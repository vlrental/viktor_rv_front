use chrono::{DateTime, Duration, Utc};
use chrono_tz::America::Vancouver;
use dioxus::prelude::*;

use crate::{api, components::Icon, Route};

fn display_date(timestamp: &str) -> String {
    DateTime::parse_from_rfc3339(timestamp)
        .map(|value| {
            value
                .with_timezone(&Vancouver)
                .format("%b %-d, %Y")
                .to_string()
        })
        .unwrap_or_else(|_| timestamp.get(0..10).unwrap_or(timestamp).to_string())
}

fn display_money(currency: &str, amount: &str) -> String {
    let value = amount.parse::<f64>().unwrap_or_default();
    format!("{currency} ${value:.2}")
}

fn status_label(status: &str) -> &'static str {
    match status {
        "pending_payment" => "Pending payment",
        "confirmed" => "Confirmed",
        "active" => "Active",
        "completed" => "Completed",
        "cancelled" => "Cancelled",
        _ => "Other",
    }
}

fn payment_label(status: &str) -> &'static str {
    match status {
        "test_paid" => "Test paid",
        "paid" => "Paid",
        "partially_paid" => "Partially paid",
        "unpaid" => "Unpaid",
        "refunded" => "Refunded",
        _ => "Recorded",
    }
}

fn is_future_booking(booking: &api::AdminBooking, now: DateTime<Utc>) -> bool {
    matches!(
        booking.status.as_str(),
        "pending_payment" | "confirmed" | "active"
    ) && DateTime::parse_from_rfc3339(&booking.ends_at)
        .map(|value| value.with_timezone(&Utc) > now)
        .unwrap_or(false)
}

#[component]
pub fn Admin() -> Element {
    let navigator = use_navigator();
    let today = Utc::now().with_timezone(&Vancouver).date_naive();
    let mut authorized = use_signal(|| None::<bool>);
    let mut active_tab = use_signal(|| "bookings".to_string());
    let mut rentals = use_signal(Vec::<api::Rental>::new);
    let mut bookings = use_signal(Vec::<api::AdminBooking>::new);
    let mut blocks = use_signal(Vec::<api::AdminAvailabilityBlock>::new);
    let mut bookings_loading = use_signal(|| true);
    let mut search = use_signal(String::new);
    let mut booking_rental = use_signal(|| "all".to_string());
    let mut booking_status = use_signal(|| "all".to_string());
    let mut rental_slug = use_signal(String::new);
    let mut starts_on = use_signal(|| (today + Duration::days(1)).to_string());
    let mut ends_on = use_signal(|| (today + Duration::days(4)).to_string());
    let mut reason = use_signal(|| "Owner use".to_string());
    let mut busy = use_signal(|| false);
    let mut error = use_signal(String::new);
    let mut success = use_signal(String::new);

    let reload_blocks = move || async move {
        match api::admin_availability_blocks().await {
            Ok(values) => blocks.set(values),
            Err(api_error) => error.set(api_error.message),
        }
    };

    let reload_bookings = move || async move {
        bookings_loading.set(true);
        match api::admin_bookings().await {
            Ok(values) => bookings.set(values),
            Err(api_error) => error.set(api_error.message),
        }
        bookings_loading.set(false);
    };

    use_effect(move || {
        spawn(async move {
            match api::auth_me().await {
                Ok(user) if user.role == "admin" => {
                    let _ = api::save_auth_user(&user);
                    authorized.set(Some(true));
                    let catalog_search = api::CatalogSearchDraft {
                        location: "Kelowna, BC".into(),
                        radius_km: 150,
                        starts_on: None,
                        ends_on: None,
                        guests: 1,
                    };
                    match api::catalog(&catalog_search).await {
                        Ok(values) => {
                            if rental_slug.peek().is_empty() {
                                if let Some(first) = values.first() {
                                    rental_slug.set(first.slug.clone());
                                }
                            }
                            rentals.set(values);
                        }
                        Err(message) => error.set(message),
                    }
                    reload_bookings().await;
                    reload_blocks().await;
                }
                Ok(_) | Err(_) => {
                    bookings_loading.set(false);
                    authorized.set(Some(false));
                }
            }
        });
    });

    let create_block = move |event: FormEvent| {
        event.prevent_default();
        let values = (
            rental_slug.read().clone(),
            starts_on.read().clone(),
            ends_on.read().clone(),
            reason.read().clone(),
        );
        async move {
            error.set(String::new());
            success.set(String::new());
            if values.0.is_empty() || values.1.is_empty() || values.2.is_empty() {
                error.set("Choose an RV and both dates.".into());
                return;
            }
            if values.2 <= values.1 {
                error.set("The reopening date must be later than the closing date.".into());
                return;
            }
            busy.set(true);
            match api::create_admin_availability_block(
                &values.0, &values.1, &values.2, &values.3,
            )
            .await
            {
                Ok(block) => {
                    success.set(format!(
                        "{} is closed from {} at 2:00 PM until {} at 11:00 AM.",
                        block.rental_name, values.1, values.2
                    ));
                    reload_blocks().await;
                }
                Err(api_error) if api_error.is_conflict() => error.set(
                    "These dates overlap a customer booking. Review the booking before closing this period."
                        .into(),
                ),
                Err(api_error) => error.set(api_error.message),
            }
            busy.set(false);
        }
    };

    let all_bookings = bookings.read().clone();
    let now = Utc::now();
    let active_bookings = all_bookings
        .iter()
        .filter(|booking| is_future_booking(booking, now))
        .count();
    let upcoming_guests: i32 = all_bookings
        .iter()
        .filter(|booking| is_future_booking(booking, now))
        .map(|booking| booking.guests)
        .sum();
    let next_thirty_days = now + Duration::days(30);
    let upcoming_arrivals = all_bookings
        .iter()
        .filter(|booking| {
            is_future_booking(booking, now)
                && DateTime::parse_from_rfc3339(&booking.starts_at)
                    .map(|value| {
                        let value = value.with_timezone(&Utc);
                        value >= now && value <= next_thirty_days
                    })
                    .unwrap_or(false)
        })
        .count();
    let search_value = search.read().trim().to_lowercase();
    let selected_rental = booking_rental.read().clone();
    let selected_status = booking_status.read().clone();
    let filtered_bookings = all_bookings
        .into_iter()
        .filter(|booking| selected_rental == "all" || booking.rental_slug == selected_rental)
        .filter(|booking| selected_status == "all" || booking.status == selected_status)
        .filter(|booking| {
            search_value.is_empty()
                || format!(
                    "{} {} {} {} {} {}",
                    booking.first_name,
                    booking.last_name,
                    booking.email,
                    booking.phone,
                    booking.booking_number,
                    booking.rental_name
                )
                .to_lowercase()
                .contains(&search_value)
        })
        .collect::<Vec<_>>();

    rsx! {
        section { class: "admin-page",
            if authorized.read().is_none() {
                div { class: "admin-state", h1 { "Checking admin access…" } }
            } else if !authorized.read().unwrap_or(false) {
                div { class: "admin-state",
                    Icon { name: "shield-alert", size: 34, color: "var(--vl-coral)" }
                    h1 { "Admin access required" }
                    p { "Sign in with an administrator account to view bookings and manage dates." }
                    button { class: "btn-forest", onclick: move |_| { navigator.push(Route::Login {}); }, "Sign in" }
                }
            } else {
                div { class: "admin-shell",
                    div { class: "admin-hero",
                        div {
                            p { class: "admin-kicker", "VL RENTAL · PRIVATE ADMIN" }
                            h1 { "Booking dashboard" }
                            p { "See every guest and reservation by RV, then close dates when a trailer is unavailable." }
                        }
                        a { class: "admin-book-link", href: "/#home-rentals",
                            Icon { name: "calendar-check", size: 17, color: "currentColor" }
                            "Customer booking"
                        }
                    }

                    div { class: "admin-metrics",
                        article { span { "ACTIVE & UPCOMING" } strong { "{active_bookings}" } small { "blocking the calendar" } }
                        article { span { "GUESTS" } strong { "{upcoming_guests}" } small { "on upcoming trips" } }
                        article { span { "NEXT 30 DAYS" } strong { "{upcoming_arrivals}" } small { "arrivals" } }
                        article { span { "ADMIN BLOCKS" } strong { "{blocks.read().len()}" } small { "closed periods" } }
                    }

                    div { class: "admin-tabs", role: "tablist", aria_label: "Admin sections",
                        button { class: if active_tab.read().as_str() == "bookings" { "active" } else { "" }, r#type: "button", role: "tab", aria_selected: active_tab.read().as_str() == "bookings", onclick: move |_| active_tab.set("bookings".into()),
                            Icon { name: "notebook-tabs", size: 17, color: "currentColor" } "Bookings"
                        }
                        button { class: if active_tab.read().as_str() == "blocks" { "active" } else { "" }, r#type: "button", role: "tab", aria_selected: active_tab.read().as_str() == "blocks", onclick: move |_| active_tab.set("blocks".into()),
                            Icon { name: "calendar-off", size: 17, color: "currentColor" } "Closed dates"
                        }
                    }

                    if active_tab.read().as_str() == "bookings" {
                        div { class: "admin-dashboard-layout",
                            aside { class: "admin-fleet",
                                div { class: "admin-fleet-head", span { "FILTER BY RV" } small { "{rentals.read().len()} units" } }
                                button { class: if selected_rental == "all" { "active" } else { "" }, r#type: "button", onclick: move |_| booking_rental.set("all".into()),
                                    span { "All RVs" } strong { "{bookings.read().len()}" }
                                }
                                for rental in rentals.read().iter() {
                                    button { key: "{rental.slug}", class: if selected_rental == rental.slug { "active" } else { "" }, r#type: "button", onclick: { let slug = rental.slug.clone(); move |_| booking_rental.set(slug.clone()) },
                                        span { "{rental.name}" }
                                        strong { "{bookings.read().iter().filter(|booking| booking.rental_slug == rental.slug).count()}" }
                                    }
                                }
                            }

                            section { class: "admin-bookings-panel",
                                div { class: "admin-bookings-toolbar",
                                    div { h2 { "All bookings" } p { "{filtered_bookings.len()} matching reservations" } }
                                    button { class: "admin-refresh", r#type: "button", disabled: *bookings_loading.read(), onclick: move |_| async move { error.set(String::new()); reload_bookings().await; },
                                        Icon { name: "refresh-cw", size: 15, color: "currentColor" }
                                        "Refresh"
                                    }
                                }
                                div { class: "admin-filters",
                                    label { class: "admin-search",
                                        Icon { name: "search", size: 17, color: "var(--vl-muted)" }
                                        input { r#type: "search", value: "{search}", oninput: move |event| search.set(event.value()), placeholder: "Guest, email, phone or booking #" }
                                    }
                                    select { class: "admin-mobile-rv-filter", aria_label: "Filter by RV", value: "{booking_rental}", onchange: move |event| booking_rental.set(event.value()),
                                        option { value: "all", "All RVs" }
                                        for rental in rentals.read().iter() { option { value: "{rental.slug}", "{rental.name}" } }
                                    }
                                    select { aria_label: "Filter by booking status", value: "{booking_status}", onchange: move |event| booking_status.set(event.value()),
                                        option { value: "all", "All statuses" }
                                        option { value: "pending_payment", "Pending payment" }
                                        option { value: "confirmed", "Confirmed" }
                                        option { value: "active", "Active" }
                                        option { value: "completed", "Completed" }
                                        option { value: "cancelled", "Cancelled" }
                                    }
                                }
                                if !error.read().is_empty() { p { class: "admin-error", role: "alert", "{error}" } }
                                if *bookings_loading.read() {
                                    div { class: "admin-bookings-loading", Icon { name: "loader-circle", size: 22, color: "var(--vl-forest)" } "Loading private booking data…" }
                                } else if filtered_bookings.is_empty() {
                                    div { class: "admin-empty", "No bookings match these filters." }
                                } else {
                                    div { class: "admin-table-wrap",
                                        table { class: "admin-bookings-table",
                                            thead { tr { th { "Guest" } th { "RV" } th { "Rental dates" } th { "Status" } th { "Payment" } th { "Total" } } }
                                            tbody { for booking in filtered_bookings.iter() {
                                                tr { key: "{booking.booking_id}",
                                                    td { div { class: "admin-guest",
                                                        strong { "{booking.first_name} {booking.last_name}" }
                                                        a { href: "mailto:{booking.email}", "{booking.email}" }
                                                        a { href: "tel:{booking.phone}", "{booking.phone}" }
                                                        small { "{booking.booking_number} · {booking.guests} guests" }
                                                    } }
                                                    td { strong { class: "admin-rv-name", "{booking.rental_name}" } }
                                                    td { span { class: "admin-date-range", "{display_date(&booking.starts_at)}" } small { "to {display_date(&booking.ends_at)}" } }
                                                    td { span { class: "admin-status admin-status-{booking.status}", "{status_label(&booking.status)}" } }
                                                    td { span { class: "admin-payment", "{payment_label(&booking.payment_status)}" } small { "Due now {display_money(&booking.currency, &booking.amount_due_now)}" } }
                                                    td { strong { "{display_money(&booking.currency, &booking.total)}" } }
                                                }
                                            } }
                                        }
                                    }
                                    div { class: "admin-booking-cards",
                                        for booking in filtered_bookings.iter() {
                                            article { key: "mobile-{booking.booking_id}", class: "admin-booking-card",
                                                div { class: "admin-booking-card-head",
                                                    div { strong { "{booking.first_name} {booking.last_name}" } small { "{booking.booking_number}" } }
                                                    span { class: "admin-status admin-status-{booking.status}", "{status_label(&booking.status)}" }
                                                }
                                                dl {
                                                    div { dt { "RV" } dd { "{booking.rental_name}" } }
                                                    div { dt { "Dates" } dd { "{display_date(&booking.starts_at)} – {display_date(&booking.ends_at)}" } }
                                                    div { dt { "Guests" } dd { "{booking.guests}" } }
                                                    div { dt { "Payment" } dd { "{payment_label(&booking.payment_status)}" } }
                                                    div { dt { "Total" } dd { "{display_money(&booking.currency, &booking.total)}" } }
                                                }
                                                div { class: "admin-booking-contact",
                                                    a { href: "mailto:{booking.email}", Icon { name: "mail", size: 14, color: "currentColor" } "{booking.email}" }
                                                    a { href: "tel:{booking.phone}", Icon { name: "phone", size: 14, color: "currentColor" } "{booking.phone}" }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        div { class: "admin-layout",
                            form { class: "admin-card", onsubmit: create_block,
                                h2 { "Add a calendar block" }
                                p { "The RV disappears from customer search for this entire period." }
                                label { r#for: "admin-rv", "RV" }
                                select { id: "admin-rv", value: "{rental_slug}", onchange: move |event| rental_slug.set(event.value()),
                                    for rental in rentals.read().iter() { option { value: "{rental.slug}", "{rental.name}" } }
                                }
                                div { class: "admin-date-grid",
                                    div { label { r#for: "admin-start", "Close from · delivery 2:00 PM" } input { id: "admin-start", r#type: "date", min: "{today}", value: "{starts_on}", onchange: move |event| starts_on.set(event.value()) } }
                                    div { label { r#for: "admin-end", "Open again · return 11:00 AM" } input { id: "admin-end", r#type: "date", min: "{today}", value: "{ends_on}", onchange: move |event| ends_on.set(event.value()) } }
                                }
                                label { r#for: "admin-reason", "Reason (admin only)" }
                                input { id: "admin-reason", maxlength: 300, value: "{reason}", oninput: move |event| reason.set(event.value()), placeholder: "Owner use, maintenance, repair…" }
                                div { class: "admin-time-rule", Icon { name: "clock-3", size: 17, color: "var(--vl-forest)" } span { "A customer may receive this RV at 2:00 PM on the day a previous rental returns at 11:00 AM." } }
                                if !error.read().is_empty() { p { class: "admin-error", role: "alert", "{error}" } }
                                if !success.read().is_empty() { p { class: "admin-success", role: "status", "{success}" } }
                                button { class: "admin-submit", r#type: "submit", disabled: *busy.read() || rentals.read().is_empty(),
                                    Icon { name: "calendar-off", size: 17, color: "currentColor" }
                                    if *busy.read() { "Closing dates…" } else { "Close dates for customers" }
                                }
                            }
                            div { class: "admin-card admin-block-list",
                                h2 { "Upcoming admin blocks" }
                                p { "Only blocks created here can be reopened here. Imported owner blocks remain protected." }
                                if blocks.read().is_empty() { div { class: "admin-empty", "No upcoming admin blocks." } }
                                else { for block in blocks.read().clone() {
                                    article { key: "{block.availability_block_id}", class: "admin-block",
                                        div { strong { "{block.rental_name}" } span { "{display_date(&block.starts_at)} at 2:00 PM → {display_date(&block.ends_at)} at 11:00 AM" } small { "{block.reason}" } }
                                        button { r#type: "button", disabled: *busy.read(), onclick: { let block_id = block.availability_block_id.clone(); move |_| { let block_id = block_id.clone(); async move { busy.set(true); error.set(String::new()); success.set(String::new()); match api::delete_admin_availability_block(&block_id).await { Ok(()) => { success.set("The dates are open for customers again.".into()); reload_blocks().await; }, Err(api_error) => error.set(api_error.message) } busy.set(false); } } }, "Reopen dates" }
                                    }
                                } }
                            }
                        }
                    }
                }
            }
        }
    }
}

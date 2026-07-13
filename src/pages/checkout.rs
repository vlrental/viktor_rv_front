use dioxus::prelude::*;

use crate::{api, components::Icon, Route};

const CSS: Asset = asset!("/assets/css/checkout.css");

#[component]
pub fn Checkout() -> Element {
    let quote = api::load_json::<api::QuoteResponse>("vl_active_quote");
    let saved_draft = api::load_json::<api::TripDraft>("vl_trip_draft");
    let recovery_slug = saved_draft.as_ref().and_then(|draft| {
        (!api::rv_delivery_ready(draft)).then(|| draft.rental_slug.clone())
    });
    let draft = saved_draft.filter(api::rv_delivery_ready);
    let user = api::current_user();
    let first_name = use_signal(String::new);
    let last_name = use_signal(String::new);
    let email = use_signal(|| user.as_ref().map(|value| value.email.clone()).unwrap_or_default());
    let phone = use_signal(String::new);
    let mut notes = use_signal(String::new);
    let mut accepted = use_signal(|| false);
    let mut busy = use_signal(|| false);
    let mut error = use_signal(String::new);
    let nav = use_navigator();
    let quote_for_confirm = quote.clone();
    let draft_for_confirm = draft.clone();
    let confirm = move |_| {
        let active_quote = quote_for_confirm.clone();
        let selected_draft = draft_for_confirm.clone();
        let values = (
            first_name.read().clone(), last_name.read().clone(), email.read().clone(),
            phone.read().clone(), notes.read().clone(), *accepted.read(),
        );
        async move {
            let Some(active_quote) = active_quote else {
                error.set("Your quote is missing. Please calculate it again.".to_string());
                return;
            };
            if !values.5 {
                error.set("Please accept the rental terms.".to_string());
                return;
            }
            if values.0.trim().len() < 2 || values.1.trim().len() < 2 || !values.2.contains('@') || values.3.trim().len() < 7 {
                error.set("Enter your full name, email, and phone number.".to_string());
                return;
            }
            busy.set(true);
            error.set(String::new());
            let booking_notes = if let Some(draft) = selected_draft.as_ref() {
                let mut parts = Vec::new();
                if !values.4.trim().is_empty() { parts.push(values.4.trim().to_string()); }
                if let Some(address) = draft.delivery_address.as_ref().filter(|value| !value.is_empty()) { parts.push(format!("Delivery address: {address}")); }
                if let Some(distance) = draft.delivery_km.as_ref() { parts.push(format!("Delivery distance: {distance} km one way")); }
                parts.push(format!("Festival/event: {}", if draft.attending_event { "yes" } else { "no" }));
                parts.push(format!("Towing after delivery: {}", if draft.towing_after_delivery { "yes" } else { "no" }));
                parts.join("\n")
            } else { values.4.clone() };
            match api::create_booking(&active_quote.quote.quote_id, &values.0, &values.1, &values.2, &values.3, &booking_notes).await {
                Ok(created) => {
                    let _ = api::save_json("vl_last_booking", &created);
                    nav.push(Route::Confirmed {});
                }
                Err(api_error) => {
                    if api_error.is_conflict() {
                        if let Some(draft) = selected_draft.as_ref() {
                            api::prepare_catalog_after_conflict(draft);
                        } else {
                            api::remove_saved("vl_active_quote");
                        }
                        nav.push(Route::Catalog {});
                    } else {
                        error.set(api_error.message);
                    }
                },
            }
            busy.set(false);
        }
    };

    rsx! {
        document::Link { rel: "stylesheet", href: CSS }
        div { class: "co-body",
            div { class: "co-breadcrumb",
                Link { class: "co-breadcrumb-a", to: Route::Catalog {}, "RV Rentals" }
                Icon { name: "chevron-right", size: 15, color: "var(--vl-muted)" }
                span { class: "co-breadcrumb-b", "Confirm booking" }
            }
            h1 { class: "co-title", "Confirm your test booking" }
            if let (Some(quote), Some(draft)) = (quote, draft) {
                div { class: "co-row",
                    div { class: "co-left",
                        div { class: "co-card",
                            div { class: "co-card-h", "Your trip" }
                            TripRow { label: "Rental", value: quote.quote.rental_slug.clone() }
                            TripRow { label: "Delivery/setup", value: format!("{} at 2:00 PM", draft.starts_on) }
                            TripRow { label: "Return", value: format!("{} at 11:00 AM · {} nights", draft.ends_on, quote.quote.units) }
                            TripRow { label: "Guests", value: format!("{} guests", draft.guests) }
                            TripRow { label: "Delivery", value: format!("Delivery and setup · {} km one way", draft.delivery_km.as_deref().unwrap_or_default()) }
                            if let Some(address) = draft.delivery_address.as_ref().filter(|value| !value.is_empty()) {
                                TripRow { label: "Address", value: address.clone() }
                            }
                            TripRow { label: "Festival or event", value: if draft.attending_event { "Yes".to_string() } else { "No".to_string() } }
                            if draft.delivery_km.is_some() {
                                TripRow { label: "Moving after setup", value: if draft.towing_after_delivery { "Yes".to_string() } else { "No, stationary stay".to_string() } }
                            }
                        }
                        div { class: "co-card",
                            div { class: "co-card-h", "Guest details" }
                            div { class: "co-field-row stack",
                                Field { label: "First name", value: first_name, kind: "text" }
                                Field { label: "Last name", value: last_name, kind: "text" }
                            }
                            div { class: "co-field-row stack",
                                Field { label: "Email", value: email, kind: "email" }
                                Field { label: "Phone", value: phone, kind: "tel" }
                            }
                            label { class: "co-field",
                                span { class: "co-field-label", "Notes (optional)" }
                                textarea { class: "co-input", value: "{notes}", oninput: move |e| notes.set(e.value()) }
                            }
                        }
                        div { class: "co-card",
                            div { class: "co-card-h", "Test payment" }
                            p { "Stripe is intentionally disabled. No card details are collected and no charge is made." }
                            label { class: "co-secure",
                                input { r#type: "checkbox", checked: *accepted.read(), onchange: move |e| accepted.set(e.checked()) }
                                span { "I accept the rental terms and understand this is a test booking." }
                            }
                        }
                    }
                    div { class: "co-summary",
                        h2 { class: "co-card-h", "Price details" }
                        for item in quote.items.iter() {
                            PriceLine { label: item.label.clone(), value: format!("CA${}", item.amount) }
                        }
                        div { class: "co-divider" }
                        div { class: "co-line",
                            span { class: "co-total-label", "Total (CAD)" }
                            span { class: "co-total-value", "CA${quote.quote.total}" }
                        }
                        if user.is_none() {
                            div { class: "co-auth-choice",
                                h2 { "Sign in to confirm" }
                                p { "Your quote is saved while you sign in." }
                                Link { class: "co-email-login", to: Route::Login {}, onclick: move |_| api::remember_auth_return("/checkout"), "Sign in or create an account" }
                            }
                        } else {
                            if !error.read().is_empty() { p { class: "auth-error", role: "alert", "{error}" } }
                            button { class: "co-pay", disabled: *busy.read(), onclick: confirm,
                                if *busy.read() { "Creating booking…" } else { "Confirm test booking" }
                            }
                        }
                    }
                }
            } else {
                div { class: "co-card",
                    h2 { "Your quote is missing or expired" }
                    p { if recovery_slug.is_some() { "A delivery address and distance are required. Return to the RV and calculate delivery again." } else { "Choose a rental, delivery address and dates to calculate a new quote." } }
                    if let Some(slug) = recovery_slug {
                        Link { class: "co-pay", to: Route::RvDetail { slug }, "Enter delivery address" }
                    } else {
                        Link { class: "co-pay", to: Route::Catalog {}, "Return to catalog" }
                    }
                }
            }
        }
    }
}

#[component]
fn Field(label: &'static str, value: Signal<String>, kind: &'static str) -> Element {
    rsx! { label { class: "co-field grow",
        span { class: "co-field-label", "{label}" }
        input { class: "co-input", r#type: kind, value: "{value}", oninput: move |e| value.set(e.value()) }
    } }
}

#[component]
fn TripRow(label: &'static str, value: String) -> Element {
    rsx! { div { class: "co-trip-row", div { class: "co-trip-col", div { class: "co-trip-label", "{label}" } div { class: "co-trip-value", "{value}" } } } }
}

#[component]
fn PriceLine(label: String, value: String) -> Element {
    rsx! { div { class: "co-line", span { class: "co-line-label", "{label}" } span { class: "co-line-value", "{value}" } } }
}

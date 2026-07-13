//! Страница RV Detail / Booking — Pencil-фреймы `l3JikE` (desktop) и `wjWTt` (mobile).

use dioxus::prelude::*;
use chrono::{DateTime, Datelike, Duration, LocalResult, Months, NaiveDate, TimeZone, Timelike, Utc};
use chrono_tz::America::Vancouver;

use crate::{api, components::Icon};
use crate::data::{rv_gallery, rv_listings, Listing, PHONE};

const CSS: Asset = asset!("/assets/css/rv_detail.css");
const IMG_HOST: Asset = asset!("/assets/img/host-viktor.webp");

#[component]
pub fn RvDetail(slug: String) -> Element {
    let catalog_search = api::load_json::<api::CatalogSearchDraft>("vl_catalog_search");
    let initial_start = catalog_search.as_ref().and_then(|value| value.starts_on.clone()).unwrap_or_default();
    let initial_end = catalog_search.as_ref().and_then(|value| value.ends_on.clone()).unwrap_or_default();
    let starts_on = use_signal(|| initial_start);
    let ends_on = use_signal(|| initial_end);
    let availability_version = use_signal(|| 0_u32);
    let api_slug = slug.clone();
    let details = use_resource(move || {
        let value = api_slug.clone();
        async move { api::rental(&value).await }
    });
    let listings = rv_listings();
    let l = listings
        .iter()
        .copied()
        .find(|l| l.slug == slug)
        .unwrap_or(listings[0]);

    rsx! {
        document::Link { rel: "stylesheet", href: CSS }
        div { class: "rvd-body",
            // Breadcrumb
            div { class: "rvd-crumb",
                Link { to: crate::Route::Catalog {}, "RV Rentals" }
                Icon { name: "chevron-right", size: 14, color: "var(--vl-muted)" }
                b { "{l.title}" }
            }
            if let Some(result) = details.read().as_ref() {
                match result {
                    Ok(value) => rsx! { div { class: "rvd-min-pill", "Live availability and pricing loaded for {value.rental.name}: {value.features.len()} features, {value.addons.len()} add-ons, {value.media.len()} photos." } },
                    Err(message) => rsx! { p { class: "auth-error", role: "alert", "Could not refresh rental data: {message}" } },
                }
            }
            TitleHead { listing: l }
            Gallery { listing: l }
            div { class: "rvd-content",
                div { class: "rvd-left",
                    Overview {}
                    div { class: "rvd-divider" }
                    Highlights {}
                    AboutRv {}
                    Amenities {}
                    GoodToKnow {}
                }
                BookingCard { listing: l, starts_on, ends_on, availability_version }
            }
        }
    }
}

#[component]
fn TitleHead(listing: Listing) -> Element {
    rsx! {
        div { class: "rvd-title-head",
            div { class: "rvd-title-left",
                h1 { class: "rvd-title", "{listing.title}" }
                div { class: "rvd-meta",
                    div { class: "rvd-meta-item",
                        Icon { name: "star", size: 15, color: "var(--vl-accent)" }
                        span { class: "rvd-meta-strong", "{listing.rating}" }
                        span { "(38 reviews)" }
                    }
                    span { "·" }
                    div { class: "rvd-meta-item",
                        Icon { name: "map-pin", size: 15, color: "var(--vl-accent)" }
                        span { "Kelowna, BC" }
                    }
                    span { "·" }
                    div { class: "rvd-pet-pill",
                        Icon { name: "paw-print", size: 14, color: "var(--vl-forest)" }
                        span { "{listing.badge}" }
                    }
                }
            }
            div { class: "rvd-actions",
                button { class: "rvd-action-btn",
                    Icon { name: "share", size: 15, color: "var(--vl-ink)" }
                    span { "Share" }
                }
                button { class: "rvd-action-btn",
                    Icon { name: "heart", size: 15, color: "var(--vl-ink)" }
                    span { "Save" }
                }
            }
        }
    }
}

#[component]
fn Gallery(listing: Listing) -> Element {
    let images = rv_gallery(listing.slug);
    let image_count = images.len();
    let mut selected = use_signal(|| None::<usize>);

    use_effect(move || {
        let is_open = selected.read().is_some();
        document::eval(&format!(
            r#"
                if (window.__vlGalleryKeyHandler) {{
                    document.removeEventListener('keydown', window.__vlGalleryKeyHandler);
                    window.__vlGalleryKeyHandler = null;
                }}
                const overlay = document.getElementById('rvd-lightbox');
                if ({is_open} && overlay) {{
                    window.__vlGalleryKeyHandler = (event) => {{
                        if (event.key === 'Escape') document.getElementById('rvd-lightbox-close')?.click();
                        if (event.key === 'ArrowLeft') document.getElementById('rvd-lightbox-prev')?.click();
                        if (event.key === 'ArrowRight') document.getElementById('rvd-lightbox-next')?.click();
                    }};
                    document.addEventListener('keydown', window.__vlGalleryKeyHandler);
                    if (!overlay.dataset.swipeReady) {{
                        overlay.dataset.swipeReady = 'true';
                        let startX = null;
                        overlay.addEventListener('touchstart', (event) => {{ startX = event.touches[0]?.clientX ?? null; }}, {{ passive: true }});
                        overlay.addEventListener('touchend', (event) => {{
                            if (startX === null) return;
                            const endX = event.changedTouches[0]?.clientX ?? startX;
                            const distance = endX - startX;
                            if (Math.abs(distance) > 50) document.getElementById(distance > 0 ? 'rvd-lightbox-prev' : 'rvd-lightbox-next')?.click();
                            startX = null;
                        }}, {{ passive: true }});
                    }}
                    overlay.focus();
                }}
            "#
        ));
    });

    rsx! {
        div { class: "rvd-gallery",
            button {
                class: "rvd-gallery-main",
                r#type: "button",
                aria_label: "Open photo 1 of {image_count}",
                style: "background-image: url('{listing.image}');",
                onclick: move |_| selected.set(Some(0)),
            }
            div { class: "rvd-gallery-grid",
                for (index, image) in images.iter().copied().enumerate().skip(1).take(6) {
                    button {
                        key: "gallery-{index}",
                        class: "rvd-gallery-tile",
                        r#type: "button",
                        aria_label: if index == 6 && image_count > 7 { "Show all {image_count} photos" } else { "Open photo {index + 1} of {image_count}" },
                        style: "background-image: url('{image}');",
                        onclick: move |_| selected.set(Some(if index == 6 && image_count > 7 { 7 } else { index })),
                        if index == 6 && image_count > 7 {
                            span { class: "rvd-gallery-more", "Show all {image_count} photos" }
                        }
                    }
                }
            }
        }

        if let Some(index) = *selected.read() {
            div {
                id: "rvd-lightbox",
                class: "rvd-lightbox",
                role: "dialog",
                aria_modal: "true",
                aria_label: "{listing.title} photo gallery",
                tabindex: "-1",
                onclick: move |_| selected.set(None),
                div {
                    class: "rvd-lightbox-content",
                    onclick: move |event| event.stop_propagation(),
                    img {
                        class: "rvd-lightbox-image",
                        src: "{images[index]}",
                        alt: "{listing.title}, photo {index + 1} of {image_count}",
                    }
                    button {
                        id: "rvd-lightbox-close",
                        class: "rvd-lightbox-close",
                        r#type: "button",
                        aria_label: "Close gallery",
                        onclick: move |_| selected.set(None),
                        Icon { name: "x", size: 24, color: "var(--vl-white)" }
                    }
                    button {
                        id: "rvd-lightbox-prev",
                        class: "rvd-lightbox-nav prev",
                        r#type: "button",
                        aria_label: "Previous photo",
                        onclick: move |_| selected.set(Some((index + image_count - 1) % image_count)),
                        Icon { name: "chevron-left", size: 30, color: "var(--vl-white)" }
                    }
                    button {
                        id: "rvd-lightbox-next",
                        class: "rvd-lightbox-nav next",
                        r#type: "button",
                        aria_label: "Next photo",
                        onclick: move |_| selected.set(Some((index + 1) % image_count)),
                        Icon { name: "chevron-right", size: 30, color: "var(--vl-white)" }
                    }
                    div { class: "rvd-lightbox-count", "{index + 1} / {image_count}" }
                }
            }
        }
    }
}

#[component]
fn Overview() -> Element {
    rsx! {
        div { class: "rvd-overview",
            div {
                div { class: "rvd-overview-t", "Entire travel trailer · hosted by Viktor" }
                div { class: "rvd-overview-s",
                    "Sleeps 10 · 32 ft · Family bunkhouse · 3-night minimum"
                }
            }
            div { class: "rvd-avatar", style: "background-image: url('{IMG_HOST}');" }
        }
    }
}

#[component]
fn Highlights() -> Element {
    rsx! {
        div { class: "rvd-highlights",
            for (icon , title , desc) in [
                ("users", "Sleeps 10", "Queen bed + double bunks"),
                ("ruler", "32 ft length", "Bumper-to-bumper, 1 slide-out"),
                ("move-horizontal", "1 slide-out", "Extra living space"),
                ("shield-check", "Family ready", "Spacious bunkhouse layout"),
            ]
            {
                div { key: "hl-{title}", class: "rvd-hl",
                    Icon { name: icon, size: 20, color: "var(--vl-forest)" }
                    div { class: "rvd-hl-t", "{title}" }
                    div { class: "rvd-hl-d", "{desc}" }
                }
            }
        }
    }
}

#[component]
fn AboutRv() -> Element {
    rsx! {
        div { class: "rvd-sec",
            h2 { class: "rvd-h", "About this RV" }
            p { class: "rvd-p",
                "A comfortable Keystone Bullet 272BHS travel trailer that's perfect for family getaways across the Okanagan. Sleeps up to 10 with a private queen bedroom, double bunks and convertible living areas, plus a full kitchen and bathroom with shower, TV, powered awning, furnace and A/C. Fully equipped and meticulously maintained, with delivery and setup at your campsite within 150 km of Kelowna."
            }
        }
    }
}

#[component]
fn Amenities() -> Element {
    rsx! {
        div { class: "rvd-sec",
            h2 { class: "rvd-h", "What this RV offers" }
            div { class: "rvd-amenities",
                for (icon , label) in [
                    ("utensils", "Full kitchen"),
                    ("refrigerator", "Fridge & freezer"),
                    ("flame", "Furnace heating"),
                    ("snowflake", "Air conditioning"),
                    ("bed-double", "Sleeps 10"),
                    ("bath", "Bathroom & shower"),
                    ("tv", "TV"),
                    ("battery-charging", "Battery & solar"),
                    ("droplets", "Fresh water tank"),
                    ("umbrella", "Power awning"),
                    ("cable", "30A hookups"),
                    ("cooking-pot", "Cookware & dishes"),
                ]
                {
                    div { key: "am-{label}", class: "rvd-am",
                        Icon { name: icon, size: 18, color: "var(--vl-forest)" }
                        span { "{label}" }
                    }
                }
            }
        }
    }
}

#[component]
fn GoodToKnow() -> Element {
    rsx! {
        div { class: "rvd-sec",
            h2 { class: "rvd-h", "Good to know" }
            div { class: "rvd-gtk",
                div { class: "rvd-gtk-col",
                    GtkCard {
                        icon: "truck",
                        title: "Delivery up to 150 km",
                        desc: "CA$150 through 50 km, then CA$3.50 per additional one-way kilometre — calculated automatically.",
                    }
                    GtkCard {
                        icon: "shield-check",
                        title: "Insurance included",
                        desc: "Every rental is covered — travel with peace of mind.",
                    }
                    GtkCard {
                        icon: "triangle-alert",
                        title: "No off-roading",
                        desc: "Keep to maintained roads — a $200 fee applies if the trailer is taken off-road.",
                    }
                }
                div { class: "rvd-gtk-col",
                    GtkCard {
                        icon: "clock",
                        title: "24/7 roadside assistance",
                        desc: "Help is a call away, wherever you camp.",
                    }
                    GtkCard {
                        icon: "file-text",
                        title: "$1,000 deposit",
                        desc: "Refundable security deposit — unused amount returned within a week.",
                    }
                    GtkCard {
                        icon: "utensils",
                        title: "Dishes & coffeemaker included",
                        desc: "Return them washed and the RV clean — a $100 cleaning fee applies otherwise.",
                    }
                }
            }
        }
    }
}

#[component]
fn GtkCard(icon: &'static str, title: &'static str, desc: &'static str) -> Element {
    rsx! {
        div { class: "rvd-gtk-card",
            div { class: "rvd-gtk-ib",
                Icon { name: icon, size: 20, color: "var(--vl-forest)" }
            }
            div {
                div { class: "rvd-gtk-t", "{title}" }
                div { class: "rvd-gtk-d", "{desc}" }
            }
        }
    }
}

#[component]
fn BookingCard(
    listing: Listing,
    mut starts_on: Signal<String>,
    mut ends_on: Signal<String>,
    mut availability_version: Signal<u32>,
) -> Element {
    let recovery_draft = api::load_json::<api::TripDraft>("vl_trip_draft")
        .filter(|draft| draft.rental_slug == listing.slug);
    let initial_delivery_address = recovery_draft
        .as_ref()
        .and_then(|draft| draft.delivery_address.clone())
        .unwrap_or_default();
    let initial_delivery_km = recovery_draft
        .as_ref()
        .and_then(|draft| draft.delivery_km.clone());
    let needs_delivery_recovery = recovery_draft.as_ref().is_some_and(|draft| {
        draft
            .delivery_address
            .as_deref()
            .map(str::trim)
            .unwrap_or_default()
            .is_empty()
            || draft.delivery_km.is_none()
    });
    let initial_guests = api::load_json::<api::CatalogSearchDraft>("vl_catalog_search").map(|value| value.guests.clamp(1, 10)).unwrap_or(2);
    let mut guests = use_signal(|| initial_guests);
    let delivery_address = use_signal(|| initial_delivery_address);
    let delivery_km = use_signal(|| initial_delivery_km);
    let addon_keys = use_signal(Vec::<String>::new);
    let attending_event = use_signal(|| false);
    let towing_after_delivery = use_signal(|| false);
    let mut busy = use_signal(|| false);
    let mut error = use_signal(String::new);
    let mut calendar_open = use_signal(|| needs_delivery_recovery);
    let mut guests_open = use_signal(|| false);
    let maximum_guests = listing.meta.split("Sleeps ").nth(1).and_then(|value| value.parse::<i32>().ok()).unwrap_or(4);
    let selected_nights = selected_date(&starts_on).zip(selected_date(&ends_on)).map(|(start, end)| (end - start).num_days()).unwrap_or(0);
    let rental_total = price_amount(listing.price) * selected_nights.max(0) as f64;
    let delivery_total = 150.0;
    let taxable_subtotal = rental_total + delivery_total;
    let gst = taxable_subtotal * 0.05;
    let pst = taxable_subtotal * 0.07;
    let refundable_deposit = 1000.0;
    let booking_total = taxable_subtotal + gst + pst + refundable_deposit;
    let saved_quote = api::load_json::<api::QuoteResponse>("vl_active_quote").filter(|value|
        value.quote.rental_slug == listing.slug
            && starts_on.read().contains(&value.quote.starts_at.get(0..10).unwrap_or_default())
            && ends_on.read().contains(&value.quote.ends_at.get(0..10).unwrap_or_default())
    );
    let nav = use_navigator();
    let reserve = move |_| {
        let draft = api::TripDraft {
            rental_slug: listing.slug.to_string(),
            starts_on: starts_on.read().clone(),
            ends_on: ends_on.read().clone(),
            guests: *guests.read(),
            addon_keys: addon_keys.read().clone(),
            delivery_km: delivery_km.read().clone(),
            delivery_address: Some(delivery_address.read().clone()),
            attending_event: *attending_event.read(),
            towing_after_delivery: *towing_after_delivery.read(),
        };
        async move {
            if draft.starts_on.is_empty() || draft.ends_on.is_empty() {
                error.set("Choose delivery and return dates first.".into());
                return;
            }
            if draft.delivery_address.as_deref().unwrap_or_default().trim().is_empty()
                || draft.delivery_km.is_none()
            {
                error.set("Open the date planner and calculate the delivery address first.".into());
                calendar_open.set(true);
                return;
            }
            busy.set(true);
            error.set(String::new());
            match api::create_quote(&draft).await {
                Ok(quote) => {
                    if api::save_json("vl_trip_draft", &draft).is_ok()
                        && api::save_json("vl_active_quote", &quote).is_ok()
                    {
                        nav.push(crate::Route::Checkout {});
                    } else {
                        error.set("Could not save your booking progress.".into());
                    }
                }
                Err(message) => {
                    if message.contains("unavailable") || message.contains("conflict") {
                        let next_version = *availability_version.read() + 1;
                        availability_version.set(next_version);
                        error.set("Those dates were just booked. The calendar has been refreshed — please choose another period.".into());
                    } else {
                        error.set(message);
                    }
                },
            }
            busy.set(false);
        }
    };
    rsx! {
        div { class: "rvd-booking",
            div { class: "rvd-price-row",
                div { class: "rvd-price",
                    span { class: "rvd-price-v", "{listing.price}" }
                    span { class: "rvd-price-u", "{listing.per}" }
                }
                div { class: "rvd-price-r",
                    Icon { name: "star", size: 14, color: "var(--vl-accent)" }
                    b { "{listing.rating}" }
                    span { "· 38 reviews" }
                }
            }
            div { class: "rvd-min-pill",
                Icon { name: "info", size: 14, color: "var(--vl-muted)" }
                span { "3-night minimum · $1,000 refundable damage deposit" }
            }
            div { class: "rvd-fields",
                div { class: "rvd-fields-dates",
                    button { class: "rvd-field rvd-date-trigger", r#type: "button", onclick: move |_| calendar_open.set(true),
                        div { class: "rvd-field-l", "DELIVERY/SETUP · 2:00 PM" }
                        div { class: "rvd-field-v",
                            if starts_on.read().is_empty() { "Choose date" } else { "{starts_on}" }
                        }
                    }
                    div { class: "rvd-field-vd" }
                    button { class: "rvd-field rvd-date-trigger", r#type: "button", onclick: move |_| calendar_open.set(true),
                        div { class: "rvd-field-l", "RETURN · 11:00 AM" }
                        div { class: "rvd-field-v",
                            if ends_on.read().is_empty() { "Choose date" } else { "{ends_on}" }
                        }
                    }
                }
                div { class: "rvd-field-hd" }
                div { class: "rvd-field-guests",
                    div { class: "rvd-guests-picker",
                        div { class: "rvd-field-l", "GUESTS" }
                        button {
                            class: "rvd-guests-trigger",
                            r#type: "button",
                            aria_expanded: *guests_open.read(),
                            onclick: move |_| {
                                let next = !*guests_open.peek();
                                guests_open.set(next);
                            },
                            span { class: "rvd-guests-trigger-main",
                                Icon { name: "users", size: 17, color: "var(--vl-forest)" }
                                span { if *guests.read() == 1 { "1 guest" } else { "{guests} guests" } }
                            }
                            Icon { name: "chevron-down", size: 17, color: "var(--vl-muted)" }
                        }
                        if *guests_open.read() {
                            div { class: "rvd-guests-menu", role: "listbox", aria_label: "Number of guests",
                                for count in 1..=maximum_guests {
                                    button {
                                        key: "guest-{count}",
                                        class: if *guests.read() == count { "rvd-guests-option selected" } else { "rvd-guests-option" },
                                        r#type: "button",
                                        role: "option",
                                        aria_selected: *guests.read() == count,
                                        onclick: move |_| {
                                            guests.set(count);
                                            guests_open.set(false);
                                        },
                                        span { if count == 1 { "1 guest" } else { "{count} guests" } }
                                        if *guests.read() == count {
                                            Icon { name: "check", size: 16, color: "var(--vl-forest)" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            div { class: "rvd-min-pill",
                Icon { name: "truck", size: 15, color: "var(--vl-forest)" }
                span { "Delivery and setup required · maximum 150 km" }
            }
            if !error.read().is_empty() { p { class: "auth-error", role: "alert", "{error}" } }
            button { class: "rvd-reserve", disabled: *busy.read(), onclick: reserve,
                if *busy.read() { "Checking availability…" } else { "Reserve" }
            }
            div { class: "rvd-note", "Test booking · no card charged · 3-night minimum" }
            if selected_nights >= 3 {
                div { class: "rvd-breakdown",
                    if let Some(quote) = saved_quote.as_ref() {
                        for item in quote.items.iter() {
                            div { class: "rvd-bd-row",
                                span { class: "rvd-bd-l", "{item.label}" }
                                span { class: "rvd-bd-v", "CA${item.amount}" }
                            }
                        }
                        div { class: "rvd-divider" }
                        div { class: "rvd-total",
                            span { class: "rvd-total-l", "Total" }
                            span { class: "rvd-total-v", "CA${quote.quote.total}" }
                        }
                    } else {
                        div { class: "rvd-bd-row",
                            span { class: "rvd-bd-l", "{listing.price} × {selected_nights} nights" }
                            span { class: "rvd-bd-v", "{money(rental_total)}" }
                        }
                        div { class: "rvd-bd-row",
                            span { class: "rvd-bd-l", "Delivery estimate" }
                            span { class: "rvd-bd-v", "Calculate address" }
                        }
                        div { class: "rvd-divider" }
                        div { class: "rvd-total",
                            span { class: "rvd-total-l", "Starting total" }
                            span { class: "rvd-total-v", "{money(booking_total)}" }
                        }
                    }
                }
            } else {
                div { class: "rvd-quote-placeholder", "Choose delivery and return dates to see the full total." }
            }
            div { class: "rvd-contact",
                Icon { name: "phone", size: 15, color: "var(--vl-forest)" }
                span { "Questions? {PHONE}" }
            }
        }
        if *calendar_open.read() {
            BookingCalendarOverlay {
                slug: listing.slug,
                price: listing.price,
                starts_on,
                ends_on,
                availability_version,
                guests,
                delivery_address,
                delivery_km,
                addon_keys,
                attending_event,
                towing_after_delivery,
                on_close: move |_| calendar_open.set(false),
            }
        }
    }
}

// ===== Live availability calendar =====

fn month_start(date: NaiveDate) -> NaiveDate {
    date.with_day(1).expect("valid first day of month")
}

fn add_months(date: NaiveDate, count: u32) -> NaiveDate {
    date.checked_add_months(Months::new(count)).expect("calendar range is valid")
}

fn calendar_cells(month: NaiveDate) -> Vec<Option<NaiveDate>> {
    let mut cells = vec![None; month.weekday().num_days_from_sunday() as usize];
    let next = add_months(month, 1);
    let mut day = month;
    while day < next {
        cells.push(Some(day));
        day += Duration::days(1);
    }
    while cells.len() % 7 != 0 { cells.push(None); }
    cells
}

type UnavailableRange = (DateTime<Utc>, DateTime<Utc>);

fn unavailable_ranges(value: &api::AvailabilityResponse) -> Vec<UnavailableRange> {
    let mut ranges = Vec::new();
    for interval in &value.unavailable {
        let Ok(start) = chrono::DateTime::parse_from_rfc3339(&interval.starts_at) else { continue };
        let Ok(end) = chrono::DateTime::parse_from_rfc3339(&interval.ends_at) else { continue };
        ranges.push((start.with_timezone(&Utc), end.with_timezone(&Utc)));
    }
    ranges
}

fn local_moment(day: NaiveDate, hour: u32) -> Option<DateTime<Utc>> {
    let naive = day.and_hms_opt(hour, 0, 0)?;
    match Vancouver.from_local_datetime(&naive) {
        LocalResult::Single(value) => Some(value.with_timezone(&Utc)),
        _ => None,
    }
}

fn range_is_available(start: DateTime<Utc>, end: DateTime<Utc>, unavailable: &[UnavailableRange]) -> bool {
    unavailable.iter().all(|(blocked_start, blocked_end)| *blocked_start >= end || *blocked_end <= start)
}

fn stay_is_available(starts_on: NaiveDate, ends_on: NaiveDate, unavailable: &[UnavailableRange]) -> bool {
    match (local_moment(starts_on, 14), local_moment(ends_on, 11)) {
        (Some(start), Some(end)) => range_is_available(start, end, unavailable),
        _ => false,
    }
}

fn minimum_stay_can_start(day: NaiveDate, minimum_nights: i64, unavailable: &[UnavailableRange]) -> bool {
    stay_is_available(day, day + Duration::days(minimum_nights), unavailable)
}

fn date_is_selectable(
    day: NaiveDate,
    selected_start: Option<NaiveDate>,
    selected_end: Option<NaiveDate>,
    minimum_nights: i64,
    unavailable: &[UnavailableRange],
) -> bool {
    if let (Some(start), None) = (selected_start, selected_end) {
        if day <= start {
            return minimum_stay_can_start(day, minimum_nights, unavailable);
        }

        return (day - start).num_days() >= minimum_nights
            && stay_is_available(start, day, unavailable);
    }

    minimum_stay_can_start(day, minimum_nights, unavailable)
}

fn next_date_selection(
    day: NaiveDate,
    selected_start: Option<NaiveDate>,
    selected_end: Option<NaiveDate>,
) -> (Option<NaiveDate>, Option<NaiveDate>) {
    if selected_start == Some(day) && selected_end.is_none() {
        return (None, None);
    }

    if selected_start.is_none() || selected_end.is_some() || day < selected_start.unwrap() {
        (Some(day), None)
    } else {
        (selected_start, Some(day))
    }
}

#[cfg(test)]
mod availability_tests {
    use super::*;

    fn day(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).unwrap()
    }

    #[test]
    fn previous_return_day_is_available_for_pickup() {
        let return_day = day(2030, 8, 10);
        let blocked = vec![(local_moment(day(2030, 8, 1), 14).unwrap(), local_moment(return_day, 11).unwrap())];
        assert!(minimum_stay_can_start(return_day, 3, &blocked));
    }

    #[test]
    fn next_pickup_day_is_available_for_return() {
        let turnover_day = day(2030, 8, 10);
        let blocked = vec![(local_moment(turnover_day, 14).unwrap(), local_moment(day(2030, 8, 13), 11).unwrap())];
        assert!(stay_is_available(day(2030, 8, 7), turnover_day, &blocked));
    }

    #[test]
    fn partial_afternoon_block_prevents_pickup() {
        let blocked_day = day(2030, 8, 10);
        let blocked = vec![(local_moment(blocked_day, 15).unwrap(), local_moment(blocked_day, 16).unwrap())];
        assert!(!minimum_stay_can_start(blocked_day, 3, &blocked));
    }

    #[test]
    fn earlier_available_day_can_replace_pickup() {
        let selected = day(2030, 8, 10);
        assert!(date_is_selectable(day(2030, 8, 8), Some(selected), None, 3, &[]));
    }

    #[test]
    fn clicking_selected_pickup_clears_selection() {
        let selected = day(2030, 8, 10);
        assert_eq!(next_date_selection(selected, Some(selected), None), (None, None));
    }

    #[test]
    fn earlier_day_restarts_selection() {
        let selected = day(2030, 8, 10);
        let earlier = day(2030, 8, 8);
        assert_eq!(next_date_selection(earlier, Some(selected), None), (Some(earlier), None));
    }
}

fn selected_date(value: &Signal<String>) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(&value.read(), "%Y-%m-%d").ok()
}

fn price_amount(price: &str) -> f64 {
    price.chars().filter(|value| value.is_ascii_digit() || *value == '.').collect::<String>().parse().unwrap_or(0.0)
}

fn money(amount: f64) -> String {
    format!("CA${amount:.2}")
}

#[component]
fn BookingCalendarOverlay(
    slug: &'static str,
    price: &'static str,
    mut guests: Signal<i32>,
    mut delivery_address: Signal<String>,
    mut delivery_km: Signal<Option<String>>,
    mut addon_keys: Signal<Vec<String>>,
    mut attending_event: Signal<bool>,
    mut towing_after_delivery: Signal<bool>,
    starts_on: Signal<String>,
    ends_on: Signal<String>,
    availability_version: Signal<u32>,
    on_close: EventHandler<()>,
) -> Element {
    let mut closing = use_signal(|| false);
    let mut address_busy = use_signal(|| false);
    let mut address_error = use_signal(String::new);
    let mut address_result = use_signal(|| None::<api::DeliveryEstimate>);
    let today = Utc::now().with_timezone(&Vancouver).date_naive();
    let initial_month = month_start(today);
    let mut visible_month = use_signal(|| selected_date(&starts_on).map(month_start).unwrap_or(initial_month));
    let availability = use_resource(move || {
        let _version = *availability_version.read();
        async move {
            api::availability(slug, &initial_month.to_string(), &add_months(initial_month, 18).to_string()).await
        }
    });
    let rental_details = use_resource(move || async move { api::rental(slug).await });
    let response = availability.read().as_ref().and_then(|result| result.as_ref().ok()).cloned();
    let availability_loaded = response.is_some();
    let unavailable = response.as_ref().map(unavailable_ranges).unwrap_or_default();
    let minimum_nights = response.as_ref().map(|value| value.minimum_nights).unwrap_or(3);
    let calendar_error = availability.read().as_ref().and_then(|result| result.as_ref().err()).cloned();
    let nights = selected_date(&starts_on).zip(selected_date(&ends_on)).map(|(start, end)| (end - start).num_days()).unwrap_or(0);
    let details = rental_details.read().as_ref().and_then(|result| result.as_ref().ok()).cloned();
    let capacity = details.as_ref().map(|value| value.rental.capacity).unwrap_or(10);
    let addons = details.as_ref().map(|value| value.addons.clone()).unwrap_or_default();
    let live_quote = use_resource(move || {
        let draft = api::TripDraft {
            rental_slug: slug.to_string(),
            starts_on: starts_on.read().clone(),
            ends_on: ends_on.read().clone(),
            guests: *guests.read(),
            addon_keys: addon_keys.read().clone(),
            delivery_km: delivery_km.read().clone(),
            delivery_address: Some(delivery_address.read().clone()),
            attending_event: *attending_event.read(),
            towing_after_delivery: *towing_after_delivery.read(),
        };
        async move {
            if draft.starts_on.is_empty() || draft.ends_on.is_empty() {
                return Err("Choose complete dates".to_string());
            }
            if draft.delivery_address.as_deref().unwrap_or_default().trim().is_empty()
                || draft.delivery_km.is_none()
            {
                return Err("Enter and calculate the delivery address".to_string());
            }
            api::create_quote(&draft).await
        }
    });
    let quote_response = live_quote.read().as_ref().and_then(|result| result.as_ref().ok()).cloned();
    let quote_error = live_quote.read().as_ref().and_then(|result| result.as_ref().err()).cloned();
    let calculate_address = move |_| {
        let address = delivery_address.read().clone();
        async move {
            address_busy.set(true);
            address_error.set(String::new());
            match api::delivery_estimate(slug, &address).await {
                Ok(result) if result.within_range => {
                    delivery_km.set(Some(result.one_way_km.clone()));
                    address_result.set(Some(result));
                }
                Ok(result) => {
                    delivery_km.set(None);
                    address_result.set(Some(result.clone()));
                    address_error.set(format!("This address is beyond the {} km delivery limit.", result.maximum_km));
                }
                Err(message) => {
                    delivery_km.set(None);
                    address_result.set(None);
                    address_error.set(message);
                }
            }
            address_busy.set(false);
        }
    };

    rsx! {
        div {
            class: if *closing.read() { "rvd-calendar-backdrop is-closing" } else { "rvd-calendar-backdrop" },
            role: "presentation",
            onclick: move |_| async move {
                if *closing.peek() { return; }
                closing.set(true);
                let _ = document::eval("await new Promise(resolve => setTimeout(resolve, 220));").await;
                on_close.call(());
            },
            div { class: "rvd-calendar-overlay", role: "dialog", aria_modal: "true", aria_label: "Choose delivery and return dates", onclick: move |event| event.stop_propagation(),
                div { class: "rvd-calendar-overlay-head",
                    div {
                        h2 { class: "rvd-avail-t", "Choose your dates" }
                        p { class: "rvd-avail-s", "Delivery/setup at 2:00 PM · return at 11:00 AM · {minimum_nights}-night minimum" }
                    }
                    button {
                        class: "rvd-calendar-close",
                        r#type: "button",
                        aria_label: "Close calendar",
                        onclick: move |_| async move {
                            if *closing.peek() { return; }
                            closing.set(true);
                            let _ = document::eval("await new Promise(resolve => setTimeout(resolve, 220));").await;
                            on_close.call(());
                        },
                        Icon { name: "x", size: 22, color: "var(--vl-ink)" }
                    }
                }
                if let Some(message) = calendar_error {
                    p { class: "auth-error", role: "alert", "Could not load availability: {message}" }
                }
                div { class: "rvd-cal-months",
                    for offset in 0..3_u32 {
                        CalendarMonth {
                            month: add_months(*visible_month.read(), offset),
                            show_prev: offset == 0,
                            show_next: offset == 2,
                            price,
                            availability_loaded,
                            unavailable: unavailable.clone(),
                            minimum_nights,
                            starts_on,
                            ends_on,
                            on_selected: move |_| {},
                            on_prev: move |_| {
                                if *visible_month.read() > initial_month {
                                    let previous = visible_month.read().checked_sub_months(Months::new(1)).unwrap();
                                    visible_month.set(previous);
                                }
                            },
                            on_next: move |_| {
                                let next = add_months(*visible_month.read(), 1);
                                if next <= add_months(initial_month, 15) { visible_month.set(next); }
                            },
                        }
                    }
                }
                if nights >= minimum_nights {
                    div { class: "rvd-trip-options",
                        div { class: "rvd-trip-option-row",
                            div {
                                div { class: "rvd-trip-option-title", Icon { name: "party-popper", size: 17, color: "var(--vl-forest)" } "Attending a festival or event?" }
                                div { class: "rvd-trip-option-help", "Tell us so the correct rental conditions can be confirmed." }
                            }
                            div { class: "rvd-choice-group",
                                button { class: if *attending_event.read() { "rvd-choice active" } else { "rvd-choice" }, r#type: "button", onclick: move |_| attending_event.set(true), "Yes" }
                                button { class: if !*attending_event.read() { "rvd-choice active" } else { "rvd-choice" }, r#type: "button", onclick: move |_| attending_event.set(false), "No" }
                            }
                        }
                        div { class: "rvd-address-card",
                                div { class: "rvd-trip-option-title", Icon { name: "map-pin", size: 17, color: "var(--vl-forest)" } "Delivery address & live distance" }
                                div { class: "rvd-trip-option-help", "From 155 Potterton Rd · CA$150 through 50 km, then CA$3.50 per additional one-way kilometre · maximum 150 km." }
                                div { class: "rvd-address-search",
                                    input { value: "{delivery_address}", placeholder: "Campsite, street address, or location", oninput: move |event| {
                                        delivery_address.set(event.value());
                                        delivery_km.set(None);
                                        address_result.set(None);
                                    } }
                                    button { r#type: "button", disabled: *address_busy.read(), onclick: calculate_address,
                                        if *address_busy.read() { "Calculating…" } else { "Calculate delivery" }
                                    }
                                }
                                if !address_error.read().is_empty() { p { class: "rvd-address-error", role: "alert", "{address_error}" } }
                                if let Some(result) = address_result.read().as_ref() {
                                    if result.within_range {
                                        div { class: "rvd-address-success",
                                            Icon { name: "check-circle-2", size: 17, color: "var(--vl-forest)" }
                                            div { strong { "{result.one_way_km} km one way · CA${result.delivery_fee} delivery" } span { "{result.resolved_address}" } }
                                        }
                                    }
                                }
                                div { class: "rvd-towing-row",
                                    span { "Will you tow the RV after delivery?" }
                                    div { class: "rvd-choice-group compact",
                                        button { class: if *towing_after_delivery.read() { "rvd-choice active" } else { "rvd-choice" }, r#type: "button", onclick: move |_| towing_after_delivery.set(true), "Yes" }
                                        button { class: if !*towing_after_delivery.read() { "rvd-choice active" } else { "rvd-choice" }, r#type: "button", onclick: move |_| towing_after_delivery.set(false), "No, it stays there" }
                                    }
                                }
                        }
                        div { class: "rvd-trip-option-row",
                            div {
                                div { class: "rvd-trip-option-title", Icon { name: "users", size: 17, color: "var(--vl-forest)" } "Guests" }
                                div { class: "rvd-trip-option-help", "Maximum {capacity} guests for this RV." }
                            }
                            div { class: "rvd-stepper",
                                button { r#type: "button", disabled: *guests.read() <= 1, onclick: move |_| {
                                    let next = (*guests.peek() - 1).max(1);
                                    guests.set(next);
                                }, "−" }
                                strong { "{guests}" }
                                button { r#type: "button", disabled: *guests.read() >= capacity, onclick: move |_| {
                                    let next = (*guests.peek() + 1).min(capacity);
                                    guests.set(next);
                                }, "+" }
                            }
                        }
                        if !addons.is_empty() {
                            div { class: "rvd-addon-picker",
                                div { class: "rvd-trip-option-title", Icon { name: "sparkles", size: 17, color: "var(--vl-forest)" } "Make your stay easier" }
                                div { class: "rvd-addon-picker-grid",
                                    for addon in addons.iter() {
                                        {
                                            let key = addon.addon_key.clone();
                                            let selected = addon_keys.read().contains(&key);
                                            rsx! { button { key: "option-{key}", class: if selected { "rvd-addon-choice selected" } else { "rvd-addon-choice" }, r#type: "button", onclick: move |_| {
                                                let mut next = addon_keys.read().clone();
                                                if let Some(index) = next.iter().position(|value| value == &key) { next.remove(index); } else { next.push(key.clone()); }
                                                addon_keys.set(next);
                                            },
                                                span { class: "rvd-addon-check", if selected { "✓" } else { "+" } }
                                                span { class: "rvd-addon-choice-copy", strong { "{addon.label}" } if addon.is_recommended { small { "Recommended" } } }
                                                b { "CA${addon.price}" }
                                            } }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                div { class: "rvd-calendar-overlay-foot",
                    if nights >= minimum_nights {
                        div { class: "rvd-calendar-trip",
                            div { class: "rvd-calendar-trip-icon",
                                Icon { name: "sparkles", size: 22, color: "var(--vl-white)" }
                            }
                            div {
                                div { class: "rvd-calendar-kicker", "YOUR OKANAGAN GETAWAY IS READY" }
                                div { class: "rvd-calendar-range", "{starts_on} → {ends_on} · {nights}-night adventure" }
                                div { class: "rvd-calendar-benefits",
                                    span { Icon { name: "zap", size: 13, color: "var(--vl-accent)" } "Instant confirmation" }
                                    span { Icon { name: "shield-check", size: 13, color: "var(--vl-accent)" } "Refundable deposit" }
                                    span { Icon { name: "check", size: 13, color: "var(--vl-accent)" } "Transparent pricing" }
                                }
                            }
                        }
                        div { class: "rvd-calendar-summary",
                            div { class: "rvd-calendar-total-label", "ESTIMATED BOOKING TOTAL" }
                            if let Some(quote) = quote_response.as_ref() {
                                div { class: "rvd-calendar-total", "CA${quote.quote.total}" }
                                div { class: "rvd-calendar-total-note", "Server-calculated total · preparation, selected add-ons, delivery, GST, PST and refundable deposit included" }
                            } else {
                                div { class: "rvd-calendar-total", "Address required" }
                                div { class: "rvd-calendar-total-note", "Calculate the delivery address to receive the exact total" }
                            }
                            if let Some(message) = quote_error.as_ref() { div { class: "rvd-calendar-quote-status", "{message}" } }
                            div { class: "rvd-calendar-actions",
                                button { class: "rvd-calendar-clear", r#type: "button", onclick: move |_| {
                                    let mut start = starts_on;
                                    let mut end = ends_on;
                                    start.set(String::new());
                                    end.set(String::new());
                                }, "Change dates" }
                                button { class: "rvd-calendar-continue", r#type: "button", disabled: quote_response.is_none(), onclick: move |_| {
                                    let selected_quote = quote_response.clone();
                                    async move {
                                    let Some(quote) = selected_quote else { return; };
                                    let draft = api::TripDraft {
                                        rental_slug: slug.to_string(),
                                        starts_on: starts_on.read().clone(),
                                        ends_on: ends_on.read().clone(),
                                        guests: *guests.read(),
                                        addon_keys: addon_keys.read().clone(),
                                        delivery_km: delivery_km.read().clone(),
                                        delivery_address: Some(delivery_address.read().clone()),
                                        attending_event: *attending_event.read(),
                                        towing_after_delivery: *towing_after_delivery.read(),
                                    };
                                    let _ = api::save_json("vl_trip_draft", &draft);
                                    let _ = api::save_json("vl_active_quote", &quote);
                                    if *closing.peek() { return; }
                                    closing.set(true);
                                    let _ = document::eval("await new Promise(resolve => setTimeout(resolve, 220));").await;
                                    on_close.call(());
                                    }
                                },
                                    span { "Use these dates" }
                                    Icon { name: "arrow-right", size: 16, color: "var(--vl-forest)" }
                                }
                            }
                        }
                    } else {
                        div { class: "rvd-calendar-prompt",
                            div { class: "rvd-calendar-trip-icon",
                                Icon { name: "calendar", size: 21, color: "var(--vl-white)" }
                            }
                            div {
                                div { class: "rvd-calendar-kicker", "PLAN YOUR ESCAPE" }
                                div { class: "rvd-calendar-prompt-title",
                                    if starts_on.read().is_empty() { "Choose your delivery date" } else { "Great start — now choose your return date" }
                                }
                                div { class: "rvd-calendar-total-note", "Three nights is all it takes to trade routine for the open road." }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn CalendarMonth(
    month: NaiveDate,
    show_prev: bool,
    show_next: bool,
    price: &'static str,
    availability_loaded: bool,
    unavailable: Vec<UnavailableRange>,
    minimum_nights: i64,
    mut starts_on: Signal<String>,
    mut ends_on: Signal<String>,
    on_selected: EventHandler<bool>,
    on_prev: EventHandler<MouseEvent>,
    on_next: EventHandler<MouseEvent>,
) -> Element {
    let start = selected_date(&starts_on);
    let end = selected_date(&ends_on);
    let title = month.format("%B %Y").to_string();
    rsx! {
        div { class: "rvd-month",
            div { class: "rvd-month-head",
                if show_prev { button { class: "rvd-month-nav", onclick: move |e| on_prev.call(e), Icon { name: "chevron-left", size: 16, color: "var(--vl-ink)" } } }
                else { div { class: "rvd-month-sp" } }
                div { class: "rvd-month-t", "{title}" }
                if show_next { button { class: "rvd-month-nav", onclick: move |e| on_next.call(e), Icon { name: "chevron-right", size: 16, color: "var(--vl-ink)" } } }
                else { div { class: "rvd-month-sp" } }
            }
            div { class: "rvd-wd",
                for (i, w) in ["S", "M", "T", "W", "T", "F", "S"].iter().enumerate() { div { key: "w-{i}", "{w}" } }
            }
            div { class: "rvd-days",
                for (i, cell) in calendar_cells(month).into_iter().enumerate() {
                    if let Some(day) = cell {
                        {
                            let now = Utc::now().with_timezone(&Vancouver);
                            let today = now.date_naive();
                            let valid_choice = date_is_selectable(day, start, end, minimum_nights, &unavailable);
                            let pickup_has_passed = day == today && now.hour() >= 14;
                            let unavailable_day = !availability_loaded || day < today || pickup_has_passed || !valid_choice;
                            let edge = start == Some(day) || end == Some(day);
                            let stay = start.zip(end).map(|(a, b)| day > a && day < b).unwrap_or(false);
                            let class = if edge { "edge" } else if stay { "stay" } else if unavailable_day { "booked" } else { "" };
                            rsx! { button {
                                key: "d-{day}", class: "rvd-day {class}", disabled: unavailable_day,
                                onclick: move |_| {
                                    let current_start = selected_date(&starts_on);
                                    let current_end = selected_date(&ends_on);
                                    let (next_start, next_end) = next_date_selection(day, current_start, current_end);
                                    match next_start {
                                        Some(value) => starts_on.set(value.to_string()),
                                        None => starts_on.set(String::new()),
                                    }
                                    match next_end {
                                        Some(value) => {
                                            ends_on.set(value.to_string());
                                            on_selected.call(true);
                                        },
                                        None => ends_on.set(String::new()),
                                    }
                                },
                                div { class: "rvd-day-n", "{day.day()}" }
                                if !unavailable_day { div { class: "rvd-day-p", "{price}" } }
                            } }
                        }
                    } else {
                        div { key: "blank-{i}", class: "rvd-day" }
                    }
                }
            }
        }
    }
}

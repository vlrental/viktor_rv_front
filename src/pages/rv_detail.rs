//! Страница RV Detail / Booking — Pencil-фреймы `l3JikE` (desktop) и `wjWTt` (mobile).

use dioxus::prelude::*;
use chrono::{DateTime, Datelike, Duration, LocalResult, Months, NaiveDate, TimeZone, Timelike, Utc};
use chrono_tz::America::Vancouver;

use crate::{api, components::Icon};
use crate::data::{
    rv_listings, Listing, IMG_BULLET, IMG_OPENRANGE, IMG_OUTBACK, IMG_ROCKWOOD, PHONE,
};

const CSS: Asset = asset!("/assets/css/rv_detail.css");
const IMG_HOST: Asset = asset!("/assets/img/host-viktor.webp");

#[component]
pub fn RvDetail(slug: String) -> Element {
    let starts_on = use_signal(String::new);
    let ends_on = use_signal(String::new);
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
            Availability { slug: l.slug, price: l.price, starts_on, ends_on, availability_version }
            AddOns {}
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
    rsx! {
        div { class: "rvd-gallery",
            div {
                class: "rvd-gallery-main",
                style: "background-image: url('{listing.image}');",
            }
            div { class: "rvd-gallery-grid",
                div { class: "rvd-gallery-row",
                    div {
                        class: "rvd-gallery-tile",
                        style: "background-image: url('{IMG_BULLET}');",
                    }
                    div {
                        class: "rvd-gallery-tile",
                        style: "background-image: url('{IMG_ROCKWOOD}');",
                    }
                }
                div { class: "rvd-gallery-row",
                    div {
                        class: "rvd-gallery-tile",
                        style: "background-image: url('{IMG_OPENRANGE}');",
                    }
                    div {
                        class: "rvd-gallery-tile",
                        style: "background-image: url('{IMG_OUTBACK}');",
                        div { class: "rvd-gallery-more",
                            Icon { name: "images", size: 17, color: "var(--vl-white)" }
                            span { "Show all 12 photos" }
                        }
                    }
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
                div { class: "rvd-overview-t", "Entire 5th wheel · hosted by Viktor" }
                div { class: "rvd-overview-s",
                    "Sleeps 4 · 32 ft · 1 slide-out · Pet-friendly · 3-night minimum"
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
                ("users", "Sleeps 4", "Queen bed + dinette"),
                ("ruler", "32 ft length", "Bumper-to-bumper, 1 slide-out"),
                ("move-horizontal", "1 slide-out", "Extra living space"),
                ("shield-check", "Pet-friendly", "Bring the dog — pets welcome"),
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
                "A comfortable, easy-to-tow 5th wheel that's perfect for family getaways across the Okanagan. Sleeps 4 with a private queen bedroom and a convertible dinette, a full kitchen and bathroom with shower, a 43\" TV, solar panels on the roof, powered awning, furnace and A/C. Fully equipped and meticulously maintained — pick it up, or let us deliver and set it up at your campsite (up to 250 km)."
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
                    ("bed-double", "Sleeps 4"),
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
                        title: "Delivery up to 250 km",
                        desc: "CA$150 within 50 km, then CA$1.75/km two-way — calculated automatically at checkout.",
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
    let mut guests = use_signal(|| 2_i32);
    let mut delivery = use_signal(|| true);
    let mut busy = use_signal(|| false);
    let mut error = use_signal(String::new);
    let nav = use_navigator();
    let reserve = move |_| {
        let draft = api::TripDraft {
            rental_slug: listing.slug.to_string(),
            starts_on: starts_on.read().clone(),
            ends_on: ends_on.read().clone(),
            guests: *guests.read(),
            addon_keys: Vec::new(),
            delivery_km: (*delivery.read()).then(|| "25".to_string()),
        };
        async move {
            if draft.starts_on.is_empty() || draft.ends_on.is_empty() {
                error.set("Choose pickup and return dates first.".into());
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
                    div { class: "rvd-field",
                        div { class: "rvd-field-l", "PICKUP · 2:00 PM" }
                        input { class: "rvd-field-v", r#type: "date", value: "{starts_on}", oninput: move |e| starts_on.set(e.value()) }
                    }
                    div { class: "rvd-field-vd" }
                    div { class: "rvd-field",
                        div { class: "rvd-field-l", "RETURN · 11:00 AM" }
                        input { class: "rvd-field-v", r#type: "date", value: "{ends_on}", oninput: move |e| ends_on.set(e.value()) }
                    }
                }
                div { class: "rvd-field-hd" }
                div { class: "rvd-field-guests",
                    div {
                        div { class: "rvd-field-l", "GUESTS" }
                        select { class: "rvd-field-v", value: "{guests}", onchange: move |e| guests.set(e.value().parse().unwrap_or(1)),
                            for count in 1..=listing.meta.split("Sleeps ").nth(1).and_then(|v| v.parse::<i32>().ok()).unwrap_or(4) {
                                option { value: "{count}", "{count} guests" }
                            }
                        }
                    }
                }
            }
            label { class: "rvd-min-pill",
                input { r#type: "checkbox", checked: *delivery.read(), onchange: move |e| delivery.set(e.checked()) }
                span { "Delivery and setup (test distance: 25 km)" }
            }
            if !error.read().is_empty() { p { class: "auth-error", role: "alert", "{error}" } }
            button { class: "rvd-reserve", disabled: *busy.read(), onclick: reserve,
                if *busy.read() { "Checking availability…" } else { "Reserve" }
            }
            div { class: "rvd-note", "Test booking · no card charged · 3-night minimum" }
            div { class: "rvd-breakdown",
                div { class: "rvd-bd-row",
                    span { class: "rvd-bd-l", "{listing.price} × 3 nights" }
                    span { class: "rvd-bd-v", "$555" }
                }
                div { class: "rvd-bd-row",
                    span { class: "rvd-bd-l", "Delivery & setup" }
                    span { class: "rvd-bd-v", "$150" }
                }
                div { class: "rvd-bd-row",
                    span { class: "rvd-bd-l", "GST (5%)" }
                    span { class: "rvd-bd-v strong", "CA$35.25" }
                }
                div { class: "rvd-bd-row",
                    span { class: "rvd-bd-l", "PST (7%)" }
                    span { class: "rvd-bd-v strong", "CA$49.35" }
                }
                div { class: "rvd-bd-row",
                    div {
                        div { class: "rvd-bd-l", "Damage deposit" }
                        div { class: "rvd-bd-s", "Refundable · collected before trip" }
                    }
                    span { class: "rvd-bd-v strong", "CA$1,000.00" }
                }
                div { class: "rvd-divider" }
                div { class: "rvd-total",
                    span { class: "rvd-total-l", "Total" }
                    span { class: "rvd-total-v", "CA$789.60" }
                }
            }
            div { class: "rvd-contact",
                Icon { name: "phone", size: 15, color: "var(--vl-forest)" }
                span { "Questions? {PHONE}" }
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
}

fn selected_date(value: &Signal<String>) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(&value.read(), "%Y-%m-%d").ok()
}

#[component]
fn Availability(
    slug: &'static str,
    price: &'static str,
    mut starts_on: Signal<String>,
    mut ends_on: Signal<String>,
    availability_version: Signal<u32>,
) -> Element {
    let initial_month = month_start(Utc::now().with_timezone(&Vancouver).date_naive());
    let mut visible_month = use_signal(|| initial_month);
    let availability = use_resource(move || {
        let _version = *availability_version.read();
        async move {
            api::availability(slug, &initial_month.to_string(), &add_months(initial_month, 18).to_string()).await
        }
    });
    let response = availability.read().as_ref().and_then(|result| result.as_ref().ok()).cloned();
    let availability_loaded = response.is_some();
    let unavailable = response.as_ref().map(unavailable_ranges).unwrap_or_default();
    let minimum_nights = response.as_ref().map(|value| value.minimum_nights).unwrap_or(3);
    let start = selected_date(&starts_on);
    let end = selected_date(&ends_on);
    let nights = start.zip(end).map(|(a, b)| (b - a).num_days()).unwrap_or(0);
    let calendar_error = availability.read().as_ref().and_then(|result| result.as_ref().err()).cloned();

    rsx! {
        div { class: "rvd-avail",
            div { class: "rvd-avail-head",
                div {
                    h2 { class: "rvd-avail-t", "Availability & instant quote" }
                    div { class: "rvd-avail-s", "Choose pickup at 2:00 PM and return at 11:00 AM · 3-night minimum." }
                }
                div { class: "rvd-legend",
                    for (bg , label) in [
                        ("background-color: var(--vl-forest);", "Pickup / return"),
                        ("background-color: var(--vl-mint);", "Your stay"),
                        ("background-color: var(--vl-white); border: 1px solid var(--vl-hair);", "Available"),
                        ("background-color: var(--vl-hair);", "Booked"),
                    ] {
                        div { key: "lg-{label}", class: "rvd-legend-item",
                            div { class: "rvd-legend-dot", style: bg }
                            span { "{label}" }
                        }
                    }
                }
            }
            if let Some(message) = calendar_error {
                p { class: "auth-error", role: "alert", "Could not load availability: {message}" }
            }
            div { class: "rvd-cal",
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
                div { class: "rvd-quote",
                    div { class: "rvd-quote-l",
                        div { class: "rvd-quote-ic", Icon { name: "zap", size: 18, color: "var(--vl-white)" } }
                        div {
                            div { class: "rvd-quote-t",
                                if nights >= 3 { "{starts_on} → {ends_on} · {nights} nights" } else { "Select at least 3 nights" }
                            }
                            div { class: "rvd-quote-s", "Live availability · final taxes calculated at checkout" }
                        }
                    }
                    if nights >= 3 {
                        div { class: "rvd-quote-r", span { class: "rvd-quote-m", "{price} × {nights}" } }
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
                            let selecting_return = start.is_some() && end.is_none();
                            let valid_choice = if selecting_return {
                                let selected_start = start.unwrap();
                                day > selected_start
                                    && (day - selected_start).num_days() >= minimum_nights
                                    && stay_is_available(selected_start, day, &unavailable)
                            } else {
                                minimum_stay_can_start(day, minimum_nights, &unavailable)
                            };
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
                                    if current_start.is_none() || current_end.is_some() || day <= current_start.unwrap() {
                                        starts_on.set(day.to_string());
                                        ends_on.set(String::new());
                                    } else {
                                        ends_on.set(day.to_string());
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

// ===== Add-ons =====

#[component]
fn AddOns() -> Element {
    rsx! {
        div { class: "rvd-addons",
            h2 { class: "rvd-avail-t", "Add-ons" }
            div { class: "rvd-addons-grid",
                div { class: "rvd-addons-col",
                    AddOnRow { icon: "bed-double", title: "Bedding and Linens", sub: "", price: "$80" }
                    AddOnRow { icon: "flame", title: "Portable BBQ", sub: "", price: "$50 + Refill" }
                    AddOnRow {
                        icon: "paw-print",
                        title: "Pet Deposit",
                        sub: "Non refundable",
                        price: "$100",
                    }
                    AddOnRow {
                        icon: "fuel",
                        title: "Propane Refill Prepayment",
                        sub: "",
                        price: "$40",
                    }
                }
                div { class: "rvd-addons-col",
                    AddOnRow {
                        icon: "droplet",
                        title: "Emptying Septic Prepayment",
                        sub: "",
                        price: "$40",
                    }
                    AddOnRow {
                        icon: "zap",
                        title: "Portable Generator",
                        sub: "Without fuel",
                        price: "$50 + Refill",
                    }
                    AddOnRow { icon: "spray-can", title: "Excessive Dirt", sub: "", price: "$200" }
                    AddOnRow { icon: "sparkles", title: "Exterior Cleaning", sub: "", price: "$80" }
                }
            }
        }
    }
}

#[component]
fn AddOnRow(
    icon: &'static str,
    title: &'static str,
    sub: &'static str,
    price: &'static str,
) -> Element {
    rsx! {
        div { class: "rvd-addon",
            div { class: "rvd-addon-l",
                div { class: "rvd-addon-ib",
                    Icon { name: icon, size: 18, color: "var(--vl-forest)" }
                }
                div {
                    div { class: "rvd-addon-t", "{title}" }
                    if !sub.is_empty() {
                        div { class: "rvd-addon-s", "{sub}" }
                    }
                }
            }
            div { class: "rvd-addon-p", "{price}" }
        }
    }
}

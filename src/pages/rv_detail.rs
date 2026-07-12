//! Страница RV Detail / Booking — Pencil-фреймы `l3JikE` (desktop) и `wjWTt` (mobile).

use dioxus::prelude::*;

use crate::{api, components::Icon};
use crate::data::{
    rv_listings, Listing, IMG_BULLET, IMG_OPENRANGE, IMG_OUTBACK, IMG_ROCKWOOD, PHONE,
};

const CSS: Asset = asset!("/assets/css/rv_detail.css");
const IMG_HOST: Asset = asset!("/assets/img/host-viktor.webp");

#[component]
pub fn RvDetail(slug: String) -> Element {
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
                BookingCard { listing: l }
            }
            Availability { price: l.price }
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
fn BookingCard(listing: Listing) -> Element {
    let mut starts_on = use_signal(|| "2026-08-12".to_string());
    let mut ends_on = use_signal(|| "2026-08-15".to_string());
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
                Err(message) => error.set(message),
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
                        div { class: "rvd-field-l", "CHECK-IN" }
                        input { class: "rvd-field-v", r#type: "date", value: "{starts_on}", oninput: move |e| starts_on.set(e.value()) }
                    }
                    div { class: "rvd-field-vd" }
                    div { class: "rvd-field",
                        div { class: "rvd-field-l", "CHECK-OUT" }
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
            div { class: "rvd-note", "Full payment due 5 days before your rental" }
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

// ===== Календарь доступности =====

/// Состояние дня в календаре.
#[derive(Clone, Copy, PartialEq)]
enum Day {
    Blank,
    Avail(u8),
    Booked(u8),
    /// Check-in / check-out — тёмная плашка.
    Edge(u8),
    /// Ночи внутри брони — мятная заливка.
    Stay(u8),
}

fn push(v: &mut Vec<Day>, from: u8, to: u8, f: fn(u8) -> Day) {
    for d in from..=to {
        v.push(f(d));
    }
}

fn july_cells() -> Vec<Day> {
    let mut v = vec![Day::Blank; 3];
    push(&mut v, 1, 3, Day::Booked);
    push(&mut v, 4, 7, Day::Avail);
    push(&mut v, 8, 9, Day::Booked);
    push(&mut v, 10, 11, Day::Avail);
    v.push(Day::Edge(12));
    push(&mut v, 13, 14, Day::Stay);
    v.push(Day::Edge(15));
    push(&mut v, 16, 18, Day::Avail);
    push(&mut v, 19, 20, Day::Booked);
    push(&mut v, 21, 25, Day::Avail);
    push(&mut v, 26, 27, Day::Booked);
    push(&mut v, 28, 30, Day::Avail);
    v.push(Day::Booked(31));
    v.push(Day::Blank);
    v
}

fn august_cells() -> Vec<Day> {
    let mut v = vec![Day::Blank; 6];
    push(&mut v, 1, 2, Day::Booked);
    push(&mut v, 3, 6, Day::Avail);
    push(&mut v, 7, 9, Day::Booked);
    push(&mut v, 10, 14, Day::Avail);
    push(&mut v, 15, 16, Day::Booked);
    push(&mut v, 17, 21, Day::Avail);
    push(&mut v, 22, 23, Day::Booked);
    push(&mut v, 24, 28, Day::Avail);
    push(&mut v, 29, 30, Day::Booked);
    v.push(Day::Avail(31));
    v.extend([Day::Blank; 5]);
    v
}

fn september_cells() -> Vec<Day> {
    let mut v = vec![Day::Blank; 2];
    push(&mut v, 1, 9, Day::Avail);
    push(&mut v, 10, 12, Day::Booked);
    push(&mut v, 13, 17, Day::Avail);
    push(&mut v, 18, 19, Day::Booked);
    push(&mut v, 20, 24, Day::Avail);
    push(&mut v, 25, 30, Day::Booked);
    v.extend([Day::Blank; 3]);
    v
}

#[component]
fn Availability(price: &'static str) -> Element {
    rsx! {
        div { class: "rvd-avail",
            div { class: "rvd-avail-head",
                div {
                    h2 { class: "rvd-avail-t", "Availability & instant quote" }
                    div { class: "rvd-avail-s",
                        "Select your check-in and check-out dates to get an instant quote for your stay."
                    }
                }
                div { class: "rvd-legend",
                    for (bg , label) in [
                        ("background-color: var(--vl-forest);", "Check-in / out"),
                        ("background-color: var(--vl-mint);", "Your stay"),
                        (
                            "background-color: var(--vl-white); border: 1px solid var(--vl-hair);",
                            "Available",
                        ),
                        ("background-color: var(--vl-hair);", "Booked"),
                    ]
                    {
                        div { key: "lg-{label}", class: "rvd-legend-item",
                            div { class: "rvd-legend-dot", style: bg }
                            span { "{label}" }
                        }
                    }
                }
            }
            div { class: "rvd-cal",
                div { class: "rvd-cal-months",
                    CalendarMonth {
                        title: "July 2026",
                        nav_prev: true,
                        nav_next: false,
                        price,
                        cells: july_cells(),
                    }
                    CalendarMonth {
                        title: "August 2026",
                        nav_prev: false,
                        nav_next: false,
                        price,
                        cells: august_cells(),
                    }
                    CalendarMonth {
                        title: "September 2026",
                        nav_prev: false,
                        nav_next: true,
                        price,
                        cells: september_cells(),
                    }
                }
                div { class: "rvd-quote",
                    div { class: "rvd-quote-l",
                        div { class: "rvd-quote-ic",
                            Icon { name: "zap", size: 18, color: "var(--vl-white)" }
                        }
                        div {
                            div { class: "rvd-quote-t", "Jul 12 → Jul 15 · 3 nights" }
                            div { class: "rvd-quote-s", "Instant quote · taxes added at checkout" }
                        }
                    }
                    div { class: "rvd-quote-r",
                        span { class: "rvd-quote-m", "{price} × 3 =" }
                        span { class: "rvd-quote-p", "$555" }
                    }
                }
            }
        }
    }
}

#[component]
fn CalendarMonth(
    title: &'static str,
    nav_prev: bool,
    nav_next: bool,
    price: &'static str,
    cells: Vec<Day>,
) -> Element {
    // Плоские дисплей-данные ячеек: (класс, номер, цена).
    let display: Vec<(String, String, &'static str)> = cells
        .iter()
        .map(|d| match *d {
            Day::Blank => (String::new(), String::new(), ""),
            Day::Avail(n) => (String::new(), n.to_string(), price),
            Day::Booked(n) => ("booked".to_string(), n.to_string(), ""),
            Day::Edge(n) => ("edge".to_string(), n.to_string(), price),
            Day::Stay(n) => ("stay".to_string(), n.to_string(), price),
        })
        .collect();

    rsx! {
        div { class: "rvd-month",
            div { class: "rvd-month-head",
                if nav_prev {
                    button { class: "rvd-month-nav",
                        Icon { name: "chevron-left", size: 16, color: "var(--vl-ink)" }
                    }
                } else {
                    div { class: "rvd-month-sp" }
                }
                div { class: "rvd-month-t", "{title}" }
                if nav_next {
                    button { class: "rvd-month-nav",
                        Icon { name: "chevron-right", size: 16, color: "var(--vl-ink)" }
                    }
                } else {
                    div { class: "rvd-month-sp" }
                }
            }
            div { class: "rvd-wd",
                for (i , w) in ["S", "M", "T", "W", "T", "F", "S"].iter().enumerate() {
                    div { key: "w-{i}", "{w}" }
                }
            }
            div { class: "rvd-days",
                for (i , (class , num , p)) in display.into_iter().enumerate() {
                    div { key: "c-{i}", class: "rvd-day {class}",
                        if !num.is_empty() {
                            div { class: "rvd-day-n", "{num}" }
                        }
                        if !p.is_empty() {
                            div { class: "rvd-day-p", "{p}" }
                        }
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

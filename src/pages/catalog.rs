use chrono::{Datelike, Months, NaiveDate, Utc};
use chrono_tz::America::Vancouver;
use dioxus::prelude::*;

use crate::data::{
    IMG_BULLET, IMG_JAYCO, IMG_OPENRANGE, IMG_OPENRANGE2, IMG_OUTBACK, IMG_ROCKWOOD,
};
use crate::{api, components::Icon, Route};

#[component]
pub fn Catalog() -> Element {
    let listings = use_resource(api::catalog);
    let saved_search = api::load_json::<api::CatalogSearchDraft>("vl_catalog_search");
    let initial_location = saved_search
        .as_ref()
        .map(|value| value.location.clone())
        .unwrap_or_else(|| "Kelowna, BC".to_string());
    let initial_radius = saved_search
        .as_ref()
        .map(|value| value.radius_km.clamp(10, 150))
        .unwrap_or(50);
    let initial_start = saved_search
        .as_ref()
        .and_then(|value| value.starts_on.as_deref())
        .and_then(|value| NaiveDate::parse_from_str(value, "%Y-%m-%d").ok());
    let initial_end = saved_search
        .as_ref()
        .and_then(|value| value.ends_on.as_deref())
        .and_then(|value| NaiveDate::parse_from_str(value, "%Y-%m-%d").ok());
    let initial_guests = saved_search
        .as_ref()
        .map(|value| value.guests.clamp(1, 10))
        .unwrap_or(2);
    let mut search_open = use_signal(|| false);
    let search_location = use_signal(|| initial_location);
    let search_radius = use_signal(|| initial_radius);
    let search_starts_on = use_signal(|| initial_start);
    let search_ends_on = use_signal(|| initial_end);
    let search_guests = use_signal(|| initial_guests);

    let location_label = format!("{} · {} km", search_location.read(), search_radius.read());
    let dates_label = catalog_date_label(*search_starts_on.read(), *search_ends_on.read());
    let guests_label = if *search_guests.read() == 1 {
        "1 guest".to_string()
    } else {
        format!("{} guests", search_guests.read())
    };
    let match_count = listings
        .read()
        .as_ref()
        .and_then(|result| result.as_ref().ok())
        .map(|values| {
            values
                .iter()
                .filter(|value| value.category == "rv" && value.capacity >= *search_guests.read())
                .count()
        })
        .unwrap_or(0);
    rsx! {
        section { class: "cat-header",
            h1 { class: "sec-title", "Explore the fleet" }
            p { class: "cat-sub", "RVs and trailers ready for your Okanagan adventure — book in minutes." }
            div { class: "cat-search",
                button { class: "cat-search-field", r#type: "button", onclick: move |_| search_open.set(true),
                    div { class: "cat-search-label", "LOCATION" }
                    div { class: "cat-search-value",
                        Icon { name: "map-pin", size: 15, color: "var(--vl-forest)" }
                        span { "{location_label}" }
                    }
                }
                div { class: "cat-search-divider" }
                button { class: "cat-search-field", r#type: "button", onclick: move |_| search_open.set(true),
                    div { class: "cat-search-label", "DATES" }
                    div { class: "cat-search-value",
                        Icon { name: "calendar", size: 15, color: "var(--vl-forest)" }
                        span { "{dates_label}" }
                    }
                }
                div { class: "cat-search-divider" }
                button { class: "cat-search-field", r#type: "button", onclick: move |_| search_open.set(true),
                    div { class: "cat-search-label", "GUESTS" }
                    div { class: "cat-search-value",
                        Icon { name: "users", size: 15, color: "var(--vl-forest)" }
                        span { "{guests_label}" }
                    }
                }
                button { class: "cat-search-btn", r#type: "button", onclick: move |_| search_open.set(true),
                    Icon { name: "search", size: 17, color: "var(--vl-white)" }
                    span { "Search" }
                }
            }
        }
        section { class: "cat-body",
            Filters {}
            div { class: "cat-results",
                div { class: "cat-results-head",
                    div { class: "cat-count", "{match_count} stays that fit your group" }
                    button { class: "cat-sort",
                        Icon { name: "arrow-up-down", size: 14, color: "var(--vl-ink)" }
                        span { "Sort: Recommended" }
                        Icon { name: "chevron-down", size: 14, color: "var(--vl-ink)" }
                    }
                }
                div { class: "cat-grid",
                    if let Some(result) = listings.read().as_ref() {
                        match result {
                            Ok(values) => rsx! { for rental in values.iter().filter(|value| value.category == "rv" && value.capacity >= *search_guests.read()) {
                                ApiListingCard { key: "{rental.slug}", rental: rental.clone() }
                            } },
                            Err(message) => rsx! { div { class: "co-card", role: "alert", "Could not load rentals: {message}" } },
                        }
                    } else {
                        div { class: "co-card", "Loading rentals…" }
                    }
                }
            }
        }
        if *search_open.read() {
            CatalogSearchOverlay {
                location: search_location,
                radius: search_radius,
                starts_on: search_starts_on,
                ends_on: search_ends_on,
                guests: search_guests,
                on_close: move |_| search_open.set(false),
            }
        }
    }
}

fn month_start(date: NaiveDate) -> NaiveDate {
    date.with_day(1).unwrap_or(date)
}

fn add_months(date: NaiveDate, count: u32) -> NaiveDate {
    date.checked_add_months(Months::new(count)).unwrap_or(date)
}

fn catalog_date_label(start: Option<NaiveDate>, end: Option<NaiveDate>) -> String {
    match (start, end) {
        (Some(start), Some(end)) if start.month() == end.month() => {
            format!("{} {} – {}", start.format("%b"), start.day(), end.day())
        }
        (Some(start), Some(end)) => {
            format!("{} – {}", start.format("%b %-d"), end.format("%b %-d"))
        }
        (Some(start), None) => format!("{} · choose return", start.format("%b %-d")),
        _ => "Add dates".to_string(),
    }
}

fn calendar_cells(month: NaiveDate) -> Vec<Option<NaiveDate>> {
    let leading = month.weekday().num_days_from_sunday() as usize;
    let next_month = add_months(month, 1);
    let days = (next_month - month).num_days() as usize;
    let mut cells = vec![None; leading];
    cells.extend((0..days).map(|offset| month.checked_add_days(chrono::Days::new(offset as u64))));
    while cells.len() % 7 != 0 {
        cells.push(None);
    }
    cells
}

fn next_catalog_date_selection(
    day: NaiveDate,
    start: Option<NaiveDate>,
    end: Option<NaiveDate>,
) -> (Option<NaiveDate>, Option<NaiveDate>) {
    match (start, end) {
        (Some(first), None) if day == first => (None, None),
        (Some(first), None) if day > first && (day - first).num_days() >= 3 => {
            (Some(first), Some(day))
        }
        (Some(first), None) if day < first => (Some(day), None),
        _ => (Some(day), None),
    }
}

#[component]
fn CatalogSearchOverlay(
    mut location: Signal<String>,
    mut radius: Signal<i32>,
    mut starts_on: Signal<Option<NaiveDate>>,
    mut ends_on: Signal<Option<NaiveDate>>,
    mut guests: Signal<i32>,
    on_close: EventHandler<()>,
) -> Element {
    let today = Utc::now().with_timezone(&Vancouver).date_naive();
    let initial_month = month_start(today);
    let mut visible_month = use_signal(|| {
        (*starts_on.read())
            .map(month_start)
            .unwrap_or(initial_month)
    });
    let mut closing = use_signal(|| false);
    let nights = starts_on
        .read()
        .zip(*ends_on.read())
        .map(|(start, end)| (end - start).num_days())
        .unwrap_or(0);
    let radius_size = 34.0 + (*radius.read() as f64 / 150.0 * 50.0);
    let guest_word = if *guests.read() == 1 {
        "guest"
    } else {
        "guests"
    };
    let close_overlay = move || async move {
        if *closing.peek() {
            return;
        }
        closing.set(true);
        let _ = document::eval("await new Promise(resolve => setTimeout(resolve, 200));").await;
        on_close.call(());
    };

    rsx! {
        div {
            class: if *closing.read() { "cat-planner-backdrop is-closing" } else { "cat-planner-backdrop" },
            role: "presentation",
            onclick: move |_| close_overlay(),
            div { class: "cat-planner", role: "dialog", aria_modal: "true", aria_label: "Plan your RV search", onclick: move |event| event.stop_propagation(),
                div { class: "cat-planner-head",
                    div {
                        div { class: "cat-planner-kicker", "OKANAGAN RV SEARCH" }
                        h2 { "Find the right RV for your trip" }
                        p { "Set your search area, travel dates and group size in one place." }
                    }
                    button { class: "cat-planner-close", r#type: "button", aria_label: "Close search", onclick: move |_| close_overlay(),
                        Icon { name: "x", size: 22, color: "var(--vl-ink)" }
                    }
                }

                div { class: "cat-planner-grid",
                    section { class: "cat-planner-location",
                        div { class: "cat-planner-section-head",
                            div { class: "cat-planner-section-icon", Icon { name: "map-pin", size: 18, color: "var(--vl-white)" } }
                            div { h3 { "Search area" } p { "Choose how far from Kelowna you want to explore." } }
                        }
                        label { class: "cat-location-input",
                            span { "SEARCH CENTRE" }
                            div { Icon { name: "map-pin", size: 17, color: "var(--vl-forest)" }
                                input { value: "{location}", oninput: move |event| location.set(event.value()), aria_label: "Search centre" }
                            }
                        }
                        div { class: "cat-radius-map",
                            div { class: "cat-map-road road-one" }
                            div { class: "cat-map-road road-two" }
                            span { class: "cat-map-label label-vernon", "Vernon" }
                            span { class: "cat-map-label label-west", "West Kelowna" }
                            span { class: "cat-map-label label-penticton", "Penticton" }
                            div { class: "cat-radius-ring", style: "width: {radius_size}%; height: {radius_size}%;" }
                            div { class: "cat-map-centre",
                                div { class: "cat-map-pin", Icon { name: "map-pin", size: 18, color: "var(--vl-white)" } }
                                strong { "{location}" }
                            }
                            div { class: "cat-map-radius-badge", strong { "{radius} km" } span { "maximum search radius" } }
                        }
                        div { class: "cat-radius-control",
                            div { span { "Search radius" } strong { "{radius} km" } }
                            input { r#type: "range", min: "10", max: "150", step: "5", value: "{radius}", aria_label: "Search radius in kilometres", oninput: move |event| {
                                if let Ok(value) = event.value().parse::<i32>() { radius.set(value.clamp(10, 150)); }
                            } }
                            div { class: "cat-radius-presets",
                                for value in [25, 50, 75, 100, 150] {
                                    button { key: "radius-{value}", r#type: "button", class: if *radius.read() == value { "active" } else { "" }, onclick: move |_| radius.set(value), "{value} km" }
                                }
                            }
                        }
                        div { class: "cat-radius-note", Icon { name: "info", size: 15, color: "var(--vl-forest)" } span { "Catalog search is limited to 150 km around Kelowna." } }
                    }

                    section { class: "cat-planner-trip",
                        div { class: "cat-planner-section-head",
                            div { class: "cat-planner-section-icon", Icon { name: "calendar", size: 18, color: "var(--vl-white)" } }
                            div { h3 { "Travel dates" } p { "All RV stays require at least 3 nights." } }
                        }
                        div { class: "cat-trip-summary",
                            div { span { "PICKUP · 2:00 PM" } strong { if let Some(date) = *starts_on.read() { "{date}" } else { "Choose date" } } }
                            Icon { name: "arrow-right", size: 18, color: "var(--vl-muted)" }
                            div { span { "RETURN · 11:00 AM" } strong { if let Some(date) = *ends_on.read() { "{date}" } else { "Choose date" } } }
                        }
                        div { class: "cat-month-nav",
                            button { r#type: "button", aria_label: "Previous month", disabled: *visible_month.read() <= initial_month, onclick: move |_| {
                                let current = *visible_month.peek();
                                if let Some(previous) = current.checked_sub_months(Months::new(1)) {
                                    if previous >= initial_month { visible_month.set(previous); }
                                }
                            }, Icon { name: "chevron-left", size: 18, color: "var(--vl-ink)" } }
                            span { "Choose pickup and return" }
                            button { r#type: "button", aria_label: "Next month", disabled: *visible_month.read() >= add_months(initial_month, 15), onclick: move |_| {
                                let current = *visible_month.peek();
                                visible_month.set(add_months(current, 1));
                            },
                                Icon { name: "chevron-right", size: 18, color: "var(--vl-ink)" }
                            }
                        }
                        div { class: "cat-calendar-months",
                            for offset in 0..2_u32 {
                                CatalogSearchMonth { month: add_months(*visible_month.read(), offset), today, starts_on, ends_on }
                            }
                        }
                        div { class: "cat-guest-row",
                            div { class: "cat-planner-section-head compact",
                                div { class: "cat-planner-section-icon", Icon { name: "users", size: 18, color: "var(--vl-white)" } }
                                div { h3 { "Guests" } p { "We'll only show RVs with enough sleeping space." } }
                            }
                            div { class: "cat-guest-stepper",
                                button { r#type: "button", disabled: *guests.read() <= 1, onclick: move |_| {
                                    let current = *guests.peek();
                                    guests.set((current - 1).max(1));
                                }, "−" }
                                div { strong { "{guests}" } span { if *guests.read() == 1 { "guest" } else { "guests" } } }
                                button { r#type: "button", disabled: *guests.read() >= 10, onclick: move |_| {
                                    let current = *guests.peek();
                                    guests.set((current + 1).min(10));
                                }, "+" }
                            }
                        }
                    }
                }

                div { class: "cat-planner-foot",
                    div { class: "cat-planner-selection",
                        Icon { name: "sparkles", size: 20, color: "var(--vl-accent)" }
                        div {
                            strong { if nights >= 3 { "{nights}-night Okanagan trip" } else { "Choose at least 3 nights" } }
                            span { "{location} · within {radius} km · {guests} {guest_word}" }
                        }
                    }
                    button { class: "cat-planner-apply", r#type: "button", onclick: move |_| {
                        let draft = api::CatalogSearchDraft {
                            location: location.read().clone(),
                            radius_km: *radius.read(),
                            starts_on: (*starts_on.read()).map(|value| value.to_string()),
                            ends_on: (*ends_on.read()).map(|value| value.to_string()),
                            guests: *guests.read(),
                        };
                        let _ = api::save_json("vl_catalog_search", &draft);
                        close_overlay()
                    },
                        Icon { name: "search", size: 18, color: "var(--vl-white)" }
                        span { "Show matching RVs" }
                    }
                }
            }
        }
    }
}

#[component]
fn CatalogSearchMonth(
    month: NaiveDate,
    today: NaiveDate,
    mut starts_on: Signal<Option<NaiveDate>>,
    mut ends_on: Signal<Option<NaiveDate>>,
) -> Element {
    let start = *starts_on.read();
    let end = *ends_on.read();
    rsx! {
        div { class: "cat-calendar-month",
            h4 { "{month.format(\"%B %Y\")}" }
            div { class: "cat-calendar-weekdays", for day in ["S", "M", "T", "W", "T", "F", "S"] { span { "{day}" } } }
            div { class: "cat-calendar-days",
                for (index, cell) in calendar_cells(month).into_iter().enumerate() {
                    if let Some(day) = cell {
                        {
                            let is_start = start == Some(day);
                            let is_end = end == Some(day);
                            let in_range = start.zip(end).map(|(first, last)| day > first && day < last).unwrap_or(false);
                            let too_short = start.filter(|_| end.is_none()).map(|first| day > first && (day - first).num_days() < 3).unwrap_or(false);
                            let disabled = day < today || too_short;
                            let class = if is_start || is_end { "selected" } else if in_range { "in-range" } else { "" };
                            rsx! { button { key: "day-{index}", r#type: "button", class, disabled, aria_label: "{day}", onclick: move |_| {
                                let current_start = *starts_on.read();
                                let current_end = *ends_on.read();
                                let (next_start, next_end) = next_catalog_date_selection(day, current_start, current_end);
                                starts_on.set(next_start);
                                ends_on.set(next_end);
                            }, span { "{day.day()}" } } }
                        }
                    } else {
                        span { key: "blank-{index}", class: "blank" }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod catalog_search_tests {
    use super::*;

    fn day(value: &str) -> NaiveDate {
        NaiveDate::parse_from_str(value, "%Y-%m-%d").unwrap()
    }

    #[test]
    fn earlier_day_restarts_catalog_range() {
        let selected = day("2030-08-10");
        assert_eq!(
            next_catalog_date_selection(day("2030-08-08"), Some(selected), None),
            (Some(day("2030-08-08")), None)
        );
    }

    #[test]
    fn valid_three_night_range_is_selected() {
        let selected = day("2030-08-10");
        assert_eq!(
            next_catalog_date_selection(day("2030-08-13"), Some(selected), None),
            (Some(selected), Some(day("2030-08-13")))
        );
    }
}

#[component]
fn ApiListingCard(rental: api::Rental) -> Element {
    let image = match rental.slug.as_str() {
        "jayco26" => IMG_JAYCO.to_string(),
        "2015-keystone-bullet" => IMG_BULLET.to_string(),
        "2014-forest-river-rockwood" => IMG_ROCKWOOD.to_string(),
        "2025-open-range-1" => IMG_OPENRANGE.to_string(),
        "2025-highland-ridge-2" => IMG_OPENRANGE2.to_string(),
        "2017-keystone-outback-ultra" => IMG_OUTBACK.to_string(),
        _ => rental.hero_image_url.clone().unwrap_or_default(),
    };
    rsx! {
        Link { class: "listing-card", to: Route::RvDetail { slug: rental.slug.clone() },
            div { class: "lc-image", style: "background-image: url('{image}');",
                div { class: "lc-badge", "Sleeps {rental.capacity}" }
            }
            div { class: "lc-body",
                div { class: "lc-title-row", div { class: "lc-title", "{rental.name}" } }
                div { class: "lc-meta", "{rental.summary}" }
                div { class: "lc-price-row", span { class: "lc-price", "${rental.base_rate}" } span { class: "lc-per", " / {rental.price_unit}" } }
            }
        }
    }
}

#[component]
fn Filters() -> Element {
    rsx! {
        aside { class: "cat-filters",
            div { class: "filter-group",
                div { class: "filter-head", "Type" }
                FilterCheck { label: "RVs", checked: true }
                FilterCheck { label: "Cooler trailers", checked: false }
            }
            div { class: "filter-divider" }
            div { class: "filter-group", style: "gap: 14px;",
                div { class: "filter-head", "Price / night" }
                div { class: "price-track",
                    div { class: "price-fill" }
                    div { class: "price-handle", style: "left: 26px;" }
                    div { class: "price-handle", style: "left: 170px;" }
                }
                div { class: "price-labels",
                    span { "$125" }
                    span { "$185+" }
                }
            }
            div { class: "filter-divider" }
            div { class: "filter-group",
                div { class: "filter-head", "Sleeps / capacity" }
                div { class: "sleep-chips",
                    for (label, active) in [("2", false), ("4", true), ("6", false), ("8", true), ("10+", false)] {
                        button {
                            key: "sleeps-{label}",
                            class: "sleep-chip",
                            style: if active {
                                "background-color: var(--vl-forest); border-color: var(--vl-forest); color: var(--vl-white);"
                            } else {
                                "background-color: var(--vl-white); border-color: var(--vl-hair); color: var(--vl-muted);"
                            },
                            "{label}"
                        }
                    }
                }
            }
            div { class: "filter-divider" }
            div { class: "filter-group",
                div { class: "filter-head", "Features" }
                FilterCheck { label: "Delivery available", checked: true }
                FilterCheck { label: "Air conditioning", checked: false }
                FilterCheck { label: "Pet friendly", checked: false }
            }
        }
    }
}

#[component]
fn FilterCheck(label: &'static str, checked: bool) -> Element {
    rsx! {
        div { class: "filter-check",
            div {
                class: "filter-box",
                style: if checked {
                    "background-color: var(--vl-forest); border-color: var(--vl-forest);"
                } else {
                    "background-color: var(--vl-white); border-color: var(--vl-hair);"
                },
                if checked {
                    Icon { name: "check", size: 13, color: "var(--vl-white)" }
                }
            }
            span { "{label}" }
        }
    }
}

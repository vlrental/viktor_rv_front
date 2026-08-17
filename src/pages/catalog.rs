use chrono::{Datelike, Months, NaiveDate, Utc};
use chrono_tz::America::Vancouver;
use dioxus::prelude::*;

use crate::data::{
    IMG_BULLET, IMG_JAYCO, IMG_OPENRANGE, IMG_OPENRANGE2, IMG_OUTBACK, IMG_ROCKWOOD,
};
use crate::{
    api,
    components::{Icon, SortDropdown},
    Route,
};

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CatalogFilters {
    pub(crate) travel_trailers: bool,
    pub(crate) fifth_wheels: bool,
    pub(crate) toy_haulers: bool,
    pub(crate) maximum_nightly_price: i32,
    pub(crate) minimum_capacity: i32,
    pub(crate) sort: String,
}

impl Default for CatalogFilters {
    fn default() -> Self {
        Self {
            travel_trailers: true,
            fifth_wheels: true,
            toy_haulers: true,
            maximum_nightly_price: 185,
            minimum_capacity: 0,
            sort: "recommended".into(),
        }
    }
}

fn rental_price(rental: &api::Rental) -> f64 {
    rental.base_rate.parse::<f64>().unwrap_or(f64::MAX)
}

fn rental_style(rental: &api::Rental) -> &'static str {
    match rental.rv_type.as_str() {
        "toy_hauler" => "toy-hauler",
        "fifth_wheel" => "fifth-wheel",
        _ => "travel-trailer",
    }
}

pub(crate) fn filtered_catalog(
    values: &[api::Rental],
    filters: &CatalogFilters,
) -> Vec<api::Rental> {
    filtered_catalog_for_guests(values, filters, None)
}

pub(crate) fn filtered_catalog_for_guests(
    values: &[api::Rental],
    filters: &CatalogFilters,
    guests: Option<i32>,
) -> Vec<api::Rental> {
    let mut rentals = values
        .iter()
        .filter(|rental| {
            let style_matches = match rental_style(rental) {
                "fifth-wheel" => filters.fifth_wheels,
                "toy-hauler" => filters.toy_haulers,
                _ => filters.travel_trailers,
            };
            style_matches
                && rental_price(rental) <= f64::from(filters.maximum_nightly_price)
                && rental.capacity >= filters.minimum_capacity
        })
        .cloned()
        .collect::<Vec<_>>();

    match filters.sort.as_str() {
        "date-fit" => {
            if let Some(guests) = guests {
                rentals.sort_by(|left, right| {
                    (left.capacity - guests)
                        .abs()
                        .cmp(&(right.capacity - guests).abs())
                        .then_with(|| rental_price(left).total_cmp(&rental_price(right)))
                });
            }
        }
        "price-low" => {
            rentals.sort_by(|left, right| rental_price(left).total_cmp(&rental_price(right)))
        }
        "price-high" => {
            rentals.sort_by(|left, right| rental_price(right).total_cmp(&rental_price(left)))
        }
        "capacity" => rentals.sort_by(|left, right| {
            right
                .capacity
                .cmp(&left.capacity)
                .then_with(|| rental_price(left).total_cmp(&rental_price(right)))
        }),
        _ => {}
    }
    rentals
}

#[component]
pub fn Catalog() -> Element {
    let navigator = use_navigator();
    use_effect(move || {
        navigator.replace(Route::Home {});
        spawn(async move {
            let _ = document::eval(
                "await new Promise(resolve => setTimeout(resolve, 180)); document.getElementById('home-rentals')?.scrollIntoView({ block: 'start' });",
            )
            .await;
        });
    });
    rsx! {
        div { class: "catalog-redirect", "Opening available RVs…" }
    }
}

#[component]
#[allow(dead_code)]
fn CatalogPage() -> Element {
    let today = Utc::now().with_timezone(&Vancouver).date_naive();
    let initial_search = normalized_catalog_search(
        api::load_json::<api::CatalogSearchDraft>("vl_catalog_search"),
        50,
        today,
    );
    let mut applied_search = use_signal(|| initial_search.clone());
    let search_version = use_signal(|| 0_u32);
    let mut search_open = use_signal(|| false);
    let mut search_location = use_signal(|| initial_search.location.clone());
    let mut search_radius = use_signal(|| initial_search.radius_km);
    let mut search_starts_on =
        use_signal(|| parse_search_date(initial_search.starts_on.as_deref()));
    let mut search_ends_on = use_signal(|| parse_search_date(initial_search.ends_on.as_deref()));
    let mut search_guests = use_signal(|| initial_search.guests);
    let mut filters = use_signal(CatalogFilters::default);
    let listings = use_resource(move || {
        let _version = *search_version.read();
        let search = applied_search.read().clone();
        async move { api::catalog(&search).await }
    });
    use_effect(move || {
        if *search_open.read() {
            let search = applied_search.read().clone();
            search_location.set(search.location);
            search_radius.set(search.radius_km);
            search_starts_on.set(parse_search_date(search.starts_on.as_deref()));
            search_ends_on.set(parse_search_date(search.ends_on.as_deref()));
            search_guests.set(search.guests);
        }
    });
    use_effect(move || {
        let search = applied_search.read().clone();
        let _ = api::save_json("vl_catalog_search", &search);
    });

    let applied = applied_search.read().clone();
    let location_label = format!("{} · {} km", applied.location, applied.radius_km);
    let applied_start = parse_search_date(applied.starts_on.as_deref());
    let applied_end = parse_search_date(applied.ends_on.as_deref());
    let dates_label = catalog_date_label(applied_start, applied_end);
    let guests_label = if applied.guests == 1 {
        "1 guest".to_string()
    } else {
        format!("{} guests", applied.guests)
    };
    let visible_rentals = listings
        .read()
        .as_ref()
        .and_then(|result| result.as_ref().ok())
        .map(|values| filtered_catalog(values, &filters.read()))
        .unwrap_or_default();
    let match_count = visible_rentals.len();
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
            Filters { filters }
            div { class: "cat-results",
                div { class: "cat-results-head",
                    div { class: "cat-count", "{match_count} stays that fit your group" }
                    SortDropdown {
                        value: filters.read().sort.clone(),
                        on_change: move |value| {
                            let mut next = filters.read().clone();
                            next.sort = value;
                            filters.set(next);
                        },
                    }
                }
                div { class: "cat-grid",
                    if let Some(result) = listings.read().as_ref() {
                        match result {
                            Ok(values) if values.is_empty() => rsx! {
                                CatalogEmptyState {
                                    dates_label: dates_label.clone(),
                                    has_dates: applied_start.is_some() && applied_end.is_some(),
                                    on_change: move |_| search_open.set(true),
                                    on_clear: move |_| {
                                        let mut next = applied_search.read().clone();
                                        next.starts_on = None;
                                        next.ends_on = None;
                                        let _ = api::save_json("vl_catalog_search", &next);
                                        applied_search.set(next);
                                        bump_search_version(search_version);
                                    },
                                }
                            },
                            Ok(_) if visible_rentals.is_empty() => rsx! {
                                CatalogFilteredEmpty { on_reset: move |_| filters.set(CatalogFilters::default()) }
                            },
                            Ok(_) => rsx! { for rental in visible_rentals.iter() {
                                ApiListingCard { key: "{rental.slug}", rental: rental.clone() }
                            } },
                            Err(message) => rsx! {
                                CatalogErrorState { message: message.clone(), on_retry: move |_| bump_search_version(search_version) }
                            },
                        }
                    } else {
                        CatalogLoadingState {}
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
                on_apply: move |_| {
                    let next = api::CatalogSearchDraft {
                        location: search_location.read().clone(),
                        radius_km: *search_radius.read(),
                        starts_on: (*search_starts_on.read()).map(|value| value.to_string()),
                        ends_on: (*search_ends_on.read()).map(|value| value.to_string()),
                        guests: *search_guests.read(),
                    };
                    applied_search.set(next);
                    bump_search_version(search_version);
                },
                on_close: move |_| search_open.set(false),
            }
        }
    }
}

fn parse_search_date(value: Option<&str>) -> Option<NaiveDate> {
    value.and_then(|value| NaiveDate::parse_from_str(value, "%Y-%m-%d").ok())
}

pub(crate) fn bump_search_version(mut version: Signal<u32>) {
    let next = *version.peek() + 1;
    version.set(next);
}

pub(crate) fn normalized_catalog_search(
    saved: Option<api::CatalogSearchDraft>,
    default_radius: i32,
    today: NaiveDate,
) -> api::CatalogSearchDraft {
    let mut search = saved.unwrap_or(api::CatalogSearchDraft {
        location: "Kelowna, BC".into(),
        radius_km: default_radius,
        starts_on: None,
        ends_on: None,
        guests: 2,
    });
    if search.location.trim().is_empty() {
        search.location = "Kelowna, BC".into();
    }
    search.radius_km = search.radius_km.clamp(10, 150);
    search.guests = search.guests.clamp(1, 10);
    let valid_dates = parse_search_date(search.starts_on.as_deref())
        .zip(parse_search_date(search.ends_on.as_deref()))
        .is_some_and(|(start, end)| start > today && (end - start).num_days() >= 1);
    if !valid_dates {
        search.starts_on = None;
        search.ends_on = None;
    }
    search
}

#[component]
pub(crate) fn CatalogLoadingState() -> Element {
    rsx! {
        for index in 0..3 {
            div { key: "catalog-skeleton-{index}", class: "listing-card catalog-skeleton", aria_hidden: "true",
                div { class: "catalog-skeleton-image" }
                div { class: "catalog-skeleton-line wide" }
                div { class: "catalog-skeleton-line" }
                div { class: "catalog-skeleton-line short" }
            }
        }
    }
}

#[component]
pub(crate) fn CatalogErrorState(message: String, on_retry: EventHandler<()>) -> Element {
    rsx! {
        div { class: "co-card catalog-result-message", role: "alert",
            h3 { "Availability could not be checked" }
            p { "No RV is being shown as available until the live calendar responds. {message}" }
            button { class: "btn-forest", r#type: "button", onclick: move |_| on_retry.call(()), "Retry" }
        }
    }
}

#[component]
pub(crate) fn CatalogEmptyState(
    dates_label: String,
    has_dates: bool,
    on_change: EventHandler<()>,
    on_clear: EventHandler<()>,
) -> Element {
    rsx! {
        div { class: "co-card catalog-result-message",
            h3 { if has_dates { "No RVs are available for these dates" } else { "No RV fits this group yet" } }
            p {
                if has_dates {
                    "Nothing is free for {dates_label}. Try another period or clear the dates."
                } else {
                    "Reduce the guest count to see matching options."
                }
            }
            div { class: "catalog-result-actions",
                button { class: "btn-forest", r#type: "button", onclick: move |_| on_change.call(()), "Change search" }
                if has_dates {
                    button { class: "catalog-clear-search", r#type: "button", onclick: move |_| on_clear.call(()), "Clear dates" }
                }
            }
        }
    }
}

pub(crate) fn month_start(date: NaiveDate) -> NaiveDate {
    date.with_day(1).unwrap_or(date)
}

pub(crate) fn add_months(date: NaiveDate, count: u32) -> NaiveDate {
    date.checked_add_months(Months::new(count)).unwrap_or(date)
}

pub(crate) fn catalog_date_label(start: Option<NaiveDate>, end: Option<NaiveDate>) -> String {
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
        (Some(first), None) if day > first => (Some(first), Some(day)),
        (Some(first), None) if day < first => (Some(day), None),
        _ => (Some(day), None),
    }
}

fn manual_date_text(value: Option<NaiveDate>) -> String {
    value
        .map(|date| date.format("%Y-%m-%d").to_string())
        .unwrap_or_default()
}

fn format_manual_date_input(value: &str) -> String {
    let digits = value
        .chars()
        .filter(|character| character.is_ascii_digit())
        .take(8)
        .collect::<String>();
    let mut formatted = String::with_capacity(10);
    for (index, character) in digits.chars().enumerate() {
        if index == 4 || index == 6 {
            formatted.push('-');
        }
        formatted.push(character);
    }
    formatted
}

fn parse_manual_date_input(value: &str) -> Option<NaiveDate> {
    (value.len() == 10)
        .then(|| NaiveDate::parse_from_str(value, "%Y-%m-%d").ok())
        .flatten()
}

const DELIVERY_MAP_SCRIPT: &str = r#"
await (async () => {
    const root = document.querySelector('.cat-radius-map');
    const container = document.getElementById('vl-delivery-map');
    if (!root || !container) return;

    const fallback = root.querySelector('.cat-map-fallback span');
    const ensureLeaflet = async () => {
        if (!document.getElementById('vl-leaflet-css')) {
            const style = document.createElement('link');
            style.id = 'vl-leaflet-css';
            style.rel = 'stylesheet';
            style.href = 'https://unpkg.com/leaflet@1.9.4/dist/leaflet.css';
            style.integrity = 'sha256-p4NxAoJBhIIN+hmNHrzRCf9tD/miZyoHS5obTRR9BMY=';
            style.crossOrigin = '';
            document.head.appendChild(style);
        }

        if (window.L) return;
        if (!window.__vlLeafletReady) {
            window.__vlLeafletReady = new Promise((resolve, reject) => {
                const script = document.createElement('script');
                script.id = 'vl-leaflet-script';
                script.src = 'https://unpkg.com/leaflet@1.9.4/dist/leaflet.js';
                script.integrity = 'sha256-20nQCchB9co0qIjJZRGuk2/Z9VM+kNiyxNV1lvTlZBo=';
                script.crossOrigin = '';
                script.onload = resolve;
                script.onerror = () => reject(new Error('Leaflet failed to load'));
                document.head.appendChild(script);
            });
        }
        await window.__vlLeafletReady;
    };

    try {
        await ensureLeaflet();
        if (!document.body.contains(container)) return;

        const base = [50.0150675, -119.3870978];
        let state = window.__vlDeliveryMapState;
        if (!state || state.container !== container) {
            if (state?.map) state.map.remove();

            const map = L.map(container, {
                zoomControl: false,
                attributionControl: true,
                scrollWheelZoom: false,
                doubleClickZoom: true,
                dragging: true,
            }).setView(base, 8);
            L.control.zoom({ position: 'topright' }).addTo(map);
            L.tileLayer('https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png', {
                maxZoom: 19,
                attribution: '&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a> contributors',
            }).addTo(map);

            const zone = L.circle(base, {
                radius: __RADIUS_KM__ * 1000,
                color: '#174D32',
                weight: 2,
                opacity: 0.92,
                fillColor: '#4A8D63',
                fillOpacity: 0.2,
            }).addTo(map);
            L.circleMarker(base, {
                radius: 8,
                color: '#FFFFFF',
                weight: 3,
                fillColor: '#174D32',
                fillOpacity: 1,
            })
                .addTo(map)
                .bindTooltip('VL Rental · Kelowna', {
                    permanent: true,
                    direction: 'top',
                    offset: [0, -10],
                    className: 'vl-base-tooltip',
                });

            state = { map, zone, container };
            window.__vlDeliveryMapState = state;
        }

        state.zone.setRadius(__RADIUS_KM__ * 1000);
        state.map.invalidateSize(false);
        state.map.fitBounds(state.zone.getBounds(), {
            padding: [22, 22],
            maxZoom: 10,
            animate: true,
            duration: 0.25,
        });
        root.classList.remove('is-error');
        root.classList.add('is-ready');
    } catch (error) {
        root.classList.remove('is-ready');
        root.classList.add('is-error');
        root.dataset.mapError = error instanceof Error ? error.message : String(error);
        if (fallback) fallback.textContent = 'Map preview is temporarily unavailable';
    }
})();
"#;

#[component]
pub(crate) fn CatalogSearchOverlay(
    mut location: Signal<String>,
    mut radius: Signal<i32>,
    mut starts_on: Signal<Option<NaiveDate>>,
    mut ends_on: Signal<Option<NaiveDate>>,
    mut guests: Signal<i32>,
    on_apply: EventHandler<()>,
    on_close: EventHandler<()>,
) -> Element {
    let today = Utc::now().with_timezone(&Vancouver).date_naive();
    let initial_month = month_start(today);
    let mut visible_month = use_signal(|| {
        (*starts_on.read())
            .map(month_start)
            .unwrap_or(initial_month)
    });
    let initial_start_input = manual_date_text(*starts_on.read());
    let initial_end_input = manual_date_text(*ends_on.read());
    let mut start_input = use_signal(|| initial_start_input);
    let mut end_input = use_signal(|| initial_end_input);
    let mut closing = use_signal(|| false);
    let nights = starts_on
        .read()
        .zip(*ends_on.read())
        .map(|(start, end)| (end - start).num_days())
        .unwrap_or(0);
    let can_apply = (starts_on.read().is_none() && ends_on.read().is_none()) || nights >= 1;
    let guest_word = if *guests.read() == 1 {
        "guest"
    } else {
        "guests"
    };
    use_effect(move || {
        let radius_km = *radius.read();
        spawn(async move {
            let script = DELIVERY_MAP_SCRIPT.replace("__RADIUS_KM__", &radius_km.to_string());
            let _ = document::eval(&script).await;
        });
    });
    use_effect(move || {
        let next = manual_date_text(*starts_on.read());
        if *start_input.peek() != next {
            start_input.set(next);
        }
    });
    use_effect(move || {
        let next = manual_date_text(*ends_on.read());
        if *end_input.peek() != next {
            end_input.set(next);
        }
    });
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
            div { class: "cat-planner-shell", role: "dialog", aria_modal: "true", aria_label: "Plan your RV search", tabindex: "-1", autofocus: true, onclick: move |event| event.stop_propagation(), onkeydown: move |event| if event.key() == Key::Escape { event.stop_propagation(); spawn(async move { close_overlay().await; }); },
              div { class: "cat-planner",
                div { class: "cat-planner-head",
                    div {
                        div { class: "cat-planner-kicker", "OKANAGAN RV SEARCH" }
                        h2 { "Plan your delivered RV trip" }
                        p { "Set the delivery radius, travel dates and group size in one place." }
                    }
                    button { class: "cat-planner-close", r#type: "button", aria_label: "Close search", onclick: move |_| close_overlay(),
                        Icon { name: "x", size: 22, color: "var(--vl-ink)" }
                    }
                }

                div { class: "cat-planner-grid",
                    section { class: "cat-planner-location",
                        div { class: "cat-planner-section-head",
                            div { class: "cat-planner-section-icon", Icon { name: "map-pin", size: 18, color: "var(--vl-white)" } }
                            div { h3 { "Delivery area" } p { "Every RV is delivered and set up within 150 km of Kelowna." } }
                        }
                        label { class: "cat-location-input",
                            span { "DELIVERY BASE" }
                            div { Icon { name: "map-pin", size: 17, color: "var(--vl-forest)" }
                                input { value: "{location}", readonly: true, aria_label: "Delivery base" }
                            }
                        }
                        div { class: "cat-radius-map", aria_label: "Interactive delivery radius map centred on Kelowna",
                            div { id: "vl-delivery-map", class: "cat-leaflet-map" }
                            div { class: "cat-map-fallback",
                                div { class: "cat-map-fallback-icon", Icon { name: "map", size: 22, color: "var(--vl-forest)" } }
                                span { "Loading delivery map…" }
                            }
                            div { class: "cat-map-radius-badge", strong { "{radius} km" } span { "maximum search radius" } }
                            div { class: "cat-map-legend",
                                i {}
                                span { "Approximate delivery area" }
                            }
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
                        div { class: "cat-radius-note", Icon { name: "info", size: 15, color: "var(--vl-forest)" } span { "Approximate area. Final eligibility and fee use driving distance, up to 150 km one way." } }
                    }

                    section { class: "cat-planner-trip",
                        div { class: "cat-planner-section-head",
                            div { class: "cat-planner-section-icon", Icon { name: "calendar", size: 18, color: "var(--vl-white)" } }
                            div { h3 { "Travel dates" } p { "Choose 1 or more nights. Short stays are priced at the 3-night minimum." } }
                        }
                        div { class: "cat-trip-summary",
                            label { class: "cat-trip-date-field",
                                span { "DELIVERY/SETUP · 2:00 PM" }
                                div { class: "cat-trip-date-control",
                                    input {
                                        r#type: "text",
                                        inputmode: "numeric",
                                        maxlength: "10",
                                        autocomplete: "off",
                                        spellcheck: "false",
                                        value: "{start_input}",
                                        placeholder: "YYYY-MM-DD",
                                        aria_label: "Delivery and setup date in YYYY-MM-DD format",
                                        oninput: move |event| {
                                            let value = format_manual_date_input(&event.value());
                                            start_input.set(value.clone());
                                            if let Some(date) = parse_manual_date_input(&value).filter(|date| *date > today) {
                                                starts_on.set(Some(date));
                                                if ends_on.peek().is_some_and(|end| end <= date) {
                                                    ends_on.set(None);
                                                }
                                                visible_month.set(month_start(date));
                                            } else if value.len() == 10 {
                                                start_input.set(manual_date_text(*starts_on.peek()));
                                            }
                                        },
                                        onblur: move |_| {
                                            let value = start_input.read().clone();
                                            if let Some(date) = parse_manual_date_input(&value).filter(|date| *date > today) {
                                                starts_on.set(Some(date));
                                                if ends_on.peek().is_some_and(|end| end <= date) {
                                                    ends_on.set(None);
                                                }
                                                visible_month.set(month_start(date));
                                            } else {
                                                start_input.set(manual_date_text(*starts_on.peek()));
                                            }
                                        }
                                    }
                                    Icon { name: "calendar-days", size: 16, color: "var(--vl-muted)" }
                                }
                            }
                            Icon { name: "arrow-right", size: 18, color: "var(--vl-muted)" }
                            label { class: "cat-trip-date-field",
                                span { "RETURN · 11:00 AM" }
                                div { class: "cat-trip-date-control",
                                    input {
                                        r#type: "text",
                                        inputmode: "numeric",
                                        maxlength: "10",
                                        autocomplete: "off",
                                        spellcheck: "false",
                                        value: "{end_input}",
                                        placeholder: "YYYY-MM-DD",
                                        disabled: starts_on.read().is_none(),
                                        aria_label: "Return date in YYYY-MM-DD format",
                                        oninput: move |event| {
                                            let value = format_manual_date_input(&event.value());
                                            end_input.set(value.clone());
                                            if let (Some(start), Some(date)) = (*starts_on.peek(), parse_manual_date_input(&value)) {
                                                if date > start {
                                                    ends_on.set(Some(date));
                                                } else if value.len() == 10 {
                                                    end_input.set(manual_date_text(*ends_on.peek()));
                                                }
                                            } else if value.len() == 10 {
                                                end_input.set(manual_date_text(*ends_on.peek()));
                                            }
                                        },
                                        onblur: move |_| {
                                            let value = end_input.read().clone();
                                            if let (Some(start), Some(date)) = (*starts_on.peek(), parse_manual_date_input(&value)) {
                                                if date > start {
                                                    ends_on.set(Some(date));
                                                } else {
                                                    end_input.set(manual_date_text(*ends_on.peek()));
                                                }
                                            } else {
                                                end_input.set(manual_date_text(*ends_on.peek()));
                                            }
                                        }
                                    }
                                    Icon { name: "calendar-days", size: 16, color: "var(--vl-muted)" }
                                }
                            }
                        }
                        div { class: "cat-month-nav",
                            span { "Choose delivery and return" }
                            div { class: "cat-month-nav-actions",
                                button { r#type: "button", aria_label: "Previous month", disabled: *visible_month.read() <= initial_month, onclick: move |_| {
                                    let current = *visible_month.peek();
                                    if let Some(previous) = current.checked_sub_months(Months::new(1)) {
                                        if previous >= initial_month { visible_month.set(previous); }
                                    }
                                }, Icon { name: "chevron-left", size: 18, color: "var(--vl-ink)" } }
                                button { r#type: "button", aria_label: "Next month", disabled: *visible_month.read() >= add_months(initial_month, 15), onclick: move |_| {
                                    let current = *visible_month.peek();
                                    visible_month.set(add_months(current, 1));
                                },
                                    Icon { name: "chevron-right", size: 18, color: "var(--vl-ink)" }
                                }
                            }
                        }
                        div { class: "cat-calendar-months",
                            for offset in 0..3_u32 {
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
                            strong { if nights >= 1 { "{nights}-night Okanagan trip" } else { "Choose your travel dates" } }
                            span { if (1..3).contains(&nights) { "Priced at 3-night minimum · {location} · {guests} {guest_word}" } else { "{location} · within {radius} km · {guests} {guest_word}" } }
                        }
                    }
                    button { class: "cat-planner-apply", r#type: "button", disabled: !can_apply, onclick: move |_| {
                        let draft = api::CatalogSearchDraft {
                            location: location.read().clone(),
                            radius_km: *radius.read(),
                            starts_on: (*starts_on.read()).map(|value| value.to_string()),
                            ends_on: (*ends_on.read()).map(|value| value.to_string()),
                            guests: *guests.read(),
                        };
                        let _ = api::save_json("vl_catalog_search", &draft);
                        on_apply.call(());
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
}

fn catalog_day_is_disabled(
    day: NaiveDate,
    today: NaiveDate,
    unavailable: bool,
    availability_pending: bool,
    availability_blocked: bool,
    is_selected_edge: bool,
) -> bool {
    availability_blocked
        || day <= today
        || unavailable
        || (availability_pending && !is_selected_edge)
}

fn calendar_day_status_label(
    day: NaiveDate,
    today: NaiveDate,
    too_short: bool,
    unavailable: bool,
    show_availability_counts: bool,
    available_rv_count: Option<usize>,
) -> String {
    if day < today {
        return "Past date; unavailable for delivery".to_string();
    }
    if day == today {
        return "Same-day delivery is unavailable; choose tomorrow or later".to_string();
    }
    if too_short {
        return "Short stay; selectable and priced at the 3-night minimum".to_string();
    }
    if unavailable && !show_availability_counts {
        return "Unavailable for this RV".to_string();
    }
    if !show_availability_counts {
        return String::new();
    }
    match available_rv_count {
        Some(0) => "No RVs available for this date selection".to_string(),
        Some(1) => "Only 1 RV available for this date selection".to_string(),
        Some(count) => format!("{count} RVs available for this date selection"),
        None => "Availability is being checked".to_string(),
    }
}

#[component]
pub(crate) fn CatalogSearchMonth(
    month: NaiveDate,
    today: NaiveDate,
    mut starts_on: Signal<Option<NaiveDate>>,
    mut ends_on: Signal<Option<NaiveDate>>,
    #[props(default)] unavailable_dates: Vec<NaiveDate>,
    #[props(default)] availability_counts: Vec<(NaiveDate, usize)>,
    #[props(default = false)] show_availability_counts: bool,
    #[props(default = false)] availability_pending: bool,
    #[props(default = false)] availability_blocked: bool,
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
                            let unavailable = unavailable_dates.contains(&day) && !is_start && !is_end;
                            let available_rv_count = availability_counts
                                .iter()
                                .find_map(|(availability_day, count)| (*availability_day == day).then_some(*count));
                            let disabled = catalog_day_is_disabled(
                                day,
                                today,
                                unavailable,
                                availability_pending,
                                availability_blocked,
                                is_start || is_end,
                            );
                            let limited = show_availability_counts
                                && available_rv_count.is_some_and(|count| (1..=2).contains(&count));
                            let class = if is_start || is_end { "selected" } else if in_range { "in-range" } else if limited { "limited" } else { "" };
                            let availability_label = calendar_day_status_label(
                                day,
                                today,
                                too_short,
                                unavailable,
                                show_availability_counts,
                                available_rv_count,
                            );
                            let aria_label = if availability_label.is_empty() {
                                day.to_string()
                            } else {
                                format!("{day}, {availability_label}")
                            };
                            rsx! { button { key: "day-{index}", r#type: "button", class, disabled, aria_label, title: availability_label, onclick: move |_| {
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

    #[test]
    fn one_night_return_completes_the_catalog_range() {
        let selected = day("2030-08-10");
        let next_day = day("2030-08-11");
        assert_eq!(
            next_catalog_date_selection(next_day, Some(selected), None),
            (Some(selected), Some(next_day))
        );
    }

    #[test]
    fn availability_error_locks_even_a_previously_selected_edge() {
        let selected = day("2030-08-10");
        assert!(catalog_day_is_disabled(
            selected,
            day("2030-08-01"),
            false,
            false,
            true,
            true,
        ));
        assert!(!catalog_day_is_disabled(
            selected,
            day("2030-08-01"),
            false,
            true,
            false,
            true,
        ));
    }

    #[test]
    fn calendar_accessibility_explains_policy_before_availability_counts() {
        let today = day("2030-08-10");
        assert_eq!(
            calendar_day_status_label(day("2030-08-09"), today, false, false, true, Some(6)),
            "Past date; unavailable for delivery"
        );
        assert_eq!(
            calendar_day_status_label(today, today, false, false, true, Some(6)),
            "Same-day delivery is unavailable; choose tomorrow or later"
        );
        assert_eq!(
            calendar_day_status_label(day("2030-08-11"), today, true, true, true, Some(0)),
            "Short stay; selectable and priced at the 3-night minimum"
        );
        assert_eq!(
            calendar_day_status_label(day("2030-08-13"), today, false, false, true, Some(1)),
            "Only 1 RV available for this date selection"
        );
        assert_eq!(
            calendar_day_status_label(day("2030-08-13"), today, false, true, false, None),
            "Unavailable for this RV"
        );
    }

    #[test]
    fn manual_date_input_is_limited_and_formatted() {
        assert_eq!(format_manual_date_input("2026081333"), "2026-08-13");
        assert_eq!(format_manual_date_input("2026-8-1"), "2026-81");
    }

    #[test]
    fn impossible_manual_date_is_rejected() {
        assert_eq!(parse_manual_date_input("2026-02-33"), None);
        assert_eq!(
            parse_manual_date_input("2026-08-13"),
            Some(day("2026-08-13"))
        );
    }

    #[test]
    fn stale_or_incomplete_saved_dates_are_cleared() {
        let saved = api::CatalogSearchDraft {
            location: "Kelowna, BC".into(),
            radius_km: 200,
            starts_on: Some("2030-07-01".into()),
            ends_on: None,
            guests: 20,
        };
        let normalized = normalized_catalog_search(Some(saved), 50, day("2030-07-02"));
        assert_eq!(normalized.starts_on, None);
        assert_eq!(normalized.ends_on, None);
        assert_eq!(normalized.radius_km, 150);
        assert_eq!(normalized.guests, 10);
    }

    #[test]
    fn valid_saved_dates_are_preserved() {
        let saved = api::CatalogSearchDraft {
            location: "Kelowna, BC".into(),
            radius_km: 75,
            starts_on: Some("2030-07-10".into()),
            ends_on: Some("2030-07-13".into()),
            guests: 4,
        };
        let normalized = normalized_catalog_search(Some(saved.clone()), 50, day("2030-07-02"));
        assert_eq!(normalized, saved);
    }

    #[test]
    fn same_day_saved_start_is_cleared() {
        let saved = api::CatalogSearchDraft {
            location: "Kelowna, BC".into(),
            radius_km: 75,
            starts_on: Some("2030-07-02".into()),
            ends_on: Some("2030-07-05".into()),
            guests: 4,
        };
        let normalized = normalized_catalog_search(Some(saved), 50, day("2030-07-02"));
        assert_eq!(normalized.starts_on, None);
        assert_eq!(normalized.ends_on, None);
    }

    fn rental(slug: &str, description: &str, price: &str, capacity: i32) -> api::Rental {
        api::Rental {
            rental_id: format!("rental-{slug}"),
            slug: slug.into(),
            name: slug.into(),
            category: "rv".into(),
            summary: String::new(),
            description: description.into(),
            model_year: Some(2025),
            manufacturer: "Test".into(),
            model: slug.into(),
            rv_type: if slug == "fifth" {
                "fifth_wheel".into()
            } else if slug == "toys" {
                "toy_hauler".into()
            } else {
                "travel_trailer".into()
            },
            length_ft: Some("24.0".into()),
            slide_outs: 1,
            pet_friendly: false,
            capacity,
            price_unit: "night".into(),
            base_rate: price.into(),
            currency: "CAD".into(),
            min_units: 3,
            refundable_deposit: "1000.00".into(),
            hero_image_url: None,
            is_active: true,
            sort_order: 0,
            review_rating: None,
            review_count: 0,
        }
    }

    #[test]
    fn catalog_filters_budget_capacity_and_style_together() {
        let rentals = vec![
            rental("couples", "lightweight travel trailer", "125.00", 4),
            rental("family", "family travel trailer", "160.00", 10),
            rental("fifth", "26-foot fifth wheel", "185.00", 4),
            rental("toys", "toy-hauler layout", "148.00", 8),
        ];
        let filters = CatalogFilters {
            fifth_wheels: false,
            maximum_nightly_price: 150,
            minimum_capacity: 8,
            ..CatalogFilters::default()
        };

        let result = filtered_catalog(&rentals, &filters);
        assert_eq!(
            result
                .iter()
                .map(|rental| rental.slug.as_str())
                .collect::<Vec<_>>(),
            ["toys"]
        );
    }

    #[test]
    fn catalog_price_sort_is_numeric() {
        let rentals = vec![
            rental("premium", "travel trailer", "185.00", 4),
            rental("value", "travel trailer", "125.00", 4),
            rental("family", "travel trailer", "160.00", 10),
        ];
        let filters = CatalogFilters {
            sort: "price-low".into(),
            ..CatalogFilters::default()
        };

        let result = filtered_catalog(&rentals, &filters);
        assert_eq!(
            result
                .iter()
                .map(|rental| rental.slug.as_str())
                .collect::<Vec<_>>(),
            ["value", "family", "premium"]
        );
    }

    #[test]
    fn date_fit_sort_prioritizes_capacity_match_then_price() {
        let rentals = vec![
            rental("large", "travel trailer", "150.00", 10),
            rental("premium-fit", "travel trailer", "180.00", 6),
            rental("value-fit", "travel trailer", "140.00", 6),
            rental("compact", "travel trailer", "120.00", 4),
        ];
        let filters = CatalogFilters {
            sort: "date-fit".into(),
            ..CatalogFilters::default()
        };

        let result = filtered_catalog_for_guests(&rentals, &filters, Some(6));
        assert_eq!(
            result
                .iter()
                .map(|rental| rental.slug.as_str())
                .collect::<Vec<_>>(),
            ["value-fit", "premium-fit", "compact", "large"]
        );
    }
}

#[component]
pub(crate) fn ApiListingCard(rental: api::Rental) -> Element {
    let image = rental_image(&rental);
    let policy_summary = format!(
        "Sleeps {} · short stays use 3-night minimum pricing · Delivery only",
        rental.capacity
    );
    rsx! {
        Link { class: "listing-card", to: Route::RvDetail { slug: rental.slug.clone() },
            div { class: "lc-image", style: "background-image: url('{image}');",
                div { class: "lc-badge", "Sleeps {rental.capacity}" }
            }
            div { class: "lc-body",
                div { class: "lc-title-row", div { class: "lc-title", "{rental.name}" } }
                div { class: "lc-meta", "{policy_summary}" }
                div { class: "lc-price-row", span { class: "lc-price", "${rental.base_rate}" } span { class: "lc-per", " / {rental.price_unit}" } }
                div { class: "lc-price-note", "Plus mandatory fees · separate refundable CA$1,000 damage deposit" }
            }
        }
    }
}

pub(crate) fn rental_image(rental: &api::Rental) -> String {
    if let Some(image) = rental
        .hero_image_url
        .as_ref()
        .filter(|value| !value.trim().is_empty())
    {
        return image.clone();
    }
    match rental.slug.as_str() {
        "jayco26" => IMG_JAYCO.to_string(),
        "2015-keystone-bullet" => IMG_BULLET.to_string(),
        "2014-forest-river-rockwood" => IMG_ROCKWOOD.to_string(),
        "2025-open-range-1" => IMG_OPENRANGE.to_string(),
        "2025-highland-ridge-2" => IMG_OPENRANGE2.to_string(),
        "2017-keystone-outback-ultra" => IMG_OUTBACK.to_string(),
        _ => String::new(),
    }
}

#[component]
pub(crate) fn CatalogFilteredEmpty(on_reset: EventHandler<()>) -> Element {
    rsx! {
        div { class: "co-card catalog-result-message", role: "status",
            h3 { "No RV matches these filters" }
            p { "Try a higher nightly budget, a smaller sleeping capacity, or another RV style." }
            button { class: "btn-forest", r#type: "button", onclick: move |_| on_reset.call(()), "Reset filters" }
        }
    }
}

#[component]
pub(crate) fn Filters(mut filters: Signal<CatalogFilters>) -> Element {
    let mut mobile_open = use_signal(|| false);
    let state = filters.read().clone();
    let active_count = usize::from(state.maximum_nightly_price < 185)
        + usize::from(state.minimum_capacity > 0)
        + usize::from(!state.travel_trailers || !state.fifth_wheels || !state.toy_haulers);
    let budget_label = if state.maximum_nightly_price >= 185 {
        "Any budget".to_string()
    } else {
        format!("Up to ${}/night", state.maximum_nightly_price)
    };
    let mobile_chevron = if *mobile_open.read() {
        "chevron-up"
    } else {
        "chevron-down"
    };
    rsx! {
        aside { class: "cat-filters",
            button { class: "filter-mobile-toggle", r#type: "button", aria_expanded: *mobile_open.read(), onclick: move |_| {
                let next = !*mobile_open.peek();
                mobile_open.set(next);
            },
                span { Icon { name: "sliders-horizontal", size: 17, color: "var(--vl-forest)" } "Filters" }
                if active_count > 0 { b { "{active_count}" } }
                Icon { name: mobile_chevron, size: 17, color: "var(--vl-ink)" }
            }
            div { class: if *mobile_open.read() { "cat-filter-panel is-open" } else { "cat-filter-panel" },
                div { class: "filter-title-row",
                    div { div { class: "filter-kicker", "REFINE RESULTS" } h2 { "Filters" } }
                    if active_count > 0 {
                        button { r#type: "button", onclick: move |_| filters.set(CatalogFilters::default()), "Reset" }
                    }
                }
                div { class: "filter-group",
                    div { class: "filter-head", "RV style" }
                    FilterCheck { label: "Travel trailers", checked: state.travel_trailers, on_toggle: move |_| {
                        let mut next = filters.read().clone(); next.travel_trailers = !next.travel_trailers; filters.set(next);
                    } }
                    FilterCheck { label: "Fifth wheels", checked: state.fifth_wheels, on_toggle: move |_| {
                        let mut next = filters.read().clone(); next.fifth_wheels = !next.fifth_wheels; filters.set(next);
                    } }
                    FilterCheck { label: "Toy haulers", checked: state.toy_haulers, on_toggle: move |_| {
                        let mut next = filters.read().clone(); next.toy_haulers = !next.toy_haulers; filters.set(next);
                    } }
                }
                div { class: "filter-divider" }
                div { class: "filter-group price-filter", style: "gap: 14px;",
                    div { class: "filter-head-row", div { class: "filter-head", "Nightly budget" } strong { "{budget_label}" } }
                    input { class: "price-range", r#type: "range", min: "125", max: "185", step: "5", value: "{state.maximum_nightly_price}", aria_label: "Maximum nightly price", oninput: move |event| {
                        if let Ok(value) = event.value().parse::<i32>() {
                            let mut next = filters.read().clone(); next.maximum_nightly_price = value.clamp(125, 185); filters.set(next);
                        }
                    } }
                    div { class: "price-labels", span { "$125" } span { "$185+" } }
                }
                div { class: "filter-divider" }
                div { class: "filter-group",
                    div { class: "filter-head", "Sleeping capacity" }
                    div { class: "sleep-chips",
                        for (label, value) in [("Any", 0), ("4+", 4), ("8+", 8), ("10", 10)] {
                            button { key: "sleeps-{label}", r#type: "button", class: if state.minimum_capacity == value { "sleep-chip active" } else { "sleep-chip" }, onclick: move |_| {
                                let mut next = filters.read().clone(); next.minimum_capacity = value; filters.set(next);
                            }, "{label}" }
                        }
                    }
                    p { class: "filter-help", "Shows RVs that can sleep at least this many guests." }
                }
            }
        }
    }
}

#[component]
fn FilterCheck(label: &'static str, checked: bool, on_toggle: EventHandler<()>) -> Element {
    rsx! {
        button { class: if checked { "filter-check active" } else { "filter-check" }, r#type: "button", aria_pressed: checked, onclick: move |_| on_toggle.call(()),
            span { class: "filter-box",
                if checked {
                    Icon { name: "check", size: 13, color: "var(--vl-white)" }
                }
            }
            span { "{label}" }
        }
    }
}

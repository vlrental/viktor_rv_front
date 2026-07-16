use chrono::NaiveDate;
use dioxus::prelude::*;

use super::booking_overlay::UnifiedBookingOverlay;
use super::catalog::{
    bump_search_version, catalog_date_label, filtered_catalog_for_guests,
    normalized_catalog_search, ApiListingCard, CatalogEmptyState, CatalogErrorState,
    CatalogFilteredEmpty, CatalogFilters, CatalogLoadingState, Filters,
};
use crate::api;
use crate::components::Icon;
use crate::data::IMG_HERO_RV;
use crate::{BookingLaunchRequest, Route};

const IMG_CTA_PARALLAX: Asset = asset!("/assets/img/generated/okanagan-rv-parallax.webp");
const CLEANUP_HERO_VIDEO: &str = r#"
if (window.__vlHeroRevealTimer) {
    window.clearTimeout(window.__vlHeroRevealTimer);
    window.__vlHeroRevealTimer = null;
}
if (window.__vlHeroPlayer) {
    try { window.__vlHeroPlayer.destroy(); } catch (_) {}
    window.__vlHeroPlayer = null;
}
if (window.__vlHeroReadyHandler && window.onYouTubeIframeAPIReady === window.__vlHeroReadyHandler) {
    window.onYouTubeIframeAPIReady = window.__vlHeroPreviousReady || null;
}
window.__vlHeroReadyHandler = null;
window.__vlHeroPreviousReady = null;
const heroIframe = document.getElementById('hero-youtube-player');
if (heroIframe) {
    heroIframe.classList.remove('is-playing');
    heroIframe.removeAttribute('src');
}
"#;

#[derive(Clone, Copy)]
enum HomeBookingEntry {
    Dates,
    Rvs,
    Guests,
    Search,
}

#[component]
pub fn Home() -> Element {
    let mut booking_launch_request = use_context::<BookingLaunchRequest>();
    let resumed_booking = use_signal(api::take_booking_auth_continuation);
    let resumed_booking_value = resumed_booking.peek().clone();
    let today = chrono::Utc::now()
        .with_timezone(&chrono_tz::America::Vancouver)
        .date_naive();
    let saved_search = resumed_booking_value
        .as_ref()
        .map(|continuation| api::CatalogSearchDraft {
            location: continuation.location.clone(),
            radius_km: continuation.radius_km,
            starts_on: Some(continuation.draft.starts_on.clone()),
            ends_on: Some(continuation.draft.ends_on.clone()),
            guests: continuation.draft.guests,
        })
        .or_else(|| api::load_json::<api::CatalogSearchDraft>("vl_catalog_search"));
    let initial_search = normalized_catalog_search(saved_search, 150, today);
    let mut applied_search = use_signal(|| initial_search.clone());
    let search_version = use_signal(|| 0_u32);
    let mut search_location = use_signal(|| initial_search.location.clone());
    let mut search_radius = use_signal(|| initial_search.radius_km);
    let mut search_starts_on = use_signal(|| search_date(initial_search.starts_on.as_deref()));
    let mut search_ends_on = use_signal(|| search_date(initial_search.ends_on.as_deref()));
    let mut search_guests = use_signal(|| 1_i32);
    let resume_after_auth = resumed_booking_value.is_some();
    let mut search_open = use_signal(move || resume_after_auth);
    let mut search_initial_step = use_signal(move || if resume_after_auth { 5_u8 } else { 1_u8 });
    use_effect(move || {
        if *booking_launch_request.0.read() {
            let search = applied_search.read();
            search_initial_step.set(booking_entry_step(&search));
            search_open.set(true);
            booking_launch_request.0.set(false);
        }
    });
    use_effect(move || {
        if *search_open.read() {
            let search = applied_search.read().clone();
            search_location.set(search.location);
            search_radius.set(search.radius_km);
            search_starts_on.set(search_date(search.starts_on.as_deref()));
            search_ends_on.set(search_date(search.ends_on.as_deref()));
            search_guests.set(1);
        }
    });
    use_effect(move || {
        let search = applied_search.read().clone();
        let _ = api::save_json("vl_catalog_search", &search);
    });
    let resumed_booking_value = resumed_booking.read().clone();
    let resumed_rental_slug = resumed_booking_value
        .as_ref()
        .map(|continuation| continuation.draft.rental_slug.clone())
        .filter(|slug| !slug.is_empty());

    rsx! {
        Hero {
            applied_search,
            search_open,
            search_initial_step,
        }
        PopularRvs {
            applied_search,
            search_version,
            search_open,
            search_initial_step,
        }
        HowItWorks {}
        MoreServices {}
        CtaBand { search_open, search_initial_step }
        if *search_open.read() {
            UnifiedBookingOverlay {
                location: search_location,
                radius: search_radius,
                starts_on: search_starts_on,
                ends_on: search_ends_on,
                guests: search_guests,
                initial_rental_slug: resumed_rental_slug,
                initial_step: *search_initial_step.read(),
                resume_after_auth: resumed_booking_value,
                on_search_change: move |_| {
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
                on_close: move |_| {
                    let next = api::CatalogSearchDraft {
                        location: search_location.read().clone(),
                        radius_km: *search_radius.read(),
                        starts_on: (*search_starts_on.read()).map(|value| value.to_string()),
                        ends_on: (*search_ends_on.read()).map(|value| value.to_string()),
                        guests: *search_guests.read(),
                    };
                    applied_search.set(next);
                    bump_search_version(search_version);
                    search_open.set(false);
                },
            }
        }
    }
}

fn search_date(value: Option<&str>) -> Option<NaiveDate> {
    value.and_then(|value| NaiveDate::parse_from_str(value, "%Y-%m-%d").ok())
}

fn booking_entry_step(search: &api::CatalogSearchDraft) -> u8 {
    if search.starts_on.is_some() && search.ends_on.is_some() {
        2
    } else {
        1
    }
}

fn home_booking_entry_step(entry: HomeBookingEntry, has_complete_dates: bool) -> u8 {
    match entry {
        HomeBookingEntry::Rvs => 2,
        HomeBookingEntry::Dates | HomeBookingEntry::Guests => 1,
        HomeBookingEntry::Search if has_complete_dates => 2,
        HomeBookingEntry::Search => 1,
    }
}

#[component]
fn Hero(
    applied_search: Signal<api::CatalogSearchDraft>,
    mut search_open: Signal<bool>,
    mut search_initial_step: Signal<u8>,
) -> Element {
    use_effect(|| {
        document::eval(
            r#"
                const iframe = document.getElementById('hero-youtube-player');
                if (!iframe || iframe.dataset.initialized === 'true') return;
                if (!window.matchMedia('(min-width: 861px)').matches) return;
                iframe.dataset.initialized = 'true';
                iframe.src = 'https://www.youtube.com/embed/?autoplay=0&controls=0&disablekb=1&enablejsapi=1&iv_load_policy=3&modestbranding=1&mute=1&playsinline=1&rel=0';

                const createPlayer = () => {
                    const playerIframe = document.getElementById('hero-youtube-player');
                    if (!playerIframe || !window.YT?.Player) return;

                    window.__vlHeroPlayer = new window.YT.Player('hero-youtube-player', {
                        events: {
                            onReady: (event) => {
                                event.target.mute();
                                event.target.loadVideoById({
                                    videoId: 'HEgbRefLY_A',
                                    startSeconds: 0
                                });
                            },
                            onStateChange: (event) => {
                                if (event.data === window.YT.PlayerState.PLAYING) {
                                    window.clearTimeout(window.__vlHeroRevealTimer);
                                    window.__vlHeroRevealTimer = window.setTimeout(() => {
                                        if (event.target.getPlayerState() === window.YT.PlayerState.PLAYING) {
                                            event.target.getIframe().classList.add('is-playing');
                                        }
                                    }, 1500);
                                }
                                if (
                                    event.data === window.YT.PlayerState.PAUSED ||
                                    event.data === window.YT.PlayerState.CUED
                                ) {
                                    window.clearTimeout(window.__vlHeroRevealTimer);
                                    event.target.getIframe().classList.remove('is-playing');
                                }
                                if (event.data === window.YT.PlayerState.ENDED) {
                                    event.target.getIframe().classList.remove('is-playing');
                                    event.target.seekTo(0);
                                    event.target.playVideo();
                                }
                            }
                        }
                    });
                };

                if (window.YT?.Player) {
                    createPlayer();
                } else {
                    const previousReady = window.onYouTubeIframeAPIReady;
                    const readyHandler = () => {
                        if (typeof previousReady === 'function') previousReady();
                        createPlayer();
                    };
                    window.__vlHeroPreviousReady = previousReady;
                    window.__vlHeroReadyHandler = readyHandler;
                    window.onYouTubeIframeAPIReady = readyHandler;

                    if (!document.querySelector('script[src="https://www.youtube.com/iframe_api"]')) {
                        const script = document.createElement('script');
                        script.src = 'https://www.youtube.com/iframe_api';
                        document.head.appendChild(script);
                    }
                }
            "#,
        );
    });
    use_drop(|| {
        document::eval(CLEANUP_HERO_VIDEO);
    });

    let search = applied_search.read().clone();
    let starts_on = search_date(search.starts_on.as_deref());
    let ends_on = search_date(search.ends_on.as_deref());
    let has_complete_dates = starts_on.is_some() && ends_on.is_some();
    let dates_label = catalog_date_label(starts_on, ends_on);
    let guests_label = if search.guests == 1 {
        "1 guest".to_string()
    } else {
        format!("{} guests", search.guests)
    };

    rsx! {
        section { class: "hero",
            div { class: "hero-media", style: "background-image: url('{IMG_HERO_RV}');",
                iframe {
                    id: "hero-youtube-player",
                    class: "hero-video",
                    title: "Kelowna, British Columbia - Drone 4K",
                    allow: "autoplay; encrypted-media; picture-in-picture",
                    tabindex: "-1",
                    "aria-hidden": "true",
                }
            }
            div { class: "hero-overlay" }
            div { class: "hero-copy",
                div { class: "eyebrow-pill",
                    Icon { name: "map-pin", size: 15, color: "var(--vl-accent)" }
                    span { "Kelowna & the Okanagan" }
                }
                h1 { class: "hero-title", "Explore nature on the open road" }
                p { class: "hero-sub",
                    "Book fully-equipped RVs in minutes. Choose your wheels, chase your adventure — we handle the logistics."
                }
            }
            div { class: "searchbar",
                button { class: "searchbar-open", r#type: "button", aria_label: "Open booking dates", onclick: move |_| { search_initial_step.set(home_booking_entry_step(HomeBookingEntry::Dates, has_complete_dates)); search_open.set(true); } }
                div { class: "search-field is-static",
                    div { class: "search-label", "DELIVERY RADIUS" }
                    div { class: "search-value",
                        Icon { name: "map-pin", size: 17, color: "var(--vl-forest)" }
                        span { "Up to {search.radius_km} km" }
                    }
                }
                div { class: "search-divider" }
                button { class: "search-field", r#type: "button", onclick: move |_| { search_initial_step.set(home_booking_entry_step(HomeBookingEntry::Rvs, has_complete_dates)); search_open.set(true); },
                    div { class: "search-label", "WHAT" }
                    div { class: "search-value",
                        Icon { name: "compass", size: 17, color: "var(--vl-forest)" }
                        span { "RVs" }
                    }
                }
                div { class: "search-divider" }
                button { class: "search-field", r#type: "button", onclick: move |_| { search_initial_step.set(home_booking_entry_step(HomeBookingEntry::Dates, has_complete_dates)); search_open.set(true); },
                    div { class: "search-label", "DATES" }
                    div { class: "search-value",
                        Icon { name: "calendar", size: 17, color: "var(--vl-forest)" }
                        span { "{dates_label}" }
                    }
                }
                div { class: "search-divider" }
                button { class: "search-field", r#type: "button", onclick: move |_| { search_initial_step.set(home_booking_entry_step(HomeBookingEntry::Guests, has_complete_dates)); search_open.set(true); },
                    div { class: "search-label", "GUESTS" }
                    div { class: "search-value",
                        Icon { name: "users", size: 17, color: "var(--vl-forest)" }
                        span { "{guests_label}" }
                    }
                }
                button { class: "search-btn", r#type: "button", onclick: move |_| { search_initial_step.set(home_booking_entry_step(HomeBookingEntry::Search, has_complete_dates)); search_open.set(true); },
                    Icon { name: "search", size: 19, color: "var(--vl-white)" }
                    span { "Search" }
                }
            }
        }
    }
}

#[cfg(test)]
mod booking_entry_tests {
    use super::*;

    fn search(starts_on: Option<&str>, ends_on: Option<&str>) -> api::CatalogSearchDraft {
        api::CatalogSearchDraft {
            location: "Kelowna, BC".into(),
            radius_km: 150,
            starts_on: starts_on.map(str::to_string),
            ends_on: ends_on.map(str::to_string),
            guests: 2,
        }
    }

    #[test]
    fn global_booking_opens_dates_when_the_range_is_incomplete() {
        assert_eq!(booking_entry_step(&search(None, None)), 1);
        assert_eq!(booking_entry_step(&search(Some("2026-08-16"), None)), 1);
        assert_eq!(booking_entry_step(&search(None, Some("2026-08-19"))), 1);
    }

    #[test]
    fn global_booking_skips_to_rv_selection_for_a_complete_range() {
        assert_eq!(
            booking_entry_step(&search(Some("2026-08-16"), Some("2026-08-19"))),
            2
        );
    }

    #[test]
    fn home_search_fields_open_their_matching_rv_steps() {
        assert_eq!(home_booking_entry_step(HomeBookingEntry::Dates, true), 1);
        assert_eq!(home_booking_entry_step(HomeBookingEntry::Guests, true), 1);
        assert_eq!(home_booking_entry_step(HomeBookingEntry::Rvs, false), 2);
        assert_eq!(home_booking_entry_step(HomeBookingEntry::Search, false), 1);
        assert_eq!(home_booking_entry_step(HomeBookingEntry::Search, true), 2);
    }
}

#[component]
fn PopularRvs(
    mut applied_search: Signal<api::CatalogSearchDraft>,
    mut search_version: Signal<u32>,
    mut search_open: Signal<bool>,
    mut search_initial_step: Signal<u8>,
) -> Element {
    let mut filters = use_signal(CatalogFilters::default);
    use_effect(move || {
        let search = applied_search.read();
        let dates_are_complete = search.starts_on.is_some() && search.ends_on.is_some();
        if !dates_are_complete && filters.peek().sort == "date-fit" {
            let mut next = filters.peek().clone();
            next.sort = "recommended".into();
            filters.set(next);
        }
    });
    let listings = use_resource(move || {
        let _version = *search_version.read();
        let search = applied_search.read().clone();
        async move { api::catalog(&search).await }
    });
    let search = applied_search.read().clone();
    let starts_on = search_date(search.starts_on.as_deref());
    let ends_on = search_date(search.ends_on.as_deref());
    let has_dates = starts_on.is_some() && ends_on.is_some();
    let dates_label = catalog_date_label(starts_on, ends_on);
    let visible_rentals = listings
        .read()
        .as_ref()
        .and_then(|result| result.as_ref().ok())
        .map(|values| {
            filtered_catalog_for_guests(values, &filters.read(), has_dates.then_some(search.guests))
        })
        .unwrap_or_default();
    let match_count = visible_rentals.len();
    let match_label = if starts_on.is_none() && ends_on.is_none() {
        format!("{} RVs fit {} guests", match_count, search.guests)
    } else {
        format!(
            "{} RVs · {} · {} guests",
            match_count, dates_label, search.guests
        )
    };
    rsx! {
        section { id: "home-rentals", class: "section bg-white home-rentals",
            div { class: "sec-header",
                div {
                    div { class: "eyebrow", "CHOOSE YOUR WHEELS" }
                    h2 { class: "sec-title", "Explore available RVs" }
                }
            }
            div { class: "home-catalog-toolbar",
                div { class: "home-match-label",
                    Icon { name: "sparkles", size: 16, color: "var(--vl-accent)" }
                    span { "{match_label}" }
                }
                label { class: "cat-sort",
                    Icon { name: "arrow-up-down", size: 14, color: "var(--vl-ink)" }
                    span { "Sort" }
                    select {
                        aria_label: "Sort RVs",
                        value: "{filters.read().sort}",
                        onchange: move |event| {
                            let mut next = filters.read().clone();
                            next.sort = event.value();
                            filters.set(next);
                        },
                        option { value: "recommended", "Recommended" }
                        if has_dates {
                            option { value: "date-fit", "Best fit for your dates" }
                        }
                        option { value: "price-low", "Price: low to high" }
                        option { value: "price-high", "Price: high to low" }
                        option { value: "capacity", "Most sleeping space" }
                    }
                    Icon { name: "chevron-down", size: 14, color: "var(--vl-ink)" }
                }
            }
            div { class: "home-catalog-layout",
                Filters { filters }
                div { class: "home-catalog-grid",
                    if let Some(result) = listings.read().as_ref() {
                        match result {
                            Ok(values) if values.is_empty() => rsx! {
                                CatalogEmptyState {
                                    dates_label: dates_label.clone(),
                                    has_dates: starts_on.is_some() && ends_on.is_some(),
                                    on_change: move |_| { search_initial_step.set(1); search_open.set(true); },
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
                                CatalogErrorState {
                                    message: message.clone(),
                                    on_retry: move |_| bump_search_version(search_version),
                                }
                            },
                        }
                    } else {
                        CatalogLoadingState {}
                    }
                }
            }
        }
    }
}

#[component]
fn HowItWorks() -> Element {
    let steps = [
        ("search", "01", "Search & compare", "Browse fully-equipped RVs with transparent, upfront pricing — no hidden fees."),
        ("calendar-check", "02", "Book online", "Reserve in minutes with fair policies and helpful guidance from our local Okanagan team."),
        ("tent-tree", "03", "Hit the road", "We deliver, level and set up your rig. You focus on the fun, we handle the logistics."),
    ];
    rsx! {
        section { class: "section bg-white", style: "padding-top: 74px; padding-bottom: 74px;",
            div { class: "hiw-header",
                div { class: "eyebrow", "HOW IT WORKS" }
                h2 { class: "hiw-title", "Book in minutes, adventure in hours" }
            }
            div { class: "card-row",
                for (icon, num, title, desc) in steps {
                    div { key: "{num}", class: "step-card",
                        div { class: "step-top",
                            div { class: "step-icon",
                                Icon { name: icon, size: 24, color: "var(--vl-white)" }
                            }
                            div { class: "step-num", "{num}" }
                        }
                        div { class: "step-title", "{title}" }
                        p { class: "step-desc", "{desc}" }
                    }
                }
            }
        }
    }
}

#[component]
fn MoreServices() -> Element {
    let services: [(&'static str, &'static str, &'static str, Route); 5] = [
        (
            "snowflake",
            "Cooler Trailers",
            "Keep food & drinks cold on any trip",
            Route::CoolerTrailers {},
        ),
        (
            "truck",
            "Delivery Services",
            "We drop off, level and set up for you",
            Route::Delivery {},
        ),
        (
            "badge-dollar-sign",
            "RV Sales",
            "Ready to own? Browse RVs for sale",
            Route::RvSales {},
        ),
        (
            "mountain",
            "Attractions",
            "Local picks for the best of Kelowna",
            Route::Attractions {},
        ),
        (
            "utensils",
            "Restaurants",
            "Where to eat around the lake",
            Route::Restaurants {},
        ),
    ];
    rsx! {
        section { class: "section bg-forest", style: "padding-top: 64px; padding-bottom: 64px;",
            div { class: "sec-header", style: "margin-bottom: 34px;",
                div {
                    div { class: "eyebrow gold", "BEYOND RENTALS" }
                    h2 { class: "sec-title on-dark", style: "font-size: 38px;", "More ways to explore with VL" }
                }
                p { class: "services-sub",
                    "One local team for your whole trip — from gear to guidance around the Okanagan."
                }
            }
            div { class: "service-row",
                for (icon, title, desc, to) in services {
                    Link { key: "{title}", class: "service-cell", to,
                        div { class: "service-icon",
                            Icon { name: icon, size: 22, color: "var(--vl-accent)" }
                        }
                        div { class: "service-title", "{title}" }
                        div { class: "service-desc", "{desc}" }
                    }
                }
            }
        }
    }
}

#[component]
fn CtaBand(mut search_open: Signal<bool>, mut search_initial_step: Signal<u8>) -> Element {
    rsx! {
        section { id: "home-parallax-cta", class: "cta-band",
            div {
                class: "cta-img",
                role: "img",
                aria_label: "Travel trailer at an Okanagan lakeside campsite",
                style: "background-image: url('{IMG_CTA_PARALLAX}');"
            }
            div { class: "cta-overlay" }
            div { class: "cta-copy",
                div { class: "eyebrow gold", "TRAVEL WITH US" }
                h2 { class: "cta-title", "Loved by guests, remembered for the fun" }
                p { class: "cta-sub",
                    "The best memories are made outdoors — tracing scenic Okanagan highways and settling into lakeside campsites. Let's get you out there."
                }
                div { class: "cta-buttons",
                    button { class: "btn-gold", r#type: "button", aria_haspopup: "dialog", onclick: move |_| { search_initial_step.set(1); search_open.set(true); },
                        Icon { name: "tent-tree", size: 18, color: "var(--vl-forest-2)" }
                        span { "Book an RV" }
                    }
                }
            }
        }
    }
}

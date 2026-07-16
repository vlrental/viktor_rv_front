//! Страница RV Detail / Booking — Pencil-фреймы `l3JikE` (desktop) и `wjWTt` (mobile).

#[cfg(test)]
use chrono::{DateTime, Duration, LocalResult, TimeZone};
use chrono::{NaiveDate, Utc};
use chrono_tz::America::Vancouver;
use dioxus::prelude::*;

use super::booking_overlay::UnifiedBookingOverlay;
use crate::data::{rv_gallery, Listing, PHONE};
use crate::{api, components::Icon, pricing, Route};

const CSS: Asset = asset!("/assets/css/rv_detail.css");
const IMG_HOST: Asset = asset!("/assets/img/host-viktor.webp");

#[component]
pub fn RvDetail(slug: String) -> Element {
    let confirmation_slug = slug.clone();
    let confirmed_booking = use_signal(move || {
        let saved =
            api::load_json::<api::CreatedBooking>("vl_post_payment_booking").filter(|created| {
                created.booking.rental_slug == confirmation_slug
                    && created.booking.status == "confirmed"
            });
        if saved.is_some() {
            api::remove_saved("vl_post_payment_booking");
        }
        saved
    });
    let rentals_href = api::frontend_path("/#home-rentals");
    let missing_rv_href = rentals_href.clone();
    let today = Utc::now().with_timezone(&Vancouver).date_naive();
    let catalog_search = super::catalog::normalized_catalog_search(
        api::load_json::<api::CatalogSearchDraft>("vl_catalog_search"),
        150,
        today,
    );
    let _ = api::save_json("vl_catalog_search", &catalog_search);
    let initial_start = catalog_search.starts_on.clone().unwrap_or_default();
    let initial_end = catalog_search.ends_on.clone().unwrap_or_default();
    let starts_on = use_signal(|| initial_start);
    let ends_on = use_signal(|| initial_end);
    let api_slug = slug.clone();
    let details = use_resource(move || {
        let value = api_slug.clone();
        async move { api::rental(&value).await }
    });
    rsx! {
        document::Link { rel: "stylesheet", href: CSS }
        div { class: "rvd-body",
            if let Some(result) = details.read().as_ref() {
                match result {
                    Ok(value) => rsx! {
                        div { class: "rvd-crumb", a { href: rentals_href, "RV Rentals" } Icon { name: "chevron-right", size: 14, color: "var(--vl-muted)" } b { "{value.rental.name}" } }
                        if let Some(created) = confirmed_booking.read().as_ref() {
                            section { class: "rvd-booking-confirmed", role: "status",
                                div { class: "rvd-booking-confirmed-icon", Icon { name: "check", size: 22, color: "var(--vl-white)" } }
                                div { span { "BOOKING CONFIRMED" } h2 { "Your {value.rental.name} is booked" } p { "Reservation {created.booking.booking_number}. {created.booking.currency} ${created.booking.amount_due_now} was confirmed by the payment system." } small { if created.notification_email_sent { "A confirmation email was sent. You can also follow the trip from your account." } else { "The booking is confirmed. Email delivery was not confirmed, so keep this reservation number and check your account." } } }
                                Link { class: "rvd-booking-confirmed-link", to: Route::Account {}, "View my booking" }
                            }
                        }
                        DynamicRvDetail { value: value.clone(), starts_on, ends_on }
                    },
                    Err(_) => rsx! { h1 { class: "rvd-min-pill", "This RV could not be found or is no longer available." } a { class: "rvd-reserve", href: missing_rv_href, "Browse available RVs" } },
                }
            } else { div { class: "rvd-min-pill", "Loading RV details…" } }
        }
    }
}

#[component]
fn DynamicRvDetail(
    value: api::RentalResponse,
    starts_on: Signal<String>,
    ends_on: Signal<String>,
) -> Element {
    let rental = value.rental.clone();
    let highlights = value
        .features
        .iter()
        .filter(|feature| feature.group_name == "highlight")
        .cloned()
        .collect::<Vec<_>>();
    let amenities = value
        .features
        .iter()
        .filter(|feature| feature.group_name == "amenity")
        .cloned()
        .collect::<Vec<_>>();
    let length_label = rental
        .length_ft
        .clone()
        .unwrap_or_else(|| "Available on request".into());
    rsx! {
        DynamicTitleHead { rental: rental.clone() }
        DynamicGallery { rental: rental.clone(), media: value.media.clone() }
        div { class: "rvd-content",
            div { class: "rvd-left",
                div { class: "rvd-overview",
                    div { div { class: "rvd-overview-t", "Entire {rv_type_label(&rental.rv_type)} · hosted by Viktor" } div { class: "rvd-overview-s", "Sleeps {rental.capacity} · {length_label} ft · {rental.slide_outs} slide-outs · 3-night minimum" } }
                    div { class: "rvd-avatar", style: "background-image: url('{IMG_HOST}');" }
                }
                div { class: "rvd-divider" }
                if highlights.is_empty() {
                    div { class: "rvd-highlights",
                        div { class: "rvd-hl", Icon { name: "users", size: 20, color: "var(--vl-forest)" } div { class: "rvd-hl-t", "Sleeps {rental.capacity}" } div { class: "rvd-hl-d", "Maximum guest capacity" } }
                        div { class: "rvd-hl", Icon { name: "ruler", size: 20, color: "var(--vl-forest)" } div { class: "rvd-hl-t", "{length_label} ft" } div { class: "rvd-hl-d", "Exterior length" } }
                        div { class: "rvd-hl", Icon { name: "move-horizontal", size: 20, color: "var(--vl-forest)" } div { class: "rvd-hl-t", "{rental.slide_outs} slide-outs" } div { class: "rvd-hl-d", "Extra living space" } }
                        div { class: "rvd-hl", Icon { name: "shield-check", size: 20, color: "var(--vl-forest)" } div { class: "rvd-hl-t", if rental.pet_friendly { "Pet friendly" } else { "Family ready" } } div { class: "rvd-hl-d", "See booking policies" } }
                    }
                } else {
                    div { class: "rvd-highlights", for feature in highlights { div { key: "{feature.feature_id}", class: "rvd-hl", ApiIcon { name: feature.icon_name.clone(), size: 20 } div { class: "rvd-hl-t", "{feature.label}" } div { class: "rvd-hl-d", "{feature.description}" } } } }
                }
                div { class: "rvd-sec", h2 { class: "rvd-h", "About this RV" } p { class: "rvd-p", "{rental.description}" } }
                div { class: "rvd-sec", h2 { class: "rvd-h", "What this RV offers" }
                    if amenities.is_empty() { p { class: "rvd-p", "Amenities are being updated. Contact us for the complete equipment list." } }
                    else { div { class: "rvd-amenities", for feature in amenities { div { key: "{feature.feature_id}", class: "rvd-am", ApiIcon { name: feature.icon_name.clone(), size: 18 } span { "{feature.label}" } } } } }
                }
                GoodToKnow {}
            }
            DynamicBookingCard { rental, starts_on, ends_on }
        }
    }
}

fn rv_type_label(value: &str) -> &'static str {
    match value {
        "fifth_wheel" => "fifth wheel",
        "toy_hauler" => "toy hauler",
        _ => "travel trailer",
    }
}

fn public_icon(value: &str) -> &'static str {
    match value {
        "flame" => "flame",
        "bed-double" => "bed-double",
        "paw-print" => "paw-print",
        "utensils" => "utensils",
        "shower-head" => "shower-head",
        "snowflake" => "snowflake",
        "wifi" => "wifi",
        "tv" => "tv",
        "battery-charging" => "battery-charging",
        "plug-zap" => "plug-zap",
        "cooking-pot" => "cooking-pot",
        "tent-tree" => "tent-tree",
        "caravan" => "caravan",
        "shield-check" => "shield-check",
        "package" => "package",
        "fuel" => "fuel",
        "trash-2" => "trash-2",
        _ => "circle-check",
    }
}

#[component]
fn ApiIcon(name: String, size: u32) -> Element {
    let safe_name = public_icon(&name);
    rsx! { i { class: "icon-{safe_name}", style: "font-size: {size}px; color: var(--vl-forest);" } }
}

#[component]
fn DynamicTitleHead(rental: api::Rental) -> Element {
    let slug = rental.slug.clone();
    let mut saved = use_signal(move || {
        api::load_json::<Vec<String>>("vl_saved_rvs")
            .unwrap_or_default()
            .contains(&slug)
    });
    let rating = rental.review_rating.clone().unwrap_or_else(|| "New".into());
    rsx! {
        div { class: "rvd-title-head", div { class: "rvd-title-left", h1 { class: "rvd-title", "{rental.name}" } div { class: "rvd-meta", div { class: "rvd-meta-item", Icon { name: "star", size: 15, color: "var(--vl-accent)" } span { class: "rvd-meta-strong", "{rating}" } span { "({rental.review_count} reviews)" } } span { "·" } div { class: "rvd-meta-item", Icon { name: "map-pin", size: 15, color: "var(--vl-accent)" } span { "Kelowna, BC" } } if rental.pet_friendly { span { "·" } div { class: "rvd-pet-pill", Icon { name: "paw-print", size: 14, color: "var(--vl-forest)" } span { "Pet friendly" } } } } }
            div { class: "rvd-actions", button { class: if saved() { "rvd-action-btn active" } else { "rvd-action-btn" }, r#type: "button", onclick: { let slug=rental.slug.clone(); move |_| { let mut values=api::load_json::<Vec<String>>("vl_saved_rvs").unwrap_or_default(); if saved(){values.retain(|value|value!=&slug);saved.set(false)}else{values.push(slug.clone());saved.set(true)} let _=api::save_json("vl_saved_rvs",&values); } }, Icon { name: "heart", size: 15, color: "var(--vl-ink)" } span { if saved() { "Saved" } else { "Save" } } } }
        }
    }
}

#[component]
fn DynamicGallery(rental: api::Rental, media: Vec<api::RentalMedia>) -> Element {
    let mut images = media
        .iter()
        .map(|item| (item.source_url.clone(), item.alt_text.clone()))
        .collect::<Vec<_>>();
    if images.is_empty() {
        if let Some(hero) = rental.hero_image_url.as_ref() {
            images.push((hero.clone(), rental.name.clone()));
        }
    }
    let count = images.len();
    let mut selected = use_signal(|| None::<usize>);
    if images.is_empty() {
        return rsx! { div { class: "rvd-gallery rvd-gallery-empty", Icon { name: "image", size: 32, color: "var(--vl-muted)" } span { "Photos are being prepared" } } };
    }
    rsx! {
        div { class: "rvd-gallery", button { class: "rvd-gallery-main", r#type: "button", aria_label: "Open photo 1 of {count}", style: "background-image: url('{images[0].0}');", onclick: move |_|selected.set(Some(0)) }
            div { class: "rvd-gallery-grid", for (index,image) in images.iter().enumerate().skip(1).take(6) { button { key: "gallery-{index}", class: "rvd-gallery-tile", r#type: "button", aria_label: "Open photo {index+1} of {count}", style: "background-image: url('{image.0}');", onclick: move |_|selected.set(Some(index)), if index==6 && count>7 { span { class:"rvd-gallery-more", "Show all {count} photos" } } } } }
        }
        if let Some(index)=selected() { div { class:"rvd-lightbox", role:"dialog", aria_modal:"true", tabindex:"-1", onkeydown:move|event|if event.key()==Key::Escape{selected.set(None)}, onclick:move |_|selected.set(None), div { class:"rvd-lightbox-content", onclick:move|event|event.stop_propagation(), img { class:"rvd-lightbox-image", src:"{images[index].0}", alt:"{images[index].1}" } button { class:"rvd-lightbox-close", r#type:"button", onclick:move |_|selected.set(None), Icon{name:"x",size:24,color:"var(--vl-white)"} } button { class:"rvd-lightbox-nav prev", r#type:"button", onclick:move |_|selected.set(Some((index+count-1)%count)), Icon{name:"chevron-left",size:30,color:"var(--vl-white)"} } button { class:"rvd-lightbox-nav next", r#type:"button", onclick:move |_|selected.set(Some((index+1)%count)), Icon{name:"chevron-right",size:30,color:"var(--vl-white)"} } div { class:"rvd-lightbox-count", "{index+1} / {count}" } } } }
    }
}

#[component]
fn DynamicBookingCard(
    rental: api::Rental,
    mut starts_on: Signal<String>,
    mut ends_on: Signal<String>,
) -> Element {
    let mut booking_open = use_signal(|| false);
    let mut booking_step = use_signal(|| 1_u8);
    let planner_starts_on = use_signal(|| selected_date(&starts_on));
    let planner_ends_on = use_signal(|| selected_date(&ends_on));
    let planner_guests = use_signal(|| 1_i32);
    let planner_location = use_signal(|| "Kelowna, BC".to_string());
    let planner_radius = use_signal(|| 150_i32);
    let nights = selected_date(&starts_on)
        .zip(selected_date(&ends_on))
        .map(|(start, end)| (end - start).num_days())
        .unwrap_or(0);
    let base = rental.base_rate.parse::<f64>().unwrap_or_default() * nights.max(0) as f64;
    let known = base + pricing::mandatory_costs(nights);
    let slug = rental.slug.clone();
    let rating = rental.review_rating.clone().unwrap_or_else(|| "New".into());
    rsx! {
        div { class:"rvd-booking", div { class:"rvd-price-row", div { class:"rvd-price", span { class:"rvd-price-v", "CA${rental.base_rate}" } span { class:"rvd-price-u", " / night" } } div { class:"rvd-price-r", Icon{name:"star",size:14,color:"var(--vl-accent)"} b { "{rating}" } } }
            div { class:"rvd-min-pill", Icon{name:"info",size:14,color:"var(--vl-muted)"} span { "3-night minimum · transparent pricing before confirmation" } }
            button { class:"rvd-summary-dates", r#type:"button", onclick:move |_|{booking_step.set(1);booking_open.set(true)}, span { "DATES" } strong { if nights>=3 { "{starts_on} → {ends_on}" } else { "Choose dates" } } Icon{name:"chevron-right",size:15,color:"var(--vl-forest)"} }
            div { class:"rvd-price-breakdown", div { class:"rvd-price-breakdown-head", strong { "What makes up your trip price" } span { "Exact delivery, add-ons and taxes are calculated in booking" } } div { span { "Base rental" } b { if nights>=3 { "{pricing::money(base)}" } else { "CA${rental.base_rate} / night" } } } div { span { "RV Preparation Fee" } b { "{pricing::money(pricing::RV_PREPARATION_FEE)}" } } div { span { "Stationary Plus Protection" } b { "CA$50.00 / night" } } if nights>=3 { div { class:"rvd-known-price", span { "Known trip costs before delivery, add-ons & tax" } b { "{pricing::money(known)}" } } } }
            div { class:"rvd-damage-deposit", div { Icon{name:"shield-check",size:17,color:"var(--vl-forest)"} strong { "Refundable CA$1,000 damage deposit" } } p { "Separate from the trip price. Due 48 hours before delivery." } }
            button { class:"rvd-reserve", r#type:"button", onclick:move |_|{booking_step.set(if nights>=3{3}else{1});booking_open.set(true)}, "Open booking" }
        }
        if booking_open() { UnifiedBookingOverlay { location:planner_location, radius:planner_radius, starts_on:planner_starts_on, ends_on:planner_ends_on, guests:planner_guests, initial_rental_slug:Some(slug), initial_step:booking_step(), resume_after_auth:None, on_search_change:move |_|{starts_on.set(planner_starts_on().map(|d|d.to_string()).unwrap_or_default());ends_on.set(planner_ends_on().map(|d|d.to_string()).unwrap_or_default())}, on_close:move |_|booking_open.set(false) } }
    }
}

#[component]
fn TitleHead(listing: Listing) -> Element {
    let mut saved = use_signal(|| {
        api::load_json::<Vec<String>>("vl_saved_rvs")
            .unwrap_or_default()
            .iter()
            .any(|slug| slug == listing.slug)
    });
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
                button { class: "rvd-action-btn", r#type: "button", onclick: move |_| {
                    document::eval(r#"
                        try {
                            if (navigator.share) {
                                await navigator.share({ title: document.title || 'VL Rental', url: location.href });
                            } else if (navigator.clipboard) {
                                await navigator.clipboard.writeText(location.href);
                            }
                        } catch (error) {
                            if (error?.name !== 'AbortError') console.warn('Could not share this RV', error);
                        }
                    "#);
                },
                    Icon { name: "share", size: 15, color: "var(--vl-ink)" }
                    span { "Share" }
                }
                button { class: if *saved.read() { "rvd-action-btn active" } else { "rvd-action-btn" }, r#type: "button", aria_pressed: *saved.read(), onclick: move |_| {
                    let mut values = api::load_json::<Vec<String>>("vl_saved_rvs").unwrap_or_default();
                    if *saved.peek() {
                        values.retain(|slug| slug != listing.slug);
                        saved.set(false);
                    } else {
                        if !values.iter().any(|slug| slug == listing.slug) { values.push(listing.slug.to_string()); }
                        saved.set(true);
                    }
                    if values.is_empty() { api::remove_saved("vl_saved_rvs"); } else { let _ = api::save_json("vl_saved_rvs", &values); }
                },
                    Icon { name: "heart", size: 15, color: "var(--vl-ink)" }
                    span { if *saved.read() { "Saved" } else { "Save" } }
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
) -> Element {
    let resumed_booking = use_signal(api::take_booking_auth_continuation);
    let resumed_booking_value = resumed_booking.peek().clone();
    let resume_after_auth = resumed_booking_value.is_some();
    let resumed_start = resumed_booking_value.as_ref().and_then(|continuation| {
        NaiveDate::parse_from_str(&continuation.draft.starts_on, "%Y-%m-%d").ok()
    });
    let resumed_end = resumed_booking_value.as_ref().and_then(|continuation| {
        NaiveDate::parse_from_str(&continuation.draft.ends_on, "%Y-%m-%d").ok()
    });
    let resumed_guests = resumed_booking_value
        .as_ref()
        .map(|continuation| continuation.draft.guests.clamp(1, 10))
        .unwrap_or(1);
    let resumed_location = resumed_booking_value
        .as_ref()
        .map(|continuation| continuation.location.clone())
        .unwrap_or_else(|| "Kelowna, BC".to_string());
    let resumed_radius = resumed_booking_value
        .as_ref()
        .map(|continuation| continuation.radius_km.clamp(10, 150))
        .unwrap_or(150);
    let mut booking_open = use_signal(move || resume_after_auth);
    let mut booking_initial_step = use_signal(move || if resume_after_auth { 5_u8 } else { 1_u8 });
    let planner_starts_on = use_signal(move || resumed_start.or_else(|| selected_date(&starts_on)));
    let planner_ends_on = use_signal(move || resumed_end.or_else(|| selected_date(&ends_on)));
    let planner_guests = use_signal(move || resumed_guests);
    let planner_location = use_signal(move || resumed_location);
    let planner_radius = use_signal(move || resumed_radius);
    let selected_nights = selected_date(&starts_on)
        .zip(selected_date(&ends_on))
        .map(|(start, end)| (end - start).num_days())
        .unwrap_or(0);
    let rental_total = price_amount(listing.price) * selected_nights.max(0) as f64;
    let protection_total = pricing::STATIONARY_PLUS_NIGHTLY_RATE * selected_nights.max(0) as f64;
    let known_trip_cost = rental_total + pricing::mandatory_costs(selected_nights);
    let resumed_booking_value = resumed_booking.read().clone();
    let overlay_rental_slug = resumed_booking_value
        .as_ref()
        .map(|continuation| continuation.draft.rental_slug.clone())
        .filter(|slug| !slug.is_empty())
        .or_else(|| Some(listing.slug.to_string()));
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
                span { "3-night minimum · transparent pricing before confirmation" }
            }
            div { class: "rvd-unified-summary",
                button { class: "rvd-summary-dates", r#type: "button", aria_label: "Choose or change booking dates", onclick: move |_| { booking_initial_step.set(1); booking_open.set(true); },
                    span { "DATES" }
                    strong { if selected_nights >= 3 { "{starts_on} → {ends_on}" } else { "Choose dates" } }
                    Icon { name: "chevron-right", size: 15, color: "var(--vl-forest)" }
                }
                div { span { "RV" } strong { "{listing.title}" } }
            }
            div { class: "rvd-price-breakdown",
                div { class: "rvd-price-breakdown-head",
                    strong { "What makes up your trip price" }
                    span { "Exact delivery and taxes are calculated in booking" }
                }
                div { span { if selected_nights >= 3 { "Base rental · {selected_nights} nights" } else { "Base rental" } } b { if selected_nights >= 3 { "{pricing::money(rental_total)}" } else { "{listing.price} / night" } } }
                div { span { "RV Preparation Fee · one time" } b { "{pricing::money(pricing::RV_PREPARATION_FEE)}" } }
                div { span { if selected_nights >= 3 { "Stationary Plus Protection · {selected_nights} nights" } else { "Stationary Plus Protection" } } b { if selected_nights >= 3 { "{pricing::money(protection_total)}" } else { "CA$50.00 / night" } } }
                div { span { "Delivery & setup" } b { "From CA$150.00" } }
                div { span { "Optional extras" } b { "Your choice" } }
                div { span { "GST + PST" } b { "Calculated" } }
                if selected_nights >= 3 {
                    div { class: "rvd-known-price", span { "Known trip costs before delivery, extras & tax" } b { "{pricing::money(known_trip_cost)}" } }
                }
            }
            div { class: "rvd-damage-deposit",
                div { Icon { name: "shield-check", size: 17, color: "var(--vl-forest)" } strong { "Refundable CA$1,000 damage deposit" } }
                p { "Separate from the trip price and the 30% booking payment. Due 48 hours before delivery." }
            }
            div { class: "rvd-payment-schedule",
                strong { "How payment works" }
                p { "More than 30 days ahead: 30% of the trip price to confirm, then the balance 30 days before delivery. Within 30 days: the full trip price is due when booked." }
            }
            div { class: "rvd-min-pill",
                Icon { name: "truck", size: 15, color: "var(--vl-forest)" }
                span { "Dates, delivery, extras and confirmation now use one booking window." }
            }
            button { class: "rvd-reserve", onclick: move |_| { booking_initial_step.set(if selected_nights >= 3 { 3 } else { 1 }); booking_open.set(true); },
                "Open booking"
            }
            div { class: "rvd-note", "Test booking · no card charged · 3-night minimum" }
            div { class: "rvd-contact",
                Icon { name: "phone", size: 15, color: "var(--vl-forest)" }
                span { "Questions? {PHONE}" }
            }
        }
        if *booking_open.read() {
            UnifiedBookingOverlay {
                location: planner_location,
                radius: planner_radius,
                starts_on: planner_starts_on,
                ends_on: planner_ends_on,
                guests: planner_guests,
                initial_rental_slug: overlay_rental_slug,
                initial_step: *booking_initial_step.read(),
                resume_after_auth: resumed_booking_value,
                on_search_change: move |_| {
                    starts_on.set((*planner_starts_on.read()).map(|date| date.to_string()).unwrap_or_default());
                    ends_on.set((*planner_ends_on.read()).map(|date| date.to_string()).unwrap_or_default());
                    let search = api::CatalogSearchDraft {
                        location: planner_location.read().clone(),
                        radius_km: *planner_radius.read(),
                        starts_on: (*planner_starts_on.read()).map(|date| date.to_string()),
                        ends_on: (*planner_ends_on.read()).map(|date| date.to_string()),
                        guests: *planner_guests.read(),
                    };
                    let _ = api::save_json("vl_catalog_search", &search);
                },
                on_close: move |_| booking_open.set(false),
            }
        }
    }
}

// ===== Live availability calendar =====

#[cfg(any())]
fn month_start(date: NaiveDate) -> NaiveDate {
    date.with_day(1).expect("valid first day of month")
}

#[cfg(any())]
fn add_months(date: NaiveDate, count: u32) -> NaiveDate {
    date.checked_add_months(Months::new(count))
        .expect("calendar range is valid")
}

#[cfg(any())]
fn calendar_cells(month: NaiveDate) -> Vec<Option<NaiveDate>> {
    let mut cells = vec![None; month.weekday().num_days_from_sunday() as usize];
    let next = add_months(month, 1);
    let mut day = month;
    while day < next {
        cells.push(Some(day));
        day += Duration::days(1);
    }
    while cells.len() % 7 != 0 {
        cells.push(None);
    }
    cells
}

#[cfg(test)]
type UnavailableRange = (DateTime<Utc>, DateTime<Utc>);

#[cfg(any())]
fn unavailable_ranges(value: &api::AvailabilityResponse) -> Vec<UnavailableRange> {
    let mut ranges = Vec::new();
    for interval in &value.unavailable {
        let Ok(start) = chrono::DateTime::parse_from_rfc3339(&interval.starts_at) else {
            continue;
        };
        let Ok(end) = chrono::DateTime::parse_from_rfc3339(&interval.ends_at) else {
            continue;
        };
        ranges.push((start.with_timezone(&Utc), end.with_timezone(&Utc)));
    }
    ranges
}

#[cfg(test)]
fn local_moment(day: NaiveDate, hour: u32) -> Option<DateTime<Utc>> {
    let naive = day.and_hms_opt(hour, 0, 0)?;
    match Vancouver.from_local_datetime(&naive) {
        LocalResult::Single(value) => Some(value.with_timezone(&Utc)),
        _ => None,
    }
}

#[cfg(test)]
fn range_is_available(
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    unavailable: &[UnavailableRange],
) -> bool {
    unavailable
        .iter()
        .all(|(blocked_start, blocked_end)| *blocked_start >= end || *blocked_end <= start)
}

#[cfg(test)]
fn stay_is_available(
    starts_on: NaiveDate,
    ends_on: NaiveDate,
    unavailable: &[UnavailableRange],
) -> bool {
    match (local_moment(starts_on, 14), local_moment(ends_on, 11)) {
        (Some(start), Some(end)) => range_is_available(start, end, unavailable),
        _ => false,
    }
}

#[cfg(test)]
fn minimum_stay_can_start(
    day: NaiveDate,
    minimum_nights: i64,
    unavailable: &[UnavailableRange],
) -> bool {
    stay_is_available(day, day + Duration::days(minimum_nights), unavailable)
}

#[cfg(test)]
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

#[cfg(test)]
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
#[allow(clippy::items_after_test_module)]
mod availability_tests {
    use super::*;

    fn day(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).unwrap()
    }

    #[test]
    fn previous_return_day_is_available_for_delivery() {
        let return_day = day(2030, 8, 10);
        let blocked = vec![(
            local_moment(day(2030, 8, 1), 14).unwrap(),
            local_moment(return_day, 11).unwrap(),
        )];
        assert!(minimum_stay_can_start(return_day, 3, &blocked));
    }

    #[test]
    fn next_delivery_day_is_available_for_return() {
        let turnover_day = day(2030, 8, 10);
        let blocked = vec![(
            local_moment(turnover_day, 14).unwrap(),
            local_moment(day(2030, 8, 13), 11).unwrap(),
        )];
        assert!(stay_is_available(day(2030, 8, 7), turnover_day, &blocked));
    }

    #[test]
    fn partial_afternoon_block_prevents_delivery() {
        let blocked_day = day(2030, 8, 10);
        let blocked = vec![(
            local_moment(blocked_day, 15).unwrap(),
            local_moment(blocked_day, 16).unwrap(),
        )];
        assert!(!minimum_stay_can_start(blocked_day, 3, &blocked));
    }

    #[test]
    fn earlier_available_day_can_replace_delivery() {
        let selected = day(2030, 8, 10);
        assert!(date_is_selectable(
            day(2030, 8, 8),
            Some(selected),
            None,
            3,
            &[]
        ));
    }

    #[test]
    fn clicking_selected_delivery_clears_selection() {
        let selected = day(2030, 8, 10);
        assert_eq!(
            next_date_selection(selected, Some(selected), None),
            (None, None)
        );
    }

    #[test]
    fn earlier_day_restarts_selection() {
        let selected = day(2030, 8, 10);
        let earlier = day(2030, 8, 8);
        assert_eq!(
            next_date_selection(earlier, Some(selected), None),
            (Some(earlier), None)
        );
    }
}

fn selected_date(value: &Signal<String>) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(&value.read(), "%Y-%m-%d").ok()
}

#[allow(dead_code)]
fn price_amount(price: &str) -> f64 {
    price
        .chars()
        .filter(|value| value.is_ascii_digit() || *value == '.')
        .collect::<String>()
        .parse()
        .unwrap_or(0.0)
}

#[cfg(any())]
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
    let mut address_suggestions = use_signal(Vec::<api::AddressSuggestion>::new);
    let mut suggestions_busy = use_signal(|| false);
    let mut suggestions_error = use_signal(String::new);
    let mut suggestions_open = use_signal(|| false);
    let mut suggestion_version = use_signal(|| 0_u32);
    let today = Utc::now().with_timezone(&Vancouver).date_naive();
    let initial_month = month_start(today);
    let mut visible_month = use_signal(|| {
        selected_date(&starts_on)
            .map(month_start)
            .unwrap_or(initial_month)
    });
    let availability = use_resource(move || {
        let _version = *availability_version.read();
        async move {
            api::availability(
                slug,
                &initial_month.to_string(),
                &add_months(initial_month, 18).to_string(),
            )
            .await
        }
    });
    let rental_details = use_resource(move || async move { api::rental(slug).await });
    let response = availability
        .read()
        .as_ref()
        .and_then(|result| result.as_ref().ok())
        .cloned();
    let availability_loaded = response.is_some();
    let unavailable = response
        .as_ref()
        .map(unavailable_ranges)
        .unwrap_or_default();
    let minimum_nights = response
        .as_ref()
        .map(|value| value.minimum_nights)
        .unwrap_or(3);
    let calendar_error = availability
        .read()
        .as_ref()
        .and_then(|result| result.as_ref().err())
        .cloned();
    let nights = selected_date(&starts_on)
        .zip(selected_date(&ends_on))
        .map(|(start, end)| (end - start).num_days())
        .unwrap_or(0);
    let details = rental_details
        .read()
        .as_ref()
        .and_then(|result| result.as_ref().ok())
        .cloned();
    let capacity = details
        .as_ref()
        .map(|value| value.rental.capacity)
        .unwrap_or(10);
    let addons = details
        .as_ref()
        .map(|value| value.addons.clone())
        .unwrap_or_default();
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
                return Err(api::ApiError::client("Choose complete dates"));
            }
            if !api::rv_delivery_ready(&draft) {
                return Err(api::ApiError::client(
                    "Enter and calculate the delivery address",
                ));
            }
            api::create_quote(&draft).await
        }
    });
    let quote_response = live_quote
        .read()
        .as_ref()
        .and_then(|result| result.as_ref().ok())
        .cloned();
    let quote_error = live_quote
        .read()
        .as_ref()
        .and_then(|result| result.as_ref().err())
        .cloned();
    let calculate_address = move |_| {
        let address = delivery_address.read().trim().to_string();
        async move {
            suggestions_open.set(false);
            if address.chars().count() < 5 {
                delivery_km.set(None);
                address_result.set(None);
                address_error.set("Enter a complete street address or campsite.".into());
                return;
            }
            address_busy.set(true);
            address_error.set(String::new());
            match api::delivery_estimate(slug, &address).await {
                Ok(result) if result.within_range => {
                    delivery_address.set(result.resolved_address.clone());
                    delivery_km.set(Some(result.one_way_km.clone()));
                    address_result.set(Some(result));
                }
                Ok(result) => {
                    delivery_km.set(None);
                    address_result.set(Some(result.clone()));
                    address_error.set(format!(
                        "This address is beyond the {} km delivery limit.",
                        result.maximum_km
                    ));
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
    let suggestion_items = address_suggestions.read().clone();
    let address_has_query = delivery_address.read().trim().chars().count() >= 3;
    let show_suggestions = *suggestions_open.read() && address_has_query;

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
                                    div { class: "rvd-address-combobox",
                                        div { class: "rvd-address-input-wrap",
                                            Icon { name: "map-pin", size: 17, color: "var(--vl-muted)" }
                                            input {
                                                value: "{delivery_address}",
                                                placeholder: "Start typing a delivery address",
                                                autocomplete: "off",
                                                spellcheck: "false",
                                                role: "combobox",
                                                aria_label: "Delivery address",
                                                aria_autocomplete: "list",
                                                aria_expanded: show_suggestions,
                                                aria_controls: "rvd-address-suggestions",
                                                onfocus: move |_| {
                                                    if delivery_address.read().trim().chars().count() >= 3 {
                                                        suggestions_open.set(true);
                                                    }
                                                },
                                                oninput: move |event| {
                                                    let value = event.value();
                                                    delivery_address.set(value.clone());
                                                    delivery_km.set(None);
                                                    address_result.set(None);
                                                    address_error.set(String::new());
                                                    suggestions_error.set(String::new());
                                                    let version = suggestion_version.peek().wrapping_add(1);
                                                    suggestion_version.set(version);
                                                    if value.trim().chars().count() < 3 {
                                                        address_suggestions.set(Vec::new());
                                                        suggestions_busy.set(false);
                                                        suggestions_open.set(false);
                                                        return;
                                                    }
                                                    suggestions_busy.set(true);
                                                    suggestions_open.set(true);
                                                    spawn(async move {
                                                        let _ = document::eval("await new Promise(resolve => setTimeout(resolve, 650));").await;
                                                        if *suggestion_version.peek() != version { return; }
                                                        match api::address_suggestions(&value).await {
                                                            Ok(items) => address_suggestions.set(items),
                                                            Err(message) => {
                                                                address_suggestions.set(Vec::new());
                                                                suggestions_error.set(message);
                                                            }
                                                        }
                                                        if *suggestion_version.peek() == version {
                                                            suggestions_busy.set(false);
                                                        }
                                                    });
                                                }
                                            }
                                            if *suggestions_busy.read() {
                                                span { class: "rvd-address-spinner", aria_label: "Searching addresses" }
                                            }
                                        }
                                        if show_suggestions {
                                            div { id: "rvd-address-suggestions", class: "rvd-address-suggestions", role: "listbox",
                                                if !suggestion_items.is_empty() {
                                                    for suggestion in suggestion_items {
                                                        button {
                                                            key: "{suggestion.display_name}",
                                                            class: "rvd-address-suggestion",
                                                            r#type: "button",
                                                            role: "option",
                                                            onclick: move |_| {
                                                                delivery_address.set(suggestion.display_name.clone());
                                                                delivery_km.set(None);
                                                                address_result.set(None);
                                                                address_error.set(String::new());
                                                                address_suggestions.set(Vec::new());
                                                                suggestions_open.set(false);
                                                            },
                                                            span { class: "rvd-address-suggestion-icon", Icon { name: "map-pin", size: 16, color: "var(--vl-forest)" } }
                                                            span { class: "rvd-address-suggestion-copy",
                                                                strong { "{suggestion.primary_text}" }
                                                                small { "{suggestion.secondary_text}" }
                                                            }
                                                        }
                                                    }
                                                } else if !*suggestions_busy.read() && suggestions_error.read().is_empty() {
                                                    div { class: "rvd-address-suggestion-status", "No matching Canadian address found. Keep typing or check the spelling." }
                                                } else if !suggestions_error.read().is_empty() {
                                                    div { class: "rvd-address-suggestion-status is-error", "{suggestions_error}" }
                                                }
                                                div { class: "rvd-address-suggestions-foot", "Canadian addresses · results prioritized near Kelowna" }
                                            }
                                        }
                                    }
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
                                    span { Icon { name: "shield-check", size: 13, color: "var(--vl-accent)" } "Refundable damage deposit" }
                                    span { Icon { name: "check", size: 13, color: "var(--vl-accent)" } "Transparent pricing" }
                                }
                            }
                        }
                        div { class: "rvd-calendar-summary",
                            div { class: "rvd-calendar-total-label", "TRIP PRICE" }
                            if let Some(quote) = quote_response.as_ref() {
                                div { class: "rvd-calendar-total", "{pricing::money(pricing::quote_trip_price(quote))}" }
                                div { class: "rvd-calendar-total-note", "Preparation, protection, selected extras, delivery, GST and PST included" }
                                div { class: "rvd-calendar-deposit-note", "Refundable CA$1,000 damage deposit · separate · charged 48 hours before delivery" }
                            } else {
                                div { class: "rvd-calendar-total", "Address required" }
                                div { class: "rvd-calendar-total-note", "Calculate the delivery address to receive the exact trip price" }
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

#[cfg(any())]
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
                            let delivery_has_passed = day == today && now.hour() >= 14;
                            let unavailable_day = !availability_loaded || day < today || delivery_has_passed || !valid_choice;
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

use chrono::{DateTime, LocalResult, Months, NaiveDate, TimeZone, Utc};
use chrono_tz::America::Vancouver;
use dioxus::prelude::*;

use super::catalog::{add_months, month_start, rental_image, CatalogSearchMonth};
use crate::{api, components::Icon, data::rv_gallery, pricing, Route};

const SAVED_DELIVERY_ADDRESSES: &str = "vl_delivery_addresses";
const MAX_SAVED_DELIVERY_ADDRESSES: usize = 5;
const SAVED_PENDING_PAYMENT: &str = "vl_pending_booking_payment";
const SAVED_POST_PAYMENT_BOOKING: &str = "vl_post_payment_booking";
const UNMOUNT_EMBEDDED_CHECKOUT: &str = r#"
if (window.__vlEmbeddedCheckout) {
  try { window.__vlEmbeddedCheckout.unmount(); } catch (_) {}
  window.__vlEmbeddedCheckout = null;
}
"#;

type UnavailableRange = (DateTime<Utc>, DateTime<Utc>);

#[derive(Clone)]
struct DatedRentalMatches {
    starts_on: Option<NaiveDate>,
    ends_on: Option<NaiveDate>,
    guests: i32,
    result: Result<Vec<api::Rental>, String>,
}

fn availability_ranges(value: &api::AvailabilityResponse) -> Vec<UnavailableRange> {
    value
        .unavailable
        .iter()
        .filter_map(|interval| {
            let start = DateTime::parse_from_rfc3339(&interval.starts_at).ok()?;
            let end = DateTime::parse_from_rfc3339(&interval.ends_at).ok()?;
            Some((start.with_timezone(&Utc), end.with_timezone(&Utc)))
        })
        .collect()
}

fn booking_moment(day: NaiveDate, hour: u32) -> Option<DateTime<Utc>> {
    let naive = day.and_hms_opt(hour, 0, 0)?;
    match Vancouver.from_local_datetime(&naive) {
        LocalResult::Single(value) => Some(value.with_timezone(&Utc)),
        _ => None,
    }
}

fn booking_stay_is_available(
    starts_on: NaiveDate,
    ends_on: NaiveDate,
    unavailable: &[UnavailableRange],
) -> bool {
    let (Some(start), Some(end)) = (booking_moment(starts_on, 14), booking_moment(ends_on, 11))
    else {
        return false;
    };
    unavailable
        .iter()
        .all(|(blocked_start, blocked_end)| *blocked_start >= end || *blocked_end <= start)
}

fn booking_date_is_selectable(
    day: NaiveDate,
    selected_start: Option<NaiveDate>,
    selected_end: Option<NaiveDate>,
    minimum_nights: i64,
    unavailable: &[UnavailableRange],
) -> bool {
    if let (Some(start), None) = (selected_start, selected_end) {
        if day <= start {
            return booking_stay_is_available(
                day,
                day + chrono::Duration::days(minimum_nights),
                unavailable,
            );
        }
        return (day - start).num_days() >= minimum_nights
            && booking_stay_is_available(start, day, unavailable);
    }
    booking_stay_is_available(
        day,
        day + chrono::Duration::days(minimum_nights),
        unavailable,
    )
}

fn step_after_rental_selection(trip_ready: bool) -> u8 {
    if trip_ready {
        3
    } else {
        1
    }
}

fn initial_booking_step(initial_step: u8, has_pending_payment: bool) -> u8 {
    if has_pending_payment {
        5
    } else {
        initial_step.clamp(1, 5)
    }
}

fn money_cents(value: &str) -> Option<i64> {
    value
        .trim()
        .parse::<f64>()
        .ok()
        .filter(|amount| amount.is_finite() && *amount >= 0.0)
        .map(|amount| (amount * 100.0).round() as i64)
}

fn booking_payment_amount_is_valid(booking: &api::Booking) -> bool {
    let Some(total) = money_cents(&booking.total) else {
        return false;
    };
    let Some(due_now) = money_cents(&booking.amount_due_now) else {
        return false;
    };
    let thirty_percent = (total * i64::from(pricing::BOOKING_DEPOSIT_PERCENT) + 50) / 100;
    due_now == total || due_now == thirty_percent
}

fn embedded_checkout_script(publishable_key: &str, client_secret: &str) -> String {
    format!(
        r#"
return await (async () => {{
  if (!window.Stripe) {{
    await new Promise((resolve, reject) => {{
      const current = document.querySelector('script[data-vl-stripe]');
      if (current) {{
        current.addEventListener('load', resolve, {{ once: true }});
        current.addEventListener('error', reject, {{ once: true }});
        if (window.Stripe) resolve();
        return;
      }}
      const script = document.createElement('script');
      script.src = 'https://js.stripe.com/v3/';
      script.async = true;
      script.dataset.vlStripe = 'true';
      script.onload = resolve;
      script.onerror = reject;
      document.head.appendChild(script);
    }});
  }}
  const root = document.getElementById('vl-embedded-checkout');
  if (!root || !window.Stripe) throw new Error('Stripe Checkout is unavailable');
  if (window.__vlEmbeddedCheckout) {{
    try {{ window.__vlEmbeddedCheckout.unmount(); }} catch (_) {{}}
  }}
  root.replaceChildren();
  const stripe = window.Stripe({publishable_key});
  const checkout = await stripe.initEmbeddedCheckout({{
    clientSecret: {client_secret},
    onComplete: () => {{ root.dataset.complete = 'true'; }}
  }});
  checkout.mount('#vl-embedded-checkout');
  window.__vlEmbeddedCheckout = checkout;
  return 'mounted';
}})();
"#
    )
}

fn rental_selection_keeps_dates(trip_ready: bool, available_for_dates: Option<bool>) -> bool {
    trip_ready && available_for_dates == Some(true)
}

const BOOKING_DELIVERY_MAP_SCRIPT: &str = r#"
await (async () => {
    const root = document.querySelector('.ub-delivery-map');
    const container = document.getElementById('vl-booking-delivery-map');
    if (!root || !container) return;

    const fallback = root.querySelector('.ub-map-fallback span');
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
        const destinationAddress = __DESTINATION_ADDRESS__;
        let state = window.__vlBookingDeliveryMapState;
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
                radius: 150000,
                color: '#174D32',
                weight: 2,
                opacity: 0.92,
                fillColor: '#4A8D63',
                fillOpacity: 0.18,
            }).addTo(map);
            L.circleMarker(base, {
                radius: 7,
                color: '#FFFFFF',
                weight: 3,
                fillColor: '#174D32',
                fillOpacity: 1,
            }).addTo(map).bindTooltip('VL Rental · Kelowna', {
                permanent: true,
                direction: 'top',
                offset: [0, -9],
                className: 'vl-base-tooltip',
            });
            state = { map, zone, container, destination: null, routeLine: null, address: '' };
            window.__vlBookingDeliveryMapState = state;
        }

        state.map.invalidateSize(false);
        if (destinationAddress && state.address !== destinationAddress) {
            if (state.destination) state.destination.remove();
            if (state.routeLine) state.routeLine.remove();
            state.destination = null;
            state.routeLine = null;
            state.address = '';
            const mapAddress = /(?:^|,)\s*canada\s*$/i.test(destinationAddress)
                ? destinationAddress
                : `${destinationAddress}, Canada`;
            const response = await fetch(`https://photon.komoot.io/api/?q=${encodeURIComponent(mapAddress)}&limit=1&lang=en&lat=${base[0]}&lon=${base[1]}`);
            if (response.ok) {
                const payload = await response.json();
                const coordinates = payload?.features?.[0]?.geometry?.coordinates;
                if (Array.isArray(coordinates) && coordinates.length >= 2) {
                    const destination = [coordinates[1], coordinates[0]];
                    state.destination = L.circleMarker(destination, {
                        radius: 7,
                        color: '#FFFFFF',
                        weight: 3,
                        fillColor: '#D9A441',
                        fillOpacity: 1,
                    }).addTo(state.map).bindTooltip('Selected delivery address', {
                        direction: 'top',
                        offset: [0, -8],
                        className: 'vl-base-tooltip',
                    });
                    state.routeLine = L.polyline([base, destination], {
                        color: '#D9A441',
                        weight: 3,
                        opacity: 0.9,
                        dashArray: '7 7',
                    }).addTo(state.map);
                    state.map.fitBounds(L.latLngBounds([base, destination]), { padding: [30, 30], maxZoom: 11 });
                    state.address = destinationAddress;
                }
            }
        } else if (!destinationAddress) {
            if (state.destination) state.destination.remove();
            if (state.routeLine) state.routeLine.remove();
            state.destination = null;
            state.routeLine = null;
            state.address = '';
            state.map.fitBounds(state.zone.getBounds(), { padding: [22, 22], maxZoom: 10 });
        }

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

const LOCK_PAGE_SCROLL: &str = r#"
(() => {
    const body = document.body;
    const root = document.documentElement;
    window.__vlBookingScrollLocks = (window.__vlBookingScrollLocks || 0) + 1;
    if (window.__vlBookingScrollLocks > 1) return;

    window.__vlBookingScrollState = {
        y: window.scrollY,
        bodyOverflow: body.style.overflow,
        bodyPosition: body.style.position,
        bodyTop: body.style.top,
        bodyWidth: body.style.width,
        rootOverscroll: root.style.overscrollBehavior
    };

    body.style.overflow = 'hidden';
    body.style.position = 'fixed';
    body.style.top = '-' + window.__vlBookingScrollState.y + 'px';
    body.style.width = '100%';
    root.style.overscrollBehavior = 'none';
})();
"#;

const UNLOCK_PAGE_SCROLL: &str = r#"
(() => {
    window.__vlBookingScrollLocks = Math.max(0, (window.__vlBookingScrollLocks || 1) - 1);
    if (window.__vlBookingScrollLocks > 0) return;

    const body = document.body;
    const root = document.documentElement;
    const state = window.__vlBookingScrollState;
    if (!state) return;

    body.style.overflow = state.bodyOverflow;
    body.style.position = state.bodyPosition;
    body.style.top = state.bodyTop;
    body.style.width = state.bodyWidth;
    root.style.overscrollBehavior = state.rootOverscroll;
    window.scrollTo(0, state.y || 0);
    delete window.__vlBookingScrollState;
})();
"#;

fn remember_delivery_address(saved: &[String], address: &str) -> Vec<String> {
    let address = address.trim();
    if address.is_empty() {
        return saved.to_vec();
    }

    let mut next = vec![address.to_string()];
    next.extend(
        saved
            .iter()
            .filter(|value| !value.trim().eq_ignore_ascii_case(address))
            .cloned(),
    );
    next.truncate(MAX_SAVED_DELIVERY_ADDRESSES);
    next
}

fn forget_delivery_address(saved: &[String], address: &str) -> Vec<String> {
    saved
        .iter()
        .filter(|value| !value.trim().eq_ignore_ascii_case(address.trim()))
        .cloned()
        .collect()
}

fn date_text(value: Option<NaiveDate>) -> String {
    value.map(|date| date.to_string()).unwrap_or_default()
}

fn display_booking_date(timestamp: &str) -> String {
    DateTime::parse_from_rfc3339(timestamp)
        .map(|value| value.with_timezone(&Vancouver).date_naive().to_string())
        .unwrap_or_else(|_| timestamp.get(0..10).unwrap_or(timestamp).to_string())
}

fn price_number(value: &str) -> f64 {
    value
        .chars()
        .filter(|character| character.is_ascii_digit() || *character == '.')
        .collect::<String>()
        .parse()
        .unwrap_or(0.0)
}

#[derive(Clone, Debug, PartialEq)]
struct OptimisticPriceLine {
    key: String,
    label: String,
    detail: Option<String>,
    amount: f64,
}

#[derive(Clone, Debug, PartialEq)]
struct OptimisticPrice {
    lines: Vec<OptimisticPriceLine>,
    total: f64,
}

fn optimistic_price(
    rental: Option<&api::Rental>,
    details: Option<&api::RentalResponse>,
    selected_addons: &[String],
    nights: i64,
    delivery: Option<&api::DeliveryEstimate>,
    previous_quote: Option<&api::QuoteResponse>,
) -> Option<OptimisticPrice> {
    let rental = rental?;
    let delivery = delivery.filter(|value| value.within_range)?;
    let nights = nights.max(0);
    let rental_amount = price_number(&rental.base_rate) * nights as f64;
    let mut lines = vec![OptimisticPriceLine {
        key: format!("rental-{}", rental.slug),
        label: format!("{} x {} night", rental.name, nights),
        detail: None,
        amount: rental_amount,
    }];
    let mut taxable_subtotal = rental_amount;

    if let Some(details) = details {
        for addon in details
            .addons
            .iter()
            .filter(|addon| selected_addons.contains(&addon.addon_key))
        {
            let quantity = if addon.charge_type == "per_unit" {
                nights as f64
            } else {
                1.0
            };
            let amount = price_number(&addon.price) * quantity;
            taxable_subtotal += amount;
            lines.push(OptimisticPriceLine {
                key: format!("addon-{}", addon.addon_key),
                label: addon.label.clone(),
                detail: (quantity > 1.0).then(|| format!("{} nights × CA${}", nights, addon.price)),
                amount,
            });
        }
    }

    let previous_fees = previous_quote
        .map(|value| {
            value
                .items
                .iter()
                .filter(|item| item.item_type == "fee")
                .map(|item| {
                    (
                        item.item_key.clone(),
                        item.label.clone(),
                        price_number(&item.amount),
                    )
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    if previous_fees.is_empty() {
        taxable_subtotal += pricing::RV_PREPARATION_FEE;
        lines.push(OptimisticPriceLine {
            key: "fee-rv-preparation".into(),
            label: "RV Preparation Fee".into(),
            detail: None,
            amount: pricing::RV_PREPARATION_FEE,
        });
    } else {
        for (key, label, amount) in previous_fees {
            taxable_subtotal += amount;
            lines.push(OptimisticPriceLine {
                key: format!("fee-{key}"),
                label,
                detail: None,
                amount,
            });
        }
    }

    let delivery_amount = price_number(&delivery.delivery_fee);
    taxable_subtotal += delivery_amount;
    lines.push(OptimisticPriceLine {
        key: "delivery".into(),
        label: "Delivery and setup".into(),
        detail: None,
        amount: delivery_amount,
    });

    let protection = pricing::STATIONARY_PLUS_NIGHTLY_RATE * nights as f64;
    lines.push(OptimisticPriceLine {
        key: "stationary-plus".into(),
        label: "Stationary Plus Protection".into(),
        detail: Some(format!(
            "{} nights × {}",
            nights,
            pricing::money(pricing::STATIONARY_PLUS_NIGHTLY_RATE)
        )),
        amount: protection,
    });

    let tax_rate = previous_quote
        .and_then(|value| {
            let previous_taxable = value
                .items
                .iter()
                .filter(|item| {
                    matches!(
                        item.item_type.as_str(),
                        "rental" | "addon" | "fee" | "delivery"
                    )
                })
                .map(|item| price_number(&item.amount))
                .sum::<f64>();
            let previous_tax = value
                .items
                .iter()
                .filter(|item| item.item_type == "tax")
                .map(|item| price_number(&item.amount))
                .sum::<f64>();
            (previous_taxable > 0.0).then_some(previous_tax / previous_taxable)
        })
        .unwrap_or(0.0);
    let tax = (taxable_subtotal * tax_rate * 100.0).round() / 100.0;
    if tax > 0.0 {
        lines.push(OptimisticPriceLine {
            key: "tax".into(),
            label: "Applicable taxes".into(),
            detail: None,
            amount: tax,
        });
    }

    Some(OptimisticPrice {
        total: taxable_subtotal + protection + tax,
        lines,
    })
}

fn booking_delivery_map_script(address: Option<&str>) -> String {
    let address_json =
        serde_json::to_string(address.unwrap_or_default()).unwrap_or_else(|_| "\"\"".to_string());
    BOOKING_DELIVERY_MAP_SCRIPT.replace("__DESTINATION_ADDRESS__", &address_json)
}

async fn scroll_to_booking_step(number: u8) {
    let script = format!(
        r#"
await new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve)));
const target = document.getElementById('ub-step-{number}');
if (target) {{
    target.scrollIntoView({{ behavior: 'smooth', block: 'start', inline: 'nearest' }});
    target.querySelector('.ub-step-head')?.focus({{ preventScroll: true }});
}}
"#
    );
    let _ = document::eval(&script).await;
}

fn addon_icon(key: &str) -> &'static str {
    if key.contains("bbq") || key.contains("bedding") {
        "sparkles"
    } else if key.contains("propane") {
        "zap"
    } else if key.contains("septic") {
        "check-circle-2"
    } else if key.contains("pet") {
        "paw-print"
    } else if key.contains("generator") {
        "zap"
    } else {
        "sparkles"
    }
}

fn addon_description(key: &str) -> &'static str {
    if key.contains("bbq") {
        "Ready for outdoor meals at your campsite; fuel not included."
    } else if key.contains("bedding") {
        "Fresh linens prepared before delivery."
    } else if key.contains("propane") {
        "Prepaid refill for an easier return."
    } else if key.contains("septic") {
        "We handle the final tank emptying."
    } else if key.contains("pet") {
        "Required when a pet joins your stay."
    } else if key.contains("generator") {
        "Portable backup power; fuel not included."
    } else if key.contains("dirt") {
        "Coverage for unusually heavy interior cleaning."
    } else if key.contains("exterior") {
        "Exterior wash after your Okanagan stay."
    } else {
        "Optional service added to this booking."
    }
}

#[allow(clippy::too_many_arguments)]
fn make_draft(
    slug: &str,
    starts_on: Option<NaiveDate>,
    ends_on: Option<NaiveDate>,
    guests: i32,
    address: &str,
    distance: Option<String>,
    addon_keys: Vec<String>,
    attending_event: bool,
    towing_after_delivery: bool,
) -> api::TripDraft {
    api::TripDraft {
        rental_slug: slug.to_string(),
        starts_on: date_text(starts_on),
        ends_on: date_text(ends_on),
        guests,
        addon_keys,
        delivery_km: distance,
        delivery_address: Some(address.trim().to_string()),
        attending_event,
        towing_after_delivery,
    }
}

#[component]
fn AnimatedMoney(id: &'static str, amount: f64) -> Element {
    let formatted = pricing::money(amount);
    use_effect(use_reactive((&amount,), move |(amount,)| {
        let id = serde_json::to_string(id).unwrap_or_else(|_| "\"ub-money\"".into());
        let script = format!(
            r#"
(function() {{
  const id = {id};
  const element = document.getElementById(id);
  if (!element) return;

  const target = {amount:.2};
  const values = window.__vlBookingMoneyValues ||= {{}};
  const frames = window.__vlBookingMoneyFrames ||= {{}};
  const format = value => 'CA$' + value.toLocaleString('en-CA', {{
    minimumFractionDigits: 2,
    maximumFractionDigits: 2
  }});

  if (frames[id]) cancelAnimationFrame(frames[id]);
  const current = Number(values[id]);
  if (!Number.isFinite(current) || window.matchMedia('(prefers-reduced-motion: reduce)').matches) {{
    values[id] = target;
    element.textContent = format(target);
    return;
  }}

  const startedAt = performance.now();
  const duration = 520;
  const change = target - current;
  element.textContent = format(current);

  const tick = now => {{
    const progress = Math.min(1, (now - startedAt) / duration);
    const eased = 1 - Math.pow(1 - progress, 3);
    const value = current + change * eased;
    values[id] = value;
    element.textContent = format(value);
    if (progress < 1) {{
      frames[id] = requestAnimationFrame(tick);
    }} else {{
      values[id] = target;
      delete frames[id];
      element.textContent = format(target);
    }}
  }};
  frames[id] = requestAnimationFrame(tick);
}})();
"#
        );
        spawn(async move {
            let _ = document::eval(&script).await;
        });
    }));

    rsx! {
        span { id, class: "ub-money-value", "{formatted}" }
    }
}

#[component]
pub(crate) fn UnifiedBookingOverlay(
    mut location: Signal<String>,
    mut radius: Signal<i32>,
    mut starts_on: Signal<Option<NaiveDate>>,
    mut ends_on: Signal<Option<NaiveDate>>,
    mut guests: Signal<i32>,
    #[props(default)] initial_rental_slug: Option<String>,
    #[props(default = 1)] initial_step: u8,
    #[props(default)] resume_after_auth: Option<api::BookingAuthContinuation>,
    on_search_change: EventHandler<()>,
    on_close: EventHandler<()>,
) -> Element {
    let resumed_draft = resume_after_auth
        .as_ref()
        .map(|continuation| continuation.draft.clone());
    let resumed_booking = resume_after_auth.is_some();
    let resumed_address = resumed_draft
        .as_ref()
        .and_then(|draft| draft.delivery_address.clone())
        .unwrap_or_default();
    let resumed_distance = resumed_draft
        .as_ref()
        .and_then(|draft| draft.delivery_km.clone());
    let resumed_addons = resumed_draft
        .as_ref()
        .map(|draft| draft.addon_keys.clone())
        .unwrap_or_default();
    let resumed_delivery_estimate = resume_after_auth
        .as_ref()
        .and_then(|continuation| continuation.delivery_estimate.clone());
    let current_user = api::current_user();
    let resumed_first_name = resume_after_auth
        .as_ref()
        .map(|continuation| continuation.first_name.clone())
        .filter(|value| !value.trim().is_empty())
        .or_else(|| current_user.as_ref().map(|value| value.first_name.clone()))
        .unwrap_or_default();
    let resumed_last_name = resume_after_auth
        .as_ref()
        .map(|continuation| continuation.last_name.clone())
        .filter(|value| !value.trim().is_empty())
        .or_else(|| current_user.as_ref().map(|value| value.last_name.clone()))
        .unwrap_or_default();
    let resumed_phone = resume_after_auth
        .as_ref()
        .map(|continuation| continuation.phone.clone())
        .filter(|value| !value.trim().is_empty())
        .or_else(|| current_user.as_ref().map(|value| value.phone.clone()))
        .unwrap_or_default();
    let resumed_notes = resume_after_auth
        .as_ref()
        .map(|continuation| continuation.notes.clone())
        .unwrap_or_default();
    let resumed_accepted_terms = resume_after_auth
        .as_ref()
        .is_some_and(|continuation| continuation.accepted_terms);
    let resumed_booking_email = resume_after_auth
        .as_ref()
        .map(|continuation| continuation.booking_email.trim().to_string())
        .filter(|email| !email.is_empty())
        .or_else(|| current_user.as_ref().map(|value| value.email.clone()))
        .unwrap_or_default();
    let initial_contact_complete = resumed_first_name.trim().len() >= 2
        && resumed_last_name.trim().len() >= 2
        && resumed_phone.trim().len() >= 7;
    let resumed_delivery_check = resumed_draft.as_ref().and_then(|draft| {
        let address = draft.delivery_address.as_deref()?.trim();
        (!draft.rental_slug.is_empty() && !address.is_empty())
            .then(|| (draft.rental_slug.clone(), address.to_string()))
    });

    // A new booking window starts with an unfiltered guest count. Do not let a
    // value persisted by an earlier catalog search silently hide smaller RVs.
    use_effect(move || {
        if !resumed_booking {
            guests.set(1);
        }
    });
    use_effect(|| {
        document::eval(LOCK_PAGE_SCROLL);
    });
    use_drop(|| {
        document::eval(UNLOCK_PAGE_SCROLL);
        document::eval(UNMOUNT_EMBEDDED_CHECKOUT);
    });

    let today = Utc::now().with_timezone(&Vancouver).date_naive();
    let initial_month = month_start(today);
    let initial_pending_payment = api::load_json::<api::CreatedBooking>(SAVED_PENDING_PAYMENT)
        .filter(|created| created.client_secret.is_some() && !created.access_token.is_empty());
    let has_pending_payment = initial_pending_payment.is_some();
    let mut visible_month = use_signal(|| {
        (*starts_on.read())
            .map(month_start)
            .unwrap_or(initial_month)
    });
    let initial_slug = resumed_draft
        .as_ref()
        .map(|draft| draft.rental_slug.clone())
        .filter(|slug| !slug.is_empty())
        .or(initial_rental_slug)
        .unwrap_or_default();
    let resumed_initial_step = if resumed_booking { 5 } else { initial_step };
    let mut open_step =
        use_signal(move || initial_booking_step(resumed_initial_step, has_pending_payment));
    let mut closing = use_signal(|| false);
    let mut selected_slug = use_signal(move || initial_slug);
    let mut delivery_address = use_signal(move || resumed_address);
    let mut delivery_km = use_signal(move || resumed_distance);
    let mut delivery_result = use_signal(move || resumed_delivery_estimate);
    let mut address_error = use_signal(String::new);
    let mut address_busy = use_signal(|| false);
    let mut suggestions_open = use_signal(|| false);
    let mut saved_addresses =
        use_signal(|| api::load_json::<Vec<String>>(SAVED_DELIVERY_ADDRESSES).unwrap_or_default());
    let mut addon_keys = use_signal(move || resumed_addons);
    let mut quote = use_signal(|| None::<api::QuoteResponse>);
    let mut quote_busy = use_signal(|| false);
    let mut quote_error = use_signal(String::new);
    let mut quote_version = use_signal(|| 0_u32);
    let mut quote_refresh_nonce = use_signal(|| 0_u32);
    let mut user = use_signal(move || current_user);
    let mut auth_profile_loaded = use_signal(|| false);
    let mut contact_editing = use_signal(move || !initial_contact_complete);
    let mut auth_email = use_signal(String::new);
    let mut auth_password = use_signal(String::new);
    let mut auth_register = use_signal(|| false);
    let mut auth_busy = use_signal(|| false);
    let mut auth_error = use_signal(String::new);
    let mut first_name = use_signal(move || resumed_first_name);
    let mut last_name = use_signal(move || resumed_last_name);
    let mut booking_email = use_signal(move || resumed_booking_email);
    let mut phone = use_signal(move || resumed_phone);
    let mut notes = use_signal(move || resumed_notes);
    let mut accepted = use_signal(move || resumed_accepted_terms);
    let mut booking_busy = use_signal(|| false);
    let mut booking_error = use_signal(String::new);
    let mut payment_config = use_signal(|| None::<api::PaymentConfig>);
    let mut payment_config_error = use_signal(String::new);
    let mut payment_config_retry = use_signal(|| 0_u32);
    let mut pending_payment = use_signal(move || initial_pending_payment);
    let mut payment_overlay_open = use_signal(move || has_pending_payment);
    let mut payment_phase = use_signal(|| "idle".to_string());
    let mut payment_attempt_nonce = use_signal(|| 0_u32);
    let navigator = use_navigator();
    let google_href = api::google_login_url();
    let google_return = use_route::<Route>().to_string();

    use_effect(move || {
        if *auth_profile_loaded.read() || user.read().is_none() {
            return;
        }
        auth_profile_loaded.set(true);
        spawn(async move {
            if let Ok(profile) = api::auth_me().await {
                let _ = api::save_auth_user(&profile);
                if first_name.peek().trim().is_empty() && !profile.first_name.trim().is_empty() {
                    first_name.set(profile.first_name.clone());
                }
                if last_name.peek().trim().is_empty() && !profile.last_name.trim().is_empty() {
                    last_name.set(profile.last_name.clone());
                }
                if phone.peek().trim().is_empty() && !profile.phone.trim().is_empty() {
                    phone.set(profile.phone.clone());
                }
                booking_email.set(profile.email.clone());
                let profile_complete = profile.first_name.trim().len() >= 2
                    && profile.last_name.trim().len() >= 2
                    && profile.phone.trim().len() >= 7;
                if profile_complete {
                    contact_editing.set(false);
                }
                user.set(Some(profile));
            }
        });
    });

    use_effect(move || {
        let Some((slug, address)) = resumed_delivery_check.clone() else {
            return;
        };
        address_busy.set(true);
        spawn(async move {
            match api::delivery_estimate(&slug, &address).await {
                Ok(result) if result.within_range => {
                    let next = remember_delivery_address(
                        &saved_addresses.peek(),
                        &result.resolved_address,
                    );
                    let _ = api::save_json(SAVED_DELIVERY_ADDRESSES, &next);
                    saved_addresses.set(next);
                    delivery_address.set(result.resolved_address.clone());
                    delivery_km.set(Some(result.one_way_km.clone()));
                    delivery_result.set(Some(result));
                    address_error.set(String::new());
                    open_step.set(5);
                }
                Ok(result) => {
                    delivery_km.set(None);
                    delivery_result.set(Some(result.clone()));
                    quote.set(None);
                    address_error.set(format!(
                        "This address is beyond the {} km delivery limit.",
                        result.maximum_km
                    ));
                    open_step.set(3);
                }
                Err(message) => {
                    delivery_km.set(None);
                    delivery_result.set(None);
                    quote.set(None);
                    address_error.set(format!(
                        "Your booking was restored, but the delivery address must be checked again: {message}"
                    ));
                    open_step.set(3);
                }
            }
            address_busy.set(false);
        });
    });

    use_effect(move || {
        let _retry = *payment_config_retry.read();
        payment_config.set(None);
        payment_config_error.set(String::new());
        spawn(async move {
            match api::payment_config().await {
                Ok(config) => payment_config.set(Some(config)),
                Err(error) => payment_config_error.set(error.message),
            }
        });
    });

    use_effect(move || {
        let _attempt = *payment_attempt_nonce.read();
        let pending = pending_payment.read().clone();
        let overlay_open = *payment_overlay_open.read();
        let config = payment_config.read().clone();
        let availability =
            api::payment_availability(config.as_ref(), !payment_config_error.read().is_empty());
        if !overlay_open || payment_phase.read().as_str() != "idle" {
            return;
        }
        let Some(created) = pending else {
            return;
        };
        if !booking_payment_amount_is_valid(&created.booking) {
            payment_phase.set("blocked".into());
            booking_error.set(
                "Checkout was blocked because the payment amount does not match the immutable booking total. No second booking was created."
                    .into(),
            );
            return;
        }
        payment_phase.set("checking".into());
        spawn(async move {
            match api::booking_status(&created.booking.booking_id, &created.access_token).await {
                Ok(status) if status.confirmed || status.status == "confirmed" => {
                    let mut confirmed = created.clone();
                    confirmed.booking.status = status.status;
                    confirmed.booking.payment_status = status.payment_status;
                    if confirmed.booking.rental_slug.is_empty() {
                        confirmed.booking.rental_slug = selected_slug.peek().clone();
                    }
                    let _ = api::save_json("vl_last_booking", &confirmed);
                    let _ = api::save_json(SAVED_POST_PAYMENT_BOOKING, &confirmed);
                    api::remove_saved(SAVED_PENDING_PAYMENT);
                    pending_payment.set(None);
                    payment_overlay_open.set(false);
                    payment_phase.set("confirmed".into());
                    let return_slug = if confirmed.booking.rental_slug.is_empty() {
                        selected_slug.peek().clone()
                    } else {
                        confirmed.booking.rental_slug.clone()
                    };
                    navigator.push(Route::RvDetail { slug: return_slug });
                    return;
                }
                Ok(status) if matches!(status.status.as_str(), "expired" | "cancelled") => {
                    api::remove_saved(SAVED_PENDING_PAYMENT);
                    pending_payment.set(None);
                    payment_phase.set("expired".into());
                    booking_error.set(
                        "This payment reservation expired. Availability and price are being checked before you try again."
                            .into(),
                    );
                    let next = quote_refresh_nonce.peek().wrapping_add(1);
                    quote_refresh_nonce.set(next);
                    return;
                }
                Ok(_) | Err(_) => {}
            }

            let config = match (availability, config) {
                (api::PaymentAvailability::Loading, _) => {
                    payment_phase.set("idle".into());
                    return;
                }
                (api::PaymentAvailability::Disabled, _) => {
                    payment_phase.set("blocked".into());
                    booking_error.set("This saved Stripe reservation cannot be reopened while payments are disabled. Check its status before creating another booking.".into());
                    return;
                }
                (api::PaymentAvailability::Blocked, _) => {
                    payment_phase.set("blocked".into());
                    booking_error.set("Secure Checkout is blocked because the Stripe test configuration could not be verified. Your existing reservation has not been recreated or charged.".into());
                    return;
                }
                (api::PaymentAvailability::TestReady, Some(config)) => config,
                _ => return,
            };
            let Some(client_secret) = created.client_secret.clone() else {
                payment_phase.set("blocked".into());
                booking_error.set("This reservation did not include a reusable Checkout session. Check its status before creating another booking.".into());
                return;
            };
            if created.access_token.is_empty() {
                payment_phase.set("blocked".into());
                booking_error.set("This browser no longer has the private booking token required to check payment status.".into());
                return;
            }
            payment_phase.set("mounting".into());
            let publishable_key =
                serde_json::to_string(&config.publishable_key).unwrap_or_else(|_| "\"\"".into());
            let client_secret =
                serde_json::to_string(&client_secret).unwrap_or_else(|_| "\"\"".into());
            let script = embedded_checkout_script(&publishable_key, &client_secret);
            match document::eval(&script).await {
                Ok(_) => payment_phase.set("checkout".into()),
                Err(_) => {
                    payment_phase.set("error".into());
                    booking_error.set(
                        "Secure test checkout could not load. Keep this window open and try again."
                            .into(),
                    );
                    return;
                }
            }

            let mut submitted = false;
            for _ in 0..800_u16 {
                let checkout_is_mounted = document::eval(
                    "return Boolean(document.getElementById('vl-embedded-checkout'));",
                )
                .join::<bool>()
                .await
                .unwrap_or(false);
                if !checkout_is_mounted {
                    return;
                }
                submitted = document::eval(
                    "return document.getElementById('vl-embedded-checkout')?.dataset.complete === 'true';",
                )
                .join::<bool>()
                .await
                .unwrap_or(false);
                if submitted {
                    break;
                }
                let _ = document::eval("await new Promise(resolve => setTimeout(resolve, 1500));")
                    .await;
            }
            if !submitted {
                payment_phase.set("delayed".into());
                booking_error.set(
                    "The secure checkout session is still open. Complete it before the reservation expires."
                        .into(),
                );
                return;
            }

            payment_phase.set("confirming".into());
            for _ in 0..120_u16 {
                match api::booking_status(&created.booking.booking_id, &created.access_token).await
                {
                    Ok(status) if status.confirmed || status.status == "confirmed" => {
                        let mut confirmed = created.clone();
                        confirmed.booking.status = status.status;
                        confirmed.booking.payment_status = status.payment_status;
                        if confirmed.booking.rental_slug.is_empty() {
                            confirmed.booking.rental_slug = selected_slug.peek().clone();
                        }
                        let _ = api::save_json("vl_last_booking", &confirmed);
                        let _ = api::save_json(SAVED_POST_PAYMENT_BOOKING, &confirmed);
                        api::remove_saved(SAVED_PENDING_PAYMENT);
                        pending_payment.set(None);
                        payment_overlay_open.set(false);
                        payment_phase.set("confirmed".into());
                        let return_slug = if confirmed.booking.rental_slug.is_empty() {
                            selected_slug.peek().clone()
                        } else {
                            confirmed.booking.rental_slug.clone()
                        };
                        navigator.push(Route::RvDetail { slug: return_slug });
                        return;
                    }
                    Ok(status) if matches!(status.status.as_str(), "expired" | "cancelled") => {
                        api::remove_saved(SAVED_PENDING_PAYMENT);
                        pending_payment.set(None);
                        payment_phase.set("expired".into());
                        booking_error.set(
                            "This payment reservation expired. Check availability and create a new booking."
                                .into(),
                        );
                        return;
                    }
                    Ok(_) | Err(_) => {}
                }
                let _ = document::eval("await new Promise(resolve => setTimeout(resolve, 1500));")
                    .await;
            }
            payment_phase.set("delayed".into());
            booking_error.set(
                "Stripe is still confirming the test payment. Keep this booking number and refresh its status shortly."
                    .into(),
            );
        });
    });

    let nights = starts_on
        .read()
        .zip(*ends_on.read())
        .map(|(start, end)| (end - start).num_days())
        .unwrap_or(0);
    let trip_ready = nights >= 3;
    let payment_locked = pending_payment.read().is_some();
    let payment_availability = api::payment_availability(
        payment_config.read().as_ref(),
        !payment_config_error.read().is_empty(),
    );
    let booking_can_submit = matches!(
        payment_availability,
        api::PaymentAvailability::Disabled | api::PaymentAvailability::TestReady
    );
    let contact_complete = first_name.read().trim().len() >= 2
        && last_name.read().trim().len() >= 2
        && booking_email.read().contains('@')
        && phone.read().trim().len() >= 7;
    let mut trip_was_ready = use_signal(|| trip_ready);
    let all_rentals = use_resource(move || {
        let query = api::CatalogSearchDraft {
            location: location.read().clone(),
            radius_km: *radius.read(),
            starts_on: None,
            ends_on: None,
            guests: *guests.read(),
        };
        async move { api::catalog(&query).await }
    });
    let available_rentals = use_resource(move || {
        let selected_starts_on = *starts_on.read();
        let selected_ends_on = *ends_on.read();
        let selected_guests = *guests.read();
        let query = api::CatalogSearchDraft {
            location: location.read().clone(),
            radius_km: *radius.read(),
            starts_on: selected_starts_on.map(|value| value.to_string()),
            ends_on: selected_ends_on.map(|value| value.to_string()),
            guests: selected_guests,
        };
        async move {
            let result = if query.starts_on.is_some() && query.ends_on.is_some() {
                api::catalog(&query).await
            } else {
                Ok(Vec::new())
            };
            DatedRentalMatches {
                starts_on: selected_starts_on,
                ends_on: selected_ends_on,
                guests: selected_guests,
                result,
            }
        }
    });
    let selected_availability = use_resource(move || {
        let slug = selected_slug.read().clone();
        let selected_start_month = (*starts_on.read()).map(month_start);
        let current_month = *visible_month.read();
        let range_start = selected_start_month
            .map(|month| month.min(current_month))
            .unwrap_or(current_month);
        let range_end = add_months(current_month, 3);
        async move {
            if slug.is_empty() {
                Ok(None)
            } else {
                api::availability(&slug, &range_start.to_string(), &range_end.to_string())
                    .await
                    .map(Some)
            }
        }
    });
    let rental_details = use_resource(move || {
        let slug = selected_slug.read().clone();
        async move {
            if slug.is_empty() {
                Ok(None)
            } else {
                api::rental(&slug).await.map(Some)
            }
        }
    });
    let address_lookup = use_resource(move || {
        let query = delivery_address.read().trim().to_string();
        async move {
            if query.chars().count() < 3 {
                return Ok(Vec::new());
            }
            let _ = document::eval("await new Promise(resolve => setTimeout(resolve, 500));").await;
            api::address_suggestions(&query).await
        }
    });
    let rental_values = all_rentals
        .read()
        .as_ref()
        .and_then(|result| result.as_ref().ok())
        .cloned()
        .unwrap_or_default();
    let all_rentals_error = all_rentals
        .read()
        .as_ref()
        .and_then(|result| result.as_ref().err())
        .cloned();
    let current_dated_matches = available_rentals
        .read()
        .as_ref()
        .cloned()
        .filter(|matches| {
            trip_ready
                && matches.starts_on == *starts_on.read()
                && matches.ends_on == *ends_on.read()
                && matches.guests == *guests.read()
        });
    let available_rental_values = current_dated_matches
        .as_ref()
        .and_then(|matches| matches.result.as_ref().ok())
        .cloned();
    let available_rentals_error = current_dated_matches
        .as_ref()
        .and_then(|matches| matches.result.as_ref().err())
        .cloned();
    let details = rental_details
        .read()
        .as_ref()
        .and_then(|result| result.as_ref().ok())
        .and_then(Clone::clone);
    let availability_response = selected_availability
        .read()
        .as_ref()
        .and_then(|result| result.as_ref().ok())
        .and_then(Clone::clone);
    let availability_error = selected_availability
        .read()
        .as_ref()
        .and_then(|result| result.as_ref().err())
        .cloned();
    let calendar_range_end = add_months(*visible_month.read(), 2);
    let calendar_required_end = calendar_range_end + chrono::Duration::days(3);
    let availability_is_current = availability_response.as_ref().is_some_and(|value| {
        value.rental_slug == *selected_slug.read()
            && NaiveDate::parse_from_str(&value.starts_on, "%Y-%m-%d")
                .is_ok_and(|start| start <= *visible_month.read())
            && NaiveDate::parse_from_str(&value.ends_on, "%Y-%m-%d")
                .is_ok_and(|end| end >= calendar_required_end)
    });
    let current_availability = availability_response
        .as_ref()
        .filter(|_| availability_is_current);
    let availability_pending = !selected_slug.read().is_empty()
        && availability_error.is_none()
        && !availability_is_current;
    let unavailable_ranges = current_availability
        .map(availability_ranges)
        .unwrap_or_default();
    let minimum_nights = current_availability
        .map(|value| value.minimum_nights)
        .unwrap_or(3);
    let mut calendar_day = *visible_month.read();
    let mut unavailable_dates = Vec::new();
    if current_availability.is_some() {
        while calendar_day < calendar_range_end {
            if !booking_date_is_selectable(
                calendar_day,
                *starts_on.read(),
                *ends_on.read(),
                minimum_nights,
                &unavailable_ranges,
            ) {
                unavailable_dates.push(calendar_day);
            }
            calendar_day += chrono::Duration::days(1);
        }
    }
    let selected_rental = rental_values
        .iter()
        .find(|rental| rental.slug == *selected_slug.read())
        .cloned()
        .or_else(|| {
            details
                .as_ref()
                .map(|value| value.rental.clone())
                .filter(|rental| rental.slug == *selected_slug.read())
        });
    let selected_name = selected_rental
        .as_ref()
        .map(|rental| rental.name.clone())
        .unwrap_or_else(|| "Choose an RV".into());
    let address_ready = delivery_km.read().is_some()
        && delivery_result
            .read()
            .as_ref()
            .is_some_and(|result| result.within_range);
    let address_query_ready = delivery_address.read().trim().chars().count() >= 3;
    let suggestion_items = address_lookup
        .read()
        .as_ref()
        .and_then(|result| result.as_ref().ok())
        .cloned()
        .unwrap_or_default();
    let suggestion_error = address_lookup
        .read()
        .as_ref()
        .and_then(|result| result.as_ref().err())
        .cloned();
    let suggestions_busy = address_query_ready && address_lookup.read().is_none();
    let optimistic_price = optimistic_price(
        selected_rental.as_ref(),
        details.as_ref(),
        &addon_keys.read(),
        nights,
        delivery_result.read().as_ref(),
        quote.read().as_ref(),
    );
    let preview_total = optimistic_price.as_ref().map(|value| value.total);

    use_effect(move || {
        let address_step_open = *open_step.read() == 3;
        let calculated_address = delivery_result
            .read()
            .as_ref()
            .map(|result| result.resolved_address.clone());
        if !address_step_open {
            return;
        }
        spawn(async move {
            let _ =
                document::eval(&booking_delivery_map_script(calculated_address.as_deref())).await;
        });
    });

    use_effect(move || {
        let ready = starts_on
            .read()
            .zip(*ends_on.read())
            .is_some_and(|(start, end)| (end - start).num_days() >= 3);
        let was_ready = *trip_was_ready.peek();
        trip_was_ready.set(ready);
        if ready && !was_ready && *open_step.peek() == 1 {
            on_search_change.call(());
            if selected_slug.peek().is_empty() {
                open_step.set(2);
            }
        }
    });

    use_effect(move || {
        if selected_slug.peek().is_empty() {
            return;
        }
        let does_not_fit_guests = all_rentals
            .read()
            .as_ref()
            .and_then(|result| result.as_ref().ok())
            .is_some_and(|values| {
                !values
                    .iter()
                    .any(|rental| rental.slug == *selected_slug.peek())
            });
        if does_not_fit_guests {
            selected_slug.set(String::new());
            addon_keys.set(Vec::new());
            delivery_km.set(None);
            delivery_result.set(None);
            quote.set(None);
            quote_error.set("This RV does not fit the current guest count. Choose another model or reduce the number of guests.".into());
            open_step.set(2);
        }
    });

    use_effect(move || {
        let current_start = *starts_on.read();
        let current_end = *ends_on.read();
        let current_guests = *guests.read();
        let ready = current_start
            .zip(current_end)
            .is_some_and(|(start, end)| (end - start).num_days() >= 3);
        if !ready || selected_slug.peek().is_empty() {
            return;
        }
        let matches_resource = available_rentals.read();
        let Some(values) = matches_resource
            .as_ref()
            .filter(|matches| {
                matches.starts_on == current_start
                    && matches.ends_on == current_end
                    && matches.guests == current_guests
            })
            .and_then(|matches| matches.result.as_ref().ok())
        else {
            return;
        };
        if values
            .iter()
            .any(|rental| rental.slug == *selected_slug.peek())
        {
            if *open_step.peek() == 1 {
                open_step.set(3);
            }
        } else {
            starts_on.set(None);
            ends_on.set(None);
            trip_was_ready.set(false);
            addon_keys.set(Vec::new());
            delivery_km.set(None);
            delivery_result.set(None);
            quote.set(None);
            quote_error.set("This RV is booked for the selected dates. Choose new dates from its live calendar.".into());
            open_step.set(1);
        }
    });

    use_effect(move || {
        let _refresh = *quote_refresh_nonce.read();
        let slug = selected_slug.read().clone();
        let start = *starts_on.read();
        let end = *ends_on.read();
        let distance = delivery_km.read().clone();
        let address = delivery_address.read().clone();
        let selected_addons = addon_keys.read().clone();
        let guest_count = *guests.read();
        if slug.is_empty()
            || start.is_none()
            || end.is_none()
            || distance.is_none()
            || address.trim().is_empty()
        {
            quote.set(None);
            quote_busy.set(false);
            return;
        }
        let version = quote_version.peek().wrapping_add(1);
        quote_version.set(version);
        let draft = make_draft(
            &slug,
            start,
            end,
            guest_count,
            &address,
            distance,
            selected_addons,
            false,
            false,
        );
        quote_busy.set(true);
        quote_error.set(String::new());
        spawn(async move {
            match api::create_quote(&draft).await {
                Ok(value) if *quote_version.peek() == version => quote.set(Some(value)),
                Err(error) if *quote_version.peek() == version && error.is_conflict() => {
                    quote.set(None);
                    quote_error.set("Server availability changed for this RV. Your selected-items estimate remains visible; choose another RV before confirmation.".into());
                }
                Err(error) if *quote_version.peek() == version => {
                    quote.set(None);
                    quote_error.set(error.message);
                }
                _ => {}
            }
            if *quote_version.peek() == version {
                quote_busy.set(false);
            }
        });
    });

    let close_overlay = move || async move {
        if *closing.peek() {
            return;
        }
        closing.set(true);
        document::eval(UNMOUNT_EMBEDDED_CHECKOUT);
        let _ = document::eval("await new Promise(resolve => setTimeout(resolve, 180));").await;
        on_close.call(());
    };

    rsx! {
        div { class: if *closing.read() { "ub-backdrop is-closing" } else { "ub-backdrop" }, onclick: move |_| close_overlay(),
            div { class: "ub-shell", role: "dialog", aria_modal: "true", aria_label: "Complete your RV booking", tabindex: "-1", autofocus: true, onclick: move |event| event.stop_propagation(), onkeydown: move |event| async move { if event.key() == Key::Escape { event.stop_propagation(); close_overlay().await; } },
                header { class: "ub-head",
                    div { div { class: "ub-kicker", "ONE-PAGE RV BOOKING" } h2 { "Build your Okanagan stay" } p { "Choose everything here. Completed sections fold into a clear summary." } }
                    button { class: "ub-close", r#type: "button", aria_label: "Close booking", onclick: move |_| close_overlay(), Icon { name: "x", size: 22, color: "var(--vl-ink)" } }
                }
                div { class: "ub-body",
                    main { class: "ub-steps",
                        BookingStep { number: 1, title: "Dates & guests", summary: if trip_ready { format!("{} → {} · {} nights · {} guests", date_text(*starts_on.read()), date_text(*ends_on.read()), nights, guests) } else { "Choose at least 3 nights".into() }, complete: trip_ready, open: *open_step.read() == 1, disabled: payment_locked, on_toggle: move |_| if !payment_locked { open_step.set(if *open_step.peek() == 1 { 0 } else { 1 }) },
                            div { class: "ub-step-content",
                                div { class: "ub-date-summary",
                                    span { "Delivery/setup · 2:00 PM" } strong { "{date_text(*starts_on.read())}" }
                                    Icon { name: "arrow-right", size: 17, color: "var(--vl-muted)" }
                                    span { "Return · 11:00 AM" } strong { "{date_text(*ends_on.read())}" }
                                }
                                if !selected_slug.read().is_empty() {
                                    div { class: if availability_error.is_some() { "ub-calendar-context is-warning" } else { "ub-calendar-context" },
                                        Icon { name: "calendar", size: 16, color: "var(--vl-forest)" }
                                        span {
                                            if availability_pending { "Loading live dates for {selected_name}…" }
                                            else if availability_error.is_some() { "Live dates could not be loaded. Choose a range and we will verify it before continuing." }
                                            else { "Showing live available dates for {selected_name}." }
                                        }
                                    }
                                }
                                div { class: "cat-month-nav",
                                    button { r#type: "button", aria_label: "Previous month", disabled: *visible_month.read() <= initial_month, onclick: move |_| { let current = *visible_month.read(); if let Some(previous) = current.checked_sub_months(Months::new(1)) { visible_month.set(previous.max(initial_month)); } }, Icon { name: "chevron-left", size: 18, color: "var(--vl-ink)" } }
                                    span { "Choose delivery and return" }
                                    button { r#type: "button", aria_label: "Next month", onclick: move |_| { let current = *visible_month.read(); visible_month.set(add_months(current, 1)); }, Icon { name: "chevron-right", size: 18, color: "var(--vl-ink)" } }
                                }
                                div { class: "cat-calendar-months ub-calendar", for offset in 0..2_u32 { CatalogSearchMonth { month: add_months(*visible_month.read(), offset), today, starts_on, ends_on, unavailable_dates: unavailable_dates.clone(), availability_pending } } }
                                div { class: "ub-guests", span { "Guests" } button { r#type: "button", disabled: *guests.read() <= 1, onclick: move |_| { let current = *guests.read(); guests.set((current - 1).max(1)); }, "−" } strong { "{guests}" } button { r#type: "button", disabled: *guests.read() >= 10, onclick: move |_| { let current = *guests.read(); guests.set((current + 1).min(10)); }, "+" } }
                            }
                        }
                        BookingStep { number: 2, title: "Choose your RV", summary: selected_name.clone(), complete: !selected_slug.read().is_empty(), open: *open_step.read() == 2, disabled: payment_locked, on_toggle: move |_| if !payment_locked { open_step.set(if *open_step.peek() == 2 { 0 } else { 2 }) },
                            div { class: "ub-step-content",
                                div { class: "ub-rv-toolbar",
                                    p { class: "ub-choice-guidance",
                                        if trip_ready { "All models are shown. Available RVs continue with these dates; any booked RV opens its calendar for new dates." }
                                        else { "Choose any RV first to see its live calendar, or choose dates first." }
                                    }
                                    div { class: "ub-rv-guests", role: "group", aria_label: "Number of guests",
                                        span { "Guests" }
                                        button { r#type: "button", aria_label: "Remove one guest", disabled: *guests.read() <= 1, onclick: move |_| { let current = *guests.read(); guests.set((current - 1).max(1)); }, "−" }
                                        strong { aria_live: "polite", "{guests}" }
                                        button { r#type: "button", aria_label: "Add one guest", disabled: *guests.read() >= 10, onclick: move |_| { let current = *guests.read(); guests.set((current + 1).min(10)); }, "+" }
                                    }
                                }
                                if all_rentals_error.is_some() { p { class: "ub-error", "RV models could not be loaded. Check your connection and try again." } }
                                else if all_rentals.read().is_none() { p { class: "ub-muted", "Loading RV models…" } }
                                else if rental_values.is_empty() { p { class: "ub-muted", "No RV fits this group size. Reduce the guest count to see available models." } }
                                else {
                                    if available_rentals_error.is_some() { p { class: "ub-error", "Live date matching is temporarily unavailable. Choose a model to open its calendar." } }
                                    div { class: "ub-rv-grid",
                                        for rental in rental_values.clone() {
                                            {
                                                let available_for_dates = available_rental_values.as_ref().map(|values| values.iter().any(|value| value.slug == rental.slug));
                                                rsx! { RentalChoice { key: "rv-{rental.slug}", rental: rental.clone(), selected: *selected_slug.read() == rental.slug, available_for_dates, on_select: move |slug| {
                                                    let keep_dates = rental_selection_keeps_dates(trip_ready, available_for_dates);
                                                    if !keep_dates {
                                                    starts_on.set(None);
                                                    ends_on.set(None);
                                                    trip_was_ready.set(false);
                                                    }
                                                    selected_slug.set(slug);
                                                    addon_keys.set(Vec::new());
                                                    delivery_km.set(None);
                                                    delivery_result.set(None);
                                                    quote.set(None);
                                                    quote_error.set(String::new());
                                                    open_step.set(step_after_rental_selection(keep_dates));
                                                } } }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        BookingStep { number: 3, title: "Delivery address", summary: if address_ready { delivery_result.read().as_ref().map(|result| format!("{} km · CA${}", result.one_way_km, result.delivery_fee)).unwrap_or_default() } else { "Enter the exact campsite or street address".into() }, complete: address_ready, open: *open_step.read() == 3, disabled: selected_slug.read().is_empty() || payment_locked, on_toggle: move |_| if !selected_slug.read().is_empty() && !payment_locked { open_step.set(if *open_step.peek() == 3 { 0 } else { 3 }) },
                            div { class: "ub-step-content",
                                div { class: "ub-address-row",
                                    div { class: "ub-address-combo",
                                        input { value: "{delivery_address}", placeholder: "Start typing a Canadian address", autocomplete: "off", role: "combobox", aria_label: "Delivery address", aria_controls: "ub-address-suggestions", aria_expanded: *suggestions_open.read() && address_query_ready, onfocus: move |_| if address_query_ready { suggestions_open.set(true); }, oninput: move |event| { let value = event.value(); delivery_address.set(value.clone()); delivery_km.set(None); delivery_result.set(None); quote.set(None); quote_error.set(String::new()); address_error.set(String::new()); suggestions_open.set(value.trim().chars().count() >= 3); } }
                                        if suggestions_busy { span { class: "rvd-address-spinner", aria_label: "Searching addresses" } }
                                        if *suggestions_open.read() && address_query_ready { div { id: "ub-address-suggestions", class: "ub-suggestions", role: "listbox",
                                            if suggestions_busy { div { class: "ub-suggestion-status", "Searching nearby Canadian addresses…" } }
                                            else if !suggestion_items.is_empty() { for suggestion in suggestion_items { button { r#type: "button", role: "option", onclick: move |_| { delivery_address.set(suggestion.display_name.clone()); suggestions_open.set(false); }, strong { "{suggestion.primary_text}" } small { "{suggestion.secondary_text}" } } } }
                                            else if let Some(message) = suggestion_error.as_ref() { div { class: "ub-suggestion-status is-error", "{message}" } }
                                            else { div { class: "ub-suggestion-status", "Keep typing the street name, city, or campground." } }
                                            div { class: "ub-suggestions-foot", "Canadian addresses · prioritized near Kelowna" }
                                        } }
                                    }
                                    button { class: "ub-primary", r#type: "button", disabled: *address_busy.read() || !address_query_ready, onclick: move |_| { let slug = selected_slug.read().clone(); let address = delivery_address.read().clone(); async move { address_busy.set(true); address_error.set(String::new()); match api::delivery_estimate(&slug, &address).await { Ok(result) if result.within_range => { let next = remember_delivery_address(&saved_addresses.read(), &result.resolved_address); let _ = api::save_json(SAVED_DELIVERY_ADDRESSES, &next); saved_addresses.set(next); delivery_address.set(result.resolved_address.clone()); delivery_km.set(Some(result.one_way_km.clone())); delivery_result.set(Some(result)); suggestions_open.set(false); open_step.set(4); }, Ok(result) => { delivery_result.set(Some(result.clone())); address_error.set(format!("This address is beyond the {} km delivery limit.", result.maximum_km)); }, Err(message) => address_error.set(message) } address_busy.set(false); } }, if *address_busy.read() { "Calculating…" } else { "Calculate delivery" } }
                                }
                                div { class: "ub-delivery-map", aria_label: "Interactive delivery map centred on Kelowna",
                                    div { id: "vl-booking-delivery-map", class: "ub-leaflet-map" }
                                    div { class: "ub-map-fallback",
                                        div { class: "cat-map-fallback-icon", Icon { name: "map", size: 22, color: "var(--vl-forest)" } }
                                        span { "Loading delivery map…" }
                                    }
                                    div { class: "ub-map-badge", strong { "150 km" } span { "maximum delivery distance" } }
                                    div { class: if address_ready { "ub-map-legend has-address" } else { "ub-map-legend" }, i {} span { if address_ready { "Kelowna base · selected address" } else { "Approximate delivery area" } } }
                                }
                                p { class: "ub-map-note", "Map area is approximate. Eligibility and price use the calculated one-way driving distance." }
                                if !saved_addresses.read().is_empty() {
                                    div { class: "ub-address-history",
                                        span { class: "ub-address-history-title", "Recently used on this device" }
                                        div { class: "ub-address-history-list",
                                            for saved_address in saved_addresses.read().clone() {
                                                div { key: "saved-{saved_address}", class: "ub-address-history-item",
                                                    button { class: "ub-address-history-select", r#type: "button", disabled: *address_busy.read(), onclick: { let address = saved_address.clone(); move |_| { let slug = selected_slug.read().clone(); let address = address.clone(); async move { delivery_address.set(address.clone()); delivery_km.set(None); delivery_result.set(None); quote.set(None); quote_error.set(String::new()); address_error.set(String::new()); suggestions_open.set(false); address_busy.set(true); match api::delivery_estimate(&slug, &address).await { Ok(result) if result.within_range => { let next = remember_delivery_address(&saved_addresses.read(), &result.resolved_address); let _ = api::save_json(SAVED_DELIVERY_ADDRESSES, &next); saved_addresses.set(next); delivery_address.set(result.resolved_address.clone()); delivery_km.set(Some(result.one_way_km.clone())); delivery_result.set(Some(result)); open_step.set(4); }, Ok(result) => { delivery_result.set(Some(result.clone())); address_error.set(format!("This address is beyond the {} km delivery limit.", result.maximum_km)); }, Err(message) => address_error.set(message) } address_busy.set(false); } } },
                                                        Icon { name: "map-pin", size: 15, color: "var(--vl-forest)" }
                                                        span { "{saved_address}" }
                                                    }
                                                    button { class: "ub-address-history-remove", r#type: "button", aria_label: "Remove saved address {saved_address}", onclick: { let address = saved_address.clone(); move |_| { let next = forget_delivery_address(&saved_addresses.read(), &address); if next.is_empty() { api::remove_saved(SAVED_DELIVERY_ADDRESSES); } else { let _ = api::save_json(SAVED_DELIVERY_ADDRESSES, &next); } saved_addresses.set(next); } }, Icon { name: "x", size: 14, color: "var(--vl-muted)" } }
                                                }
                                            }
                                        }
                                    }
                                }
                                if !address_error.read().is_empty() { p { class: "ub-error", role: "alert", "{address_error}" } }
                                if let Some(result) = delivery_result.read().as_ref().filter(|result| result.within_range) { div { class: "ub-success", Icon { name: "check-circle-2", size: 17, color: "var(--vl-forest)" } span { "{result.resolved_address} · {result.one_way_km} km one way" } } }
                            }
                        }
                        BookingStep { number: 4, title: "Extras & trip details", summary: if addon_keys.read().is_empty() { "No extras selected".into() } else { format!("{} extras selected", addon_keys.read().len()) }, complete: address_ready, open: *open_step.read() == 4, disabled: !address_ready || payment_locked, on_toggle: move |_| if address_ready && !payment_locked { open_step.set(if *open_step.peek() == 4 { 0 } else { 4 }) },
                            div { class: "ub-step-content",
                                if let Some(details) = details.as_ref() {
                                    div { class: "ub-addon-grid",
                                        for addon in details.addons.iter() {
                                            {
                                                let key = addon.addon_key.clone();
                                                let selected = addon_keys.read().contains(&key);
                                                rsx! {
                                                    button { key: "addon-{key}", class: if selected { "ub-addon active" } else { "ub-addon" }, r#type: "button", onclick: move |_| {
                                                        let mut next = addon_keys.read().clone();
                                                        if next.contains(&key) { next.retain(|value| value != &key); } else { next.push(key.clone()); }
                                                        quote_busy.set(true);
                                                        quote_error.set(String::new());
                                                        addon_keys.set(next);
                                                    },
                                                        span { class: "ub-addon-icon", Icon { name: addon_icon(&key), size: 18, color: "var(--vl-forest)" } }
                                                        span { class: "ub-addon-copy", strong { "{addon.label}" } small { "{addon_description(&key)}" } if addon.is_recommended { em { "Recommended" } } }
                                                        span { class: "ub-addon-price", b { "CA${addon.price}" } small { if addon.charge_type == "per_unit" { "per night" } else { "one-time" } } }
                                                        span { class: "ub-addon-toggle", if selected { "✓" } else { "+" } }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                div { class: "ub-binary-row", div { class: "ub-binary-copy", Icon { name: "party-popper", size: 18, color: "var(--vl-forest)" } div { strong { "Festival or special event" } small { "Festival and special-event bookings are not available." } } } div { class: "ub-segmented is-policy", button { r#type: "button", disabled: true, "Yes" } button { class: "active", r#type: "button", disabled: true, "No" } } }
                                div { class: "ub-binary-row", div { class: "ub-binary-copy", Icon { name: "move", size: 18, color: "var(--vl-forest)" } div { strong { "RV movement after delivery" } small { "The RV must remain at the delivery and setup location." } } } div { class: "ub-segmented is-policy", button { r#type: "button", disabled: true, "Yes" } button { class: "active", r#type: "button", disabled: true, "No" } } }
                                div { class: "ub-binary-row", div { class: "ub-binary-copy", Icon { name: "truck", size: 18, color: "var(--vl-forest)" } div { strong { "Delivery only" } small { "Customer pickup is not available; we deliver and set up every RV." } } } div { class: "ub-segmented is-policy", button { class: "active", r#type: "button", disabled: true, "Yes" } button { r#type: "button", disabled: true, "No" } } }
                                button { class: "ub-primary ub-next", r#type: "button", onclick: move |_| open_step.set(5), "Review booking" }
                            }
                        }
                        BookingStep { number: 5, title: "Guest details & confirmation", summary: String::from(if user.read().is_some() { "Ready to confirm" } else { "Sign in, then confirm" }), complete: false, open: *open_step.read() == 5, disabled: !address_ready, on_toggle: move |_| if address_ready { open_step.set(if *open_step.peek() == 5 { 0 } else { 5 }) },
                            div { class: "ub-step-content",
                                if user.read().is_none() { div { class: "ub-auth",
                                    h3 { if *auth_register.read() { "Create your account" } else { "Sign in to confirm" } }
                                    a { class: "ub-google", href: google_href, onclick: move |event| {
                                        let draft = make_draft(
                                            &selected_slug.read(),
                                            *starts_on.read(),
                                            *ends_on.read(),
                                            *guests.read(),
                                            &delivery_address.read(),
                                            delivery_km.read().clone(),
                                            addon_keys.read().clone(),
                                            false,
                                            false,
                                        );
                                        let continuation = api::BookingAuthContinuation {
                                            draft: draft.clone(),
                                            location: location.read().clone(),
                                            radius_km: *radius.read(),
                                            delivery_estimate: delivery_result.read().clone(),
                                            first_name: first_name.read().clone(),
                                            last_name: last_name.read().clone(),
                                            booking_email: booking_email.read().clone(),
                                            phone: phone.read().clone(),
                                            notes: notes.read().clone(),
                                            accepted_terms: *accepted.read(),
                                        };
                                        match api::save_booking_auth_continuation(&continuation) {
                                            Ok(()) => {
                                                let search = api::CatalogSearchDraft {
                                                    location: location.read().clone(),
                                                    radius_km: *radius.read(),
                                                    starts_on: Some(draft.starts_on),
                                                    ends_on: Some(draft.ends_on),
                                                    guests: draft.guests,
                                                };
                                                let _ = api::save_json("vl_catalog_search", &search);
                                                on_search_change.call(());
                                                api::remember_auth_return(&google_return);
                                            }
                                            Err(message) => {
                                                event.prevent_default();
                                                auth_error.set(message);
                                            }
                                        }
                                    },
                                        span { class: "auth-google-mark", "G" }
                                        "Continue with Google"
                                    }
                                    div { class: "ub-auth-divider", span { "or use email" } }
                                    div { class: "ub-field-grid", input { r#type: "email", autocomplete: "email", value: "{auth_email}", placeholder: "Email", oninput: move |event| auth_email.set(event.value()) } input { r#type: "password", autocomplete: if *auth_register.read() { "new-password" } else { "current-password" }, value: "{auth_password}", placeholder: "Password", oninput: move |event| auth_password.set(event.value()) } }
                                    if !auth_error.read().is_empty() { p { class: "ub-error", "{auth_error}" } }
                                    div { class: "ub-auth-actions", button { class: "ub-primary", r#type: "button", disabled: *auth_busy.read(), onclick: move |_| { let email = auth_email.read().clone(); let password = auth_password.read().clone(); let register = *auth_register.read(); async move { auth_busy.set(true); auth_error.set(String::new()); match api::login(&email, &password, register).await { Ok(tokens) => match api::save_session(&tokens) { Ok(()) => { booking_email.set(tokens.user.email.clone()); user.set(Some(tokens.user)); }, Err(message) => auth_error.set(message) }, Err(_) => auth_error.set("Check your email and password, then try again.".into()) } auth_busy.set(false); } }, if *auth_busy.read() { "Please wait…" } else if *auth_register.read() { "Create account" } else { "Sign in" } } button { r#type: "button", onclick: move |_| { let current = *auth_register.read(); auth_register.set(!current); }, if *auth_register.read() { "I already have an account" } else { "Create an account" } } }
                                } } else if let Some(created) = pending_payment.read().as_ref() { div { class: "ub-payment-reserved",
                                    div { class: "ub-stripe-payment-head",
                                        div { Icon { name: "shield-check", size: 20, color: "var(--vl-forest)" } div { h3 { "Reservation ready for payment" } p { "Booking {created.booking.booking_number} is held. Secure Checkout opens above this booking." } } }
                                        span { "TEST MODE" }
                                    }
                                    p { "Your dates remain reserved while this Checkout session is active. Closing the payment window will not create another booking." }
                                    button { class: "ub-primary", r#type: "button", onclick: move |_| { booking_error.set(String::new()); payment_overlay_open.set(true); payment_phase.set("idle".into()); let next = payment_attempt_nonce().wrapping_add(1); payment_attempt_nonce.set(next); }, "Open secure payment" }
                                } } else { div { class: "ub-fields",
                                    if let Some(authenticated_user) = user.read().as_ref() {
                                        if contact_complete && !*contact_editing.read() {
                                            div { class: "ub-contact-summary",
                                                div { Icon { name: "check-circle-2", size: 18, color: "var(--vl-forest)" } span { strong { "{first_name} {last_name}" } small { "{authenticated_user.email} · {phone}" } } }
                                                button { r#type: "button", onclick: move |_| contact_editing.set(true), "Edit contact" }
                                            }
                                        } else {
                                            div { class: "ub-contact-intro", Icon { name: "user-round", size: 18, color: "var(--vl-forest)" } span { strong { "Complete your booking contact" } small { "You are signed in as {authenticated_user.email}. Add any missing name or phone details once." } } }
                                            div { class: "ub-field-grid", input { autocomplete: "given-name", value: "{first_name}", placeholder: "First name", oninput: move |event| first_name.set(event.value()) } input { autocomplete: "family-name", value: "{last_name}", placeholder: "Last name", oninput: move |event| last_name.set(event.value()) } }
                                            div { class: "ub-field-grid", input { r#type: "email", readonly: true, value: "{booking_email}", aria_label: "Signed-in booking email" } input { r#type: "tel", autocomplete: "tel", value: "{phone}", placeholder: "Phone", oninput: move |event| phone.set(event.value()) } }
                                            if contact_complete { button { class: "ub-contact-done", r#type: "button", onclick: move |_| contact_editing.set(false), "Use these contact details" } }
                                        }
                                    }
                                    textarea { value: "{notes}", placeholder: "Notes (optional)", oninput: move |event| notes.set(event.value()) }
                                    label { class: "ub-terms", input { r#type: "checkbox", checked: *accepted.read(), disabled: !booking_can_submit, onchange: move |event| accepted.set(event.checked()) } span { if payment_availability == api::PaymentAvailability::TestReady { "I accept the rental terms and authorize this Stripe test-mode payment." } else { "I accept the rental terms and understand this is a test booking with no card charge." } } }
                                    if payment_availability == api::PaymentAvailability::Loading { p { class: "ub-muted", role: "status", "Checking whether this environment uses Stripe test payments…" } }
                                    if payment_availability == api::PaymentAvailability::Blocked { p { class: "ub-error", role: "alert", if payment_config_error.read().is_empty() { "Checkout is blocked because the returned Stripe mode, key, or account is not the approved test configuration." } else { "Payment configuration could not be verified. Retry before creating a reservation." } } button { r#type: "button", onclick: move |_| { let next = payment_config_retry().wrapping_add(1); payment_config_retry.set(next); }, "Retry payment configuration" } }
                                    if !booking_error.read().is_empty() { p { class: "ub-error", role: "alert", "{booking_error}" } }
                                    button { class: "ub-primary ub-confirm", r#type: "button", disabled: *booking_busy.read() || *quote_busy.read() || quote.read().is_none() || !booking_can_submit, onclick: move |_| { let active_quote = quote.read().clone(); let values = (first_name.read().clone(), last_name.read().clone(), booking_email.read().clone(), phone.read().clone(), notes.read().clone(), *accepted.read()); let draft = make_draft(&selected_slug.read(), *starts_on.read(), *ends_on.read(), *guests.read(), &delivery_address.read(), delivery_km.read().clone(), addon_keys.read().clone(), false, false); async move { if !booking_can_submit { booking_error.set("Payment configuration must be verified before a booking can be created.".into()); return; } else if !values.5 { booking_error.set("Please accept the rental terms.".into()); return; } else if values.0.trim().len() < 2 || values.1.trim().len() < 2 || !values.2.contains('@') || values.3.trim().len() < 7 { booking_error.set("Enter your full name, email, and phone number.".into()); return; } let Some(active_quote) = active_quote else { booking_error.set("Wait for the price calculation to finish.".into()); return; }; booking_busy.set(true); booking_error.set(String::new()); let booking_notes = format!("{}\nDelivery address: {}\nDelivery distance: {} km one way\nFestival/event: no\nTowing after delivery: no\nDelivery only: yes", values.4.trim(), delivery_address.read(), delivery_km.read().clone().unwrap_or_default()); match api::create_booking(&active_quote.quote.quote_id, &values.0, &values.1, &values.2, &values.3, &booking_notes).await { Ok(created) => { let _ = api::save_json("vl_trip_draft", &draft); let _ = api::save_json("vl_active_quote", &active_quote); if created.client_secret.is_some() && !created.access_token.is_empty() { let _ = api::save_json(SAVED_PENDING_PAYMENT, &created); payment_overlay_open.set(true); payment_phase.set("idle".into()); pending_payment.set(Some(created)); } else if created.booking.status == "confirmed" || created.booking.payment_status == "test_paid" { let _ = api::save_json("vl_last_booking", &created); let _ = api::save_json(SAVED_POST_PAYMENT_BOOKING, &created); let return_slug = if created.booking.rental_slug.is_empty() { selected_slug.peek().clone() } else { created.booking.rental_slug.clone() }; navigator.push(Route::RvDetail { slug: return_slug }); } else { booking_error.set("The booking was reserved, but Stripe Checkout was not returned. Please contact support before trying again.".into()); } }, Err(error) => booking_error.set(error.message) } booking_busy.set(false); } }, if *booking_busy.read() { "Creating reservation…" } else if payment_availability == api::PaymentAvailability::TestReady { "Continue to secure test payment" } else { "Confirm test booking" } }
                                } }
                            }
                        }
                    }
                    aside { class: "ub-summary",
                        h3 { aria_live: "polite", if let Some(created) = pending_payment.read().as_ref() { "{created.booking.currency} ${created.booking.total}" } else if *quote_busy.read() { if let Some(value) = preview_total { AnimatedMoney { id: "ub-trip-price", amount: value } } else { "Updating…" } } else if let Some(value) = quote.read().as_ref() { AnimatedMoney { id: "ub-trip-price", amount: pricing::quote_trip_price(value) } } else if let Some(value) = preview_total { AnimatedMoney { id: "ub-trip-price", amount: value } } else { "Complete delivery" } }
                        p { if pending_payment.read().is_some() { "This immutable trip price is locked to the active Stripe reservation. The refundable damage deposit is separate." } else if *quote_busy.read() { "Updating the exact trip price…" } else if quote.read().is_some() { "Trip price with preparation, protection, delivery, selected extras and taxes. The refundable damage deposit is separate." } else if preview_total.is_some() { "Known trip costs are shown. Exact taxes are updating; the refundable damage deposit is separate." } else { "Your trip price appears after the delivery address is calculated." } }
                        button { class: "ub-summary-dates", r#type: "button", disabled: !trip_ready || payment_locked, onclick: move |_| if !payment_locked { open_step.set(1); spawn(async move { scroll_to_booking_step(1).await; }); },
                            span { "Dates" }
                            b { if let Some(created) = pending_payment.read().as_ref() { "{display_booking_date(&created.booking.starts_at)} → {display_booking_date(&created.booking.ends_at)}" } else if trip_ready { "{date_text(*starts_on.read())} → {date_text(*ends_on.read())} · {nights} nights" } else { "Choose dates" } }
                            small { if payment_locked { "Locked" } else { "Edit dates" } }
                        }
                        if let Some(created) = pending_payment.read().as_ref() { div { class: "ub-price-lines ub-locked-price",
                            div { span { "Booked RV" } b { "{created.booking.rental_name}" } }
                            div { class: "total", span { "Trip price CAD" } b { "{created.booking.currency} ${created.booking.total}" } }
                            div { class: "due-now", span { if created.booking.amount_due_now == created.booking.total { "Due now · 100%" } else { "Due now · 30%" } } b { "{created.booking.currency} ${created.booking.amount_due_now}" } }
                            small { "Stripe Checkout is created from this same backend amount. Dates, RV and extras cannot change while this reservation is active." }
                        } } else if *quote_busy.read() {
                            if let Some(value) = optimistic_price.as_ref() { div { class: "ub-price-lines", for item in value.lines.iter() { if item.key.starts_with("rental-") { button { key: "optimistic-{item.key}-{item.amount}", class: "ub-price-line is-editable", r#type: "button", aria_label: "Edit dates for {item.label}", onclick: move |_| { open_step.set(1); spawn(async move { scroll_to_booking_step(1).await; }); }, span { "{item.label}" if let Some(detail) = item.detail.as_ref() { small { "{detail}" } } } b { class: "ub-line-price", "{pricing::money(item.amount)}" } } } else { div { key: "optimistic-{item.key}-{item.amount}", class: "ub-price-line", span { "{item.label}" if let Some(detail) = item.detail.as_ref() { small { "{detail}" } } } b { class: "ub-line-price", "{pricing::money(item.amount)}" } } } } div { class: "total", span { "Trip price CAD" } b { AnimatedMoney { id: "ub-trip-price-total", amount: value.total } } } } }
                        } else if let Some(value) = quote.read().as_ref() { div { class: "ub-price-lines", for item in value.items.iter().filter(|item| item.item_type != "deposit") { if item.item_type == "rental" { button { key: "line-{item.item_key}-{item.amount}", class: "ub-price-line is-editable", r#type: "button", aria_label: "Edit dates for {item.label}", onclick: move |_| { open_step.set(1); spawn(async move { scroll_to_booking_step(1).await; }); }, span { "{item.label}" } b { class: "ub-line-price", "CA${item.amount}" } } } else { div { key: "line-{item.item_key}-{item.amount}", class: "ub-price-line", span { "{item.label}" if item.item_key == "stationary_plus" { small { "{item.quantity} nights × CA${item.unit_price}" } } } b { class: "ub-line-price", "CA${item.amount}" } } } } div { class: "total", span { "Trip price CAD" } b { AnimatedMoney { id: "ub-trip-price-total", amount: pricing::quote_trip_price(value) } } } } }
                        div { class: "ub-deposit-card",
                            div { span { "REFUNDABLE DAMAGE DEPOSIT" } b { "{pricing::money(pricing::DAMAGE_DEPOSIT)}" } }
                            p { "Charged separately {pricing::DAMAGE_DEPOSIT_DUE_HOURS} hours before delivery and refunded to the original payment method after return and inspection, less any documented damage." }
                        }
                        div { class: "ub-payment-note",
                            b { "Payment timing" }
                            span { "{pricing::BOOKING_DEPOSIT_PERCENT}% of the trip price to confirm when booked more than {pricing::BALANCE_DUE_DAYS} days ahead; the balance is due {pricing::BALANCE_DUE_DAYS} days before delivery. Trips within {pricing::BALANCE_DUE_DAYS} days are paid in full when booked." }
                        }
                        if !quote_error.read().is_empty() { p { class: "ub-error", "{quote_error}" } }
                        div { class: "ub-summary-trip", span { "Dates" } b { if let Some(created) = pending_payment.read().as_ref() { "{display_booking_date(&created.booking.starts_at)} → {display_booking_date(&created.booking.ends_at)}" } else if trip_ready { "{date_text(*starts_on.read())} → {date_text(*ends_on.read())}" } else { "Not selected" } } span { "RV" } b { if let Some(created) = pending_payment.read().as_ref() { "{created.booking.rental_name}" } else { "{selected_name}" } } span { "Price" } b { if payment_locked { "Locked to booking" } else if address_ready { "Calculated" } else { "Required" } } }
                        div { class: "ub-test-note", Icon { name: "shield-check", size: 17, color: "var(--vl-forest)" } span { match payment_availability { api::PaymentAvailability::TestReady => "Stripe test mode: test cards only. Live payments remain disabled.", api::PaymentAvailability::Disabled => "Test mode: no card is collected and no charge is made.", api::PaymentAvailability::Loading => "Checking the test payment configuration…", api::PaymentAvailability::Blocked => "Payment configuration is blocked until the approved Stripe test account is verified.", } } }
                    }
                }
            }
            if let Some(created) = pending_payment.read().as_ref().filter(|_| *payment_overlay_open.read()) {
                div { class: "ub-payment-layer", onclick: move |event| { event.stop_propagation(); document::eval(UNMOUNT_EMBEDDED_CHECKOUT); payment_overlay_open.set(false); payment_phase.set("idle".into()); },
                    section { class: "ub-payment-dialog", role: "dialog", aria_modal: "true", aria_label: "Secure payment for your RV booking", tabindex: "-1", autofocus: true, onclick: move |event| event.stop_propagation(), onkeydown: move |event| if event.key() == Key::Escape { event.stop_propagation(); document::eval(UNMOUNT_EMBEDDED_CHECKOUT); payment_overlay_open.set(false); payment_phase.set("idle".into()); },
                        header { class: "ub-payment-dialog-head",
                            div {
                                span { class: "ub-kicker", "SECURE TEST PAYMENT" }
                                h2 { "Complete your reservation" }
                                p { "{created.booking.booking_number} · {created.booking.rental_name}" }
                            }
                            div { class: "ub-payment-dialog-actions",
                                span { "TEST MODE" }
                                button { r#type: "button", aria_label: "Close secure payment", onclick: move |_| { document::eval(UNMOUNT_EMBEDDED_CHECKOUT); payment_overlay_open.set(false); payment_phase.set("idle".into()); }, Icon { name: "x", size: 21, color: "var(--vl-ink)" } }
                            }
                        }
                        div { class: "ub-payment-dialog-summary",
                            span { "Due now" }
                            strong { "{created.booking.currency} ${created.booking.amount_due_now}" }
                            small { if created.booking.amount_due_now == created.booking.total { "Full locked trip price: {created.booking.currency} ${created.booking.total}. Your booking stays reserved if you close and reopen this window." } else { "30% of the locked trip price {created.booking.currency} ${created.booking.total}. Your booking stays reserved if you close and reopen this window." } }
                        }
                        div { class: "ub-payment-dialog-body",
                            if payment_phase.read().as_str() == "checking" { p { class: "ub-stripe-state", "Checking webhook-backed booking status…" } }
                            if payment_phase.read().as_str() == "mounting" { p { class: "ub-stripe-state", "Loading secure Checkout…" } }
                            if payment_availability == api::PaymentAvailability::TestReady { div { id: "vl-embedded-checkout", class: "ub-embedded-checkout", aria_label: "Stripe test checkout" } }
                            if matches!(payment_phase.read().as_str(), "confirming" | "delayed") { div { class: "ub-confirming-payment", Icon { name: "loader-circle", size: 17, color: "var(--vl-forest)" } span { strong { "Confirming payment…" } small { "We return to this RV only after the backend confirms the Stripe webhook. Do not submit a second payment." } } } }
                            if !booking_error.read().is_empty() { p { class: "ub-error", role: "alert", "{booking_error}" } }
                            if matches!(payment_phase.read().as_str(), "error" | "blocked" | "delayed") { button { class: "ub-primary", r#type: "button", onclick: move |_| { booking_error.set(String::new()); payment_phase.set("idle".into()); let next = payment_attempt_nonce().wrapping_add(1); payment_attempt_nonce.set(next); }, "Check status and reopen Checkout" } }
                            if payment_availability == api::PaymentAvailability::Blocked { button { class: "ub-payment-secondary", r#type: "button", onclick: move |_| { let next = payment_config_retry().wrapping_add(1); payment_config_retry.set(next); payment_phase.set("idle".into()); }, "Retry test configuration" } }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn RentalChoice(
    rental: api::Rental,
    selected: bool,
    available_for_dates: Option<bool>,
    on_select: EventHandler<String>,
) -> Element {
    let slug = rental.slug.clone();
    let fallback_image = rental_image(&rental);
    let mut images = rv_gallery(&rental.slug)
        .into_iter()
        .map(|asset| asset.to_string())
        .collect::<Vec<_>>();
    if images.is_empty() {
        images.push(fallback_image);
    }
    let image_count = images.len();
    let mut image_index = use_signal(|| 0_usize);
    let current_image = images
        .get(*image_index.read() % image_count)
        .cloned()
        .unwrap_or_default();
    let image_number = *image_index.read() + 1;
    let mut reviews_open = use_signal(|| false);
    let mut reviews_busy = use_signal(|| false);
    let mut reviews_error = use_signal(String::new);
    let mut reviews = use_signal(|| None::<api::RentalReviewsResponse>);
    let rating = rental.review_rating.clone().unwrap_or_else(|| "New".into());
    let rounded_rating = rental
        .review_rating
        .as_deref()
        .and_then(|value| value.parse::<f64>().ok())
        .map(|value| value.round() as i32)
        .unwrap_or(0);
    let review_label = if rental.review_count == 1 {
        "1 review".to_string()
    } else {
        format!("{} reviews", rental.review_count)
    };
    rsx! {
        article {
            class: if selected { "ub-rv active" } else if available_for_dates == Some(false) { "ub-rv is-unavailable" } else { "ub-rv" },
            div { class: "ub-rv-image", style: "background-image: url('{current_image}');",
                button { class: "ub-rv-image-select", r#type: "button", aria_label: "Select {rental.name}", onclick: { let slug = slug.clone(); move |_| on_select.call(slug.clone()) } }
                span { "Sleeps {rental.capacity}" }
                if selected { b { "Selected" } }
                if image_count > 1 {
                    button { class: "ub-rv-gallery-arrow prev", r#type: "button", aria_label: "Previous photo of {rental.name}", onclick: move |event| { event.stop_propagation(); let current = *image_index.peek(); image_index.set(if current == 0 { image_count - 1 } else { current - 1 }); }, Icon { name: "chevron-left", size: 16, color: "var(--vl-ink)" } }
                    button { class: "ub-rv-gallery-arrow next", r#type: "button", aria_label: "Next photo of {rental.name}", onclick: move |event| { event.stop_propagation(); let current = *image_index.peek(); image_index.set((current + 1) % image_count); }, Icon { name: "chevron-right", size: 16, color: "var(--vl-ink)" } }
                    small { class: "ub-rv-image-count", "{image_number} / {image_count}" }
                }
            }
            div { class: "ub-rv-body",
                button { class: "ub-rv-select-copy", r#type: "button", onclick: { let slug = slug.clone(); move |_| on_select.call(slug.clone()) },
                    strong { "{rental.name}" }
                    small { "{rental.category}" }
                    if let Some(available) = available_for_dates {
                        span { class: if available { "ub-rv-date-status is-available" } else { "ub-rv-date-status is-unavailable" },
                            if available { "Available for your dates" } else { "Choose new dates" }
                        }
                    }
                    p { "{rental.summary}" }
                    div { b { "CA${rental.base_rate}" } span { " / {rental.price_unit}" } }
                }
                button { class: "ub-rv-rating", r#type: "button", aria_label: "Read {review_label} for {rental.name}", onclick: { let slug = slug.clone(); move |event| { event.stop_propagation(); reviews_open.set(true); if reviews.peek().is_none() && !*reviews_busy.peek() { reviews_busy.set(true); reviews_error.set(String::new()); let slug = slug.clone(); spawn(async move { match api::rental_reviews(&slug).await { Ok(value) => reviews.set(Some(value)), Err(message) => reviews_error.set(message) } reviews_busy.set(false); }); } } },
                    RatingStars { rating: rounded_rating }
                    b { "{rating}" }
                    span { "({rental.review_count})" }
                    small { "Read comments" }
                }
            }
            if *reviews_open.read() {
                div { class: "ub-review-backdrop", onclick: move |_| reviews_open.set(false),
                    section { class: "ub-review-modal", role: "dialog", aria_modal: "true", aria_label: "Reviews for {rental.name}", tabindex: "-1", autofocus: true, onclick: move |event| event.stop_propagation(), onkeydown: move |event| { if event.key() == Key::Escape { event.stop_propagation(); reviews_open.set(false); } },
                        header { div { RatingStars { rating: rounded_rating } h3 { "Guest reviews" } p { "{rental.name}" } } button { r#type: "button", aria_label: "Close reviews", onclick: move |_| reviews_open.set(false), Icon { name: "x", size: 20, color: "var(--vl-ink)" } } }
                        div { class: "ub-review-scroll",
                            if *reviews_busy.read() { p { class: "ub-review-state", "Loading reviews…" } }
                            else if !reviews_error.read().is_empty() { p { class: "ub-error", role: "alert", "{reviews_error}" } }
                            else if let Some(value) = reviews.read().as_ref() {
                                div { class: "ub-review-summary", b { if let Some(average) = value.summary.average_rating.as_ref() { "{average}" } else { "New" } } span { "out of 5 · {value.summary.review_count} verified reviews" } }
                                if value.reviews.is_empty() { p { class: "ub-review-state", "No guest comments yet." } }
                                for review in value.reviews.iter() {
                                    article { class: "ub-review-item", key: "{review.rental_review_id}",
                                        div { RatingStars { rating: review.rating } b { "{review.rating}/5" } time { "{review.created_at.get(0..10).unwrap_or(&review.created_at)}" } }
                                        if !review.title.is_empty() { h4 { "{review.title}" } }
                                        p { "{review.body}" }
                                        small { "{review.reviewer_name} · Verified booking" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn RatingStars(rating: i32) -> Element {
    rsx! {
        span { class: "ub-rating-stars", aria_label: "{rating} out of 5 stars",
            for value in 1..=5_i32 {
                span { key: "rating-star-{value}", class: if value <= rating { "filled" } else { "" }, "★" }
            }
        }
    }
}

#[component]
fn BookingStep(
    number: u8,
    title: &'static str,
    summary: String,
    complete: bool,
    open: bool,
    #[props(default = false)] disabled: bool,
    on_toggle: EventHandler<()>,
    children: Element,
) -> Element {
    let chevron = if open { "chevron-up" } else { "chevron-down" };
    rsx! {
        section { id: "ub-step-{number}", class: if open { "ub-step is-open" } else if complete { "ub-step is-complete" } else { "ub-step" },
            button { class: "ub-step-head", r#type: "button", disabled, aria_expanded: open, onclick: move |_| on_toggle.call(()),
                span { class: "ub-step-number", if complete { Icon { name: "check", size: 16, color: "var(--vl-white)" } } else { "{number}" } }
                span { class: "ub-step-title", strong { "{title}" } small { "{summary}" } }
                Icon { name: chevron, size: 18, color: "var(--vl-muted)" }
            }
            if open { {children} }
        }
    }
}

#[cfg(test)]
mod saved_address_tests {
    use super::*;

    #[test]
    fn newest_address_is_first_and_duplicates_are_removed() {
        let saved = vec![
            "1198 Raymer Avenue, Kelowna, BC".into(),
            "Bear Creek Provincial Park".into(),
        ];
        let next = remember_delivery_address(&saved, "  1198 raymer avenue, kelowna, bc  ");

        assert_eq!(
            next,
            vec![
                "1198 raymer avenue, kelowna, bc",
                "Bear Creek Provincial Park"
            ]
        );
    }

    #[test]
    fn address_history_is_limited_and_can_be_deleted() {
        let saved = (1..=6)
            .map(|number| format!("Address {number}"))
            .collect::<Vec<_>>();
        let next = remember_delivery_address(&saved, "Newest address");

        assert_eq!(next.len(), MAX_SAVED_DELIVERY_ADDRESSES);
        assert_eq!(next[0], "Newest address");
        assert!(!forget_delivery_address(&next, "Address 2").contains(&"Address 2".to_string()));
    }

    #[test]
    fn optimistic_price_updates_selected_addons_before_the_server_responds() {
        let rental = api::Rental {
            slug: "test-rv".into(),
            name: "Test RV".into(),
            category: "rv".into(),
            summary: String::new(),
            description: String::new(),
            capacity: 4,
            price_unit: "night".into(),
            base_rate: "100.00".into(),
            currency: "CAD".into(),
            min_units: 3,
            refundable_deposit: "1000.00".into(),
            hero_image_url: None,
            review_rating: None,
            review_count: 0,
        };
        let details = api::RentalResponse {
            rental: rental.clone(),
            media: Vec::new(),
            features: Vec::new(),
            addons: vec![api::RentalAddon {
                addon_key: "portable_bbq".into(),
                label: "Portable BBQ".into(),
                price: "50.00".into(),
                charge_type: "fixed".into(),
                is_recommended: false,
            }],
        };
        let delivery = api::DeliveryEstimate {
            resolved_address: "Kelowna, BC".into(),
            one_way_km: "22.0".into(),
            round_trip_km: "44.0".into(),
            delivery_fee: "150.00".into(),
            maximum_km: "150.0".into(),
            within_range: true,
        };
        let previous_quote = api::QuoteResponse {
            quote: api::Quote {
                quote_id: "quote".into(),
                rental_slug: rental.slug.clone(),
                starts_at: String::new(),
                ends_at: String::new(),
                guests: 2,
                units: 4,
                currency: "CAD".into(),
                subtotal: "847.00".into(),
                tax_total: "77.64".into(),
                refundable_deposit: "1000.00".into(),
                total: "924.64".into(),
                expires_at: String::new(),
            },
            items: vec![
                api::QuoteItem {
                    item_type: "rental".into(),
                    item_key: "test-rv".into(),
                    label: "Test RV x 4 night".into(),
                    quantity: "4".into(),
                    unit_price: "100.00".into(),
                    amount: "400.00".into(),
                },
                api::QuoteItem {
                    item_type: "fee".into(),
                    item_key: "rv_preparation".into(),
                    label: "RV Preparation Fee".into(),
                    quantity: "1".into(),
                    unit_price: "97.00".into(),
                    amount: "97.00".into(),
                },
                api::QuoteItem {
                    item_type: "delivery".into(),
                    item_key: "delivery".into(),
                    label: "Delivery and setup".into(),
                    quantity: "1".into(),
                    unit_price: "150.00".into(),
                    amount: "150.00".into(),
                },
                api::QuoteItem {
                    item_type: "protection".into(),
                    item_key: "stationary_plus".into(),
                    label: "Stationary Plus Protection".into(),
                    quantity: "4".into(),
                    unit_price: "50.00".into(),
                    amount: "200.00".into(),
                },
                api::QuoteItem {
                    item_type: "tax".into(),
                    item_key: "combined_tax".into(),
                    label: "Applicable taxes".into(),
                    quantity: "1".into(),
                    unit_price: "77.64".into(),
                    amount: "77.64".into(),
                },
            ],
        };

        let without_addon = optimistic_price(
            Some(&rental),
            Some(&details),
            &[],
            4,
            Some(&delivery),
            Some(&previous_quote),
        )
        .unwrap();
        let with_addon = optimistic_price(
            Some(&rental),
            Some(&details),
            &["portable_bbq".into()],
            4,
            Some(&delivery),
            Some(&previous_quote),
        )
        .unwrap();

        assert!(with_addon
            .lines
            .iter()
            .any(|line| line.label == "Portable BBQ"));
        assert_eq!(with_addon.total - without_addon.total, 56.0);
    }

    #[test]
    fn map_address_is_embedded_as_safe_json() {
        let script = booking_delivery_map_script(Some("Camp \"Okanagan\""));

        assert!(script.contains("\"Camp \\\"Okanagan\\\"\""));
        assert!(!script.contains("__DESTINATION_ADDRESS__"));
    }

    #[test]
    fn choosing_an_rv_first_returns_to_its_calendar() {
        assert_eq!(step_after_rental_selection(false), 1);
        assert_eq!(step_after_rental_selection(true), 3);
    }

    #[test]
    fn a_saved_payment_always_reopens_the_confirmation_step() {
        assert_eq!(initial_booking_step(1, true), 5);
        assert_eq!(initial_booking_step(4, false), 4);
        assert_eq!(initial_booking_step(9, false), 5);
    }

    #[test]
    fn embedded_checkout_script_returns_the_async_mount_result() {
        let script = embedded_checkout_script("\"pk_test_example\"", "\"secret_example\"");

        assert!(script.trim_start().starts_with("return await (async () =>"));
        assert!(script.contains("window.Stripe(\"pk_test_example\")"));
        assert!(script.contains("clientSecret: \"secret_example\""));
    }

    #[test]
    fn checkout_due_now_must_match_the_immutable_trip_total() {
        let mut booking = api::Booking {
            booking_id: "booking-1".into(),
            booking_number: "VL-1".into(),
            rental_slug: "test-rv".into(),
            rental_name: "Test RV".into(),
            status: "pending_payment".into(),
            payment_status: "unpaid".into(),
            starts_at: "2030-08-30T21:00:00Z".into(),
            ends_at: "2030-09-03T18:00:00Z".into(),
            currency: "CAD".into(),
            total: "1406.24".into(),
            amount_due_now: "421.87".into(),
            review_id: None,
            can_review: false,
        };

        assert!(booking_payment_amount_is_valid(&booking));
        booking.amount_due_now = "1406.24".into();
        assert!(booking_payment_amount_is_valid(&booking));
        booking.amount_due_now = "344.71".into();
        assert!(!booking_payment_amount_is_valid(&booking));
    }

    #[test]
    fn rental_selection_keeps_dates_only_when_server_marks_it_available() {
        assert!(rental_selection_keeps_dates(true, Some(true)));
        assert!(!rental_selection_keeps_dates(true, Some(false)));
        assert!(!rental_selection_keeps_dates(true, None));
        assert!(!rental_selection_keeps_dates(false, Some(true)));
    }

    #[test]
    fn selected_rv_calendar_respects_turnover_times() {
        let return_day = NaiveDate::from_ymd_opt(2030, 8, 10).unwrap();
        let blocked = vec![(
            booking_moment(NaiveDate::from_ymd_opt(2030, 8, 1).unwrap(), 14).unwrap(),
            booking_moment(return_day, 11).unwrap(),
        )];

        assert!(booking_date_is_selectable(
            return_day, None, None, 3, &blocked,
        ));
        assert!(!booking_stay_is_available(
            NaiveDate::from_ymd_opt(2030, 8, 8).unwrap(),
            NaiveDate::from_ymd_opt(2030, 8, 11).unwrap(),
            &blocked,
        ));
    }
}

use chrono::{DateTime, LocalResult, Months, NaiveDate, TimeZone, Utc};
use chrono_tz::America::Vancouver;
use dioxus::prelude::*;

use super::{
    catalog::{add_months, month_start, rental_fallback_image, rental_image, CatalogSearchMonth},
    terms::TermsAgreementContent,
};
use crate::{
    api,
    components::{Icon, ReviewForm},
    data::rv_gallery,
    pricing, AuthSession, Route,
};

const SAVED_DELIVERY_ADDRESSES: &str = "vl_delivery_addresses";
const MAX_SAVED_DELIVERY_ADDRESSES: usize = 5;
const SAVED_PENDING_PAYMENT: &str = "vl_pending_booking_payment";
const SAVED_POST_PAYMENT_BOOKING: &str = "vl_post_payment_booking";
const SAVED_DEPOSIT_INSTRUCTIONS: &str = "vl_damage_deposit_instructions";
const DAMAGE_DEPOSIT_ETRANSFER_EMAIL: &str = "protrailercare@gmail.com";
const MOBILE_CALENDAR_BREAKPOINT: f64 = 860.0;
const CALENDAR_SWIPE_THRESHOLD: f64 = 48.0;
const IMG_DESTINATION_FINTRY: Asset = asset!(
    "/assets/img/booking-destination-fintry.webp",
    AssetOptions::image().with_jpg()
);
const IMG_DESTINATION_BEAR_CREEK: Asset = asset!(
    "/assets/img/booking-destination-bear-creek.webp",
    AssetOptions::image().with_jpg()
);
const IMG_DESTINATION_SHUSWAP_LAKE: Asset = asset!(
    "/assets/img/booking-destination-shuswap-lake.webp",
    AssetOptions::image().with_jpg()
);
const IMG_DESTINATION_KEKULI_BAY: Asset = asset!(
    "/assets/img/booking-destination-kekuli-bay.webp",
    AssetOptions::image().with_jpg()
);
const UNMOUNT_EMBEDDED_CHECKOUT: &str = r#"
if (window.__vlEmbeddedCheckout) {
  try { window.__vlEmbeddedCheckout.destroy(); }
  catch (_) { try { window.__vlEmbeddedCheckout.unmount(); } catch (_) {} }
  window.__vlEmbeddedCheckout = null;
}
"#;
const GUIDE_TO_PAYMENT_TERMS: &str = r#"
await new Promise(resolve => window.setTimeout(resolve, 280));
const target = document.getElementById('ub-payment-terms-confirmation');
if (target) {
  const reducedMotion = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
  target.scrollIntoView({
    behavior: reducedMotion ? 'auto' : 'smooth',
    block: 'center',
    inline: 'nearest'
  });
  target.querySelector('input[type="checkbox"]')?.focus({ preventScroll: true });
}
"#;
const CLEANUP_BOOKING_EPHEMERALS: &str = r#"
const mapState = window.__vlBookingDeliveryMapState;
if (mapState?.map) {
  try { mapState.map.remove(); } catch (_) {}
}
delete window.__vlBookingDeliveryMapState;

const moneyFrames = window.__vlBookingMoneyFrames;
if (moneyFrames) {
  Object.values(moneyFrames).forEach(frame => cancelAnimationFrame(frame));
}
delete window.__vlBookingMoneyFrames;
delete window.__vlBookingMoneyValues;
"#;

type UnavailableRange = (DateTime<Utc>, DateTime<Utc>);

#[derive(Clone, PartialEq)]
struct DestinationRecommendation {
    id: &'static str,
    name: &'static str,
    region: &'static str,
    description: &'static str,
    address: &'static str,
    image: Asset,
}

fn destination_recommendations() -> Vec<DestinationRecommendation> {
    vec![
        DestinationRecommendation {
            id: "fintry",
            name: "Fintry Provincial Park",
            region: "Okanagan Lake · Fintry",
            description: "Waterfront camping, a natural sand beach and the historic Fintry Estate.",
            address: "7655 Fintry Delta Rd, Kelowna, BC V1Z 3B2",
            image: IMG_DESTINATION_FINTRY,
        },
        DestinationRecommendation {
            id: "bear-creek",
            name: "Bear Creek Provincial Park",
            region: "West Kelowna · Okanagan Lake",
            description:
                "Lakeside camping with a sandy beach, creek and canyon trails close to Kelowna.",
            address: "107 Westside Rd, Kelowna, BC V1Z 3S4",
            image: IMG_DESTINATION_BEAR_CREEK,
        },
        DestinationRecommendation {
            id: "shuswap-lake",
            name: "Shuswap Lake Provincial Park",
            region: "Scotch Creek · Shuswap Lake",
            description:
                "A popular family campground with a long beach, play areas and a boat launch.",
            address: "4120 Squilax-Anglemont Rd, Scotch Creek, BC V0E 1M5",
            image: IMG_DESTINATION_SHUSWAP_LAKE,
        },
        DestinationRecommendation {
            id: "kekuli-bay",
            name: "Kekuli Bay Provincial Park",
            region: "Vernon · Kalamalka Lake",
            description:
                "Lake-view campsites beside the Okanagan Rail Trail, swimming and boating.",
            address: "421 High Ridge Rd, Vernon, BC V1H 1G1",
            image: IMG_DESTINATION_KEKULI_BAY,
        },
    ]
}

pub(crate) fn has_saved_pending_payment() -> bool {
    api::load_sensitive_json::<api::CreatedBooking>(SAVED_PENDING_PAYMENT)
        .is_some_and(|created| created.client_secret.is_some() && !created.access_token.is_empty())
}

pub(crate) fn should_open_booking_overlay(
    resume_after_auth: bool,
    has_pending_payment: bool,
) -> bool {
    resume_after_auth || has_pending_payment
}

fn booking_overlay_close_blocked(
    closing: bool,
    booking_busy: bool,
    auth_busy: bool,
    all_in_busy: bool,
    edit_booking_busy: bool,
) -> bool {
    closing || booking_busy || auth_busy || all_in_busy || edit_booking_busy
}

fn booking_creation_recovery_message(error: &api::ApiError) -> Option<&'static str> {
    if error.code == "service_unavailable" {
        Some(
            "Secure payment could not start. Your dates and selections are still here; the exact price is refreshing so you can try again.",
        )
    } else if error.is_conflict() {
        Some(
            "Availability or the saved price changed. Your selections are still here; review the refreshed total and try again.",
        )
    } else {
        None
    }
}

fn payment_overlay_close_blocked(
    all_in_busy: bool,
    edit_booking_busy: bool,
    payment_phase: &str,
) -> bool {
    all_in_busy || edit_booking_busy || matches!(payment_phase, "switching" | "confirming")
}

fn payment_terms_allow_checkout(accepted: bool) -> bool {
    accepted
}

#[derive(Clone)]
struct DatedRentalMatches {
    starts_on: Option<NaiveDate>,
    ends_on: Option<NaiveDate>,
    guests: i32,
    result: Result<Vec<api::Rental>, String>,
}

#[derive(Clone)]
struct FleetAvailabilityMatch {
    starts_on: NaiveDate,
    ends_on: NaiveDate,
    guests: i32,
    result: Result<api::FleetAvailabilityResponse, String>,
}

fn availability_ranges(value: &api::AvailabilityResponse) -> Result<Vec<UnavailableRange>, String> {
    value
        .unavailable
        .iter()
        .map(|interval| {
            let start = DateTime::parse_from_rfc3339(&interval.starts_at)
                .map_err(|_| {
                    "Live availability returned an invalid blocked start time.".to_string()
                })?
                .with_timezone(&Utc);
            let end = DateTime::parse_from_rfc3339(&interval.ends_at)
                .map_err(|_| "Live availability returned an invalid blocked end time.".to_string())?
                .with_timezone(&Utc);
            if end <= start {
                return Err("Live availability returned an invalid blocked date range.".into());
            }
            Ok((start, end))
        })
        .collect()
}

fn validated_booking_availability(
    value: &api::AvailabilityResponse,
    expected_slug: &str,
    required_start: NaiveDate,
    required_end: NaiveDate,
) -> Result<Vec<UnavailableRange>, String> {
    let response_start = NaiveDate::parse_from_str(&value.starts_on, "%Y-%m-%d")
        .map_err(|_| "Live availability returned an invalid start date.".to_string())?;
    let response_end = NaiveDate::parse_from_str(&value.ends_on, "%Y-%m-%d")
        .map_err(|_| "Live availability returned an invalid end date.".to_string())?;
    if value.rental_slug != expected_slug
        || response_start > required_start
        || response_end < required_end
        || value.timezone != "America/Vancouver"
        || value.delivery_time != "14:00"
        || value.return_time != "11:00"
        || value.minimum_nights < 3
    {
        return Err("Live availability returned an incomplete schedule. Please retry.".into());
    }
    availability_ranges(value)
}

fn validated_fleet_availability(
    value: &api::FleetAvailabilityResponse,
    required_start: NaiveDate,
    required_end: NaiveDate,
) -> Result<(Vec<Vec<UnavailableRange>>, Vec<NaiveDate>), String> {
    let response_start = NaiveDate::parse_from_str(&value.starts_on, "%Y-%m-%d")
        .map_err(|_| "Fleet availability returned an invalid start date.".to_string())?;
    let response_end = NaiveDate::parse_from_str(&value.ends_on, "%Y-%m-%d")
        .map_err(|_| "Fleet availability returned an invalid end date.".to_string())?;
    if response_start > required_start
        || response_end < required_end
        || value.timezone != "America/Vancouver"
        || value.delivery_time != "14:00"
        || value.return_time != "11:00"
        || value.minimum_nights != 3
        || value.total_rentals != value.rentals.len()
    {
        return Err("Fleet availability returned an incomplete schedule. Please retry.".into());
    }

    let mut rental_slugs = std::collections::HashSet::new();
    let mut schedules = Vec::with_capacity(value.rentals.len());
    for rental in &value.rentals {
        if rental.rental_slug.trim().is_empty() || !rental_slugs.insert(rental.rental_slug.as_str())
        {
            return Err("Fleet availability returned an invalid RV schedule. Please retry.".into());
        }
        let availability = api::AvailabilityResponse {
            rental_slug: rental.rental_slug.clone(),
            starts_on: value.starts_on.clone(),
            ends_on: value.ends_on.clone(),
            unavailable: rental.unavailable.clone(),
            delivery_time: value.delivery_time.clone(),
            return_time: value.return_time.clone(),
            timezone: value.timezone.clone(),
            minimum_nights: value.minimum_nights,
        };
        schedules.push(availability_ranges(&availability)?);
    }

    let unavailable_start_dates = value
        .unavailable_start_dates
        .iter()
        .map(|day| {
            let day = NaiveDate::parse_from_str(day, "%Y-%m-%d").map_err(|_| {
                "Fleet availability returned an invalid unavailable date.".to_string()
            })?;
            if day < response_start || day >= response_end {
                return Err("Fleet availability returned an out-of-range unavailable date.".into());
            }
            Ok(day)
        })
        .collect::<Result<Vec<_>, String>>()?;
    let unavailable_start_set = unavailable_start_dates
        .iter()
        .copied()
        .collect::<std::collections::HashSet<_>>();
    if unavailable_start_set.len() != unavailable_start_dates.len() {
        return Err("Fleet availability returned duplicate unavailable dates.".into());
    }
    let last_start = required_end - chrono::Duration::days(value.minimum_nights);
    let mut day = required_start;
    while day < last_start {
        let no_rv_available =
            fleet_available_rental_count(day, None, None, value.minimum_nights, &schedules) == 0;
        if unavailable_start_set.contains(&day) != no_rv_available {
            return Err("Fleet availability returned inconsistent unavailable dates.".into());
        }
        day += chrono::Duration::days(1);
    }

    Ok((schedules, unavailable_start_dates))
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
        let blocked_until = day.max(start + chrono::Duration::days(minimum_nights));
        return booking_stay_is_available(start, blocked_until, unavailable);
    }
    booking_stay_is_available(
        day,
        day + chrono::Duration::days(minimum_nights),
        unavailable,
    )
}

fn fleet_available_rental_count(
    day: NaiveDate,
    selected_start: Option<NaiveDate>,
    selected_end: Option<NaiveDate>,
    minimum_nights: i64,
    rental_schedules: &[Vec<UnavailableRange>],
) -> usize {
    rental_schedules
        .iter()
        .filter(|unavailable| {
            booking_date_is_selectable(
                day,
                selected_start,
                selected_end,
                minimum_nights,
                unavailable,
            )
        })
        .count()
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

fn calendar_swipe_month_delta(start: (f64, f64), end: (f64, f64)) -> i32 {
    let distance_x = end.0 - start.0;
    let distance_y = end.1 - start.1;
    if distance_x.abs() < CALENDAR_SWIPE_THRESHOLD || distance_x.abs() <= distance_y.abs() * 1.2 {
        0
    } else if distance_x < 0.0 {
        1
    } else {
        -1
    }
}

fn mobile_calendar_swipe_enabled() -> bool {
    web_sys::window()
        .and_then(|window| window.inner_width().ok())
        .and_then(|width| width.as_f64())
        .is_some_and(|width| width <= MOBILE_CALENDAR_BREAKPOINT)
}

fn money_cents(value: &str) -> Option<i64> {
    let value = value.trim();
    let (whole, fraction) = value.split_once('.').unwrap_or((value, ""));
    if whole.is_empty()
        || !whole.bytes().all(|byte| byte.is_ascii_digit())
        || fraction.len() > 2
        || !fraction.bytes().all(|byte| byte.is_ascii_digit())
    {
        return None;
    }
    let whole = whole.parse::<i64>().ok()?;
    let fraction = match fraction.len() {
        0 => 0,
        1 => fraction.parse::<i64>().ok()?.checked_mul(10)?,
        _ => fraction.parse::<i64>().ok()?,
    };
    whole.checked_mul(100)?.checked_add(fraction)
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

fn created_payment_amount_is_valid(created: &api::CreatedBooking) -> bool {
    if created.payment_option != "all_in" {
        return booking_payment_amount_is_valid(&created.booking);
    }
    let Some(offer) = created.all_in_offer.as_ref() else {
        return false;
    };
    let (Some(trip), Some(deposit), Some(total)) = (
        money_cents(&offer.trip_price),
        money_cents(&offer.refundable_deposit),
        money_cents(&offer.total_due_today),
    ) else {
        return false;
    };
    trip == money_cents(&created.booking.total).unwrap_or_default()
        && deposit > 0
        && trip.checked_add(deposit) == Some(total)
        && offer.currency == created.booking.currency
}

fn booking_payment_percent(booking: &api::Booking) -> Option<i32> {
    let total = money_cents(&booking.total)?;
    let due_now = money_cents(&booking.amount_due_now)?;
    if due_now == total {
        Some(100)
    } else if due_now == (total * i64::from(pricing::BOOKING_DEPOSIT_PERCENT) + 50) / 100 {
        Some(pricing::BOOKING_DEPOSIT_PERCENT)
    } else {
        None
    }
}

fn money_from_cents(cents: i64) -> String {
    format!("{}.{:02}", cents / 100, cents.rem_euclid(100))
}

fn booking_remaining_balance(booking: &api::Booking) -> Option<String> {
    let remaining = money_cents(&booking.total)? - money_cents(&booking.amount_due_now)?;
    (remaining > 0).then(|| money_from_cents(remaining))
}

fn quote_matches_booking(quote: &api::QuoteResponse, booking: &api::Booking) -> bool {
    !booking.quote_id.is_empty()
        && quote.quote.quote_id == booking.quote_id
        && money_cents(&quote.quote.total) == money_cents(&booking.total)
}

fn draft_matches_booking(draft: &api::TripDraft, booking: &api::Booking) -> bool {
    !booking.rental_slug.is_empty()
        && draft.rental_slug == booking.rental_slug
        && draft.starts_on == display_booking_date(&booking.starts_at)
        && draft.ends_on == display_booking_date(&booking.ends_at)
}

fn fill_booking_rental(booking: &mut api::Booking, rental_slug: String, rental_name: String) {
    if booking.rental_slug.is_empty() {
        booking.rental_slug = rental_slug;
    }
    if booking.rental_name.is_empty() {
        booking.rental_name = rental_name;
    }
}

fn embedded_checkout_script(publishable_key: &str, client_secret: &str) -> String {
    format!(
        r#"
return await (async () => {{
  if (!window.Stripe) {{
    await new Promise((resolve, reject) => {{
      const timeout = window.setTimeout(() => reject(new Error('Stripe.js timed out')), 15000);
      const loaded = () => {{ window.clearTimeout(timeout); resolve(); }};
      const failed = () => {{ window.clearTimeout(timeout); reject(new Error('Stripe.js failed to load')); }};
      const current = document.querySelector('script[data-vl-stripe]');
      if (current) {{
        current.addEventListener('load', loaded, {{ once: true }});
        current.addEventListener('error', failed, {{ once: true }});
        if (window.Stripe) loaded();
        return;
      }}
      const script = document.createElement('script');
      script.src = 'https://js.stripe.com/v3/';
      script.async = true;
      script.dataset.vlStripe = 'true';
      script.onload = loaded;
      script.onerror = failed;
      document.head.appendChild(script);
    }});
  }}
  const root = document.getElementById('vl-embedded-checkout');
  if (!root || !window.Stripe) throw new Error('Stripe Checkout is unavailable');
  if (window.__vlEmbeddedCheckout) {{
    try {{ window.__vlEmbeddedCheckout.destroy(); }}
    catch (_) {{ try {{ window.__vlEmbeddedCheckout.unmount(); }} catch (_) {{}} }}
    window.__vlEmbeddedCheckout = null;
  }}
  root.replaceChildren();
  const stripe = window.Stripe({publishable_key});
  const checkout = await stripe.initEmbeddedCheckout({{
    fetchClientSecret: async () => {client_secret},
    onComplete: () => {{ root.dataset.complete = 'true'; }}
  }});
  checkout.mount('#vl-embedded-checkout');
  window.__vlEmbeddedCheckout = checkout;
  return 'mounted';
}})();
"#
    )
}

fn checkout_status_poll_due(iteration: u16, checkout_complete: bool) -> bool {
    checkout_complete || iteration.is_multiple_of(4)
}

fn payment_checkout_may_start(overlay_open: bool, terms_accepted: bool, phase: &str) -> bool {
    overlay_open && terms_accepted && phase == "idle"
}

fn rental_selection_keeps_dates(trip_ready: bool, available_for_dates: Option<bool>) -> bool {
    trip_ready && available_for_dates == Some(true)
}

fn displayed_rental_choices<'a, T>(
    all_rentals: &'a [T],
    available_rentals: Option<&'a [T]>,
    trip_ready: bool,
) -> &'a [T] {
    if trip_ready {
        available_rentals.unwrap_or_default()
    } else {
        all_rentals
    }
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

fn display_booking_moment(timestamp: &str) -> String {
    DateTime::parse_from_rfc3339(timestamp)
        .map(|value| {
            value
                .with_timezone(&Vancouver)
                .format("%A, %B %-d, %Y · %-I:%M %p")
                .to_string()
        })
        .unwrap_or_else(|_| timestamp.to_string())
}

fn booking_stay_nights(booking: &api::Booking) -> Option<i64> {
    let starts_on = DateTime::parse_from_rfc3339(&booking.starts_at)
        .ok()?
        .with_timezone(&Vancouver)
        .date_naive();
    let ends_on = DateTime::parse_from_rfc3339(&booking.ends_at)
        .ok()?
        .with_timezone(&Vancouver)
        .date_naive();
    let nights = (ends_on - starts_on).num_days();
    (nights >= 1).then_some(nights)
}

fn display_deposit_due(timestamp: &str) -> String {
    DateTime::parse_from_rfc3339(timestamp)
        .map(|value| {
            value
                .with_timezone(&Vancouver)
                .format("%B %-d, %Y at %-I:%M %p")
                .to_string()
        })
        .unwrap_or_else(|_| timestamp.to_string())
}

fn display_delivery_deposit_due(delivery_timestamp: &str) -> String {
    DateTime::parse_from_rfc3339(delivery_timestamp)
        .map(|value| display_deposit_due(&(value - chrono::Duration::hours(48)).to_rfc3339()))
        .unwrap_or_else(|_| delivery_timestamp.to_string())
}

fn price_number(value: &str) -> f64 {
    value
        .chars()
        .filter(|character| character.is_ascii_digit() || *character == '.')
        .collect::<String>()
        .parse()
        .unwrap_or(0.0)
}

fn delivery_distance_detail(delivery: &api::DeliveryEstimate) -> String {
    let one_way_km = price_number(&delivery.one_way_km);
    if one_way_km <= api::DELIVERY_INCLUDED_KM {
        format!("{} km one way · covered by CA$150", delivery.one_way_km)
    } else {
        format!(
            "{} km one way · {:.1} km extra × CA${:.2} · each way",
            delivery.one_way_km,
            one_way_km - api::DELIVERY_INCLUDED_KM,
            api::DELIVERY_PRICE_PER_LEG_KM
        )
    }
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
    let requested_nights = nights.max(0);
    let billable_nights = if requested_nights > 0 {
        requested_nights.max(3)
    } else {
        0
    };
    let rental_amount = price_number(&rental.base_rate) * billable_nights as f64;
    let mut lines = vec![OptimisticPriceLine {
        key: format!("rental-{}", rental.slug),
        label: if requested_nights < billable_nights {
            format!("{} x {} nights (minimum)", rental.name, billable_nights)
        } else {
            format!("{} x {} nights", rental.name, billable_nights)
        },
        detail: (requested_nights < billable_nights).then(|| {
            let label = if requested_nights == 1 {
                "night"
            } else {
                "nights"
            };
            format!("Your selected stay: {requested_nights} {label}")
        }),
        amount: rental_amount,
    }];
    let mut taxable_subtotal = rental_amount;

    if let Some(details) = details {
        for addon in details
            .addons
            .iter()
            .filter(|addon| addon_quantity(selected_addons, &addon.addon_key) > 0)
        {
            let selected_quantity = addon_quantity(selected_addons, &addon.addon_key);
            let quantity = if addon.charge_type == "per_unit" {
                billable_nights as f64
            } else {
                selected_quantity as f64
            };
            let amount = price_number(&addon.price) * quantity;
            taxable_subtotal += amount;
            lines.push(OptimisticPriceLine {
                key: format!("addon-{}", addon.addon_key),
                label: addon.label.clone(),
                detail: if is_bedding_addon(&addon.addon_key) {
                    Some(format!(
                        "{} {} × CA${}",
                        selected_quantity,
                        if selected_quantity == 1 {
                            "bed"
                        } else {
                            "beds"
                        },
                        addon.price
                    ))
                } else {
                    (quantity > 1.0)
                        .then(|| format!("{} nights × CA${}", billable_nights, addon.price))
                },
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
        detail: Some(delivery_distance_detail(delivery)),
        amount: delivery_amount,
    });

    let protection = pricing::stationary_plus_amount(billable_nights);
    lines.push(OptimisticPriceLine {
        key: "stationary-plus".into(),
        label: "Stationary Plus Protection".into(),
        detail: Some(pricing::stationary_plus_detail(billable_nights)),
        amount: protection,
    });

    let mut tax = 0.0;
    if let Some(previous) = previous_quote {
        let previous_taxable = previous
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
        let previous_protection = previous
            .items
            .iter()
            .filter(|item| item.item_type == "protection")
            .map(|item| price_number(&item.amount))
            .sum::<f64>();
        let primary_tax = previous
            .items
            .iter()
            .find(|item| item.item_key == "tax_primary");
        let secondary_tax = previous
            .items
            .iter()
            .find(|item| item.item_key == "tax_secondary");

        if primary_tax.is_some() || secondary_tax.is_some() {
            if let Some(item) = primary_tax {
                let previous_base = previous_taxable + previous_protection;
                if previous_base > 0.0 {
                    let rate = price_number(&item.amount) / previous_base;
                    let amount = ((taxable_subtotal + protection) * rate * 100.0).round() / 100.0;
                    tax += amount;
                    lines.push(OptimisticPriceLine {
                        key: item.item_key.clone(),
                        label: item.label.clone(),
                        detail: None,
                        amount,
                    });
                }
            }
            if let Some(item) = secondary_tax {
                if previous_taxable > 0.0 {
                    let rate = price_number(&item.amount) / previous_taxable;
                    let amount = (taxable_subtotal * rate * 100.0).round() / 100.0;
                    tax += amount;
                    lines.push(OptimisticPriceLine {
                        key: item.item_key.clone(),
                        label: item.label.clone(),
                        detail: None,
                        amount,
                    });
                }
            }
        } else if previous_taxable > 0.0 {
            let previous_tax = previous
                .items
                .iter()
                .filter(|item| item.item_type == "tax")
                .map(|item| price_number(&item.amount))
                .sum::<f64>();
            tax = (taxable_subtotal * (previous_tax / previous_taxable) * 100.0).round() / 100.0;
            if tax > 0.0 {
                lines.push(OptimisticPriceLine {
                    key: "tax".into(),
                    label: "Applicable taxes".into(),
                    detail: None,
                    amount: tax,
                });
            }
        }
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

async fn guide_to_payment_terms() {
    let _ = document::eval(GUIDE_TO_PAYMENT_TERMS).await;
}

fn addon_description(key: &str) -> &'static str {
    if key == "bbq_fuel" {
        "Fuel supplied for the portable BBQ."
    } else if key == "generator_fuel" {
        "Fuel supplied for the portable generator."
    } else if key.contains("bbq") {
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

const BEDDING_ADDON_KEY: &str = "linens";
const MAX_BEDDING_QUANTITY: usize = 4;

fn is_bedding_addon(key: &str) -> bool {
    key == BEDDING_ADDON_KEY
}

fn addon_quantity(addon_keys: &[String], key: &str) -> usize {
    addon_keys
        .iter()
        .filter(|selected| selected.as_str() == key)
        .count()
}

fn remove_one_addon(addon_keys: &mut Vec<String>, key: &str) {
    if let Some(index) = addon_keys.iter().rposition(|selected| selected == key) {
        addon_keys.remove(index);
    }
}

fn selected_addon_count(addon_keys: &[String]) -> usize {
    addon_keys
        .iter()
        .collect::<std::collections::HashSet<_>>()
        .len()
}

#[component]
fn AddonApiIcon(name: String) -> Element {
    let safe = match name.as_str() {
        "flame" | "bed-double" | "paw-print" | "utensils" | "shower-head" | "snowflake"
        | "wifi" | "tv" | "battery-charging" | "plug-zap" | "cooking-pot" | "tent-tree"
        | "caravan" | "shield-check" | "package" | "fuel" | "trash-2" | "circle-check"
        | "sparkles" => name,
        _ => "sparkles".into(),
    };
    rsx! { i { class: "icon-{safe}", style: "font-size: 18px; color: var(--vl-forest);" } }
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
fn DestinationRecommendationCard(
    recommendation: DestinationRecommendation,
    on_select: EventHandler<String>,
) -> Element {
    let address = recommendation.address.to_string();
    rsx! {
        article { class: "ub-destination-card",
            img {
                class: "ub-destination-image",
                src: "{recommendation.image}",
                alt: "Scenic lake view for {recommendation.name}",
                loading: "lazy",
            }
            div { class: "ub-destination-card-body",
                span { class: "ub-destination-region", "{recommendation.region}" }
                h3 { "{recommendation.name}" }
                p { "{recommendation.description}" }
                div { class: "ub-destination-address",
                    Icon { name: "map-pin", size: 14, color: "var(--vl-muted)" }
                    span { "{recommendation.address}" }
                }
                button {
                    class: "ub-destination-select",
                    r#type: "button",
                    onclick: move |_| on_select.call(address.clone()),
                    span { "Use this address" }
                    Icon { name: "arrow-right", size: 14, color: "var(--vl-white)" }
                }
            }
        }
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
    let mut user = use_context::<AuthSession>().0;
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
    let current_user = user.read().clone();
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
        document::eval(CLEANUP_BOOKING_EPHEMERALS);
    });

    let today = Utc::now().with_timezone(&Vancouver).date_naive();
    let initial_month = month_start(today);
    let initial_pending_payment =
        api::load_sensitive_json::<api::CreatedBooking>(SAVED_PENDING_PAYMENT)
            .filter(|created| created.client_secret.is_some() && !created.access_token.is_empty());
    let initial_locked_quote =
        api::load_json::<api::QuoteResponse>("vl_active_quote").filter(|quote| {
            initial_pending_payment
                .as_ref()
                .is_some_and(|created| quote_matches_booking(quote, &created.booking))
        });
    let initial_payment_draft = api::load_json::<api::TripDraft>("vl_trip_draft").filter(|draft| {
        initial_pending_payment
            .as_ref()
            .is_some_and(|created| draft_matches_booking(draft, &created.booking))
    });
    let has_pending_payment = initial_pending_payment.is_some();
    let initial_deposit_instructions =
        api::load_sensitive_json::<api::CreatedBooking>(SAVED_DEPOSIT_INSTRUCTIONS);
    use_effect(move || {
        if !has_pending_payment && !resumed_booking {
            api::remove_saved("vl_active_quote");
            api::remove_saved("vl_trip_draft");
        }
    });
    let mut visible_month = use_signal(|| {
        (*starts_on.read())
            .map(month_start)
            .unwrap_or(initial_month)
    });
    let mut calendar_swipe_start = use_signal(|| None::<(f64, f64)>);
    let initial_slug = resumed_draft
        .as_ref()
        .map(|draft| draft.rental_slug.clone())
        .filter(|slug| !slug.is_empty())
        .or_else(|| {
            initial_locked_quote
                .as_ref()
                .map(|quote| quote.quote.rental_slug.clone())
                .filter(|slug| !slug.is_empty())
        })
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
    let mut address_request_version = use_signal(|| 0_u32);
    let mut suggestions_open = use_signal(|| false);
    let mut destination_picker_open = use_signal(|| false);
    let mut saved_addresses =
        use_signal(|| api::load_json::<Vec<String>>(SAVED_DELIVERY_ADDRESSES).unwrap_or_default());
    let mut addon_keys = use_signal(move || resumed_addons);
    let mut quote = use_signal(move || initial_locked_quote);
    let mut quote_busy = use_signal(|| false);
    let mut quote_error = use_signal(String::new);
    let mut stationary_plus_details_open = use_signal(|| false);
    let mut addon_notice = use_signal(String::new);
    let mut quote_version = use_signal(|| 0_u32);
    let mut quote_refresh_nonce = use_signal(|| 0_u32);
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
    let mut accepted = use_signal(move || resumed_accepted_terms || has_pending_payment);
    let mut booking_busy = use_signal(|| false);
    let mut booking_error = use_signal(String::new);
    let mut payment_config = use_signal(|| None::<api::PaymentConfig>);
    let mut payment_config_error = use_signal(String::new);
    let mut payment_config_retry = use_signal(|| 0_u32);
    let mut rental_choices_retry = use_signal(|| 0_u32);
    let mut selected_availability_retry = use_signal(|| 0_u32);
    let mut fleet_availability_retry = use_signal(|| 0_u32);
    let mut pending_payment = use_signal(move || initial_pending_payment);
    let mut payment_overlay_open = use_signal(move || has_pending_payment);
    let mut payment_phase = use_signal(|| "idle".to_string());
    let mut payment_attempt_nonce = use_signal(|| 0_u32);
    let mut payment_terms_accepted = use_signal(move || resumed_accepted_terms);
    let mut payment_terms_open = use_signal(|| false);
    let mut deposit_overlay_booking = use_signal(move || initial_deposit_instructions);
    let mut deposit_copy_state = use_signal(String::new);
    let mut all_in_busy = use_signal(|| false);
    let mut all_in_error = use_signal(String::new);
    let mut edit_booking_confirm_open = use_signal(|| false);
    let mut edit_booking_busy = use_signal(|| false);
    let mut edit_booking_error = use_signal(String::new);
    let navigator = use_navigator();
    let google_href = api::google_login_url();
    let facebook_href = api::FACEBOOK_AUTH_ENABLED.then(api::facebook_login_url);
    let route = use_route::<Route>();
    let google_return = api::current_auth_return_path().unwrap_or_else(|| route.to_string());
    let facebook_return = google_return.clone();

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
        let request_version = address_request_version.peek().wrapping_add(1);
        address_request_version.set(request_version);
        address_busy.set(true);
        spawn(async move {
            let result = api::delivery_estimate(&slug, &address).await;
            if *address_request_version.peek() != request_version
                || selected_slug.peek().as_str() != slug.as_str()
            {
                return;
            }
            match result {
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
        let overlay_open = *payment_overlay_open.read();
        let terms_accepted = *payment_terms_accepted.read();
        let has_pending_payment = pending_payment.read().is_some();
        if overlay_open && has_pending_payment && !terms_accepted {
            spawn(guide_to_payment_terms());
        }
    });

    use_effect(move || {
        let attempt = *payment_attempt_nonce.read();
        let pending = pending_payment.read().clone();
        let overlay_open = *payment_overlay_open.read();
        let terms_accepted = *payment_terms_accepted.read();
        let config = payment_config.read().clone();
        let availability =
            api::payment_availability(config.as_ref(), !payment_config_error.read().is_empty());
        if !payment_checkout_may_start(overlay_open, terms_accepted, payment_phase.read().as_str())
        {
            return;
        }
        let Some(created) = pending else {
            return;
        };
        if !created_payment_amount_is_valid(&created) {
            payment_phase.set("blocked".into());
            booking_error.set(
                "Checkout was blocked because the payment amount does not match the immutable booking total. No second booking was created."
                    .into(),
            );
            return;
        }
        payment_phase.set("checking".into());
        spawn(async move {
            let status_result =
                api::booking_status(&created.booking.booking_id, &created.access_token).await;
            if *payment_attempt_nonce.peek() != attempt || !*payment_overlay_open.peek() {
                return;
            }
            match status_result {
                Ok(status) if status.confirmed || status.status == "confirmed" => {
                    let mut confirmed = created.clone();
                    confirmed.booking.status = status.status;
                    confirmed.booking.payment_status = status.payment_status;
                    confirmed.damage_deposit = status.damage_deposit;
                    if confirmed.booking.rental_slug.is_empty() {
                        confirmed.booking.rental_slug = selected_slug.peek().clone();
                    }
                    let _ = api::save_sensitive_json("vl_last_booking", &confirmed);
                    let _ = api::save_sensitive_json(SAVED_POST_PAYMENT_BOOKING, &confirmed);
                    let _ = api::save_sensitive_json(SAVED_DEPOSIT_INSTRUCTIONS, &confirmed);
                    api::remove_sensitive_saved(SAVED_PENDING_PAYMENT);
                    pending_payment.set(None);
                    payment_overlay_open.set(false);
                    payment_phase.set("confirmed".into());
                    deposit_overlay_booking.set(Some(confirmed));
                    return;
                }
                Ok(status) if matches!(status.status.as_str(), "expired" | "cancelled") => {
                    api::remove_sensitive_saved(SAVED_PENDING_PAYMENT);
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

            if *payment_attempt_nonce.peek() != attempt || !*payment_overlay_open.peek() {
                return;
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
                    booking_error.set("Secure Checkout is blocked because the Stripe configuration could not be verified. Your existing reservation has not been recreated or charged.".into());
                    return;
                }
                (api::PaymentAvailability::Ready, Some(config)) => config,
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
                Ok(_)
                    if *payment_attempt_nonce.peek() == attempt && *payment_overlay_open.peek() =>
                {
                    payment_phase.set("checkout".into())
                }
                Ok(_) => {
                    document::eval(UNMOUNT_EMBEDDED_CHECKOUT);
                    return;
                }
                Err(_) => {
                    if *payment_attempt_nonce.peek() != attempt || !*payment_overlay_open.peek() {
                        return;
                    }
                    payment_phase.set("error".into());
                    booking_error.set(
                        "Secure test checkout could not load. Keep this window open and try again."
                            .into(),
                    );
                    return;
                }
            }

            let mut submitted = false;
            for poll_iteration in 0..800_u16 {
                if *payment_attempt_nonce.peek() != attempt || !*payment_overlay_open.peek() {
                    return;
                }
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
                if checkout_status_poll_due(poll_iteration, submitted) {
                    let status_result =
                        api::booking_status(&created.booking.booking_id, &created.access_token)
                            .await;
                    if *payment_attempt_nonce.peek() != attempt || !*payment_overlay_open.peek() {
                        return;
                    }
                    match status_result {
                        Ok(status) if status.confirmed || status.status == "confirmed" => {
                            let mut confirmed = created.clone();
                            confirmed.booking.status = status.status;
                            confirmed.booking.payment_status = status.payment_status;
                            confirmed.damage_deposit = status.damage_deposit;
                            if confirmed.booking.rental_slug.is_empty() {
                                confirmed.booking.rental_slug = selected_slug.peek().clone();
                            }
                            let _ = api::save_sensitive_json("vl_last_booking", &confirmed);
                            let _ =
                                api::save_sensitive_json(SAVED_POST_PAYMENT_BOOKING, &confirmed);
                            let _ =
                                api::save_sensitive_json(SAVED_DEPOSIT_INSTRUCTIONS, &confirmed);
                            api::remove_sensitive_saved(SAVED_PENDING_PAYMENT);
                            pending_payment.set(None);
                            payment_overlay_open.set(false);
                            payment_phase.set("confirmed".into());
                            deposit_overlay_booking.set(Some(confirmed));
                            return;
                        }
                        Ok(status) if matches!(status.status.as_str(), "expired" | "cancelled") => {
                            api::remove_sensitive_saved(SAVED_PENDING_PAYMENT);
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
                }
                if submitted {
                    break;
                }
                let _ = document::eval("await new Promise(resolve => setTimeout(resolve, 1500));")
                    .await;
            }
            if *payment_attempt_nonce.peek() != attempt || !*payment_overlay_open.peek() {
                return;
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
                if *payment_attempt_nonce.peek() != attempt || !*payment_overlay_open.peek() {
                    return;
                }
                let status_result =
                    api::booking_status(&created.booking.booking_id, &created.access_token).await;
                if *payment_attempt_nonce.peek() != attempt || !*payment_overlay_open.peek() {
                    return;
                }
                match status_result {
                    Ok(status) if status.confirmed || status.status == "confirmed" => {
                        let mut confirmed = created.clone();
                        confirmed.booking.status = status.status;
                        confirmed.booking.payment_status = status.payment_status;
                        confirmed.damage_deposit = status.damage_deposit;
                        if confirmed.booking.rental_slug.is_empty() {
                            confirmed.booking.rental_slug = selected_slug.peek().clone();
                        }
                        let _ = api::save_sensitive_json("vl_last_booking", &confirmed);
                        let _ = api::save_sensitive_json(SAVED_POST_PAYMENT_BOOKING, &confirmed);
                        let _ = api::save_sensitive_json(SAVED_DEPOSIT_INSTRUCTIONS, &confirmed);
                        api::remove_sensitive_saved(SAVED_PENDING_PAYMENT);
                        pending_payment.set(None);
                        payment_overlay_open.set(false);
                        payment_phase.set("confirmed".into());
                        deposit_overlay_booking.set(Some(confirmed));
                        return;
                    }
                    Ok(status) if matches!(status.status.as_str(), "expired" | "cancelled") => {
                        api::remove_sensitive_saved(SAVED_PENDING_PAYMENT);
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
            if *payment_attempt_nonce.peek() != attempt || !*payment_overlay_open.peek() {
                return;
            }
            payment_phase.set("delayed".into());
            booking_error.set(
                "Stripe is still confirming the payment. Keep this booking number and refresh its status shortly."
                    .into(),
            );
        });
    });

    let nights = starts_on
        .read()
        .zip(*ends_on.read())
        .map(|(start, end)| (end - start).num_days())
        .unwrap_or(0);
    let trip_ready = nights >= 1;
    let payment_locked = pending_payment.read().is_some();
    let payment_availability = api::payment_availability(
        payment_config.read().as_ref(),
        !payment_config_error.read().is_empty(),
    );
    let booking_can_submit = matches!(
        payment_availability,
        api::PaymentAvailability::Disabled | api::PaymentAvailability::Ready
    );
    let contact_complete = first_name.read().trim().len() >= 2
        && last_name.read().trim().len() >= 2
        && booking_email.read().contains('@')
        && phone.read().trim().len() >= 7;
    let mut trip_was_ready = use_signal(|| trip_ready);
    let all_rentals = use_resource(move || {
        let _retry = *rental_choices_retry.read();
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
        let _retry = *rental_choices_retry.read();
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
        let _retry = *selected_availability_retry.read();
        let slug = selected_slug.read().clone();
        let selected_start_month = (*starts_on.read()).map(month_start);
        let current_month = *visible_month.read();
        let range_start = selected_start_month
            .map(|month| month.min(current_month))
            .unwrap_or(current_month);
        let range_end = add_months(current_month, 3) + chrono::Duration::days(3);
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
    let fleet_availability = use_resource(move || {
        let _retry = *fleet_availability_retry.read();
        let selected_start_month = (*starts_on.read()).map(month_start);
        let current_month = *visible_month.read();
        let range_start = selected_start_month
            .map(|month| month.min(current_month))
            .unwrap_or(current_month);
        let range_end = add_months(current_month, 3) + chrono::Duration::days(3);
        let selected_guests = *guests.read();
        async move {
            let result = api::fleet_availability(
                &range_start.to_string(),
                &range_end.to_string(),
                selected_guests,
            )
            .await;
            FleetAvailabilityMatch {
                starts_on: range_start,
                ends_on: range_end,
                guests: selected_guests,
                result,
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
    let reconciliation_details = rental_details;
    use_effect(move || {
        let active_keys = reconciliation_details
            .read()
            .as_ref()
            .and_then(|result| result.as_ref().ok())
            .and_then(|result| result.as_ref())
            .map(|details| {
                details
                    .addons
                    .iter()
                    .filter(|addon| addon.is_active)
                    .map(|addon| addon.addon_key.clone())
                    .collect::<std::collections::HashSet<_>>()
            });
        let Some(active_keys) = active_keys else {
            return;
        };
        let mut selected = addon_keys.read().clone();
        let previous_len = selected.len();
        selected.retain(|key| active_keys.contains(key));
        if selected.len() != previous_len {
            addon_keys.set(selected);
            addon_notice.set(
                "One or more previously selected extras are no longer available and were removed."
                    .into(),
            );
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
    let rental_choice_values = displayed_rental_choices(
        &rental_values,
        available_rental_values.as_deref(),
        trip_ready,
    )
    .to_vec();
    let rental_choices_loading = if trip_ready {
        current_dated_matches.is_none() && available_rentals_error.is_none()
    } else {
        all_rentals.read().is_none()
    };
    let rental_choices_error = if trip_ready {
        available_rentals_error.clone()
    } else {
        all_rentals_error.clone()
    };
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
    let request_availability_error = selected_availability
        .read()
        .as_ref()
        .and_then(|result| result.as_ref().err())
        .cloned();
    let calendar_range_end = add_months(*visible_month.read(), 3);
    let calendar_required_end = calendar_range_end + chrono::Duration::days(3);
    let fleet_range_start = (*starts_on.read())
        .map(month_start)
        .map(|month| month.min(*visible_month.read()))
        .unwrap_or(*visible_month.read());
    let fleet_range_end = calendar_required_end;
    let current_fleet_match = fleet_availability
        .read()
        .as_ref()
        .cloned()
        .filter(|matches| {
            matches.starts_on == fleet_range_start
                && matches.ends_on == fleet_range_end
                && matches.guests == *guests.read()
        });
    let fleet_response = current_fleet_match
        .as_ref()
        .and_then(|matches| matches.result.as_ref().ok());
    let request_fleet_availability_error = current_fleet_match
        .as_ref()
        .and_then(|matches| matches.result.as_ref().err())
        .cloned();
    let validated_fleet = fleet_response.map(|value| {
        validated_fleet_availability(value, *visible_month.read(), calendar_required_end)
    });
    let contract_fleet_availability_error = validated_fleet
        .as_ref()
        .and_then(|result| result.as_ref().err())
        .cloned();
    let fleet_availability_error =
        request_fleet_availability_error.or(contract_fleet_availability_error);
    let fleet_availability_is_current =
        fleet_response.is_some() && fleet_availability_error.is_none();
    let fleet_availability_pending =
        current_fleet_match.is_none() && fleet_availability_error.is_none();
    let (fleet_schedules, _server_unavailable_start_dates) =
        validated_fleet.and_then(Result::ok).unwrap_or_default();
    let validated_availability = availability_response.as_ref().map(|value| {
        validated_booking_availability(
            value,
            &selected_slug.read(),
            *visible_month.read(),
            calendar_required_end,
        )
    });
    let contract_availability_error = validated_availability
        .as_ref()
        .and_then(|result| result.as_ref().err())
        .cloned();
    let availability_error = request_availability_error.or(contract_availability_error);
    let unavailable_ranges = validated_availability
        .and_then(Result::ok)
        .unwrap_or_default();
    let selected_calendar_active = !selected_slug.read().is_empty();
    let availability_is_current = if selected_calendar_active {
        availability_response.is_some() && availability_error.is_none()
    } else {
        fleet_availability_is_current
    };
    let availability_pending = if selected_calendar_active {
        availability_error.is_none() && !availability_is_current
    } else {
        fleet_availability_pending
    };
    let calendar_error = if selected_calendar_active {
        availability_error.clone()
    } else {
        fleet_availability_error.clone()
    };
    let calendar_blocked = calendar_error.is_some();
    let minimum_nights = if selected_calendar_active {
        availability_response
            .as_ref()
            .filter(|_| availability_is_current)
            .map(|value| value.minimum_nights)
            .unwrap_or(3)
    } else {
        fleet_response
            .filter(|_| availability_is_current)
            .map(|value| value.minimum_nights)
            .unwrap_or(3)
    };
    let mut calendar_day = *visible_month.read();
    let mut unavailable_dates = Vec::new();
    let mut availability_counts = Vec::new();
    if availability_is_current {
        while calendar_day < calendar_range_end {
            if selected_calendar_active {
                if !booking_date_is_selectable(
                    calendar_day,
                    *starts_on.read(),
                    *ends_on.read(),
                    minimum_nights,
                    &unavailable_ranges,
                ) {
                    unavailable_dates.push(calendar_day);
                }
            } else {
                let available_count = fleet_available_rental_count(
                    calendar_day,
                    *starts_on.read(),
                    *ends_on.read(),
                    minimum_nights,
                    &fleet_schedules,
                );
                availability_counts.push((calendar_day, available_count));
                if available_count == 0 {
                    unavailable_dates.push(calendar_day);
                }
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
    let delivery_distance = delivery_result
        .read()
        .as_ref()
        .map(delivery_distance_detail);

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
            .is_some_and(|(start, end)| (end - start).num_days() >= 1);
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
            let invalidated_request = address_request_version.peek().wrapping_add(1);
            address_request_version.set(invalidated_request);
            address_busy.set(false);
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
            .is_some_and(|(start, end)| (end - start).num_days() >= 1);
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
            let invalidated_request = address_request_version.peek().wrapping_add(1);
            address_request_version.set(invalidated_request);
            address_busy.set(false);
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
        if pending_payment.read().is_some() {
            let invalidated_version = quote_version.peek().wrapping_add(1);
            quote_version.set(invalidated_version);
            quote_busy.set(false);
            return;
        }
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
            let invalidated_version = quote_version.peek().wrapping_add(1);
            quote_version.set(invalidated_version);
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
        if booking_overlay_close_blocked(
            *closing.peek(),
            *booking_busy.peek(),
            *auth_busy.peek(),
            *all_in_busy.peek(),
            *edit_booking_busy.peek(),
        ) {
            return;
        }
        closing.set(true);
        let next_attempt = payment_attempt_nonce().wrapping_add(1);
        payment_attempt_nonce.set(next_attempt);
        document::eval(UNMOUNT_EMBEDDED_CHECKOUT);
        let _ = document::eval("await new Promise(resolve => setTimeout(resolve, 180));").await;
        on_close.call(());
    };
    let mut close_payment_overlay = move || {
        if payment_overlay_close_blocked(
            *all_in_busy.peek(),
            *edit_booking_busy.peek(),
            payment_phase.peek().as_str(),
        ) {
            return;
        }
        let next_attempt = payment_attempt_nonce().wrapping_add(1);
        payment_attempt_nonce.set(next_attempt);
        document::eval(UNMOUNT_EMBEDDED_CHECKOUT);
        payment_terms_open.set(false);
        payment_overlay_open.set(false);
        payment_phase.set("idle".into());
    };
    let mut finish_deposit_overlay = move || {
        let return_slug = deposit_overlay_booking
            .peek()
            .as_ref()
            .map(|created| created.booking.rental_slug.clone())
            .filter(|slug| !slug.is_empty())
            .unwrap_or_else(|| selected_slug.peek().clone());
        api::remove_sensitive_saved(SAVED_DEPOSIT_INSTRUCTIONS);
        deposit_overlay_booking.set(None);
        navigator.push(Route::RvDetail { slug: return_slug });
    };

    let selected_name_for_booking = selected_name.clone();
    let deposit_overlay = deposit_overlay_booking.read().clone();
    let current_delivery_address = delivery_address.read().trim().to_string();
    let payment_delivery_address = (!current_delivery_address.is_empty())
        .then_some(current_delivery_address)
        .or_else(|| {
            initial_payment_draft
                .as_ref()
                .and_then(|draft| draft.delivery_address.clone())
                .map(|address| address.trim().to_string())
                .filter(|address| !address.is_empty())
        });
    rsx! {
        div { class: if *closing.read() { "ub-backdrop is-closing" } else { "ub-backdrop" }, onclick: move |_| close_overlay(),
            div { class: "ub-shell", role: "dialog", aria_modal: "true", aria_label: "Complete your RV booking", tabindex: "-1", autofocus: true, onclick: move |event| event.stop_propagation(), onkeydown: move |event| { if event.key() == Key::Escape { event.stop_propagation(); spawn(async move { close_overlay().await; }); } },
                header { class: "ub-head",
                    div { div { class: "ub-kicker", "ONE-PAGE RV BOOKING" } h2 { "Build your Okanagan stay" } p { "Choose everything here. Completed sections fold into a clear summary." } }
                    button { class: "ub-close", r#type: "button", disabled: booking_overlay_close_blocked(*closing.read(), *booking_busy.read(), *auth_busy.read(), *all_in_busy.read(), *edit_booking_busy.read()), aria_label: "Close booking", onclick: move |_| close_overlay(), Icon { name: "x", size: 22, color: "var(--vl-ink)" } }
                }
                div { class: "ub-body",
                    main { class: "ub-steps",
                        BookingStep { number: 1, title: "Dates & guests", summary: if trip_ready { format!("{} → {} · {} · {} guests", date_text(*starts_on.read()), date_text(*ends_on.read()), pricing::night_count_label(nights), guests) } else { "Choose delivery and return dates".into() }, complete: trip_ready, open: *open_step.read() == 1, disabled: payment_locked, on_toggle: move |_| if !payment_locked { open_step.set(if *open_step.peek() == 1 { 0 } else { 1 }) },
                            div { class: "ub-step-content",
                                div { class: "ub-date-summary",
                                    span { "Delivery/setup · 2:00 PM" } strong { "{date_text(*starts_on.read())}" }
                                    Icon { name: "arrow-right", size: 17, color: "var(--vl-muted)" }
                                    span { "Return · 11:00 AM" } strong { "{date_text(*ends_on.read())}" }
                                }
                                if trip_ready && nights < 3 {
                                    div { class: "ub-short-stay-note", role: "status",
                                        Icon { name: "badge-check", size: 17, color: "var(--vl-forest)" }
                                        span {
                                            strong { "Your {nights}-night stay is available." }
                                            small { "The rental and per-night items are priced at the 3-night minimum. Your return date stays unchanged." }
                                        }
                                    }
                                }
                                div { class: if calendar_error.is_some() { "ub-calendar-context is-warning" } else { "ub-calendar-context" }, role: if calendar_error.is_some() { "alert" } else { "status" }, aria_live: "polite",
                                    Icon { name: "calendar", size: 16, color: "var(--vl-forest)" }
                                    span {
                                        if selected_calendar_active {
                                            if availability_pending { "Loading live dates for {selected_name}…" }
                                            else if calendar_error.is_some() { "Live dates could not be verified. Retry before choosing dates for this RV." }
                                            else { "Showing live available dates for {selected_name}." }
                                        } else if availability_pending {
                                            "Checking every RV for these dates…"
                                        } else if calendar_error.is_some() {
                                            "Fleet availability could not be verified. Retry before choosing dates."
                                        } else if fleet_response.is_some_and(|value| value.total_rentals == 0) {
                                            "No RV fits this guest count. Reduce guests to unlock bookable dates."
                                        } else if starts_on.read().is_some() && ends_on.read().is_none() {
                                            "Choose a return date. Each open date keeps at least one same RV available for your whole stay."
                                        } else {
                                            "Closed dates mean every RV that fits your guests is booked for a 3-night stay. Gold dates have only 1–2 RVs left."
                                        }
                                    }
                                    if calendar_error.is_some() {
                                        if selected_calendar_active {
                                            button { r#type: "button", onclick: move |_| selected_availability_retry.set(selected_availability_retry().wrapping_add(1)), "Retry" }
                                        } else {
                                            button { r#type: "button", onclick: move |_| fleet_availability_retry.set(fleet_availability_retry().wrapping_add(1)), "Retry" }
                                        }
                                    }
                                }
                                div { class: "cat-month-nav",
                                    span { "Choose delivery and return" }
                                    div { class: "cat-month-nav-actions",
                                        button { r#type: "button", aria_label: "Previous month", disabled: *visible_month.read() <= initial_month, onclick: move |_| { let current = *visible_month.read(); if let Some(previous) = current.checked_sub_months(Months::new(1)) { visible_month.set(previous.max(initial_month)); } }, Icon { name: "chevron-left", size: 18, color: "var(--vl-ink)" } }
                                        button { r#type: "button", aria_label: "Next month", onclick: move |_| { let current = *visible_month.read(); visible_month.set(add_months(current, 1)); }, Icon { name: "chevron-right", size: 18, color: "var(--vl-ink)" } }
                                    }
                                }
                                div {
                                    class: "cat-calendar-months ub-calendar",
                                    ontouchstart: move |event| {
                                        if !mobile_calendar_swipe_enabled() {
                                            calendar_swipe_start.set(None);
                                            return;
                                        }
                                        let touches = event.touches();
                                        if touches.len() != 1 {
                                            calendar_swipe_start.set(None);
                                            return;
                                        }
                                        let point = touches[0].client_coordinates();
                                        calendar_swipe_start.set(Some((point.x, point.y)));
                                    },
                                    ontouchend: move |event| {
                                        let start = *calendar_swipe_start.read();
                                        calendar_swipe_start.set(None);
                                        if !mobile_calendar_swipe_enabled() {
                                            return;
                                        }
                                        let Some(start) = start else { return; };
                                        let changed = event.touches_changed();
                                        let Some(touch) = changed.first() else { return; };
                                        let point = touch.client_coordinates();
                                        match calendar_swipe_month_delta(start, (point.x, point.y)) {
                                            1 => {
                                                event.prevent_default();
                                                let current = *visible_month.read();
                                                visible_month.set(add_months(current, 1));
                                            }
                                            -1 => {
                                                event.prevent_default();
                                                let current = *visible_month.read();
                                                if current > initial_month {
                                                    if let Some(previous) = current.checked_sub_months(Months::new(1)) {
                                                        visible_month.set(previous.max(initial_month));
                                                    }
                                                }
                                            }
                                            _ => {}
                                        }
                                    },
                                    ontouchcancel: move |_| calendar_swipe_start.set(None),
                                    for offset in 0..3_u32 { CatalogSearchMonth { month: add_months(*visible_month.read(), offset), today, starts_on, ends_on, unavailable_dates: unavailable_dates.clone(), availability_counts: availability_counts.clone(), show_availability_counts: !selected_calendar_active && availability_is_current, availability_pending, availability_blocked: calendar_blocked } }
                                }
                                div { class: "ub-guests", role: "group", aria_label: "Number of guests", span { "Guests" } button { r#type: "button", aria_label: "Remove one guest", disabled: *guests.read() <= 1, onclick: move |_| { let current = *guests.read(); guests.set((current - 1).max(1)); }, "−" } strong { aria_live: "polite", "{guests}" } button { r#type: "button", aria_label: "Add one guest", disabled: *guests.read() >= 10, onclick: move |_| { let current = *guests.read(); guests.set((current + 1).min(10)); }, "+" } }
                            }
                        }
                        BookingStep { number: 2, title: "Choose your RV", summary: selected_name.clone(), complete: !selected_slug.read().is_empty(), open: *open_step.read() == 2, disabled: payment_locked, on_toggle: move |_| if !payment_locked { open_step.set(if *open_step.peek() == 2 { 0 } else { 2 }) },
                            div { class: "ub-step-content",
                                div { class: "ub-rv-toolbar",
                                    p { class: "ub-choice-guidance",
                                        if trip_ready { "Only RVs available for your selected dates are shown." }
                                        else { "Choose any RV first to see its live calendar, or choose dates first." }
                                    }
                                    div { class: "ub-rv-guests", role: "group", aria_label: "Number of guests",
                                        span { "Guests" }
                                        button { r#type: "button", aria_label: "Remove one guest", disabled: *guests.read() <= 1, onclick: move |_| { let current = *guests.read(); guests.set((current - 1).max(1)); }, "−" }
                                        strong { aria_live: "polite", "{guests}" }
                                        button { r#type: "button", aria_label: "Add one guest", disabled: *guests.read() >= 10, onclick: move |_| { let current = *guests.read(); guests.set((current + 1).min(10)); }, "+" }
                                    }
                                }
                                if rental_choices_error.is_some() {
                                    div { class: "ub-error", role: "alert",
                                        p {
                                            if trip_ready { "Live availability could not be checked. Retry or choose different dates." }
                                            else { "RV models could not be loaded. Check your connection and retry." }
                                        }
                                        button { r#type: "button", onclick: move |_| rental_choices_retry.set(rental_choices_retry().wrapping_add(1)), "Retry" }
                                    }
                                }
                                else if rental_choices_loading { p { class: "ub-muted", if trip_ready { "Checking live availability…" } else { "Loading RV models…" } } }
                                else if rental_choice_values.is_empty() && trip_ready {
                                    div { class: "ub-rv-empty",
                                        strong { "No RV is available for these dates" }
                                        p { "Choose another date range to see bookable models." }
                                        button { class: "ub-primary", r#type: "button", onclick: move |_| { open_step.set(1); spawn(async move { scroll_to_booking_step(1).await; }); }, "Choose new dates" }
                                    }
                                }
                                else if rental_choice_values.is_empty() { p { class: "ub-muted", "No RV fits this group size. Reduce the guest count to see available models." } }
                                else {
                                    div { class: "ub-rv-grid",
                                        for rental in rental_choice_values.clone() {
                                            {
                                                let available_for_dates = available_rental_values.as_ref().map(|values| values.iter().any(|value| value.slug == rental.slug));
                                                rsx! { RentalChoice { key: "rv-{rental.slug}", rental: rental.clone(), selected: *selected_slug.read() == rental.slug, available_for_dates, on_select: move |slug| {
                                                    let keep_dates = rental_selection_keeps_dates(trip_ready, available_for_dates);
                                                    if !keep_dates {
                                                    starts_on.set(None);
                                                    ends_on.set(None);
                                                    trip_was_ready.set(false);
                                                    }
                                                    address_request_version.set(address_request_version().wrapping_add(1));
                                                    address_busy.set(false);
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
                                        input { value: "{delivery_address}", placeholder: "Start typing a Canadian address", autocomplete: "off", role: "combobox", aria_label: "Delivery address", aria_controls: "ub-address-suggestions", aria_expanded: *suggestions_open.read() && address_query_ready, onfocus: move |_| if address_query_ready { suggestions_open.set(true); }, oninput: move |event| { let value = event.value(); address_request_version.set(address_request_version().wrapping_add(1)); address_busy.set(false); delivery_address.set(value.clone()); delivery_km.set(None); delivery_result.set(None); quote.set(None); quote_error.set(String::new()); address_error.set(String::new()); suggestions_open.set(value.trim().chars().count() >= 3); } }
                                        if suggestions_busy { span { class: "rvd-address-spinner", aria_label: "Searching addresses" } }
                                        if *suggestions_open.read() && address_query_ready { div { id: "ub-address-suggestions", class: "ub-suggestions", role: "listbox",
                                            if suggestions_busy { div { class: "ub-suggestion-status", "Searching nearby Canadian addresses…" } }
                                            else if !suggestion_items.is_empty() { for suggestion in suggestion_items { button { r#type: "button", role: "option", onclick: move |_| { address_request_version.set(address_request_version().wrapping_add(1)); address_busy.set(false); delivery_address.set(suggestion.display_name.clone()); delivery_km.set(None); delivery_result.set(None); quote.set(None); quote_error.set(String::new()); address_error.set(String::new()); suggestions_open.set(false); }, strong { "{suggestion.primary_text}" } small { "{suggestion.secondary_text}" } } } }
                                            else if let Some(message) = suggestion_error.as_ref() { div { class: "ub-suggestion-status is-error", "{message}" } }
                                            else { div { class: "ub-suggestion-status", "Keep typing the street name, city, or campground." } }
                                            div { class: "ub-suggestions-foot", "Canadian addresses · prioritized near Kelowna" }
                                        } }
                                    }
                                    button { class: "ub-primary", r#type: "button", disabled: *address_busy.read() || !address_query_ready, onclick: move |_| { let slug = selected_slug.read().clone(); let address = delivery_address.read().clone(); let request_version = address_request_version().wrapping_add(1); address_request_version.set(request_version); async move { address_busy.set(true); address_error.set(String::new()); let result = api::delivery_estimate(&slug, &address).await; if *address_request_version.peek() != request_version || selected_slug.peek().as_str() != slug.as_str() || delivery_address.peek().as_str() != address.as_str() { return; } match result { Ok(result) if result.within_range => { let next = remember_delivery_address(&saved_addresses.read(), &result.resolved_address); let _ = api::save_json(SAVED_DELIVERY_ADDRESSES, &next); saved_addresses.set(next); delivery_address.set(result.resolved_address.clone()); delivery_km.set(Some(result.one_way_km.clone())); delivery_result.set(Some(result)); suggestions_open.set(false); open_step.set(4); }, Ok(result) => { delivery_result.set(Some(result.clone())); address_error.set(format!("This address is beyond the {} km delivery limit.", result.maximum_km)); }, Err(message) => address_error.set(message) } address_busy.set(false); } }, if *address_busy.read() { "Calculating…" } else { "Calculate delivery" } }
                                }
                                button {
                                    class: "ub-destination-trigger",
                                    r#type: "button",
                                    disabled: *address_busy.read(),
                                    aria_label: "Choose a popular provincial park delivery address",
                                    onclick: move |_| {
                                        suggestions_open.set(false);
                                        destination_picker_open.set(true);
                                    },
                                    span { class: "ub-destination-trigger-icon",
                                        Icon { name: "map-pinned", size: 18, color: "var(--vl-forest)" }
                                    }
                                    span { class: "ub-destination-trigger-copy",
                                        strong { "Choose a popular campground" }
                                        small { "Fintry, Bear Creek, Shuswap Lake or Kekuli Bay" }
                                    }
                                    Icon { name: "arrow-right", size: 17, color: "var(--vl-forest)" }
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
                                                    button { class: "ub-address-history-select", r#type: "button", disabled: *address_busy.read(), onclick: { let address = saved_address.clone(); move |_| { let slug = selected_slug.read().clone(); let address = address.clone(); let request_version = address_request_version().wrapping_add(1); address_request_version.set(request_version); async move { delivery_address.set(address.clone()); delivery_km.set(None); delivery_result.set(None); quote.set(None); quote_error.set(String::new()); address_error.set(String::new()); suggestions_open.set(false); address_busy.set(true); let result = api::delivery_estimate(&slug, &address).await; if *address_request_version.peek() != request_version || selected_slug.peek().as_str() != slug.as_str() || delivery_address.peek().as_str() != address.as_str() { return; } match result { Ok(result) if result.within_range => { let next = remember_delivery_address(&saved_addresses.read(), &result.resolved_address); let _ = api::save_json(SAVED_DELIVERY_ADDRESSES, &next); saved_addresses.set(next); delivery_address.set(result.resolved_address.clone()); delivery_km.set(Some(result.one_way_km.clone())); delivery_result.set(Some(result)); open_step.set(4); }, Ok(result) => { delivery_result.set(Some(result.clone())); address_error.set(format!("This address is beyond the {} km delivery limit.", result.maximum_km)); }, Err(message) => address_error.set(message) } address_busy.set(false); } } },
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
                        BookingStep { number: 4, title: "Extras & trip details", summary: if addon_keys.read().is_empty() { "No extras selected".into() } else { format!("{} extras selected", selected_addon_count(&addon_keys.read())) }, complete: address_ready, open: *open_step.read() == 4, disabled: !address_ready || payment_locked, on_toggle: move |_| if address_ready && !payment_locked { open_step.set(if *open_step.peek() == 4 { 0 } else { 4 }) },
                            div { class: "ub-step-content",
                                if let Some(details) = details.as_ref() {
                                    div { class: "ub-addon-grid",
                                        for addon in details.addons.iter() {
                                            {
                                                let key = addon.addon_key.clone();
                                                let quantity = addon_quantity(&addon_keys.read(), &key);
                                                let selected = quantity > 0;
                                                let is_bedding = is_bedding_addon(&key);
                                                let displayed_price = if is_bedding && selected {
                                                    pricing::money(price_number(&addon.price) * quantity as f64)
                                                } else {
                                                    format!("CA${}", addon.price)
                                                };
                                                if is_bedding {
                                                    rsx! {
                                                    div { key: "addon-{key}", class: if selected { "ub-addon ub-addon-bedding active" } else { "ub-addon ub-addon-bedding" },
                                                        span { class: "ub-addon-icon", AddonApiIcon { name: addon.icon_name.clone() } }
                                                        span { class: "ub-addon-copy", strong { "{addon.label}" } small { if addon.description.trim().is_empty() { "Fresh linens prepared for each selected bed before delivery." } else { "{addon.description}" } } if addon.is_recommended { em { "Recommended" } } }
                                                        span { class: "ub-addon-price", b { "{displayed_price}" } small { if quantity > 1 { "{quantity} × CA${addon.price}" } else { "per bed" } } }
                                                        span { class: "ub-addon-quantity", role: "group", aria_label: "Number of beds for bedding and linens",
                                                            button { r#type: "button", disabled: quantity == 0, aria_label: "Remove one bedding set", onclick: { let key = key.clone(); move |_| { let mut next = addon_keys.read().clone(); remove_one_addon(&mut next, &key); quote_busy.set(true); quote_error.set(String::new()); addon_notice.set(String::new()); addon_keys.set(next); } }, "−" }
                                                            output { aria_live: "polite", aria_label: "{quantity} bedding sets selected", "{quantity}" }
                                                            button { r#type: "button", disabled: quantity >= MAX_BEDDING_QUANTITY, aria_label: "Add one bedding set", onclick: { let key = key.clone(); move |_| { let mut next = addon_keys.read().clone(); if addon_quantity(&next, &key) < MAX_BEDDING_QUANTITY { next.push(key.clone()); quote_busy.set(true); quote_error.set(String::new()); addon_notice.set(String::new()); addon_keys.set(next); } } }, "+" }
                                                        }
                                                    }
                                                    }
                                                } else {
                                                    rsx! {
                                                    button { key: "addon-{key}", class: if selected { "ub-addon active" } else { "ub-addon" }, r#type: "button", onclick: move |_| {
                                                        let mut next = addon_keys.read().clone();
                                                        if next.contains(&key) { next.retain(|value| value != &key); } else { next.push(key.clone()); }
                                                        quote_busy.set(true);
                                                        quote_error.set(String::new());
                                                        addon_notice.set(String::new());
                                                        addon_keys.set(next);
                                                    },
                                                        span { class: "ub-addon-icon", AddonApiIcon { name: addon.icon_name.clone() } }
                                                        span { class: "ub-addon-copy", strong { "{addon.label}" } small { if addon.description.trim().is_empty() { "{addon_description(&key)}" } else { "{addon.description}" } } if addon.is_recommended { em { "Recommended" } } }
                                                        span { class: "ub-addon-price", b { "{displayed_price}" } small { if addon.charge_type == "per_unit" { "per night" } else { "one-time" } } }
                                                        span { class: "ub-addon-toggle", if selected { "✓" } else { "+" } }
                                                    }
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
                                    a { class: "ub-google", href: google_href, aria_disabled: *auth_busy.read(), onclick: move |event| {
                                        if *auth_busy.peek() {
                                            event.prevent_default();
                                            return;
                                        }
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
                                    if let Some(facebook_href) = facebook_href.clone() {
                                        a { class: "ub-facebook", href: facebook_href, aria_disabled: *auth_busy.read(), onclick: move |event| {
                                            if *auth_busy.peek() {
                                                event.prevent_default();
                                                return;
                                            }
                                            let draft = make_draft(&selected_slug.read(), *starts_on.read(), *ends_on.read(), *guests.read(), &delivery_address.read(), delivery_km.read().clone(), addon_keys.read().clone(), false, false);
                                            let continuation = api::BookingAuthContinuation { draft: draft.clone(), location: location.read().clone(), radius_km: *radius.read(), delivery_estimate: delivery_result.read().clone(), first_name: first_name.read().clone(), last_name: last_name.read().clone(), booking_email: booking_email.read().clone(), phone: phone.read().clone(), notes: notes.read().clone(), accepted_terms: *accepted.read() };
                                            match api::save_booking_auth_continuation(&continuation) {
                                                Ok(()) => { let search = api::CatalogSearchDraft { location: location.read().clone(), radius_km: *radius.read(), starts_on: Some(draft.starts_on), ends_on: Some(draft.ends_on), guests: draft.guests }; let _ = api::save_json("vl_catalog_search", &search); on_search_change.call(()); api::remember_auth_return(&facebook_return); }
                                                Err(message) => { event.prevent_default(); auth_error.set(message); }
                                            }
                                        },
                                            span { class: "auth-facebook-mark", "f" }
                                            "Continue with Facebook"
                                        }
                                    }
                                    div { class: "ub-auth-divider", span { "or use email" } }
                                    div { class: "ub-field-grid", input { r#type: "email", autocomplete: "email", value: "{auth_email}", placeholder: "Email", disabled: *auth_busy.read(), oninput: move |event| auth_email.set(event.value()) } input { r#type: "password", autocomplete: if *auth_register.read() { "new-password" } else { "current-password" }, value: "{auth_password}", placeholder: "Password", disabled: *auth_busy.read(), oninput: move |event| auth_password.set(event.value()) } }
                                    if !auth_error.read().is_empty() { p { class: "ub-error", "{auth_error}" } }
                                    div { class: "ub-auth-actions", button { class: "ub-primary", r#type: "button", disabled: *auth_busy.read(), onclick: move |_| { let email = auth_email.read().clone(); let password = auth_password.read().clone(); let register = *auth_register.read(); async move { auth_busy.set(true); auth_error.set(String::new()); match api::login(&email, &password, register).await { Ok(tokens) => match api::save_session(&tokens) { Ok(()) => { booking_email.set(tokens.user.email.clone()); user.set(Some(tokens.user)); }, Err(message) => auth_error.set(message) }, Err(_) => auth_error.set("Check your email and password, then try again.".into()) } auth_busy.set(false); } }, if *auth_busy.read() { "Please wait…" } else if *auth_register.read() { "Create account" } else { "Sign in" } } button { r#type: "button", disabled: *auth_busy.read(), onclick: move |_| { let current = *auth_register.read(); auth_register.set(!current); }, if *auth_register.read() { "I already have an account" } else { "Create an account" } } }
                                } } else if let Some(created) = pending_payment.read().as_ref() { div { class: "ub-payment-reserved",
                                    div { class: "ub-stripe-payment-head",
                                        div { Icon { name: "shield-check", size: 20, color: "var(--vl-forest)" } div { h3 { "Reservation ready for payment" } p { "Booking {created.booking.booking_number} is held. Secure Checkout opens above this booking." } } }
                                        span { "TEST MODE" }
                                    }
                                    p { "Your dates remain reserved while this Checkout session is active. To change any trip detail, release this unpaid reservation first." }
                                    div { class: "ub-payment-reserved-actions",
                                        button { class: "ub-primary", r#type: "button", onclick: move |_| { booking_error.set(String::new()); payment_overlay_open.set(true); payment_phase.set("idle".into()); let next = payment_attempt_nonce().wrapping_add(1); payment_attempt_nonce.set(next); }, "Open secure payment" }
                                        button { class: "ub-change-booking", r#type: "button", onclick: move |_| { edit_booking_error.set(String::new()); edit_booking_confirm_open.set(true); }, Icon { name: "pencil", size: 15, color: "var(--vl-forest)" } span { "Change booking details" } }
                                    }
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
                                    label { class: "ub-terms",
                                        input { r#type: "checkbox", checked: *accepted.read(), disabled: !booking_can_submit, onchange: move |event| accepted.set(event.checked()) }
                                        span {
                                            "I accept the "
                                            button { class: "ub-inline-terms-link", r#type: "button", onclick: move |event| { event.stop_propagation(); payment_terms_open.set(true); }, "RV Rental Terms & Conditions" }
                                            if payment_availability == api::PaymentAvailability::Ready { " and authorize this Stripe payment." } else { " and understand this is a booking with no card charge." }
                                        }
                                    }
                                    if payment_availability == api::PaymentAvailability::Loading { p { class: "ub-muted", role: "status", "Checking the Stripe payment configuration…" } }
                                    if payment_availability == api::PaymentAvailability::Blocked { p { class: "ub-error", role: "alert", if payment_config_error.read().is_empty() { "Checkout is blocked because the returned Stripe mode, key, or account could not be verified." } else { "Payment configuration could not be verified. Retry before creating a reservation." } } button { r#type: "button", onclick: move |_| { let next = payment_config_retry().wrapping_add(1); payment_config_retry.set(next); }, "Retry payment configuration" } }
                                    if !booking_error.read().is_empty() { p { class: "ub-error", role: "alert", "{booking_error}" } }
                                    button { class: "ub-primary ub-confirm", r#type: "button", disabled: *booking_busy.read() || *quote_busy.read() || quote.read().is_none() || !booking_can_submit, onclick: move |_| { let active_quote = quote.read().clone(); let values = (first_name.read().clone(), last_name.read().clone(), booking_email.read().clone(), phone.read().clone(), notes.read().clone(), *accepted.read()); let draft = make_draft(&selected_slug.read(), *starts_on.read(), *ends_on.read(), *guests.read(), &delivery_address.read(), delivery_km.read().clone(), addon_keys.read().clone(), false, false); let rental_slug = selected_slug.read().clone(); let rental_name = selected_name_for_booking.clone(); async move { if !booking_can_submit { booking_error.set("Payment configuration must be verified before a booking can be created.".into()); return; } else if !values.5 { booking_error.set("Please accept the rental terms.".into()); return; } else if values.0.trim().len() < 2 || values.1.trim().len() < 2 || !values.2.contains('@') || values.3.trim().len() < 7 { booking_error.set("Enter your full name, email, and phone number.".into()); return; } let Some(active_quote) = active_quote else { booking_error.set("Wait for the price calculation to finish.".into()); return; }; booking_busy.set(true); booking_error.set(String::new()); let booking_notes = format!("{}\nFestival/event: no\nTowing after delivery: no\nDelivery only: yes", values.4.trim()); match api::create_booking(&active_quote.quote.quote_id, &values.0, &values.1, &values.2, &values.3, &booking_notes).await { Ok(mut created) => { fill_booking_rental(&mut created.booking, rental_slug, rental_name); let _ = api::save_json("vl_trip_draft", &draft); let _ = api::save_json("vl_active_quote", &active_quote); if created.client_secret.is_some() && !created.access_token.is_empty() { let _ = api::save_sensitive_json(SAVED_PENDING_PAYMENT, &created); payment_terms_accepted.set(false); payment_terms_open.set(false); payment_attempt_nonce.set(payment_attempt_nonce().wrapping_add(1)); payment_overlay_open.set(true); payment_phase.set("idle".into()); pending_payment.set(Some(created)); } else if created.booking.status == "confirmed" || created.booking.payment_status == "test_paid" { let _ = api::save_sensitive_json("vl_last_booking", &created); let _ = api::save_sensitive_json(SAVED_POST_PAYMENT_BOOKING, &created); let _ = api::save_sensitive_json(SAVED_DEPOSIT_INSTRUCTIONS, &created); deposit_overlay_booking.set(Some(created)); } else { booking_error.set("The booking was reserved, but Stripe Checkout was not returned. Please contact support before trying again.".into()); } }, Err(error) if booking_creation_recovery_message(&error).is_some() => { let message = booking_creation_recovery_message(&error).unwrap_or("The saved booking price changed. Review the refreshed total and try again."); api::remove_saved("vl_active_quote"); quote.set(None); let next_quote = quote_refresh_nonce().wrapping_add(1); quote_refresh_nonce.set(next_quote); booking_error.set(message.into()); }, Err(error) => booking_error.set(error.message) } booking_busy.set(false); } }, if *booking_busy.read() { "Creating reservation…" } else if payment_availability == api::PaymentAvailability::Ready { "Continue to secure payment" } else { "Confirm booking" } }
                                } }
                            }
                        }
                    }
                    aside { class: "ub-summary",
                        h3 { aria_live: "polite", if let Some(created) = pending_payment.read().as_ref() { "{created.booking.currency} ${created.booking.total}" } else if *quote_busy.read() { if let Some(value) = preview_total { AnimatedMoney { id: "ub-trip-price", amount: value } } else { "Updating…" } } else if let Some(value) = quote.read().as_ref() { AnimatedMoney { id: "ub-trip-price", amount: pricing::quote_trip_price(value) } } else if let Some(value) = preview_total { AnimatedMoney { id: "ub-trip-price", amount: value } } else { "Complete delivery" } }
                        p { if pending_payment.read().is_some() { "This immutable trip price is locked to the active Stripe reservation. The refundable damage deposit is separate." } else if *quote_busy.read() { "Updating the exact trip price…" } else if quote.read().is_some() { "Trip price with preparation, protection, delivery, selected extras and taxes. The refundable damage deposit is separate." } else if preview_total.is_some() { "Known trip costs are shown. Exact taxes are updating; the refundable damage deposit is separate." } else { "Your trip price appears after the delivery address is calculated." } }
                        button { class: "ub-summary-dates", r#type: "button", disabled: !trip_ready || payment_locked, onclick: move |_| if !payment_locked { open_step.set(1); spawn(async move { scroll_to_booking_step(1).await; }); },
                            span { "Dates" }
                            b { if let Some(created) = pending_payment.read().as_ref() { "{display_booking_date(&created.booking.starts_at)} → {display_booking_date(&created.booking.ends_at)}" } else if trip_ready { "{date_text(*starts_on.read())} → {date_text(*ends_on.read())} · {pricing::night_count_label(nights)}" } else { "Choose dates" } }
                            small { if payment_locked { "Locked" } else { "Edit dates" } }
                        }
                        if let Some(created) = pending_payment.read().as_ref() { div { class: "ub-price-lines ub-locked-price",
                            div { span { "Booked RV" } b { if created.booking.rental_name.is_empty() { "{selected_name}" } else { "{created.booking.rental_name}" } } }
                            if let Some(value) = quote.read().as_ref().filter(|value| quote_matches_booking(value, &created.booking)) {
                                for item in value.items.iter().filter(|item| item.item_type != "deposit") {
                                    if item.item_key == "stationary_plus" {
                                        StationaryPlusPriceLine {
                                            key: "locked-{item.item_key}-{item.amount}",
                                            label: item.label.clone(),
                                            detail: pricing::stationary_plus_detail(i64::from(value.quote.units)),
                                            amount: format!("CA${}", item.amount),
                                            expanded: *stationary_plus_details_open.read(),
                                            on_toggle: move |_| {
                                                let next = !*stationary_plus_details_open.read();
                                                stationary_plus_details_open.set(next);
                                            },
                                        }
                                    } else {
                                        div { key: "locked-{item.item_key}-{item.amount}", span { "{item.label}" if item.item_type == "delivery" { if let Some(detail) = delivery_distance.as_ref() { small { "{detail}" } } } else if item.item_key == BEDDING_ADDON_KEY { small { "{item.quantity} beds × CA${item.unit_price}" } } } b { class: "ub-line-price", "CA${item.amount}" } }
                                    }
                                }
                            }
                            div { class: "total", span { "Full trip price" } b { "{created.booking.currency} ${created.booking.total}" } }
                            div { class: "due-now", span { "Due now · {booking_payment_percent(&created.booking).unwrap_or(0)}%" } b { "{created.booking.currency} ${created.booking.amount_due_now}" } small { "{booking_payment_percent(&created.booking).unwrap_or(0)}% × {created.booking.currency} ${created.booking.total}" } }
                            if let Some(remaining) = booking_remaining_balance(&created.booking) { div { class: "remaining-balance", span { "Remaining balance · 70%" if let Some(due_at) = created.booking.balance_due_at.as_deref() { small { "Due {display_booking_date(due_at)} — 30 days before delivery" } } } b { "{created.booking.currency} ${remaining}" } } }
                            small { "The Stripe Checkout amount is locked to this booking. Closing and reopening the window cannot create a second reservation." }
                        } } else if *quote_busy.read() {
                            if let Some(value) = optimistic_price.as_ref() { div { class: "ub-price-lines", for item in value.lines.iter() { if item.key.starts_with("rental-") { button { key: "optimistic-{item.key}-{item.amount}", class: "ub-price-line is-editable", r#type: "button", aria_label: "Edit dates for {item.label}", onclick: move |_| { open_step.set(1); spawn(async move { scroll_to_booking_step(1).await; }); }, span { "{item.label}" if let Some(detail) = item.detail.as_ref() { small { "{detail}" } } } b { class: "ub-line-price", "{pricing::money(item.amount)}" } } } else if item.key == "stationary-plus" { StationaryPlusPriceLine { key: "optimistic-{item.key}-{item.amount}", label: item.label.clone(), detail: item.detail.clone().unwrap_or_default(), amount: pricing::money(item.amount), expanded: *stationary_plus_details_open.read(), on_toggle: move |_| { let next = !*stationary_plus_details_open.read(); stationary_plus_details_open.set(next); } } } else { div { key: "optimistic-{item.key}-{item.amount}", class: "ub-price-line", span { "{item.label}" if let Some(detail) = item.detail.as_ref() { small { "{detail}" } } } b { class: "ub-line-price", "{pricing::money(item.amount)}" } } } } div { class: "total", span { "Trip price CAD" } b { AnimatedMoney { id: "ub-trip-price-total", amount: value.total } } } } }
                        } else if let Some(value) = quote.read().as_ref() { div { class: "ub-price-lines", for item in value.items.iter().filter(|item| item.item_type != "deposit") { if item.item_type == "rental" { button { key: "line-{item.item_key}-{item.amount}", class: "ub-price-line is-editable", r#type: "button", aria_label: "Edit dates for {item.label}", onclick: move |_| { open_step.set(1); spawn(async move { scroll_to_booking_step(1).await; }); }, span { "{item.label}" } b { class: "ub-line-price", "CA${item.amount}" } } } else if item.item_key == "stationary_plus" { StationaryPlusPriceLine { key: "line-{item.item_key}-{item.amount}", label: item.label.clone(), detail: pricing::stationary_plus_detail(i64::from(value.quote.units)), amount: format!("CA${}", item.amount), expanded: *stationary_plus_details_open.read(), on_toggle: move |_| { let next = !*stationary_plus_details_open.read(); stationary_plus_details_open.set(next); } } } else { div { key: "line-{item.item_key}-{item.amount}", class: "ub-price-line", span { "{item.label}" if item.item_type == "delivery" { if let Some(detail) = delivery_distance.as_ref() { small { "{detail}" } } } else if item.item_key == BEDDING_ADDON_KEY { small { "{item.quantity} beds × CA${item.unit_price}" } } } b { class: "ub-line-price", "CA${item.amount}" } } } } div { class: "total", span { "Trip price CAD" } b { AnimatedMoney { id: "ub-trip-price-total", amount: pricing::quote_trip_price(value) } } } } }
                        div { class: "ub-deposit-card",
                            div { span { "REFUNDABLE DAMAGE DEPOSIT" } b { "{pricing::money(pricing::DAMAGE_DEPOSIT)}" } }
                            p { "Send separately by Interac e-Transfer to {DAMAGE_DEPOSIT_ETRANSFER_EMAIL} no later than {pricing::DAMAGE_DEPOSIT_DUE_HOURS} hours before delivery. It is refundable after return and inspection, less documented damage." }
                        }
                        div { class: "ub-payment-note",
                            b { "Payment timing" }
                            span { "{pricing::BOOKING_DEPOSIT_PERCENT}% of the trip price to confirm when booked more than {pricing::BALANCE_DUE_DAYS} days ahead; the balance is due {pricing::BALANCE_DUE_DAYS} days before delivery. Trips within {pricing::BALANCE_DUE_DAYS} days are paid in full when booked." }
                        }
                        if !quote_error.read().is_empty() { p { class: "ub-error", "{quote_error}" } }
                        if !addon_notice.read().is_empty() { p { class: "ub-notice", "{addon_notice}" } }
                        div { class: "ub-summary-trip", span { "Dates" } b { if let Some(created) = pending_payment.read().as_ref() { "{display_booking_date(&created.booking.starts_at)} → {display_booking_date(&created.booking.ends_at)}" } else if trip_ready { "{date_text(*starts_on.read())} → {date_text(*ends_on.read())}" } else { "Not selected" } } span { "RV" } b { if let Some(created) = pending_payment.read().as_ref() { if created.booking.rental_name.is_empty() { "{selected_name}" } else { "{created.booking.rental_name}" } } else { "{selected_name}" } } span { "Price" } b { if payment_locked { "Locked to booking" } else if address_ready { "Calculated" } else { "Required" } } }
                        div { class: "ub-test-note", Icon { name: "shield-check", size: 17, color: "var(--vl-forest)" } span { match payment_availability { api::PaymentAvailability::Ready => "Secure Stripe card payment is enabled.", api::PaymentAvailability::Disabled => "Payments are disabled; no card is collected or charged.", api::PaymentAvailability::Loading => "Checking the Stripe payment configuration…", api::PaymentAvailability::Blocked => "Payment configuration is blocked until the approved Stripe account is verified.", } } }
                        div { class: if *accepted.read() { "ub-payment-terms-gate is-accepted" } else { "ub-payment-terms-gate" },
                            label {
                                input { r#type: "checkbox", checked: *accepted.read(), disabled: !booking_can_submit || payment_locked, onchange: move |event| { let is_accepted = event.checked(); accepted.set(is_accepted); payment_terms_accepted.set(is_accepted); } }
                                span { "I have read and agree to the" }
                            }
                            button { class: "ub-payment-terms-link", r#type: "button", onclick: move |_| payment_terms_open.set(true), "RV Rental Terms & Conditions" }
                            small { if payment_locked { "Terms were accepted before this reservation was created." } else if *accepted.read() { "Confirmed. You can continue to secure payment." } else { "Required before the reservation and payment can continue." } }
                        }
                    }
                }
            }
            if *destination_picker_open.read() {
                div {
                    class: "ub-destination-layer",
                    onclick: move |event| {
                        event.stop_propagation();
                        destination_picker_open.set(false);
                    },
                    section {
                        class: "ub-destination-dialog",
                        role: "dialog",
                        aria_modal: "true",
                        aria_labelledby: "ub-destination-title",
                        tabindex: "-1",
                        autofocus: true,
                        onclick: move |event| event.stop_propagation(),
                        onkeydown: move |event| {
                            if event.key() == Key::Escape {
                                event.stop_propagation();
                                destination_picker_open.set(false);
                            }
                        },
                        header { class: "ub-destination-head",
                            div {
                                span { class: "ub-destination-kicker", "POPULAR RV DESTINATIONS" }
                                h2 { id: "ub-destination-title", "Choose a delivery destination" }
                                p { "Pick a popular campground to fill the address instantly. Delivery eligibility and price are calculated next." }
                            }
                            button {
                                class: "ub-close",
                                r#type: "button",
                                aria_label: "Close destination recommendations",
                                onclick: move |_| destination_picker_open.set(false),
                                Icon { name: "x", size: 21, color: "var(--vl-ink)" }
                            }
                        }
                        div { class: "ub-destination-scroll",
                            div { class: "ub-destination-grid",
                                for recommendation in destination_recommendations() {
                                    DestinationRecommendationCard {
                                        key: "destination-{recommendation.id}",
                                        recommendation,
                                        on_select: move |address: String| {
                                            destination_picker_open.set(false);
                                            let slug = selected_slug.read().clone();
                                            let request_version = address_request_version().wrapping_add(1);
                                            address_request_version.set(request_version);
                                            async move {
                                                delivery_address.set(address.clone());
                                                delivery_km.set(None);
                                                delivery_result.set(None);
                                                quote.set(None);
                                                quote_error.set(String::new());
                                                address_error.set(String::new());
                                                suggestions_open.set(false);
                                                address_busy.set(true);
                                                let result = api::delivery_estimate(&slug, &address).await;
                                                if *address_request_version.peek() != request_version
                                                    || selected_slug.peek().as_str() != slug.as_str()
                                                    || delivery_address.peek().as_str() != address.as_str()
                                                {
                                                    return;
                                                }
                                                match result {
                                                    Ok(result) if result.within_range => {
                                                        let next = remember_delivery_address(
                                                            &saved_addresses.read(),
                                                            &result.resolved_address,
                                                        );
                                                        let _ = api::save_json(SAVED_DELIVERY_ADDRESSES, &next);
                                                        saved_addresses.set(next);
                                                        delivery_address.set(result.resolved_address.clone());
                                                        delivery_km.set(Some(result.one_way_km.clone()));
                                                        delivery_result.set(Some(result));
                                                        open_step.set(4);
                                                    }
                                                    Ok(result) => {
                                                        delivery_result.set(Some(result.clone()));
                                                        address_error.set(format!(
                                                            "This address is beyond the {} km delivery limit.",
                                                            result.maximum_km
                                                        ));
                                                        open_step.set(3);
                                                    }
                                                    Err(message) => {
                                                        address_error.set(message);
                                                        open_step.set(3);
                                                    }
                                                }
                                                address_busy.set(false);
                                            }
                                        },
                                    }
                                }
                            }
                            div { class: "ub-destination-note",
                                Icon { name: "info", size: 16, color: "var(--vl-forest)" }
                                span { "Selecting a park fills the delivery field and runs the existing distance, 150 km limit and fee calculation." }
                            }
                        }
                    }
                }
            }
            if let Some(created) = pending_payment.read().as_ref().filter(|_| *payment_overlay_open.read()) {
                div { class: "ub-payment-layer", onclick: move |event| { event.stop_propagation(); close_payment_overlay(); },
                    section { class: "ub-payment-dialog", role: "dialog", aria_modal: "true", aria_label: "Secure payment for your RV booking", tabindex: "-1", autofocus: true, onclick: move |event| event.stop_propagation(), onkeydown: move |event| if event.key() == Key::Escape { event.stop_propagation(); close_payment_overlay(); },
                        header { class: "ub-payment-dialog-head",
                            div {
                                span { class: "ub-kicker", "SECURE TEST PAYMENT" }
                                h2 { "Complete your reservation" }
                                p { "{created.booking.booking_number} · " if created.booking.rental_name.is_empty() { "{selected_name}" } else { "{created.booking.rental_name}" } }
                            }
                            div { class: "ub-payment-dialog-actions",
                                span { "TEST MODE" }
                                button { r#type: "button", disabled: payment_overlay_close_blocked(*all_in_busy.read(), *edit_booking_busy.read(), payment_phase.read().as_str()), aria_label: "Close secure payment", onclick: move |_| close_payment_overlay(), Icon { name: "x", size: 21, color: "var(--vl-ink)" } }
                            }
                        }
                        div { class: "ub-payment-dialog-content",
                            div { class: "ub-payment-dialog-body",
                                div { class: "ub-payment-dialog-summary",
                                    if created.payment_option == "all_in" {
                                        span { "DUE NOW · TRIP + REFUNDABLE DEPOSIT" }
                                        strong { "{created.all_in_offer.as_ref().map(|offer| offer.currency.as_str()).unwrap_or(created.booking.currency.as_str())} ${created.all_in_offer.as_ref().map(|offer| offer.total_due_today.as_str()).unwrap_or(created.booking.total.as_str())}" }
                                        small { "One transaction pays the full trip price and the refundable damage deposit today." }
                                    } else {
                                        span { "DUE NOW · {booking_payment_percent(&created.booking).unwrap_or(0)}% OF TRIP PRICE" }
                                        strong { "{created.booking.currency} ${created.booking.amount_due_now}" }
                                        small { if booking_payment_percent(&created.booking) == Some(100) { "The full trip price is charged today." } else { "The remaining 70% is not charged today." } }
                                    }
                                }
                                if payment_phase.read().as_str() == "switching" { p { class: "ub-stripe-state", "Replacing the unpaid Checkout securely…" } }
                                if payment_phase.read().as_str() == "checking" { p { class: "ub-stripe-state", "Checking webhook-backed booking status…" } }
                                if payment_phase.read().as_str() == "mounting" { p { class: "ub-stripe-state", "Loading secure Checkout…" } }
                                if !payment_terms_allow_checkout(*payment_terms_accepted.read()) {
                                    div { class: "ub-payment-terms-wait", role: "status", aria_live: "polite",
                                        Icon { name: "shield-check", size: 24, color: "var(--vl-forest)" }
                                        strong { "Accept the rental terms to continue" }
                                        p { "We moved the booking receipt to the required checkbox. Confirm the Terms & Conditions there, and Secure Checkout will load here automatically." }
                                        button { class: "ub-payment-terms-guide", r#type: "button", onclick: move |_| { spawn(guide_to_payment_terms()); },
                                            span { "Review and accept terms" }
                                            Icon { name: "arrow-right", size: 16, color: "currentColor" }
                                        }
                                    }
                                } else if payment_availability == api::PaymentAvailability::Ready {
                                    div { id: "vl-embedded-checkout", class: "ub-embedded-checkout", aria_label: "Stripe secure checkout" }
                                }
                                if matches!(payment_phase.read().as_str(), "confirming" | "delayed") { div { class: "ub-confirming-payment", Icon { name: "loader-circle", size: 17, color: "var(--vl-forest)" } span { strong { "Confirming payment…" } small { "We return to this RV only after the backend confirms the Stripe webhook. Do not submit a second payment." } } } }
                                if !booking_error.read().is_empty() { p { class: "ub-error", role: "alert", "{booking_error}" } }
                                if matches!(payment_phase.read().as_str(), "error" | "blocked" | "delayed") { button { class: "ub-primary", r#type: "button", onclick: move |_| { booking_error.set(String::new()); payment_phase.set("idle".into()); let next = payment_attempt_nonce().wrapping_add(1); payment_attempt_nonce.set(next); }, "Check status and reopen Checkout" } }
                                if payment_availability == api::PaymentAvailability::Blocked { button { class: "ub-payment-secondary", r#type: "button", onclick: move |_| { let next = payment_config_retry().wrapping_add(1); payment_config_retry.set(next); payment_phase.set("idle".into()); }, "Retry payment configuration" } }
                            }
                            aside { class: "ub-payment-price-summary", aria_label: "Payment schedule",
                                h3 { "Booking receipt" }
                                div { class: "ub-payment-booking-details",
                                    figure { class: "ub-payment-selected-rv",
                                        if let Some(rental) = selected_rental.as_ref() {
                                            img { src: "{rental_image(rental)}", alt: "Selected trailer for this booking", loading: "eager" }
                                        } else if !created.booking.rental_slug.is_empty() {
                                            img { src: "{rental_fallback_image(&created.booking.rental_slug)}", alt: "Selected trailer for this booking", loading: "eager" }
                                        }
                                        figcaption {
                                            span { class: "ub-payment-booking-kicker", "SELECTED TRAILER" }
                                            strong { if created.booking.rental_name.is_empty() { "{selected_name}" } else { "{created.booking.rental_name}" } }
                                        }
                                    }
                                    dl { class: "ub-payment-trip-list",
                                        div {
                                            dt { "Booking" }
                                            dd { "{created.booking.booking_number}" }
                                        }
                                        div {
                                            dt { "Delivery & setup" }
                                            dd { "{display_booking_moment(&created.booking.starts_at)}" }
                                        }
                                        div {
                                            dt { "Return" }
                                            dd { "{display_booking_moment(&created.booking.ends_at)}" }
                                        }
                                        if let Some(stay_nights) = booking_stay_nights(&created.booking) {
                                            div { dt { "Stay" } dd { "{pricing::night_count_label(stay_nights)}" } }
                                        }
                                        if let Some(value) = quote.read().as_ref().filter(|value| quote_matches_booking(value, &created.booking)) {
                                            div { dt { "Guests" } dd { "{value.quote.guests}" } }
                                        }
                                        div { dt { "Service" } dd { "Delivery, leveling & setup" } }
                                        if let Some(payment_delivery_address) = payment_delivery_address.as_ref() {
                                            div { dt { "Address" } dd { "{payment_delivery_address}" } }
                                        }
                                    }
                                }
                                if let Some(value) = quote.read().as_ref().filter(|value| quote_matches_booking(value, &created.booking)) {
                                    h4 { class: "ub-payment-breakdown-title", "What you are paying for" }
                                    div { class: "ub-payment-price-breakdown",
                                        for item in value.items.iter().filter(|item| item.item_type != "deposit") {
                                            div { key: "payment-{item.item_key}-{item.amount}", class: "ub-payment-price-row",
                                                span {
                                                    "{item.label}"
                                                    if item.item_type == "rental" {
                                                        small { "{item.quantity} billable nights × {created.booking.currency} ${item.unit_price}" }
                                                    } else if item.item_key == "stationary_plus" {
                                                        small { "{pricing::stationary_plus_detail(i64::from(value.quote.units))}" }
                                                    } else if item.item_type == "delivery" {
                                                        small { if let Some(detail) = delivery_distance.as_ref() { "{detail} · delivery and campsite setup" } else { "Delivery and campsite setup included" } }
                                                    } else if item.item_key == "rv_preparation" {
                                                        small { "One-time RV preparation service" }
                                                    } else if item.item_key == BEDDING_ADDON_KEY {
                                                        small { "{item.quantity} beds × {created.booking.currency} ${item.unit_price}" }
                                                    } else if item.item_type == "addon" {
                                                        small { "{item.quantity} × {created.booking.currency} ${item.unit_price}" }
                                                    }
                                                }
                                                strong { "{created.booking.currency} ${item.amount}" }
                                            }
                                        }
                                    }
                                }
                                div { class: "ub-payment-price-row is-total", span { "Full trip price" } strong { "{created.booking.currency} ${created.booking.total}" } }
                                if created.payment_option == "all_in" {
                                    if let Some(offer) = created.all_in_offer.as_ref() {
                                        div { class: "ub-all-in is-selected",
                                            div { class: "ub-all-in-head", span { "PAY EVERYTHING NOW" } b { "SELECTED" } }
                                            p { "One Stripe transaction today. No balance or deposit payment later." }
                                            div { class: "ub-all-in-row", span { "Full trip price" } strong { "{offer.currency} ${offer.trip_price}" } }
                                            div { class: "ub-all-in-row", span { "Refundable damage deposit" } strong { "{offer.currency} ${offer.refundable_deposit}" } }
                                            div { class: "ub-all-in-total", span { "Total paid today" } strong { "{offer.currency} ${offer.total_due_today}" } }
                                            button { r#type: "button", disabled: *all_in_busy.read(), onclick: move |_| { let created = pending_payment.read().clone(); async move { let Some(mut created) = created else { return; }; all_in_busy.set(true); all_in_error.set(String::new()); booking_error.set(String::new()); payment_phase.set("switching".into()); payment_attempt_nonce.set(payment_attempt_nonce().wrapping_add(1)); document::eval(UNMOUNT_EMBEDDED_CHECKOUT); match api::switch_booking_to_scheduled(&created.booking.booking_id, &created.access_token).await { Ok(response) => { created.client_secret = Some(response.checkout_client_secret); created.checkout_session_id = Some(response.checkout_session_id); created.payment_option = response.payment_option; let _ = api::save_sensitive_json(SAVED_PENDING_PAYMENT, &created); pending_payment.set(Some(created)); }, Err(error) => all_in_error.set(error.message) } all_in_busy.set(false); payment_phase.set("idle".into()); payment_attempt_nonce.set(payment_attempt_nonce().wrapping_add(1)); } }, if *all_in_busy.read() { "Restoring 30% payment…" } else { "Return to 30% payment" } }
                                            small { "Return to 30% today, with the remaining 70% due 30 days before delivery and the refundable deposit charged separately later." }
                                        }
                                    }
                                } else {
                                    div { class: "ub-payment-price-now", div { span { "Due now · {booking_payment_percent(&created.booking).unwrap_or(0)}%" } strong { "{created.booking.currency} ${created.booking.amount_due_now}" } } small { "{booking_payment_percent(&created.booking).unwrap_or(0)}% × {created.booking.currency} ${created.booking.total} — charged today" } }
                                    if let Some(remaining) = booking_remaining_balance(&created.booking) { div { class: "ub-payment-price-row", span { "Remaining balance · 70%" if let Some(due_at) = created.booking.balance_due_at.as_deref() { small { "Due {display_booking_date(due_at)} — 30 days before delivery" } } } strong { "{created.booking.currency} ${remaining}" } } }
                                    div { class: "ub-payment-price-row is-deposit", span { "Refundable damage deposit" small { "Interac e-Transfer to {DAMAGE_DEPOSIT_ETRANSFER_EMAIL}, due {pricing::DAMAGE_DEPOSIT_DUE_HOURS} hours before delivery. Not charged through Stripe." } } strong { "{pricing::money(pricing::DAMAGE_DEPOSIT)}" } }
                                    if let Some(offer) = created.all_in_offer.as_ref() {
                                        div { class: "ub-all-in",
                                            div { class: "ub-all-in-head", span { "PAY EVERYTHING NOW" } b { "ONE PAYMENT" } }
                                            p { "Pay the full trip price and refundable deposit together today." }
                                            div { class: "ub-all-in-row", span { "Full trip price" } strong { "{offer.currency} ${offer.trip_price}" } }
                                            div { class: "ub-all-in-row", span { "Refundable damage deposit" } strong { "{offer.currency} ${offer.refundable_deposit}" } }
                                            div { class: "ub-all-in-total", span { "Total today" } strong { "{offer.currency} ${offer.total_due_today}" } }
                                            button { r#type: "button", disabled: *all_in_busy.read(), onclick: move |_| { let created = pending_payment.read().clone(); async move { let Some(mut created) = created else { return; }; all_in_busy.set(true); all_in_error.set(String::new()); booking_error.set(String::new()); payment_phase.set("switching".into()); payment_attempt_nonce.set(payment_attempt_nonce().wrapping_add(1)); document::eval(UNMOUNT_EMBEDDED_CHECKOUT); match api::switch_booking_to_all_in(&created.booking.booking_id, &created.access_token).await { Ok(response) => { created.client_secret = Some(response.checkout_client_secret); created.checkout_session_id = Some(response.checkout_session_id); created.payment_option = response.payment_option; created.all_in_offer = Some(response.offer); let _ = api::save_sensitive_json(SAVED_PENDING_PAYMENT, &created); pending_payment.set(Some(created)); }, Err(error) => all_in_error.set(error.message) } all_in_busy.set(false); payment_phase.set("idle".into()); payment_attempt_nonce.set(payment_attempt_nonce().wrapping_add(1)); } }, if *all_in_busy.read() { "Preparing one payment…" } else { "Pay {offer.currency} ${offer.total_due_today} now" } }
                                            small { "The CA$1,000 deposit is still refundable after return and inspection, less any documented damage." }
                                        }
                                    }
                                }
                                if !all_in_error.read().is_empty() { p { class: "ub-error", role: "alert", "{all_in_error}" } }
                                button { class: "ub-change-booking ub-change-booking-payment", r#type: "button", onclick: move |_| { edit_booking_error.set(String::new()); edit_booking_confirm_open.set(true); }, Icon { name: "pencil", size: 15, color: "var(--vl-forest)" } span { "Change booking details" } small { "Cancels this unpaid payment session, releases the dates, and recalculates your updated trip." } }
                                div { id: "ub-payment-terms-confirmation", class: if *payment_terms_accepted.read() { "ub-payment-terms-gate is-accepted" } else { "ub-payment-terms-gate needs-attention" },
                                    label {
                                        input {
                                            r#type: "checkbox",
                                            checked: *payment_terms_accepted.read(),
                                            onchange: move |event| {
                                                let is_accepted = event.checked();
                                                payment_terms_accepted.set(is_accepted);
                                                let next_attempt = payment_attempt_nonce().wrapping_add(1);
                                                payment_attempt_nonce.set(next_attempt);
                                                booking_error.set(String::new());
                                                if is_accepted {
                                                    payment_phase.set("idle".into());
                                                } else {
                                                    document::eval(UNMOUNT_EMBEDDED_CHECKOUT);
                                                    payment_phase.set("awaiting_terms".into());
                                                }
                                            }
                                        }
                                        span { "I have read and agree to the" }
                                    }
                                    button { class: "ub-payment-terms-link", r#type: "button", onclick: move |_| payment_terms_open.set(true), "RV Rental Terms & Conditions" }
                                    small { if *payment_terms_accepted.read() { "Confirmed. Secure Checkout is available." } else { "Required before payment can continue." } }
                                }
                            }
                        }
                    }
                }
            }
            if *payment_terms_open.read() {
                div { class: "ub-terms-layer", onclick: move |event| { event.stop_propagation(); payment_terms_open.set(false); },
                    section { class: "ub-terms-dialog", role: "dialog", aria_modal: "true", aria_labelledby: "ub-terms-title", tabindex: "-1", autofocus: true, onclick: move |event| event.stop_propagation(), onkeydown: move |event| if event.key() == Key::Escape { event.stop_propagation(); payment_terms_open.set(false); },
                        header { class: "ub-terms-dialog-head",
                            div {
                                span { class: "ub-kicker", "VL RENTAL · KELOWNA, BC" }
                                h2 { id: "ub-terms-title", "RV Rental Terms & Conditions" }
                                p { "Effective August 4, 2026 · Your booking and payment details stay open underneath." }
                            }
                            button { r#type: "button", aria_label: "Close terms and return to payment", onclick: move |_| payment_terms_open.set(false), Icon { name: "x", size: 21, color: "var(--vl-ink)" } }
                        }
                        div { class: "ub-terms-dialog-content",
                            TermsAgreementContent {}
                        }
                        footer { class: "ub-terms-dialog-actions",
                            button { class: "ub-primary", r#type: "button", onclick: move |_| payment_terms_open.set(false), "Back to payment" }
                        }
                    }
                }
            }
            if let Some(created) = deposit_overlay.as_ref() {
                div { class: "ub-deposit-layer", onclick: move |event| { event.stop_propagation(); finish_deposit_overlay(); },
                    section { class: "ub-deposit-dialog", role: "dialog", aria_modal: "true", aria_labelledby: "ub-deposit-title", aria_describedby: "ub-deposit-description", tabindex: "-1", autofocus: true, onclick: move |event| event.stop_propagation(), onkeydown: move |event| if event.key() == Key::Escape { event.stop_propagation(); finish_deposit_overlay(); },
                        header { class: "ub-deposit-head",
                            div { class: "ub-deposit-success", Icon { name: "check", size: 23, color: "white" } }
                            div {
                                span { class: "ub-kicker", "STRIPE PAYMENT CONFIRMED" }
                                h2 { id: "ub-deposit-title", "One last payment before your trip" }
                                p { id: "ub-deposit-description", "Your RV is reserved. The refundable damage deposit is paid separately by Interac e-Transfer." }
                            }
                            button { class: "ub-close", r#type: "button", aria_label: "Close damage deposit instructions", onclick: move |_| finish_deposit_overlay(), Icon { name: "x", size: 21, color: "var(--vl-ink)" } }
                        }
                        div { class: "ub-deposit-content",
                            div { class: "ub-deposit-amount",
                                span { "REFUNDABLE DAMAGE DEPOSIT" }
                                strong { "CA$1,000" }
                                small { "Not charged through Stripe" }
                            }
                            div { class: "ub-deposit-due",
                                span { "DUE 48 HOURS BEFORE DELIVERY" }
                                strong { "{created.damage_deposit.as_ref().map(|deposit| display_deposit_due(&deposit.due_at)).unwrap_or_else(|| display_delivery_deposit_due(&created.booking.starts_at))}" }
                            }
                            div { class: "ub-deposit-transfer",
                                span { "SEND INTERAC E-TRANSFER TO" }
                                div {
                                    strong { "{created.damage_deposit.as_ref().and_then(|deposit| deposit.transfer_email.as_deref()).unwrap_or(DAMAGE_DEPOSIT_ETRANSFER_EMAIL)}" }
                                    button { r#type: "button", onclick: move |_| async move { let script = format!("return navigator.clipboard.writeText({});", serde_json::to_string(DAMAGE_DEPOSIT_ETRANSFER_EMAIL).unwrap_or_else(|_| "\"\"".into())); match document::eval(&script).await { Ok(_) => deposit_copy_state.set("Email copied".into()), Err(_) => deposit_copy_state.set("Copy failed — select the email above".into()) } }, Icon { name: "copy", size: 16, color: "var(--vl-forest)" } span { if deposit_copy_state.read().is_empty() { "Copy" } else { "{deposit_copy_state}" } } }
                                }
                                small { "Include booking #{created.booking.booking_number} in the e-Transfer message." }
                            }
                            div { class: "ub-deposit-notice", Icon { name: "shield-check", size: 20, color: "var(--vl-forest)" } p { strong { "Delivery waits for verification" } "Your account changes to Paid only after VL Rental confirms receipt. We then email you and vlrental.ca@gmail.com." } }
                            div { class: "ub-deposit-actions",
                                a { href: "mailto:{DAMAGE_DEPOSIT_ETRANSFER_EMAIL}?subject=Damage%20deposit%20for%20booking%20{created.booking.booking_number}", "Email transfer details" }
                                button { class: "ub-primary", r#type: "button", onclick: move |_| finish_deposit_overlay(), "View my booking" }
                            }
                        }
                    }
                }
            }
            if *edit_booking_confirm_open.read() {
                div { class: "ub-edit-confirm-layer", onclick: move |event| { event.stop_propagation(); if !*edit_booking_busy.read() { edit_booking_confirm_open.set(false); edit_booking_error.set(String::new()); } },
                    section { class: "ub-edit-confirm", role: "alertdialog", aria_modal: "true", aria_labelledby: "ub-edit-confirm-title", aria_describedby: "ub-edit-confirm-description", tabindex: "-1", autofocus: true, onclick: move |event| event.stop_propagation(), onkeydown: move |event| if event.key() == Key::Escape { event.stop_propagation(); if !*edit_booking_busy.read() { edit_booking_confirm_open.set(false); edit_booking_error.set(String::new()); } },
                        div { class: "ub-edit-confirm-icon", Icon { name: "calendar-cog", size: 23, color: "var(--vl-forest)" } }
                        h2 { id: "ub-edit-confirm-title", "Change this booking?" }
                        p { id: "ub-edit-confirm-description", "The current unpaid reservation and Stripe Checkout session will be cancelled. Your dates will be released while you update the trip, and the server will calculate a new price before payment." }
                        if !edit_booking_error.read().is_empty() { p { class: "ub-error", role: "alert", "{edit_booking_error}" } }
                        div { class: "ub-edit-confirm-actions",
                            button { r#type: "button", disabled: *edit_booking_busy.read(), onclick: move |_| { edit_booking_confirm_open.set(false); edit_booking_error.set(String::new()); }, "Keep current payment" }
                            button { class: "ub-primary", r#type: "button", disabled: *edit_booking_busy.read(), onclick: move |_| { let created = pending_payment.read().clone(); async move { let Some(created) = created else { edit_booking_confirm_open.set(false); return; }; edit_booking_busy.set(true); edit_booking_error.set(String::new()); match api::expire_pending_booking_for_edit(&created.booking.booking_id, &created.access_token).await { Ok(()) => { let next_attempt = payment_attempt_nonce().wrapping_add(1); payment_attempt_nonce.set(next_attempt); document::eval(UNMOUNT_EMBEDDED_CHECKOUT); api::remove_sensitive_saved(SAVED_PENDING_PAYMENT); api::remove_saved("vl_active_quote"); pending_payment.set(None); payment_terms_accepted.set(false); payment_terms_open.set(false); payment_overlay_open.set(false); payment_phase.set("idle".into()); edit_booking_confirm_open.set(false); quote.set(None); accepted.set(false); booking_error.set(String::new()); addon_notice.set("Your previous unpaid reservation was released. Update any details; the exact server price will refresh automatically.".into()); open_step.set(1); let next_quote = quote_refresh_nonce().wrapping_add(1); quote_refresh_nonce.set(next_quote); spawn(async move { scroll_to_booking_step(1).await; }); }, Err(error) if error.is_conflict() => edit_booking_error.set("This reservation can no longer be changed because its payment or status has already advanced. Close this message and check the payment status before continuing.".into()), Err(error) => edit_booking_error.set(error.message) } edit_booking_busy.set(false); } }, if *edit_booking_busy.read() { "Releasing reservation…" } else { "Release and edit booking" } }
                        }
                    }
                }
            }
        }
    }
}

fn set_review_like_count(
    reviews: &mut api::RentalReviewsResponse,
    review_id: &str,
    like_count: i64,
) {
    if let Some(review) = reviews
        .reviews
        .iter_mut()
        .find(|review| review.rental_review_id == review_id)
    {
        review.like_count = like_count.max(0);
    }
}

fn set_review_like_membership(
    context: &mut api::RentalReviewContext,
    review_id: &str,
    liked: bool,
) {
    context.liked_review_ids.retain(|id| id != review_id);
    if liked {
        context.liked_review_ids.push(review_id.to_string());
    }
}

fn review_like_action_disabled(can_like: bool, is_liked: bool, busy: bool) -> bool {
    (!can_like && !is_liked) || busy
}

fn rounded_review_rating(rating: &str) -> i32 {
    rating
        .parse::<f64>()
        .ok()
        .map(|value| value.round() as i32)
        .unwrap_or_default()
}

fn is_public_booking_review_rating(rating: &str) -> bool {
    rating
        .parse::<f64>()
        .is_ok_and(|value| (4.0..=5.0).contains(&value))
}

fn displayed_review_rating(rating: &str) -> String {
    let trimmed = rating.trim_end_matches('0').trim_end_matches('.');
    if trimmed.is_empty() {
        rating.to_string()
    } else {
        trimmed.to_string()
    }
}

fn booking_review_source_label(source: &str) -> &'static str {
    match source {
        "rvezy" => "Verified on RVezy",
        "outdoorsy" => "Verified on Outdoorsy",
        _ => "Verified booking",
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
    let mut review_context = use_signal(|| None::<api::RentalReviewContext>);
    let mut like_busy = use_signal(std::collections::HashSet::<String>::new);
    let mut like_error = use_signal(String::new);
    let mut review_publish_busy = use_signal(|| false);
    let live_summary = reviews.read().as_ref().map(|value| value.summary.clone());
    let rating = live_summary
        .as_ref()
        .and_then(|summary| summary.average_rating.clone())
        .or_else(|| rental.review_rating.clone())
        .unwrap_or_else(|| "New".into());
    let review_count = live_summary
        .as_ref()
        .map(|summary| summary.review_count)
        .unwrap_or(rental.review_count);
    let rounded_rating = rating
        .parse::<f64>()
        .ok()
        .map(|value| value.round() as i32)
        .unwrap_or(0);
    let review_label = if review_count == 1 {
        "1 review".to_string()
    } else {
        format!("{review_count} reviews")
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
                button { class: "ub-rv-rating", r#type: "button", aria_label: "Read {review_label} for {rental.name}", onclick: { let slug = slug.clone(); move |event| { event.stop_propagation(); reviews_open.set(true); like_error.set(String::new()); if reviews.peek().is_none() && !*reviews_busy.peek() { reviews_busy.set(true); reviews_error.set(String::new()); let slug_for_reviews = slug.clone(); spawn(async move { match api::rental_reviews(&slug_for_reviews).await { Ok(value) => reviews.set(Some(value)), Err(message) => reviews_error.set(message) } reviews_busy.set(false); }); }; if api::access_token().is_some() && review_context.peek().is_none() { let slug_for_context = slug.clone(); spawn(async move { if let Ok(context) = api::rental_review_context(&slug_for_context).await { review_context.set(Some(context)); } }); } } },
                    RatingStars { rating: rounded_rating }
                    b { "{rating}" }
                    span { "({review_count})" }
                    small { "Read comments" }
                }
            }
            if *reviews_open.read() {
                div { class: "ub-review-backdrop", onclick: move |_| if !review_publish_busy() { reviews_open.set(false); },
                    section { class: "ub-review-modal", role: "dialog", aria_modal: "true", aria_label: "Reviews for {rental.name}", tabindex: "-1", autofocus: true, onclick: move |event| event.stop_propagation(), onkeydown: move |event| { if event.key() == Key::Escape { event.stop_propagation(); if !review_publish_busy() { reviews_open.set(false); } } },
                        header { div { RatingStars { rating: rounded_rating } h3 { "Guest reviews" } p { "{rental.name}" } } button { r#type: "button", disabled: review_publish_busy(), aria_label: "Close reviews", onclick: move |_| if !review_publish_busy() { reviews_open.set(false); }, Icon { name: "x", size: 20, color: "var(--vl-ink)" } } }
                        div { class: "ub-review-scroll",
                            if *reviews_busy.read() { p { class: "ub-review-state", "Loading reviews…" } }
                            else if !reviews_error.read().is_empty() { p { class: "ub-error", role: "alert", "{reviews_error}" } }
                            else if let Some(value) = reviews.read().as_ref() {
                                div { class: "ub-review-summary", b { if let Some(average) = value.summary.average_rating.as_ref() { "{average}" } else { "New" } } span { "out of 5 · {value.summary.review_count} guest reviews" } }
                                if !like_error.read().is_empty() { p { class: "ub-review-action-error", role: "alert", "{like_error}" } }
                                if let Some(context) = review_context.read().as_ref() {
                                    if let Some(booking_id) = context.reviewable_booking_id.as_ref() {
                                        ReviewForm { booking_id: booking_id.clone(), rental_name: rental.name.clone(), on_busy_change: move |value| review_publish_busy.set(value), on_published: { let slug = slug.clone(); move |_| { let slug = slug.clone(); spawn(async move { if let Ok(value) = api::rental_reviews(&slug).await { reviews.set(Some(value)); }; if let Ok(context) = api::rental_review_context(&slug).await { review_context.set(Some(context)); } }); } } }
                                    } else {
                                        p { class: "ub-review-policy", match context.review_state.as_str() { "used" => "Your review opportunity for this trip has already been used.", "waiting_for_return" => "You can write a review after the RV has been returned.", _ => "Reviews are available after a completed, paid RV trip." } }
                                    }
                                } else if api::access_token().is_none() {
                                    p { class: "ub-review-policy", "Sign in to write a review after return. Likes are available to customers with a paid booking." }
                                }
                                if !value
                                    .reviews
                                    .iter()
                                    .any(|review| is_public_booking_review_rating(&review.rating))
                                {
                                    p { class: "ub-review-state", "No guest comments yet." }
                                }
                                for review in value
                                    .reviews
                                    .iter()
                                    .filter(|review| is_public_booking_review_rating(&review.rating))
                                {
                                    {
                                        let rating_label = displayed_review_rating(&review.rating);
                                        let source_label = booking_review_source_label(&review.source);
                                        let reviewed_at_label = if review.reviewed_at_label.is_empty() {
                                            review.created_at.get(0..10).unwrap_or(&review.created_at)
                                        } else {
                                            &review.reviewed_at_label
                                        };
                                        rsx! { article { class: "ub-review-item", key: "{review.rental_review_id}",
                                        div { RatingStars { rating: rounded_review_rating(&review.rating) } b { "{rating_label}/5" } time { "{reviewed_at_label}" } }
                                        if !review.title.is_empty() { h4 { "{review.title}" } }
                                        if !review.body.is_empty() { p { "{review.body}" } }
                                        div { class: "ub-review-foot", small { "{review.reviewer_name} · {source_label}" }
                                            if let Some(context) = review_context.read().as_ref() {
                                                if context.own_review_ids.contains(&review.rental_review_id) {
                                                    span { class: "ub-like-own", "Your review · {review.like_count} likes" }
                                                } else {
                                                    button {
                                                        class: if context.liked_review_ids.contains(&review.rental_review_id) { "ub-like active" } else { "ub-like" },
                                                        r#type: "button",
                                                        aria_label: if context.liked_review_ids.contains(&review.rental_review_id) { "Unlike this review" } else { "Like this review" },
                                                        disabled: review_like_action_disabled(
                                                            context.can_like,
                                                            context.liked_review_ids.contains(&review.rental_review_id),
                                                            like_busy.read().contains(&review.rental_review_id),
                                                        ),
                                                        onclick: {
                                                            let review_id = review.rental_review_id.clone();
                                                            let was_liked = context.liked_review_ids.contains(&review.rental_review_id);
                                                            let displayed_like_count = review.like_count;
                                                            move |_| {
                                                                like_error.set(String::new());
                                                                let previous_like_count = reviews
                                                                    .read()
                                                                    .as_ref()
                                                                    .and_then(|value| value.reviews.iter().find(|item| item.rental_review_id == review_id))
                                                                    .map(|item| item.like_count)
                                                                    .unwrap_or(displayed_like_count);
                                                                let optimistic_reviews = reviews.read().clone();
                                                                if let Some(mut current) = optimistic_reviews {
                                                                    set_review_like_count(
                                                                        &mut current,
                                                                        &review_id,
                                                                        previous_like_count + if was_liked { -1 } else { 1 },
                                                                    );
                                                                    reviews.set(Some(current));
                                                                }
                                                                let optimistic_context = review_context.read().clone();
                                                                if let Some(mut current) = optimistic_context {
                                                                    set_review_like_membership(&mut current, &review_id, !was_liked);
                                                                    review_context.set(Some(current));
                                                                }
                                                                like_busy.write().insert(review_id.clone());
                                                                let review_id_for_request = review_id.clone();
                                                                spawn(async move {
                                                                    match api::set_rental_review_like(&review_id_for_request, !was_liked).await {
                                                                        Ok(result) => {
                                                                            let latest_reviews = reviews.read().clone();
                                                                            if let Some(mut current) = latest_reviews {
                                                                                set_review_like_count(&mut current, &result.rental_review_id, result.like_count);
                                                                                reviews.set(Some(current));
                                                                            }
                                                                            let latest_context = review_context.read().clone();
                                                                            if let Some(mut current) = latest_context {
                                                                                set_review_like_membership(&mut current, &result.rental_review_id, result.liked);
                                                                                review_context.set(Some(current));
                                                                            }
                                                                        }
                                                                        Err(_) => {
                                                                            let latest_reviews = reviews.read().clone();
                                                                            if let Some(mut current) = latest_reviews {
                                                                                set_review_like_count(&mut current, &review_id_for_request, previous_like_count);
                                                                                reviews.set(Some(current));
                                                                            }
                                                                            let latest_context = review_context.read().clone();
                                                                            if let Some(mut current) = latest_context {
                                                                                set_review_like_membership(&mut current, &review_id_for_request, was_liked);
                                                                                review_context.set(Some(current));
                                                                            }
                                                                            like_error.set("The like could not be updated. Please try again.".into());
                                                                        }
                                                                    }
                                                                    like_busy.write().remove(&review_id_for_request);
                                                                });
                                                            }
                                                        },
                                                        "♥ {review.like_count}"
                                                    }
                                                }
                                            } else { span { class: "ub-like-own", "♥ {review.like_count}" } }
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
    }
}

#[component]
fn RatingStars(rating: i32) -> Element {
    rsx! {
        span { class: "ub-rating-stars", aria_label: "{rating} out of 5 stars",
            for value in 1..=5_i32 {
                span { key: "rating-star-{value}", class: if value <= rating { "filled" } else { "" }, aria_hidden: "true", "★" }
            }
        }
    }
}

#[component]
fn StationaryPlusPriceLine(
    label: String,
    detail: String,
    amount: String,
    expanded: bool,
    on_toggle: EventHandler<()>,
) -> Element {
    rsx! {
        div { class: if expanded { "ub-price-line ub-protection-line is-open" } else { "ub-price-line ub-protection-line" },
            button {
                class: "ub-protection-summary",
                r#type: "button",
                aria_expanded: expanded,
                aria_controls: "ub-stationary-plus-details",
                onclick: move |_| on_toggle.call(()),
                span {
                    "{label}"
                    small { "{detail} · Coverage details" }
                }
                b { class: "ub-line-price", "{amount}" }
                Icon { name: "chevron-down", size: 17, color: "currentColor" }
            }
            if expanded {
                div { id: "ub-stationary-plus-details", class: "ub-protection-details",
                    h4 { "Coverage details" }
                    ul {
                        li { strong { "Delivered & parked: " } "For stationary rentals delivered to a campground or property; the guest does not drive or tow the RV." }
                        li { strong { "Reduced rate: " } "Costs less than driving protection plans because public-road transit and collision exposure are excluded." }
                        li { strong { "Site protection: " } "Covers eligible physical damage, comprehensive incidents, and site liability while the RV is parked and occupied or left stationary." }
                    }
                    p { "Coverage is subject to the rental agreement and applicable policy terms." }
                }
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
    fn booking_close_is_blocked_during_irreversible_work() {
        assert!(booking_overlay_close_blocked(
            false, true, false, false, false
        ));
        assert!(booking_overlay_close_blocked(
            false, false, true, false, false
        ));
        assert!(booking_overlay_close_blocked(
            false, false, false, true, false
        ));
        assert!(booking_overlay_close_blocked(
            false, false, false, false, true
        ));
        assert!(!booking_overlay_close_blocked(
            false, false, false, false, false
        ));
    }

    #[test]
    fn secure_checkout_waits_for_the_payment_terms_confirmation() {
        assert!(!payment_terms_allow_checkout(false));
        assert!(payment_terms_allow_checkout(true));
    }

    #[test]
    fn checkout_failures_refresh_the_quote_without_discarding_the_draft() {
        let unavailable = api::ApiError {
            status: 503,
            code: "service_unavailable".into(),
            message: "safe server message".into(),
        };
        let stale_quote = api::ApiError {
            status: 409,
            code: "conflict".into(),
            message: "conflict".into(),
        };
        let validation = api::ApiError {
            status: 400,
            code: "validation_error".into(),
            message: "validation".into(),
        };

        assert!(booking_creation_recovery_message(&unavailable)
            .unwrap()
            .contains("selections are still here"));
        assert!(booking_creation_recovery_message(&stale_quote)
            .unwrap()
            .contains("selections are still here"));
        assert!(booking_creation_recovery_message(&validation).is_none());
    }

    #[test]
    fn payment_close_is_blocked_while_checkout_is_being_changed_or_confirmed() {
        assert!(payment_overlay_close_blocked(false, false, "switching"));
        assert!(payment_overlay_close_blocked(false, false, "confirming"));
        assert!(payment_overlay_close_blocked(true, false, "idle"));
        assert!(payment_overlay_close_blocked(false, true, "idle"));
        assert!(!payment_overlay_close_blocked(false, false, "idle"));
    }

    #[test]
    fn popular_destination_addresses_are_complete_and_unique() {
        let destinations = destination_recommendations();
        let unique_addresses = destinations
            .iter()
            .map(|destination| destination.address)
            .collect::<std::collections::HashSet<_>>();

        assert_eq!(destinations.len(), 4);
        assert_eq!(unique_addresses.len(), destinations.len());
        assert!(destinations
            .iter()
            .all(|destination| destination.address.contains(", BC ")));
        assert!(destinations
            .iter()
            .any(|destination| destination.name == "Kekuli Bay Provincial Park"));
    }

    #[test]
    fn delivery_price_line_explains_the_each_way_charge_after_forty_km() {
        let delivery = api::DeliveryEstimate {
            resolved_address: "Bear Creek Provincial Park".into(),
            one_way_km: "48.4".into(),
            round_trip_km: "96.8".into(),
            delivery_fee: "192.00".into(),
            maximum_km: "150.0".into(),
            within_range: true,
        };

        assert_eq!(
            delivery_distance_detail(&delivery),
            "48.4 km one way · 8.4 km extra × CA$2.50 · each way"
        );
    }

    #[test]
    fn delivery_price_line_marks_distances_through_forty_km_as_covered() {
        let delivery = api::DeliveryEstimate {
            resolved_address: "Kelowna, BC".into(),
            one_way_km: "30.0".into(),
            round_trip_km: "60.0".into(),
            delivery_fee: "150.00".into(),
            maximum_km: "150.0".into(),
            within_range: true,
        };

        assert_eq!(
            delivery_distance_detail(&delivery),
            "30.0 km one way · covered by CA$150"
        );
    }

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
    fn horizontal_calendar_swipes_change_months() {
        assert_eq!(calendar_swipe_month_delta((180.0, 120.0), (90.0, 130.0)), 1);
        assert_eq!(
            calendar_swipe_month_delta((90.0, 120.0), (180.0, 110.0)),
            -1
        );
    }

    #[test]
    fn short_or_vertical_calendar_gestures_do_not_change_months() {
        assert_eq!(calendar_swipe_month_delta((100.0, 100.0), (70.0, 103.0)), 0);
        assert_eq!(
            calendar_swipe_month_delta((100.0, 100.0), (155.0, 175.0)),
            0
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
            rental_id: "rental-id".into(),
            slug: "test-rv".into(),
            name: "Test RV".into(),
            category: "rv".into(),
            summary: String::new(),
            description: String::new(),
            model_year: Some(2025),
            manufacturer: "Test".into(),
            model: "RV".into(),
            rv_type: "travel_trailer".into(),
            length_ft: Some("24.0".into()),
            slide_outs: 1,
            pet_friendly: false,
            capacity: 4,
            price_unit: "night".into(),
            base_rate: "100.00".into(),
            currency: "CAD".into(),
            min_units: 3,
            refundable_deposit: "1000.00".into(),
            hero_image_url: None,
            is_active: true,
            sort_order: 0,
            review_rating: None,
            review_count: 0,
        };
        let details = api::RentalResponse {
            rental: rental.clone(),
            media: Vec::new(),
            features: Vec::new(),
            addons: vec![
                api::RentalAddon {
                    addon_id: "addon-id".into(),
                    addon_key: "portable_bbq".into(),
                    label: "Portable BBQ".into(),
                    description: "Portable barbecue".into(),
                    icon_name: "cooking-pot".into(),
                    price: "50.00".into(),
                    charge_type: "fixed".into(),
                    is_recommended: false,
                    is_active: true,
                    sort_order: 0,
                },
                api::RentalAddon {
                    addon_id: "linens-id".into(),
                    addon_key: BEDDING_ADDON_KEY.into(),
                    label: "Bedding and linens".into(),
                    description: "Fresh linens".into(),
                    icon_name: "sparkles".into(),
                    price: "40.00".into(),
                    charge_type: "per_booking".into(),
                    is_recommended: false,
                    is_active: true,
                    sort_order: 1,
                },
            ],
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
                subtotal: "827.00".into(),
                tax_total: "86.64".into(),
                refundable_deposit: "1000.00".into(),
                total: "913.64".into(),
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
                    quantity: "1".into(),
                    unit_price: "180.00".into(),
                    amount: "180.00".into(),
                },
                api::QuoteItem {
                    item_type: "tax".into(),
                    item_key: "tax_primary".into(),
                    label: "GST (5%)".into(),
                    quantity: "1".into(),
                    unit_price: "41.35".into(),
                    amount: "41.35".into(),
                },
                api::QuoteItem {
                    item_type: "tax".into(),
                    item_key: "tax_secondary".into(),
                    label: "PST (7%)".into(),
                    quantity: "1".into(),
                    unit_price: "45.29".into(),
                    amount: "45.29".into(),
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
        let with_four_beds = optimistic_price(
            Some(&rental),
            Some(&details),
            &vec![BEDDING_ADDON_KEY.into(); MAX_BEDDING_QUANTITY],
            4,
            Some(&delivery),
            Some(&previous_quote),
        )
        .unwrap();
        let short_stay =
            optimistic_price(Some(&rental), Some(&details), &[], 1, Some(&delivery), None).unwrap();

        assert!(with_addon
            .lines
            .iter()
            .any(|line| line.label == "Portable BBQ"));
        assert_eq!(without_addon.total, 913.64);
        assert!(without_addon
            .lines
            .iter()
            .any(|line| line.label == "GST (5%)"));
        let short_rental = short_stay
            .lines
            .iter()
            .find(|line| line.key.starts_with("rental-"))
            .unwrap();
        assert_eq!(short_rental.amount, 300.0);
        assert!(short_rental.label.contains("3 nights (minimum)"));
        assert_eq!(
            short_rental.detail.as_deref(),
            Some("Your selected stay: 1 night")
        );
        assert_eq!(
            short_stay
                .lines
                .iter()
                .find(|line| line.key == "stationary-plus")
                .unwrap()
                .amount,
            150.0
        );
        assert!(without_addon
            .lines
            .iter()
            .any(|line| line.label == "PST (7%)"));
        assert_eq!(with_addon.total - without_addon.total, 56.0);
        let bedding = with_four_beds
            .lines
            .iter()
            .find(|line| line.key == "addon-linens")
            .unwrap();
        assert_eq!(bedding.amount, 160.0);
        assert_eq!(bedding.detail.as_deref(), Some("4 beds × CA$40.00"));
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
    fn a_saved_payment_reopens_the_booking_overlay_after_refresh() {
        assert!(should_open_booking_overlay(false, true));
        assert!(should_open_booking_overlay(true, false));
        assert!(!should_open_booking_overlay(false, false));
    }

    #[test]
    fn embedded_checkout_script_returns_the_async_mount_result() {
        let script = embedded_checkout_script("\"pk_test_example\"", "\"secret_example\"");

        assert!(script.trim_start().starts_with("return await (async () =>"));
        assert!(script.contains("window.Stripe(\"pk_test_example\")"));
        assert!(script.contains("fetchClientSecret: async () => \"secret_example\""));
        assert!(script.contains("window.__vlEmbeddedCheckout.destroy()"));
        assert!(UNMOUNT_EMBEDDED_CHECKOUT.contains(".destroy()"));
    }

    #[test]
    fn embedded_checkout_waits_for_payment_layer_terms() {
        assert!(!payment_checkout_may_start(true, false, "idle"));
        assert!(payment_checkout_may_start(true, true, "idle"));
        assert!(!payment_checkout_may_start(false, true, "idle"));
        assert!(!payment_checkout_may_start(true, true, "checking"));
    }

    #[test]
    fn payment_terms_guidance_targets_the_required_checkbox_smoothly() {
        assert!(GUIDE_TO_PAYMENT_TERMS.contains("ub-payment-terms-confirmation"));
        assert!(GUIDE_TO_PAYMENT_TERMS.contains("scrollIntoView"));
        assert!(GUIDE_TO_PAYMENT_TERMS.contains("behavior: reducedMotion ? 'auto' : 'smooth'"));
        assert!(GUIDE_TO_PAYMENT_TERMS.contains("focus({ preventScroll: true })"));
    }

    #[test]
    fn checkout_status_is_polled_without_waiting_for_stripe_on_complete() {
        assert!(checkout_status_poll_due(0, false));
        assert!(!checkout_status_poll_due(1, false));
        assert!(checkout_status_poll_due(4, false));
        assert!(checkout_status_poll_due(1, true));
    }

    #[test]
    fn checkout_due_now_must_match_the_immutable_trip_total() {
        let mut booking = api::Booking {
            booking_id: "booking-1".into(),
            booking_number: "VL-1".into(),
            quote_id: "quote-1".into(),
            rental_slug: "test-rv".into(),
            rental_name: "Test RV".into(),
            status: "pending_payment".into(),
            payment_status: "unpaid".into(),
            starts_at: "2030-08-30T21:00:00Z".into(),
            ends_at: "2030-09-03T18:00:00Z".into(),
            currency: "CAD".into(),
            total: "1406.24".into(),
            amount_due_now: "421.87".into(),
            payment_option: "scheduled".into(),
            refundable_deposit_paid: false,
            paid_transaction_total: None,
            damage_deposit_status: String::new(),
            damage_deposit_due_at: None,
            damage_deposit_collection_method: "e_transfer".into(),
            damage_deposit_transfer_email: None,
            balance_due_at: None,
            payment_expires_at: None,
            review_id: None,
            can_review: false,
            review_opportunity_used: false,
        };

        assert!(booking_payment_amount_is_valid(&booking));
        booking.amount_due_now = "1406.24".into();
        assert!(booking_payment_amount_is_valid(&booking));
        booking.amount_due_now = "344.71".into();
        assert!(!booking_payment_amount_is_valid(&booking));

        booking.amount_due_now = "421.87".into();
        let mut created = api::CreatedBooking {
            booking,
            access_token: "private-token".into(),
            notification_email_sent: false,
            client_secret: Some("cs_test_secret".into()),
            checkout_session_id: Some("cs_test_all_in".into()),
            payment_enabled: true,
            payment_expires_at: None,
            checkout_url: None,
            payment_option: "all_in".into(),
            all_in_offer: Some(api::AllInPaymentOffer {
                trip_price: "1406.24".into(),
                refundable_deposit: "1000.00".into(),
                total_due_today: "2406.24".into(),
                currency: "CAD".into(),
            }),
            damage_deposit: None,
        };
        assert!(created_payment_amount_is_valid(&created));
        created.all_in_offer.as_mut().unwrap().total_due_today = "2406.25".into();
        assert!(!created_payment_amount_is_valid(&created));
    }

    #[test]
    fn payment_schedule_uses_cents_and_rejects_a_stale_saved_quote() {
        let booking = api::Booking {
            booking_id: "booking-1".into(),
            booking_number: "VL-1".into(),
            quote_id: "quote-1".into(),
            rental_slug: "test-rv".into(),
            rental_name: "Test RV".into(),
            status: "pending_payment".into(),
            payment_status: "unpaid".into(),
            starts_at: "2030-08-30T21:00:00Z".into(),
            ends_at: "2030-09-03T18:00:00Z".into(),
            currency: "CAD".into(),
            total: "1965.44".into(),
            amount_due_now: "589.63".into(),
            payment_option: "scheduled".into(),
            refundable_deposit_paid: false,
            paid_transaction_total: None,
            damage_deposit_status: String::new(),
            damage_deposit_due_at: None,
            damage_deposit_collection_method: "e_transfer".into(),
            damage_deposit_transfer_email: None,
            balance_due_at: Some("2030-07-31T21:00:00Z".into()),
            payment_expires_at: None,
            review_id: None,
            can_review: false,
            review_opportunity_used: false,
        };
        let mut quote = api::QuoteResponse {
            quote: api::Quote {
                quote_id: "quote-1".into(),
                rental_slug: "test-rv".into(),
                starts_at: booking.starts_at.clone(),
                ends_at: booking.ends_at.clone(),
                guests: 2,
                units: 4,
                currency: "CAD".into(),
                subtotal: "1787.00".into(),
                tax_total: "178.44".into(),
                refundable_deposit: "1000.00".into(),
                total: "1965.44".into(),
                expires_at: String::new(),
            },
            items: Vec::new(),
        };

        assert_eq!(booking_payment_percent(&booking), Some(30));
        assert_eq!(
            booking_remaining_balance(&booking).as_deref(),
            Some("1375.81")
        );
        assert_eq!(booking_stay_nights(&booking), Some(4));
        assert_eq!(
            display_booking_moment(&booking.starts_at),
            "Friday, August 30, 2030 · 2:00 PM"
        );
        assert_eq!(
            display_booking_moment(&booking.ends_at),
            "Tuesday, September 3, 2030 · 11:00 AM"
        );
        assert!(quote_matches_booking(&quote, &booking));
        let matching_draft = api::TripDraft {
            rental_slug: "test-rv".into(),
            starts_on: "2030-08-30".into(),
            ends_on: "2030-09-03".into(),
            guests: 2,
            addon_keys: Vec::new(),
            delivery_km: Some("32.0".into()),
            delivery_address: Some("Bear Creek Provincial Park".into()),
            attending_event: false,
            towing_after_delivery: false,
        };
        assert!(draft_matches_booking(&matching_draft, &booking));
        quote.quote.total = "1965.45".into();
        assert!(!quote_matches_booking(&quote, &booking));
    }

    #[test]
    fn payment_amount_parser_accepts_only_plain_non_negative_cad_values() {
        assert_eq!(money_cents("1965.44"), Some(196_544));
        assert_eq!(money_cents("1.2"), Some(120));
        assert_eq!(money_cents("1"), Some(100));
        assert_eq!(money_cents("1e3"), None);
        assert_eq!(money_cents("-1.00"), None);
        assert_eq!(money_cents("1.001"), None);
        assert_eq!(money_cents("NaN"), None);
    }

    #[test]
    fn rental_selection_keeps_dates_only_when_server_marks_it_available() {
        assert!(rental_selection_keeps_dates(true, Some(true)));
        assert!(!rental_selection_keeps_dates(true, Some(false)));
        assert!(!rental_selection_keeps_dates(true, None));
        assert!(!rental_selection_keeps_dates(false, Some(true)));
    }

    #[test]
    fn selected_dates_hide_rentals_missing_from_live_availability() {
        let all = ["available", "booked"];
        let available = ["available"];

        assert_eq!(
            displayed_rental_choices(&all, Some(&available), true),
            available
        );
        assert_eq!(displayed_rental_choices(&all, Some(&available), false), all);
        assert!(displayed_rental_choices(&all, None, true).is_empty());
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

    #[test]
    fn fleet_calendar_counts_only_rvs_available_for_the_entire_stay() {
        let starts_on = NaiveDate::from_ymd_opt(2030, 8, 1).unwrap();
        let ends_on = NaiveDate::from_ymd_opt(2030, 8, 4).unwrap();
        let blocked_at_start = vec![(
            booking_moment(starts_on, 14).unwrap(),
            booking_moment(NaiveDate::from_ymd_opt(2030, 8, 2).unwrap(), 11).unwrap(),
        )];
        let blocked_at_end = vec![(
            booking_moment(NaiveDate::from_ymd_opt(2030, 8, 3).unwrap(), 14).unwrap(),
            booking_moment(ends_on, 11).unwrap(),
        )];
        let schedules = vec![blocked_at_start, blocked_at_end, Vec::new()];

        assert_eq!(
            fleet_available_rental_count(starts_on, None, None, 3, &schedules),
            1
        );
        assert_eq!(
            fleet_available_rental_count(ends_on, Some(starts_on), None, 3, &schedules[..2],),
            0
        );
    }

    #[test]
    fn fleet_calendar_allows_same_day_turnover() {
        let return_day = NaiveDate::from_ymd_opt(2030, 8, 10).unwrap();
        let schedules = vec![vec![(
            booking_moment(NaiveDate::from_ymd_opt(2030, 8, 1).unwrap(), 14).unwrap(),
            booking_moment(return_day, 11).unwrap(),
        )]];

        assert_eq!(
            fleet_available_rental_count(return_day, None, None, 3, &schedules),
            1
        );
    }

    #[test]
    fn fleet_calendar_allows_a_short_return_and_protects_three_nights() {
        let starts_on = NaiveDate::from_ymd_opt(2030, 8, 30).unwrap();
        let valid_return = NaiveDate::from_ymd_opt(2030, 9, 2).unwrap();
        let too_early = NaiveDate::from_ymd_opt(2030, 9, 1).unwrap();
        let schedules = vec![Vec::new()];

        assert_eq!(
            fleet_available_rental_count(valid_return, Some(starts_on), None, 3, &schedules,),
            1
        );
        assert_eq!(
            fleet_available_rental_count(too_early, Some(starts_on), None, 3, &schedules,),
            1
        );

        let blocked_during_minimum = vec![vec![(
            booking_moment(NaiveDate::from_ymd_opt(2030, 9, 1).unwrap(), 14).unwrap(),
            booking_moment(NaiveDate::from_ymd_opt(2030, 9, 2).unwrap(), 11).unwrap(),
        )]];
        assert_eq!(
            fleet_available_rental_count(
                too_early,
                Some(starts_on),
                None,
                3,
                &blocked_during_minimum,
            ),
            0
        );
    }

    #[test]
    fn fleet_availability_contract_is_validated_before_dates_unlock() {
        let response = api::FleetAvailabilityResponse {
            starts_on: "2030-08-01".into(),
            ends_on: "2030-10-04".into(),
            delivery_time: "14:00".into(),
            return_time: "11:00".into(),
            timezone: "America/Vancouver".into(),
            minimum_nights: 3,
            total_rentals: 1,
            rentals: vec![api::FleetRentalAvailability {
                rental_slug: "test-rv".into(),
                unavailable: Vec::new(),
            }],
            unavailable_start_dates: Vec::new(),
        };
        assert!(validated_fleet_availability(
            &response,
            NaiveDate::from_ymd_opt(2030, 8, 1).unwrap(),
            NaiveDate::from_ymd_opt(2030, 10, 4).unwrap(),
        )
        .is_ok());

        let mut malformed = response;
        malformed.total_rentals = 2;
        assert!(validated_fleet_availability(
            &malformed,
            NaiveDate::from_ymd_opt(2030, 8, 1).unwrap(),
            NaiveDate::from_ymd_opt(2030, 10, 4).unwrap(),
        )
        .is_err());
    }

    #[test]
    fn booking_calendar_fails_closed_on_an_invalid_schedule_response() {
        let response = api::AvailabilityResponse {
            rental_slug: "test-rv".into(),
            starts_on: "2030-08-01".into(),
            ends_on: "2030-10-04".into(),
            unavailable: vec![api::UnavailableInterval {
                starts_at: "invalid".into(),
                ends_at: "2030-08-13T18:00:00Z".into(),
            }],
            delivery_time: "14:00".into(),
            return_time: "11:00".into(),
            timezone: "America/Vancouver".into(),
            minimum_nights: 3,
        };
        assert!(validated_booking_availability(
            &response,
            "test-rv",
            NaiveDate::from_ymd_opt(2030, 8, 1).unwrap(),
            NaiveDate::from_ymd_opt(2030, 10, 4).unwrap(),
        )
        .is_err());
    }

    #[test]
    fn review_like_updates_are_scoped_to_one_review() {
        let review = |id: &str, like_count| api::RentalReview {
            rental_review_id: id.into(),
            rental_slug: "test-rv".into(),
            rating: "5.00".into(),
            title: String::new(),
            body: "A verified review".into(),
            reviewer_name: "Guest".into(),
            source: "vl_rental".into(),
            source_url: None,
            reviewed_at_label: "Aug 1, 2030".into(),
            like_count,
            created_at: "2030-08-01T12:00:00Z".into(),
        };
        let mut response = api::RentalReviewsResponse {
            summary: api::RentalReviewSummary {
                average_rating: Some("5.00".into()),
                review_count: 2,
            },
            reviews: vec![review("first", 3), review("second", 7)],
        };

        set_review_like_count(&mut response, "first", 4);

        assert_eq!(response.reviews[0].like_count, 4);
        assert_eq!(response.reviews[1].like_count, 7);
    }

    #[test]
    fn booking_review_modal_only_accepts_four_to_five_star_reviews() {
        for rating in [
            "1", "2.00", "3", "3.75", "5.01", "6", "inf", "NaN", "invalid",
        ] {
            assert!(!is_public_booking_review_rating(rating));
        }
        for rating in ["4", "4.00", "4.75", "5", "5.00"] {
            assert!(is_public_booking_review_rating(rating));
        }
    }

    #[test]
    fn review_like_membership_is_deduplicated_and_reversible() {
        let mut context = api::RentalReviewContext {
            can_like: true,
            liked_review_ids: vec!["first".into(), "first".into(), "second".into()],
            own_review_ids: Vec::new(),
            reviewable_booking_id: None,
            review_state: "used".into(),
        };

        set_review_like_membership(&mut context, "first", true);
        assert_eq!(context.liked_review_ids, vec!["second", "first"]);

        set_review_like_membership(&mut context, "first", false);
        assert_eq!(context.liked_review_ids, vec!["second"]);
    }

    #[test]
    fn existing_like_can_be_removed_after_like_eligibility_is_lost() {
        assert!(!review_like_action_disabled(false, true, false));
        assert!(review_like_action_disabled(false, false, false));
        assert!(review_like_action_disabled(true, true, true));
    }

    #[test]
    fn damage_deposit_fallback_is_exactly_48_hours_before_delivery() {
        assert_eq!(
            display_delivery_deposit_due("2026-09-04T21:00:00Z"),
            "September 2, 2026 at 2:00 PM"
        );
    }

    #[test]
    fn fuel_addons_have_distinct_customer_descriptions() {
        assert_eq!(
            addon_description("bbq_fuel"),
            "Fuel supplied for the portable BBQ."
        );
        assert_eq!(
            addon_description("generator_fuel"),
            "Fuel supplied for the portable generator."
        );
        assert_eq!(
            addon_description("generator"),
            "Portable backup power; fuel not included."
        );
    }
}

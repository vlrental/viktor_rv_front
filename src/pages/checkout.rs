use dioxus::prelude::*;

use super::booking_overlay::has_saved_pending_payment;
use crate::{api, BookingLaunchRequest, Route};

fn restored_checkout_search(draft: &api::TripDraft) -> api::CatalogSearchDraft {
    api::CatalogSearchDraft {
        location: "Kelowna, BC".into(),
        radius_km: 150,
        starts_on: (!draft.starts_on.trim().is_empty()).then(|| draft.starts_on.clone()),
        ends_on: (!draft.ends_on.trim().is_empty()).then(|| draft.ends_on.clone()),
        guests: draft.guests.clamp(1, 10),
    }
}

/// Compatibility route for older saved links.
///
/// Booking confirmation now belongs exclusively to the unified Home overlay.
/// Rendering the retired checkout form here could create a pending Stripe
/// reservation and then incorrectly present it as confirmed, so this route
/// only restores safe search context and immediately redirects into the
/// canonical flow.
#[component]
pub fn Checkout() -> Element {
    let navigator = use_navigator();
    let mut booking_launch_request = use_context::<BookingLaunchRequest>();

    use_effect(move || {
        let has_pending_payment = has_saved_pending_payment();
        if !has_pending_payment {
            if let Some(draft) = api::load_json::<api::TripDraft>("vl_trip_draft") {
                let search = restored_checkout_search(&draft);
                let _ = api::save_json("vl_catalog_search", &search);
            }
            booking_launch_request.0.set(true);
        }
        navigator.replace(Route::Home {});
    });

    rsx! {
        div { hidden: true, aria_live: "polite", "Opening the secure booking window" }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn draft() -> api::TripDraft {
        api::TripDraft {
            rental_slug: "example-rv".into(),
            starts_on: "2026-09-10".into(),
            ends_on: "2026-09-13".into(),
            guests: 99,
            addon_keys: Vec::new(),
            delivery_km: Some("25".into()),
            delivery_address: Some("Kelowna, BC".into()),
            attending_event: false,
            towing_after_delivery: false,
        }
    }

    #[test]
    fn legacy_checkout_restores_only_safe_search_context() {
        let search = restored_checkout_search(&draft());

        assert_eq!(search.starts_on.as_deref(), Some("2026-09-10"));
        assert_eq!(search.ends_on.as_deref(), Some("2026-09-13"));
        assert_eq!(search.guests, 10);
        assert_eq!(search.radius_km, 150);
    }

    #[test]
    fn compatibility_route_cannot_submit_or_confirm_a_booking() {
        let source = include_str!("checkout.rs");

        assert!(!source.contains(concat!("api::", "create_booking(")));
        assert!(!source.contains(concat!("vl_last_", "booking")));
        assert!(source.contains("navigator.replace(Route::Home {})"));
    }
}

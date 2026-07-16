use dioxus::prelude::*;

use crate::data::{rv_listings, IMG_BULLET};
use crate::Route;
use crate::{api, components::Icon, pricing};

const CSS: Asset = asset!("/assets/css/confirmed.css");

/// Страница подтверждения брони — Pencil-фреймы pivyP (desktop) / V4VB2P (mobile).
#[component]
pub fn Confirmed() -> Element {
    let created = api::load_sensitive_json::<api::CreatedBooking>("vl_last_booking");
    let draft = api::load_json::<api::TripDraft>("vl_trip_draft");
    if created.is_none() {
        return rsx! {
            document::Link { rel: "stylesheet", href: CSS }
            div { class: "cf-body",
                div { class: "cf-inner",
                    h1 { class: "cf-title", "No confirmed booking found" }
                    p { class: "cf-sub", "Complete the booking window before opening this page." }
                    div { class: "cf-buttons", Link { class: "cf-btn-primary", to: Route::Home {}, "Browse available RVs" } }
                }
            }
        };
    }
    let booking_number = created
        .as_ref()
        .map(|value| value.booking.booking_number.clone())
        .unwrap_or_else(|| "Unavailable".into());
    rsx! {
        document::Link { rel: "stylesheet", href: CSS }
        div { class: "cf-body",
            div { class: "cf-inner",
                div { class: "cf-check",
                    Icon { name: "check", size: 44, color: "var(--vl-white)" }
                }
                h1 { class: "cf-title", "Booking confirmed!" }
                p { class: "cf-sub",
                    if created.as_ref().map(|value| value.notification_email_sent).unwrap_or(false) {
                        "Your booking is stored and a confirmation email has been sent."
                    } else {
                        "Your booking is stored. We could not send the email, but your confirmation is valid."
                    }
                }
                div { class: "cf-ref",
                    Icon { name: "ticket", size: 16, color: "var(--vl-ink)" }
                    span { class: "cf-ref-t", "Confirmation #{booking_number}" }
                }
                SummaryCard { created, draft }
                div { class: "cf-buttons",
                    Link { class: "cf-btn-primary", to: Route::Account {}, "View my booking" }
                    Link { class: "cf-btn-secondary", to: Route::Home {}, "Back to home" }
                }
            }
        }
    }
}

#[component]
fn SummaryCard(created: Option<api::CreatedBooking>, draft: Option<api::TripDraft>) -> Element {
    let booking = created.as_ref().map(|value| &value.booking);
    let rental_listing = draft.as_ref().and_then(|value| {
        rv_listings()
            .into_iter()
            .find(|listing| listing.slug == value.rental_slug)
    });
    let rental_label = rental_listing
        .map(|listing| listing.title.to_string())
        .or_else(|| draft.as_ref().map(|value| value.rental_slug.clone()))
        .unwrap_or_else(|| "Rental".into());
    let rental_image = rental_listing
        .map(|listing| listing.image)
        .unwrap_or(IMG_BULLET);
    rsx! {
        div { class: "cf-summary",
            div { class: "cf-item",
                div {
                    class: "cf-item-img",
                    style: "background-image: url('{rental_image}');",
                }
                div { class: "cf-item-c",
                    div { class: "cf-item-t", "{rental_label}" }
                    div { class: "cf-item-r",
                        Icon { name: "map-pin", size: 14, color: "var(--vl-muted)" }
                        span { "Kelowna, BC · Delivery calculated" }
                    }
                }
            }
            div { class: "cf-divider" }
            div { class: "cf-grid",
                GridCell { label: "Delivery/setup", value: draft.as_ref().map(|value| format!("{} at 2:00 PM", value.starts_on)).unwrap_or_default() }
                GridCell { label: "Return", value: draft.as_ref().map(|value| format!("{} at 11:00 AM", value.ends_on)).unwrap_or_default() }
                GridCell { label: "Guests", value: format!("{} guests", draft.as_ref().map(|value| value.guests).unwrap_or_default()) }
                GridCell { label: "Trip price", value: format!("CA${}", booking.map(|value| value.total.as_str()).unwrap_or("0.00")) }
                GridCell { label: "Booking payment", value: format!("CA${}", booking.map(|value| value.amount_due_now.as_str()).unwrap_or("0.00")) }
                GridCell { label: "Refundable damage deposit", value: pricing::money(pricing::DAMAGE_DEPOSIT) }
            }
            div { class: "cf-payment-note", "The separate refundable CA$1,000 damage deposit is charged 48 hours before delivery. After return and inspection, it is refunded to the original payment method less any documented damage. The secure payment link is sent by email." }
            div { class: "cf-divider" }
            div { class: "cf-host",
                div { class: "cf-host-l",
                    Icon { name: "phone", size: 16, color: "var(--vl-ink)" }
                    span { "Questions? Call VL Rental" }
                }
                span { class: "cf-host-p", "+1 (250) 878 5874" }
            }
        }
    }
}

#[component]
fn GridCell(label: &'static str, value: String) -> Element {
    rsx! {
        div { class: "cf-cell",
            div { class: "cf-cell-l", "{label}" }
            div { class: "cf-cell-v", "{value}" }
        }
    }
}

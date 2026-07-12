use dioxus::prelude::*;

use crate::{api, components::Icon};
use crate::data::IMG_JAYCO;
use crate::Route;

const CSS: Asset = asset!("/assets/css/confirmed.css");

/// Страница подтверждения брони — Pencil-фреймы pivyP (desktop) / V4VB2P (mobile).
#[component]
pub fn Confirmed() -> Element {
    let created = api::load_json::<api::CreatedBooking>("vl_last_booking");
    let draft = api::load_json::<api::TripDraft>("vl_trip_draft");
    let booking_number = created.as_ref().map(|value| value.booking.booking_number.clone()).unwrap_or_else(|| "Unavailable".into());
    rsx! {
        document::Link { rel: "stylesheet", href: CSS }
        div { class: "cf-body",
            div { class: "cf-inner",
                div { class: "cf-check",
                    Icon { name: "check", size: 44, color: "var(--vl-white)" }
                }
                h1 { class: "cf-title", "Booking confirmed!" }
                p { class: "cf-sub", "Your test booking is stored. No payment was charged." }
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
    let rental_label = draft.as_ref().map(|value| value.rental_slug.clone()).unwrap_or_else(|| "Rental".into());
    rsx! {
        div { class: "cf-summary",
            div { class: "cf-item",
                div {
                    class: "cf-item-img",
                    style: "background-image: url('{IMG_JAYCO}');",
                }
                div { class: "cf-item-c",
                    div { class: "cf-item-t", "{rental_label}" }
                    div { class: "cf-item-r",
                        Icon { name: "map-pin", size: 14, color: "var(--vl-muted)" }
                        span { "Kelowna, BC · Delivery included" }
                    }
                }
            }
            div { class: "cf-divider" }
            div { class: "cf-grid",
                GridCell { label: "Check-in", value: draft.as_ref().map(|value| value.starts_on.clone()).unwrap_or_default() }
                GridCell { label: "Check-out", value: draft.as_ref().map(|value| value.ends_on.clone()).unwrap_or_default() }
                GridCell { label: "Guests", value: format!("{} guests", draft.as_ref().map(|value| value.guests).unwrap_or_default()) }
                GridCell { label: "Test total", value: format!("CA${}", booking.map(|value| value.total.as_str()).unwrap_or("0.00")) }
            }
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

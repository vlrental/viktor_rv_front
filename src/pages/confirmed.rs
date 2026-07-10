use dioxus::prelude::*;

use crate::components::Icon;
use crate::data::IMG_JAYCO;
use crate::Route;

const CSS: Asset = asset!("/assets/css/confirmed.css");

/// Страница подтверждения брони — Pencil-фреймы pivyP (desktop) / V4VB2P (mobile).
#[component]
pub fn Confirmed() -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: CSS }
        div { class: "cf-body",
            div { class: "cf-inner",
                div { class: "cf-check",
                    Icon { name: "check", size: 44, color: "var(--vl-white)" }
                }
                h1 { class: "cf-title", "Booking confirmed!" }
                p { class: "cf-sub",
                    "You're all set, Alex. We've emailed your confirmation and our team will reach out about delivery. Adventure awaits!"
                }
                div { class: "cf-ref",
                    Icon { name: "ticket", size: 16, color: "var(--vl-ink)" }
                    span { class: "cf-ref-t", "Confirmation #VL-2026-1842" }
                }
                SummaryCard {}
                div { class: "cf-buttons",
                    Link { class: "cf-btn-primary", to: Route::Catalog {}, "View my booking" }
                    Link { class: "cf-btn-secondary", to: Route::Home {}, "Back to home" }
                }
            }
        }
    }
}

#[component]
fn SummaryCard() -> Element {
    rsx! {
        div { class: "cf-summary",
            div { class: "cf-item",
                div {
                    class: "cf-item-img",
                    style: "background-image: url('{IMG_JAYCO}');",
                }
                div { class: "cf-item-c",
                    div { class: "cf-item-t", "Jayco 26' 5th Wheel" }
                    div { class: "cf-item-r",
                        Icon { name: "map-pin", size: 14, color: "var(--vl-muted)" }
                        span { "Kelowna, BC · Delivery included" }
                    }
                }
            }
            div { class: "cf-divider" }
            div { class: "cf-grid",
                GridCell { label: "Check-in", value: "Jul 12, 2026" }
                GridCell { label: "Check-out", value: "Jul 15, 2026" }
                GridCell { label: "Guests", value: "4 guests" }
                GridCell { label: "Total paid", value: "CA$879.20" }
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
fn GridCell(label: &'static str, value: &'static str) -> Element {
    rsx! {
        div { class: "cf-cell",
            div { class: "cf-cell-l", "{label}" }
            div { class: "cf-cell-v", "{value}" }
        }
    }
}

use dioxus::prelude::*;

use crate::api;
use crate::components::Icon;
use crate::data::IMG_JAYCO;
use crate::Route;

const CSS: Asset = asset!("/assets/css/checkout.css");

/// Страница оформления брони — Pencil-фреймы Q6oqIF (desktop) / QyGrm (mobile).
#[component]
pub fn Checkout() -> Element {
    let signed_in = api::current_user().is_some();
    rsx! {
        document::Link { rel: "stylesheet", href: CSS }
        div { class: "co-body",
            div { class: "co-breadcrumb",
                span { class: "co-breadcrumb-a", "Jayco 26' 5th Wheel" }
                Icon { name: "chevron-right", size: 15, color: "var(--vl-muted)" }
                span { class: "co-breadcrumb-b", "Confirm and pay" }
            }
            h1 { class: "co-title", "Confirm and pay" }
            div { class: "co-row",
                div { class: "co-left",
                    YourTripCard {}
                    AddOnsCard {}
                    GuestDetailsCard {}
                    PaymentCard {}
                }
                SummaryCard { signed_in }
            }
        }
    }
}

#[component]
fn YourTripCard() -> Element {
    rsx! {
        div { class: "co-card",
            div { class: "co-card-h", "Your trip" }
            TripRow { label: "Dates", value: "Jul 12 – 15, 2026 · 3 nights" }
            TripRow { label: "Guests", value: "4 guests" }
            TripRow { label: "Delivery", value: "Deliver & set up at campsite" }
        }
    }
}

#[component]
fn TripRow(label: &'static str, value: &'static str) -> Element {
    rsx! {
        div { class: "co-trip-row",
            div { class: "co-trip-col",
                div { class: "co-trip-label", "{label}" }
                div { class: "co-trip-value", "{value}" }
            }
            span { class: "co-trip-edit", "Edit" }
        }
    }
}

#[component]
fn AddOnsCard() -> Element {
    rsx! {
        div { class: "co-card",
            div { class: "co-card-h", "Add-ons" }
            AddOnRow { title: "Bedding and Linens", price: "$80", checked: true }
            AddOnRow { title: "Portable BBQ", price: "$50 + Refill", checked: false }
            AddOnRow { title: "Pet Deposit (non refundable)", price: "$100", checked: false }
        }
    }
}

#[component]
fn AddOnRow(title: &'static str, price: &'static str, checked: bool) -> Element {
    rsx! {
        div { class: "co-addon",
            div { class: "co-addon-l",
                div {
                    class: if checked { "co-addon-box checked" } else { "co-addon-box" },
                    if checked {
                        Icon { name: "check", size: 14, color: "var(--vl-white)" }
                    }
                }
                span { class: "co-addon-t", "{title}" }
            }
            span { class: "co-addon-p", "{price}" }
        }
    }
}

#[component]
fn GuestDetailsCard() -> Element {
    rsx! {
        div { class: "co-card",
            div { class: "co-card-h", "Guest details" }
            div { class: "co-field",
                label { class: "co-field-label", "Full name" }
                input { class: "co-input", r#type: "text", placeholder: "Alex Johnson" }
            }
            div { class: "co-field-row stack",
                div { class: "co-field grow",
                    label { class: "co-field-label", "Email" }
                    input { class: "co-input", r#type: "text", placeholder: "alex@email.com" }
                }
                div { class: "co-field grow",
                    label { class: "co-field-label", "Phone" }
                    input { class: "co-input", r#type: "text", placeholder: "+1 (250) 000 0000" }
                }
            }
        }
    }
}

#[component]
fn PaymentCard() -> Element {
    rsx! {
        div { class: "co-card",
            div { class: "co-card-h", "Payment" }
            div { class: "co-field",
                label { class: "co-field-label", "Card number" }
                input { class: "co-input", r#type: "text", placeholder: "1234 5678 9012 3456" }
            }
            div { class: "co-field-row",
                div { class: "co-field grow",
                    label { class: "co-field-label", "Expiry" }
                    input { class: "co-input", r#type: "text", placeholder: "MM / YY" }
                }
                div { class: "co-field grow",
                    label { class: "co-field-label", "CVC" }
                    input { class: "co-input", r#type: "text", placeholder: "123" }
                }
                div { class: "co-field grow",
                    label { class: "co-field-label", "ZIP / Postal" }
                    input { class: "co-input", r#type: "text", placeholder: "V1Y 0A0" }
                }
            }
            div { class: "co-secure",
                Icon { name: "lock", size: 15, color: "var(--vl-muted)" }
                span { "Payments are secure and encrypted. You won't be charged until confirmed." }
            }
        }
    }
}

#[component]
fn SummaryCard(signed_in: bool) -> Element {
    let google_href = api::google_login_url();
    rsx! {
        div { class: "co-summary",
            div { class: "co-sum-item",
                div {
                    class: "co-sum-img",
                    style: "background-image: url('{IMG_JAYCO}');",
                }
                div { class: "co-sum-c",
                    div { class: "co-sum-t", "Jayco 26' 5th Wheel" }
                    div { class: "co-sum-r",
                        Icon { name: "star", size: 14, color: "var(--vl-accent)" }
                        span { "4.9 · 38 reviews" }
                    }
                }
            }
            div { class: "co-divider" }
            div { class: "co-breakdown",
                PriceLine { label: "$185 × 3 nights", value: "$555", bold: false }
                PriceLine { label: "Delivery & setup", value: "$150", bold: false }
                PriceLine { label: "Bedding and linens", value: "$80", bold: false }
                PriceLine { label: "GST (5%)", value: "CA$39.25", bold: true }
                PriceLine { label: "PST (7%)", value: "CA$54.95", bold: true }
                div { class: "co-line",
                    div { class: "co-line-lc",
                        span { class: "co-line-label", "Damage deposit" }
                        span { class: "co-line-sub", "Refundable · collected before trip" }
                    }
                    span { class: "co-line-value bold", "CA$1,000.00" }
                }
                div { class: "co-divider" }
                div { class: "co-line",
                    span { class: "co-total-label", "Total (CAD)" }
                    span { class: "co-total-value", "CA$879.20" }
                }
            }
            if signed_in {
                Link { class: "co-pay", to: Route::Confirmed {}, "Confirm and pay" }
            } else {
                div { class: "co-auth-choice",
                    h2 { "Sign in to confirm" }
                    p { "Your trip details stay here while you sign in." }
                    a {
                        class: "co-google",
                        href: google_href,
                        onclick: move |_| api::remember_auth_return("/checkout"),
                        span { class: "co-google-mark", "G" }
                        "Continue with Google"
                    }
                    Link { class: "co-email-login", to: Route::Login {}, "Use email and password" }
                }
            }
            div { class: "co-pay5",
                Icon { name: "calendar-clock", size: 14, color: "var(--vl-muted)" }
                span { "Full payment due 5 days before your rental" }
            }
        }
    }
}

#[component]
fn PriceLine(label: &'static str, value: &'static str, bold: bool) -> Element {
    rsx! {
        div { class: "co-line",
            span { class: "co-line-label", "{label}" }
            span {
                class: if bold { "co-line-value bold" } else { "co-line-value" },
                "{value}"
            }
        }
    }
}

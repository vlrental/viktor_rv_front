use crate::{api, components::ReviewForm, pricing, AuthSession, Route};
use dioxus::prelude::*;

const DAMAGE_DEPOSIT_ETRANSFER_EMAIL: &str = "protrailercare@gmail.com";

fn local_booking_moment(value: &str) -> String {
    crate::timezone::format_local_moment(value)
}

fn redirect_to_inline_auth(register: bool) {
    api::request_inline_auth(register, None);
    if let Some(window) = web_sys::window() {
        let _ = window.location().replace(&api::frontend_path("/"));
    }
}

#[component]
pub fn Login() -> Element {
    use_effect(|| redirect_to_inline_auth(false));
    rsx! {}
}

#[component]
pub fn Register() -> Element {
    use_effect(|| redirect_to_inline_auth(true));
    rsx! {}
}

#[component]
pub fn AuthCallback() -> Element {
    rsx! { div { hidden: true, aria_live: "polite", "Completing sign in" } }
}

#[component]
pub fn Account() -> Element {
    let rentals_href = api::frontend_path("/#home-rentals");
    let nav = use_navigator();
    let mut auth_session = use_context::<AuthSession>().0;
    let user = auth_session.read().clone();
    let mut bookings = use_signal(Vec::<api::Booking>::new);
    let mut load_error = use_signal(String::new);
    let mut loaded = use_signal(|| false);
    let mut review_booking = use_signal(|| None::<String>);
    let mut logout_busy = use_signal(|| false);
    let mut bookings_request_version = use_signal(|| 0_u32);
    let user_id = user.as_ref().map(|value| value.user_id.clone());
    use_effect(use_reactive((&user_id,), move |(user_id,)| {
        let request_version = bookings_request_version.peek().wrapping_add(1);
        bookings_request_version.set(request_version);
        let Some(expected_user_id) = user_id else {
            bookings.set(Vec::new());
            load_error.set(String::new());
            review_booking.set(None);
            loaded.set(true);
            return;
        };
        loaded.set(false);
        load_error.set(String::new());
        spawn(async move {
            let result = api::my_bookings().await;
            let same_user = auth_session
                .peek()
                .as_ref()
                .is_some_and(|value| value.user_id == expected_user_id);
            if *bookings_request_version.peek() != request_version || !same_user {
                return;
            }
            match result {
                Ok(values) => bookings.set(values),
                Err(message) => load_error.set(message),
            }
            loaded.set(true);
        });
    }));
    rsx! { section { class: "account-page",
        if let Some(user) = user {
            div { class: "account-head", p { class: "auth-kicker", "Your account" } h1 { "Bookings and account" } p { "{user.email}" } span { class: "account-role", "{user.role}" } }
            if !*loaded.read() {
                div { class: "account-empty", h2 { "Loading your bookings…" } }
            } else if !load_error.read().is_empty() {
                div { class: "account-empty", p { class: "auth-error", role: "alert", "{load_error}" } }
            } else if bookings.read().is_empty() {
                div { class: "account-empty", h2 { "No bookings yet" } p { "Your confirmed rentals will appear here." } a { class: "btn-forest", href: rentals_href, "Browse rentals" } }
            } else {
                div { class: "account-bookings",
                    for booking in bookings.read().iter() {
                        article { class: "co-card", key: "{booking.booking_id}",
                            p { class: "auth-kicker", "{booking.status} · {booking.payment_status}" }
                            h2 { "#{booking.booking_number}" }
                            p { "Delivery/setup: {local_booking_moment(&booking.starts_at)} · Return: {local_booking_moment(&booking.ends_at)}" }
                            p { "Trip price: {booking.currency} {booking.total}" }
                            p { "Booking payment: {booking.currency} {booking.amount_due_now}" }
                            div { class: if matches!(booking.damage_deposit_status.as_str(), "succeeded" | "released" | "captured") { "account-deposit is-paid" } else { "account-deposit" },
                                div { class: "account-deposit-head",
                                    span { "REFUNDABLE DAMAGE DEPOSIT" }
                                    strong { "{pricing::money(pricing::DAMAGE_DEPOSIT)}" }
                                }
                                if matches!(booking.damage_deposit_status.as_str(), "succeeded" | "released" | "captured") {
                                    p { strong { "Paid" } "VL Rental verified your Interac e-Transfer. A confirmation was emailed to you and the administrator." }
                                } else {
                                    p { strong { "Awaiting e-Transfer" } "Not charged through Stripe. Send to " b { "{booking.damage_deposit_transfer_email.as_deref().unwrap_or(DAMAGE_DEPOSIT_ETRANSFER_EMAIL)}" } "." }
                                    if let Some(due_at) = booking.damage_deposit_due_at.as_deref() { small { "Due {local_booking_moment(due_at)} · 48 hours before delivery" } }
                                    button { r#type: "button", onclick: move |_| async move { let _ = document::eval("return navigator.clipboard.writeText('protrailercare@gmail.com');").await; }, "Copy e-Transfer email" }
                                }
                                small { "Delivery is blocked until the trip price and damage deposit are fully paid." }
                            }
                            if booking.can_review {
                                button { class: "account-review-open", r#type: "button", onclick: { let booking_id = booking.booking_id.clone(); move |_| review_booking.set(Some(booking_id.clone())) }, "Leave a verified review" }
                            } else if booking.review_opportunity_used || booking.review_id.is_some() {
                                p { class: "account-review-sent", "★ Review submitted" }
                            }
                            if review_booking.read().as_deref() == Some(booking.booking_id.as_str()) {
                                div { class: "account-review-form",
                                    ReviewForm { booking_id: booking.booking_id.clone(), rental_name: booking.rental_name.clone(), on_cancel: move |_| review_booking.set(None), on_published: { let booking_id = booking.booking_id.clone(); move |review: api::RentalReview| { let mut next = bookings.read().clone(); if let Some(item) = next.iter_mut().find(|item| item.booking_id == booking_id) { item.can_review = false; item.review_opportunity_used = true; item.review_id = Some(review.rental_review_id); } bookings.set(next); review_booking.set(None); } } }
                                }
                            }
                        }
                    }
                }
            }
            button { class: "account-signout", disabled: logout_busy(), onclick: move |_| async move { logout_busy.set(true); api::logout().await; auth_session.set(None); logout_busy.set(false); nav.push(Route::Login {}); }, if logout_busy() { "Signing out…" } else { "Sign out" } }
        } else {
            div { class: "account-empty", h1 { "Sign in required" } Link { class: "btn-forest", to: Route::Login {}, "Sign in" } }
        }
    } }
}

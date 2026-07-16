use crate::{api, components::ReviewForm, pricing, AuthSession, Route};
use dioxus::prelude::*;

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
    let has_user = user.is_some();
    let mut bookings = use_signal(Vec::<api::Booking>::new);
    let mut load_error = use_signal(String::new);
    let mut loaded = use_signal(|| false);
    let mut review_booking = use_signal(|| None::<String>);
    use_effect(move || {
        if !has_user {
            loaded.set(true);
            return;
        }
        spawn(async move {
            match api::my_bookings().await {
                Ok(values) => bookings.set(values),
                Err(message) => load_error.set(message),
            }
            loaded.set(true);
        });
    });
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
                            p { "Refundable damage deposit: {pricing::money(pricing::DAMAGE_DEPOSIT)} · separate · charged 48 hours before delivery" }
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
            button { class: "account-signout", onclick: move |_| async move { api::logout().await; auth_session.set(None); nav.push(Route::Login {}); }, "Sign out" }
        } else {
            div { class: "account-empty", h1 { "Sign in required" } Link { class: "btn-forest", to: Route::Login {}, "Sign in" } }
        }
    } }
}

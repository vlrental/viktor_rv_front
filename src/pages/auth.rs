use crate::{api, pricing, Route};
use dioxus::prelude::*;

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
    let user = api::current_user();
    let has_user = user.is_some();
    let mut bookings = use_signal(Vec::<api::Booking>::new);
    let mut load_error = use_signal(String::new);
    let mut loaded = use_signal(|| false);
    let mut review_booking = use_signal(|| None::<String>);
    let mut review_rating = use_signal(|| 5_i32);
    let mut review_title = use_signal(String::new);
    let mut review_body = use_signal(String::new);
    let mut review_error = use_signal(String::new);
    let mut review_busy = use_signal(|| false);
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
                            p { "Delivery/setup: {booking.starts_at} · Return: {booking.ends_at}" }
                            p { "Trip price: {booking.currency} {booking.total}" }
                            p { "Booking payment: {booking.currency} {booking.amount_due_now}" }
                            p { "Refundable damage deposit: {pricing::money(pricing::DAMAGE_DEPOSIT)} · separate · charged 48 hours before delivery" }
                            if booking.can_review {
                                button { class: "account-review-open", r#type: "button", onclick: { let booking_id = booking.booking_id.clone(); move |_| { review_booking.set(Some(booking_id.clone())); review_rating.set(5); review_title.set(String::new()); review_body.set(String::new()); review_error.set(String::new()); } }, "Leave a verified review" }
                            } else if booking.review_id.is_some() {
                                p { class: "account-review-sent", "★ Review submitted" }
                            }
                            if review_booking.read().as_deref() == Some(booking.booking_id.as_str()) {
                                div { class: "account-review-form",
                                    h3 { "Review {booking.rental_name}" }
                                    p { "Your rating" }
                                    div { class: "account-review-stars", role: "group", aria_label: "Rating out of 5",
                                        for value in 1..=5_i32 {
                                            button { key: "review-star-{value}", class: if value <= *review_rating.read() { "active" } else { "" }, r#type: "button", aria_label: "{value} out of 5 stars", onclick: move |_| review_rating.set(value), "★" }
                                        }
                                    }
                                    input { maxlength: "80", value: "{review_title}", placeholder: "Short title (optional)", oninput: move |event| review_title.set(event.value()) }
                                    textarea { maxlength: "2000", value: "{review_body}", placeholder: "Tell future guests about your experience", oninput: move |event| review_body.set(event.value()) }
                                    if !review_error.read().is_empty() { p { class: "auth-error", role: "alert", "{review_error}" } }
                                    div { class: "account-review-actions",
                                        button { r#type: "button", onclick: move |_| review_booking.set(None), "Cancel" }
                                        button { class: "btn-forest", r#type: "button", disabled: *review_busy.read(), onclick: { let booking_id = booking.booking_id.clone(); move |_| { let booking_id = booking_id.clone(); let title = review_title.read().clone(); let body = review_body.read().clone(); async move { if body.trim().chars().count() < 10 { review_error.set("Write at least 10 characters about your experience.".into()); return; } review_busy.set(true); review_error.set(String::new()); match api::create_rental_review(&booking_id, *review_rating.read(), &title, &body).await { Ok(review) => { let mut next = bookings.read().clone(); if let Some(item) = next.iter_mut().find(|item| item.booking_id == booking_id) { item.can_review = false; item.review_id = Some(review.rental_review_id); } bookings.set(next); review_booking.set(None); }, Err(error) => review_error.set(error.message) } review_busy.set(false); } } }, if *review_busy.read() { "Publishing…" } else { "Publish review" } }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            button { class: "account-signout", onclick: move |_| { api::clear_session(); nav.push(Route::Login {}); }, "Sign out" }
        } else {
            div { class: "account-empty", h1 { "Sign in required" } Link { class: "btn-forest", to: Route::Login {}, "Sign in" } }
        }
    } }
}

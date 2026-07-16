use dioxus::prelude::*;

use crate::api;
use crate::components::Icon;
use crate::data::PHONE;
use crate::{booking_launch_requires_home, BookingLaunchRequest, Route};

const LOGO: Asset = asset!("/assets/img/logo.png");

/// Шапка сайта. На Home — прозрачный overlay поверх hero, на остальных — белая с волосяной линией.
#[component]
pub fn Header() -> Element {
    let route = use_route::<Route>();
    let navigator = use_navigator();
    let mut booking_launch_request = use_context::<BookingLaunchRequest>();
    let inline_auth = api::take_inline_auth_request();
    let initial_account_open = inline_auth.is_some();
    let initial_register = inline_auth.as_ref().is_some_and(|value| value.0);
    let initial_error = inline_auth.and_then(|value| value.1).unwrap_or_default();
    let mut account_open = use_signal(move || initial_account_open);
    let mut mobile_open = use_signal(|| false);
    let mut register = use_signal(move || initial_register);
    let mut email = use_signal(String::new);
    let mut password = use_signal(String::new);
    let mut error = use_signal(move || initial_error);
    let mut busy = use_signal(|| false);
    let mut session_version = use_signal(|| 0_u32);
    let _session_version = *session_version.read();
    let current_user = api::current_user();
    let rentals_href = api::frontend_path("/#home-rentals");
    let auth_return_route = route.clone();
    let overlay = matches!(route, Route::Home {});
    let class = if overlay {
        "site-header overlay"
    } else {
        "site-header"
    };
    let mobile_menu_label = if *mobile_open.read() {
        "Close navigation"
    } else {
        "Open navigation"
    };
    let mobile_menu_icon = if *mobile_open.read() { "x" } else { "menu" };
    let submit = move |_| {
        let email_value = email.read().clone();
        let password_value = password.read().clone();
        let creating_account = *register.read();
        async move {
            busy.set(true);
            error.set(String::new());
            match api::login(&email_value, &password_value, creating_account).await {
                Ok(tokens) => match api::save_session(&tokens) {
                    Ok(()) => {
                        session_version += 1;
                        account_open.set(false);
                        password.set(String::new());
                    }
                    Err(message) => error.set(message),
                },
                Err(_) => error.set("Check your email and password, then try again.".into()),
            }
            busy.set(false);
        }
    };

    rsx! {
        header {
            class: "{class}",
            onkeydown: move |event| if event.key() == Key::Escape {
                if *account_open.peek() {
                    account_open.set(false);
                } else if *mobile_open.peek() {
                    mobile_open.set(false);
                }
            },
            Link { class: "brand", to: Route::Home {},
                img { class: "brand-mark", src: LOGO, alt: "VL Rental" }
                span { class: "brand-word", "VL Rental" }
            }
            nav { class: if *mobile_open.read() { "nav-menu is-open" } else { "nav-menu" },
                a { class: "nav-link", href: rentals_href, onclick: move |_| mobile_open.set(false), "RV Rentals" }
                Link { class: "nav-link", to: Route::CoolerTrailers {}, onclick: move |_| mobile_open.set(false), "Cooler Trailers" }
                Link { class: "nav-link", to: Route::Delivery {}, onclick: move |_| mobile_open.set(false), "Delivery" }
                Link { class: "nav-link", to: Route::RvSales {}, onclick: move |_| mobile_open.set(false), "RV Sales" }
                Link { class: "nav-link nav-menu-contact", to: Route::Contact {}, onclick: move |_| mobile_open.set(false), "Contact" }
            }
            div { class: "nav-right",
                button { class: "nav-burger", r#type: "button", aria_label: mobile_menu_label, aria_expanded: *mobile_open.read(), onclick: move |_| { let next = !*mobile_open.peek(); mobile_open.set(next); account_open.set(false); }, Icon { name: mobile_menu_icon, size: 21, color: "currentColor" } }
                a { class: "nav-phone", href: "tel:+12508785874",
                    Icon { name: "phone", size: 15, color: "var(--vl-accent)" }
                    span { "{PHONE}" }
                }
                Link { class: "nav-link", to: Route::Contact {}, "Contact" }
                div { class: "nav-account-wrap",
                    button {
                        class: if *account_open.read() { "nav-account active" } else { "nav-account" },
                        r#type: "button",
                        aria_label: "Account",
                        aria_expanded: *account_open.read(),
                        onclick: move |_| {
                            let next = !*account_open.peek();
                            account_open.set(next);
                            mobile_open.set(false);
                            error.set(String::new());
                        },
                        Icon { name: "circle-user-round", size: 20, color: "currentColor" }
                    }
                    if *account_open.read() {
                        button { class: "nav-account-dismiss", r#type: "button", aria_label: "Close account panel", onclick: move |_| account_open.set(false) }
                        aside { class: "nav-account-panel", onclick: move |event| event.stop_propagation(),
                            if let Some(user) = current_user.clone() {
                                div { class: "nav-account-panel-head",
                                    div { class: "nav-account-avatar", Icon { name: "circle-user-round", size: 24, color: "var(--vl-white)" } }
                                    div {
                                        div { class: "nav-account-kicker", "SIGNED IN" }
                                        strong { "{user.email}" }
                                    }
                                }
                                p { class: "nav-account-copy", "Your bookings and trip details are ready whenever you need them." }
                                Link { class: "nav-account-primary", to: Route::Account {}, onclick: move |_| account_open.set(false),
                                    span { "View my bookings" }
                                    Icon { name: "arrow-right", size: 16, color: "var(--vl-white)" }
                                }
                                if user.role == "admin" {
                                    Link { class: "nav-account-admin", to: Route::Admin {}, onclick: move |_| account_open.set(false),
                                        Icon { name: "layout-dashboard", size: 16, color: "var(--vl-forest)" }
                                        span { "Open admin dashboard" }
                                    }
                                }
                                button { class: "nav-account-signout", r#type: "button", onclick: move |_| async move {
                                    api::logout().await;
                                    session_version += 1;
                                    account_open.set(false);
                                }, "Sign out" }
                            } else {
                                div { class: "nav-account-panel-head",
                                    div { class: "nav-account-avatar", Icon { name: "key-round", size: 22, color: "var(--vl-white)" } }
                                    div {
                                        div { class: "nav-account-kicker", "WELCOME TO VL RENTAL" }
                                        strong { if *register.read() { "Create your account" } else { "Sign in without leaving" } }
                                    }
                                }
                                p { class: "nav-account-copy", "Keep your dates and continue booking right where you are." }
                                a { class: "nav-account-google", href: api::google_login_url(), onclick: move |_| api::remember_auth_return(&auth_return_route.to_string()),
                                    span { class: "auth-google-mark", "G" }
                                    "Continue with Google"
                                }
                                div { class: "nav-account-or", span { "or" } }
                                label { r#for: "header-auth-email", "Email" }
                                input { id: "header-auth-email", r#type: "email", autocomplete: "email", value: "{email}", oninput: move |event| email.set(event.value()), placeholder: "you@example.com" }
                                label { r#for: "header-auth-password", "Password" }
                                input { id: "header-auth-password", r#type: "password", autocomplete: if *register.read() { "new-password" } else { "current-password" }, value: "{password}", oninput: move |event| password.set(event.value()), placeholder: "At least 10 characters" }
                                if !error.read().is_empty() { p { class: "nav-account-error", role: "alert", "{error}" } }
                                button { class: "nav-account-primary", r#type: "button", disabled: *busy.read(), onclick: submit,
                                    if *busy.read() { "Please wait…" } else if *register.read() { "Create account" } else { "Sign in" }
                                }
                                button { class: "nav-account-switch", r#type: "button", onclick: move |_| {
                                    let next = !*register.peek();
                                    register.set(next);
                                    error.set(String::new());
                                },
                                    if *register.read() { "Already registered? Sign in" } else { "New here? Create an account" }
                                }
                            }
                        }
                    }
                }
                if current_user.as_ref().is_some_and(|user| user.role == "admin") {
                    Link { class: "nav-admin-cta", to: Route::Admin {},
                        Icon { name: "shield-check", size: 15, color: "currentColor" }
                        span { "Admin dashboard" }
                    }
                }
                button {
                    class: "nav-cta",
                    r#type: "button",
                    aria_haspopup: "dialog",
                    onclick: move |_| {
                        mobile_open.set(false);
                        account_open.set(false);
                        booking_launch_request.0.set(true);
                        if booking_launch_requires_home(&route) {
                            navigator.push(Route::Home {});
                        }
                    },
                    "Book now"
                }
            }
        }
    }
}

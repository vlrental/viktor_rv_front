use dioxus::prelude::*;

use crate::api;
use crate::components::Icon;
use crate::data::PHONE;
use crate::Route;

const LOGO: Asset = asset!("/assets/img/logo.png");

/// Шапка сайта. На Home — прозрачный overlay поверх hero, на остальных — белая с волосяной линией.
#[component]
pub fn Header() -> Element {
    let route = use_route::<Route>();
    let mut account_open = use_signal(|| false);
    let mut register = use_signal(|| false);
    let mut email = use_signal(String::new);
    let mut password = use_signal(String::new);
    let mut error = use_signal(String::new);
    let mut busy = use_signal(|| false);
    let mut session_version = use_signal(|| 0_u32);
    let _session_version = *session_version.read();
    let current_user = api::current_user();
    let overlay = matches!(route, Route::Home {});
    let class = if overlay {
        "site-header overlay"
    } else {
        "site-header"
    };
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
        header { class: "{class}",
            Link { class: "brand", to: Route::Home {},
                img { class: "brand-mark", src: LOGO, alt: "VL Rental" }
                span { class: "brand-word", "VL Rental" }
            }
            nav { class: "nav-menu",
                Link { class: "nav-link", to: Route::Catalog {}, "RV Rentals" }
                Link { class: "nav-link", to: Route::CoolerTrailers {}, "Cooler Trailers" }
                Link { class: "nav-link", to: Route::Delivery {}, "Delivery" }
                Link { class: "nav-link", to: Route::RvSales {}, "RV Sales" }
            }
            div { class: "nav-right",
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
                                button { class: "nav-account-signout", r#type: "button", onclick: move |_| {
                                    api::clear_session();
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
                                a { class: "nav-account-google", href: api::google_login_url(), onclick: move |_| api::remember_auth_return(&route.to_string()),
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
                Link { class: "nav-cta", to: Route::Catalog {}, "Book now" }
            }
        }
    }
}

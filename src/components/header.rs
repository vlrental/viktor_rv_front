use dioxus::prelude::*;

use crate::api;
use crate::components::Icon;
use crate::data::PHONE;
use crate::{
    booking_launch_requires_home, push_notifications, AuthSession, BookingLaunchRequest, Route,
};

const LOGO: Asset = asset!(
    "/assets/img/logo.png",
    AssetOptions::image().with_size(ImageSize::Manual {
        width: 120,
        height: 120,
    })
);

fn focus_mobile_menu_toggle() {
    document::eval(
        r#"
            requestAnimationFrame(() => {
                document.querySelector('.nav-burger')?.focus({ preventScroll: true });
            });
        "#,
    );
}

/// Шапка сайта. На Home — прозрачный overlay поверх hero, на остальных — белая с волосяной линией.
#[component]
pub fn Header() -> Element {
    let route = use_route::<Route>();
    let navigator = use_navigator();
    let mut booking_launch_request = use_context::<BookingLaunchRequest>();
    let mut auth_session = use_context::<AuthSession>().0;
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
    let mut push_state = use_signal(|| "loading".to_string());
    let mut push_busy = use_signal(|| false);
    let mut push_error = use_signal(String::new);
    let mut push_status_version = use_signal(|| 0_u32);
    let current_user = auth_session.read().clone();
    let signed_in = current_user.is_some();
    let rentals_href = api::frontend_path("/#home-rentals");
    let auth_return_route = api::current_auth_return_path().unwrap_or_else(|| route.to_string());
    let facebook_auth_return_route = auth_return_route.clone();
    let mobile_booking_route = route.clone();
    let mobile_navigator = navigator;
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
    let push_icon: &'static str = if push_state.read().as_str() == "enabled" {
        "bell-ring"
    } else {
        "bell"
    };
    use_effect(use_reactive((&signed_in,), move |(signed_in,)| {
        let version = push_status_version.peek().wrapping_add(1);
        push_status_version.set(version);
        if !signed_in {
            push_state.set("available".into());
            return;
        }
        spawn(async move {
            let status = push_notifications::status().await;
            if *push_status_version.peek() == version && auth_session.peek().is_some() {
                push_state.set(status);
            }
        });
    }));
    use_effect(move || {
        if *mobile_open.read() {
            document::eval(
                r#"
                    if (window.__vlMobileMenuKeyHandler) {
                        document.removeEventListener('keydown', window.__vlMobileMenuKeyHandler);
                    }
                    window.__vlMobileMenuKeyHandler = (event) => {
                        if (event.key !== 'Escape') return;
                        const toggle = document.querySelector('.nav-burger.is-open');
                        toggle?.focus({ preventScroll: true });
                        toggle?.click();
                    };
                    document.addEventListener('keydown', window.__vlMobileMenuKeyHandler);
                    const focusFirstMenuLink = () => {
                        document.querySelector('#mobile-navigation .nav-link')?.focus({ preventScroll: true });
                    };
                    requestAnimationFrame(() => requestAnimationFrame(() => {
                        focusFirstMenuLink();
                    }));
                    window.clearTimeout(window.__vlMobileMenuFocusTimer);
                    window.__vlMobileMenuFocusTimer = window.setTimeout(() => {
                        const toggle = document.querySelector('.nav-burger.is-open');
                        if (toggle && document.activeElement === toggle) focusFirstMenuLink();
                    }, 420);
                "#,
            );
        } else {
            document::eval(
                r#"
                    if (window.__vlMobileMenuKeyHandler) {
                        document.removeEventListener('keydown', window.__vlMobileMenuKeyHandler);
                        window.__vlMobileMenuKeyHandler = null;
                    }
                    window.clearTimeout(window.__vlMobileMenuFocusTimer);
                    window.__vlMobileMenuFocusTimer = null;
                "#,
            );
        }
    });
    use_drop(|| {
        document::eval(
            r#"
                if (window.__vlMobileMenuKeyHandler) {
                    document.removeEventListener('keydown', window.__vlMobileMenuKeyHandler);
                    window.__vlMobileMenuKeyHandler = null;
                }
                window.clearTimeout(window.__vlMobileMenuFocusTimer);
                window.__vlMobileMenuFocusTimer = null;
            "#,
        );
    });
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
                        auth_session.set(Some(tokens.user));
                        account_open.set(true);
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
                if *account_open.peek() && !*busy.peek() {
                    account_open.set(false);
                } else if *mobile_open.peek() {
                    mobile_open.set(false);
                    focus_mobile_menu_toggle();
                }
            },
            Link { class: "brand", to: Route::Home {},
                img { class: "brand-mark", src: LOGO, alt: "VL Rental" }
                span { class: "brand-word", "VL Rental" }
            }
            button {
                class: if *mobile_open.read() { "mobile-menu-backdrop is-open" } else { "mobile-menu-backdrop" },
                r#type: "button",
                aria_label: "Close navigation",
                tabindex: "-1",
                onwheel: move |event| event.prevent_default(),
                onclick: move |_| {
                    mobile_open.set(false);
                    focus_mobile_menu_toggle();
                },
            }
            nav {
                id: "mobile-navigation",
                class: if *mobile_open.read() { "nav-menu is-open" } else { "nav-menu" },
                aria_label: "Primary navigation",
                div { class: "mobile-menu-head",
                    div {
                        span { "EXPLORE" }
                        strong { "Plan your Okanagan stay" }
                    }
                    span { class: "mobile-menu-status", "MENU" }
                }
                a { class: "nav-link", href: rentals_href, aria_label: "RV Rentals", onclick: move |_| mobile_open.set(false), "RV Rentals" }
                Link { class: "nav-link", to: Route::ParksInOurRange {}, aria_label: "Parks in Our Range", onclick: move |_| mobile_open.set(false), "Parks in Our Range" }
                Link { class: "nav-link", to: Route::Delivery {}, aria_label: "Delivery", onclick: move |_| mobile_open.set(false), "Delivery" }
                Link { class: "nav-link", to: Route::RvSales {}, aria_label: "RV Sales", onclick: move |_| mobile_open.set(false), "RV Sales" }
                Link { class: "nav-link nav-menu-contact", to: Route::Contact {}, aria_label: "Contact", onclick: move |_| mobile_open.set(false), "Contact" }
                div { class: "mobile-menu-footer",
                    a { class: "mobile-menu-phone", href: "tel:+12508785874", aria_label: "Call +1 (250) 878 5874",
                        Icon { name: "phone", size: 16, color: "currentColor" }
                        span { "Call {PHONE}" }
                    }
                    button {
                        class: "mobile-menu-book",
                        r#type: "button",
                        aria_label: "Book now",
                        aria_haspopup: "dialog",
                        disabled: *busy.read(),
                        onclick: move |_| {
                            mobile_open.set(false);
                            account_open.set(false);
                            booking_launch_request.0.set(true);
                            if booking_launch_requires_home(&mobile_booking_route) {
                                mobile_navigator.push(Route::Home {});
                            }
                        },
                        span { "Book now" }
                        Icon { name: "arrow-up-right", size: 17, color: "currentColor" }
                    }
                }
            }
            div { class: "nav-right",
                button {
                    class: if *mobile_open.read() { "nav-burger is-open" } else { "nav-burger" },
                    r#type: "button",
                    aria_label: mobile_menu_label,
                    aria_controls: "mobile-navigation",
                    aria_expanded: *mobile_open.read(),
                    onclick: move |_| {
                        let next = !*mobile_open.peek();
                        mobile_open.set(next);
                        account_open.set(false);
                    },
                    span { class: "nav-burger-line" }
                    span { class: "nav-burger-line" }
                    span { class: "nav-burger-line" }
                }
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
                        disabled: *busy.read(),
                        onclick: move |_| {
                            let next = !*account_open.peek();
                            account_open.set(next);
                            mobile_open.set(false);
                            error.set(String::new());
                        },
                        Icon { name: "circle-user-round", size: 20, color: "currentColor" }
                    }
                    if *account_open.read() {
                        button { class: "nav-account-dismiss", r#type: "button", disabled: *busy.read(), aria_label: "Close account panel", onclick: move |_| if !*busy.peek() { account_open.set(false); } }
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
                                button {
                                    class: "nav-account-admin",
                                    r#type: "button",
                                    disabled: *busy.read() || *push_busy.read() || matches!(push_state.read().as_str(), "loading" | "denied" | "unsupported"),
                                    onclick: move |_| async move {
                                        push_busy.set(true);
                                        push_error.set(String::new());
                                        let enabled = push_state.read().as_str() == "enabled";
                                        let result = if enabled {
                                            push_notifications::disable().await
                                        } else {
                                            push_notifications::enable().await
                                        };
                                        match result {
                                            Ok(()) => push_state.set(if enabled { "available".into() } else { "enabled".into() }),
                                            Err(message) => push_error.set(message),
                                        }
                                        push_busy.set(false);
                                    },
                                    Icon {
                                        name: push_icon,
                                        size: 16,
                                        color: "var(--vl-forest)"
                                    }
                                    span {
                                        if *push_busy.read() {
                                            "Please wait…"
                                        } else {
                                            match push_state.read().as_str() {
                                                "enabled" => "Notifications on",
                                                "denied" => "Notifications blocked in browser",
                                                "unsupported" => "Notifications unavailable",
                                                "loading" => "Checking notifications…",
                                                _ => "Turn on notifications",
                                            }
                                        }
                                    }
                                }
                                if !push_error.read().is_empty() {
                                    p { class: "nav-account-error", role: "alert", "{push_error}" }
                                }
                                button { class: "nav-account-signout", r#type: "button", disabled: *busy.read() || *push_busy.read(), onclick: move |_| async move {
                                    busy.set(true);
                                    api::logout().await;
                                    auth_session.set(None);
                                    busy.set(false);
                                    account_open.set(false);
                                }, if *busy.read() { "Signing out…" } else { "Sign out" } }
                            } else {
                                div { class: "nav-account-panel-head",
                                    div { class: "nav-account-avatar", Icon { name: "key-round", size: 22, color: "var(--vl-white)" } }
                                    div {
                                        div { class: "nav-account-kicker", "WELCOME TO VL RENTAL" }
                                        strong { if *register.read() { "Create your account" } else { "Sign in without leaving" } }
                                    }
                                }
                                p { class: "nav-account-copy", "Keep your dates and continue booking right where you are." }
                                a { class: "nav-account-google", href: api::google_login_url(), aria_disabled: *busy.read(), onclick: move |event| { if *busy.peek() { event.prevent_default(); } else { api::remember_auth_return(&auth_return_route); } },
                                    span { class: "auth-google-mark", "G" }
                                    "Continue with Google"
                                }
                                if api::FACEBOOK_AUTH_ENABLED {
                                    a { class: "nav-account-facebook", href: api::facebook_login_url(), aria_disabled: *busy.read(), onclick: move |event| { if *busy.peek() { event.prevent_default(); } else { api::remember_auth_return(&facebook_auth_return_route); } },
                                        span { class: "auth-facebook-mark", "f" }
                                        "Continue with Facebook"
                                    }
                                }
                                div { class: "nav-account-or", span { "or" } }
                                label { r#for: "header-auth-email", "Email" }
                                input { id: "header-auth-email", r#type: "email", autocomplete: "email", value: "{email}", disabled: *busy.read(), oninput: move |event| email.set(event.value()), placeholder: "you@example.com" }
                                label { r#for: "header-auth-password", "Password" }
                                input { id: "header-auth-password", r#type: "password", autocomplete: if *register.read() { "new-password" } else { "current-password" }, value: "{password}", disabled: *busy.read(), oninput: move |event| password.set(event.value()), placeholder: "At least 10 characters" }
                                if !error.read().is_empty() { p { class: "nav-account-error", role: "alert", "{error}" } }
                                button { class: "nav-account-primary", r#type: "button", disabled: *busy.read(), onclick: submit,
                                    if *busy.read() { "Please wait…" } else if *register.read() { "Create account" } else { "Sign in" }
                                }
                                button { class: "nav-account-switch", r#type: "button", disabled: *busy.read(), onclick: move |_| {
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
                    disabled: *busy.read(),
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

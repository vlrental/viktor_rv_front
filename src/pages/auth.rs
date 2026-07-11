use crate::{api, Route};
use dioxus::prelude::*;

#[component]
fn AuthForm(register: bool) -> Element {
    let mut email = use_signal(String::new);
    let mut password = use_signal(String::new);
    let mut error = use_signal(String::new);
    let mut busy = use_signal(|| false);
    let nav = use_navigator();
    let google_href = api::google_login_url();
    let submit = move |_| {
        let email_value = email.read().clone();
        let password_value = password.read().clone();
        async move {
            busy.set(true);
            error.set(String::new());
            match api::login(&email_value, &password_value, register).await {
                Ok(tokens) => match api::save_session(&tokens) {
                    Ok(()) if tokens.user.role == "admin" => {
                        nav.push(Route::Account {});
                    }
                    Ok(()) => {
                        nav.push(Route::Checkout {});
                    }
                    Err(message) => error.set(message),
                },
                Err(_) => error.set("Check your email and password, then try again.".into()),
            }
            busy.set(false);
        }
    };

    rsx! {
        section { class: "auth-page",
            div { class: "auth-visual",
                div { class: "auth-visual-copy",
                    span { class: "auth-kicker", "VL Rental account" }
                    h1 { if register { "Book with confidence" } else { "Welcome back" } }
                    p { "Keep your booking details, dates, and rental history in one place." }
                }
            }
            div { class: "auth-panel",
                div { class: "auth-form",
                    h2 { if register { "Create your account" } else { "Sign in" } }
                    p { class: "auth-intro", if register { "Use Google or create a password." } else { "Continue to your booking or dashboard." } }
                    a { class: "auth-google", href: google_href,
                        span { class: "auth-google-mark", "G" }
                        "Continue with Google"
                    }
                    div { class: "auth-divider", span { "or" } }
                    label { r#for: "auth-email", "Email" }
                    input { id: "auth-email", r#type: "email", autocomplete: "email", value: "{email}", oninput: move |e| email.set(e.value()), placeholder: "you@example.com" }
                    label { r#for: "auth-password", "Password" }
                    input { id: "auth-password", r#type: "password", autocomplete: if register { "new-password" } else { "current-password" }, value: "{password}", oninput: move |e| password.set(e.value()), placeholder: "At least 10 characters" }
                    if !error.read().is_empty() { p { class: "auth-error", role: "alert", "{error}" } }
                    button { class: "auth-submit", disabled: *busy.read(), onclick: submit,
                        if *busy.read() { "Please wait…" } else if register { "Create account" } else { "Sign in" }
                    }
                    p { class: "auth-switch",
                        if register { "Already have an account? " } else { "New to VL Rental? " }
                        Link { to: if register { Route::Login {} } else { Route::Register {} }, if register { "Sign in" } else { "Create account" } }
                    }
                }
            }
        }
    }
}

#[component]
pub fn Login() -> Element {
    rsx! { AuthForm { register: false } }
}
#[component]
pub fn Register() -> Element {
    rsx! { AuthForm { register: true } }
}

#[component]
pub fn AuthCallback() -> Element {
    let nav = use_navigator();
    use_effect(move || {
        if let Some(hash) = web_sys::window().and_then(|w| w.location().hash().ok()) {
            let values = hash
                .trim_start_matches('#')
                .split('&')
                .filter_map(|part| part.split_once('='))
                .collect::<std::collections::HashMap<_, _>>();
            if let (Some(access), Some(refresh), Some(user_id), Some(email), Some(role)) = (
                values.get("access_token"),
                values.get("refresh_token"),
                values.get("user_id"),
                values.get("email"),
                values.get("role"),
            ) {
                let tokens = api::AuthTokens {
                    access_token: urlencoding::decode(access).unwrap_or_default().into_owned(),
                    refresh_token: urlencoding::decode(refresh)
                        .unwrap_or_default()
                        .into_owned(),
                    user: api::AuthUser {
                        user_id: urlencoding::decode(user_id)
                            .unwrap_or_default()
                            .into_owned(),
                        email: urlencoding::decode(email).unwrap_or_default().into_owned(),
                        role: urlencoding::decode(role).unwrap_or_default().into_owned(),
                    },
                };
                let _ = api::save_session(&tokens);
                if api::take_auth_return().as_deref() == Some("/checkout") {
                    nav.replace(Route::Checkout {});
                } else {
                    nav.replace(Route::Account {});
                }
            }
        }
    });
    rsx! { div { class: "auth-callback", "Finishing Google sign in…" } }
}

#[component]
pub fn Account() -> Element {
    let nav = use_navigator();
    let user = api::current_user();
    rsx! { section { class: "account-page",
        if let Some(user) = user {
            div { class: "account-head", p { class: "auth-kicker", "Your account" } h1 { "Bookings and account" } p { "{user.email}" } span { class: "account-role", "{user.role}" } }
            div { class: "account-empty", h2 { "No bookings yet" } p { "Your confirmed rentals will appear here." } Link { class: "btn-forest", to: Route::Catalog {}, "Browse rentals" } }
            button { class: "account-signout", onclick: move |_| { api::clear_session(); nav.push(Route::Login {}); }, "Sign out" }
        } else {
            div { class: "account-empty", h1 { "Sign in required" } Link { class: "btn-forest", to: Route::Login {}, "Sign in" } }
        }
    } }
}

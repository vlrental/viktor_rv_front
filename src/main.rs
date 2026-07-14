mod api;
mod components;
mod data;
mod pages;
mod pricing;

use components::{Footer, Header};
use pages::*;

use dioxus::prelude::*;

const MAIN_CSS: Asset = asset!("/assets/main.css");

const FONTS_URL: &str = "https://fonts.googleapis.com/css2?family=Newsreader:ital,opsz,wght@0,6..72,400..700;1,6..72,400..600&family=Inter:wght@400;500;600;700&family=Geist+Mono:wght@400;500;600&display=swap";

fn main() {
    //
    dioxus::launch(App);
}

#[derive(Clone, Routable, PartialEq)]
#[rustfmt::skip]
pub enum Route {
    #[layout(SiteShell)]
        #[route("/")]
        Home {},
        #[route("/catalog")]
        Catalog {},
        #[route("/rv/:slug")]
        RvDetail { slug: String },
        #[route("/checkout")]
        Checkout {},
        #[route("/confirmed")]
        Confirmed {},
        #[route("/contact")]
        Contact {},
        #[route("/about")]
        About {},
        #[route("/attractions")]
        Attractions {},
        #[route("/restaurants")]
        Restaurants {},
        #[route("/cooler-trailers")]
        CoolerTrailers {},
        #[route("/delivery")]
        Delivery {},
        #[route("/rv-sales")]
        RvSales {},
        #[route("/terms")]
        Terms {},
        #[route("/login")]
        Login {},
        #[route("/register")]
        Register {},
        #[route("/auth/callback")]
        AuthCallback {},
        #[route("/account")]
        Account {},
        #[route("/admin")]
        Admin {},
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Link { rel: "preconnect", href: "https://fonts.googleapis.com" }
        document::Link {
            rel: "preconnect",
            href: "https://fonts.gstatic.com",
            crossorigin: "anonymous",
        }
        document::Link { rel: "stylesheet", href: FONTS_URL }
        document::Link {
            rel: "stylesheet",
            href: "https://unpkg.com/lucide-static@latest/font/lucide.css",
        }
        Router::<Route> {}
    }
}

#[component]
fn SiteShell() -> Element {
    rsx! {
        AuthSessionBridge {}
        div { class: "site-shell",
            Header {}
            main { class: "site-main", Outlet::<Route> {} }
            Footer {}
        }
    }
}

#[component]
fn AuthSessionBridge() -> Element {
    let compatibility_callback = matches!(use_route::<Route>(), Route::AuthCallback {});
    use_effect(move || {
        let result = api::finish_google_sign_in();
        match result {
            Ok(Some(path)) => {
                if let Some(window) = web_sys::window() {
                    let _ = window.location().replace(&api::frontend_path(&path));
                }
            }
            Ok(None) if compatibility_callback => {
                let path = api::take_auth_return().unwrap_or_else(|| "/".to_string());
                if api::take_google_auth_pending() {
                    api::request_inline_auth(
                        false,
                        Some("Google sign in was not completed. Please try again."),
                    );
                }
                if let Some(window) = web_sys::window() {
                    let _ = window.location().replace(&api::frontend_path(&path));
                }
            }
            Err(message) => {
                let _ = api::take_google_auth_pending();
                let path = api::take_auth_return().unwrap_or_else(|| "/".to_string());
                api::request_inline_auth(false, Some(&message));
                if let Some(window) = web_sys::window() {
                    let _ = window.location().replace(&api::frontend_path(&path));
                }
            }
            Ok(None) => {}
        }
    });
    rsx! {}
}

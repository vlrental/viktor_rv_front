mod components;
mod data;
mod pages;

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
        #[route("/rv/:id")]
        RvDetail { id: u32 },
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
        div { class: "site-shell",
            Header {}
            main { class: "site-main", Outlet::<Route> {} }
            Footer {}
        }
    }
}

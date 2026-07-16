mod api;
mod components;
mod data;
mod pages;
mod pricing;
mod timezone;

use components::{Footer, Header};
use pages::*;

use dioxus::prelude::*;

const MAIN_CSS: Asset = asset!(
    "/assets/main.css",
    AssetOptions::css().with_static_head(true)
);
// Route styles must be available before client-side navigation. Loading these
// only from inside each page causes a visible unstyled frame on the first visit
// to that route; subsequent visits merely hide the issue behind the cache.
const ABOUT_CSS: Asset = asset!(
    "/assets/css/about.css",
    AssetOptions::css().with_static_head(true)
);
const ATTRACTIONS_CSS: Asset = asset!(
    "/assets/css/attractions.css",
    AssetOptions::css().with_static_head(true)
);
const CHECKOUT_CSS: Asset = asset!(
    "/assets/css/checkout.css",
    AssetOptions::css().with_static_head(true)
);
const CONFIRMED_CSS: Asset = asset!(
    "/assets/css/confirmed.css",
    AssetOptions::css().with_static_head(true)
);
const CONTACT_CSS: Asset = asset!(
    "/assets/css/contact.css",
    AssetOptions::css().with_static_head(true)
);
const COOLER_TRAILERS_CSS: Asset = asset!(
    "/assets/css/cooler_trailers.css",
    AssetOptions::css().with_static_head(true)
);
const DELIVERY_CSS: Asset = asset!(
    "/assets/css/delivery.css",
    AssetOptions::css().with_static_head(true)
);
const RESTAURANTS_CSS: Asset = asset!(
    "/assets/css/restaurants.css",
    AssetOptions::css().with_static_head(true)
);
const RV_DETAIL_CSS: Asset = asset!(
    "/assets/css/rv_detail.css",
    AssetOptions::css().with_static_head(true)
);
const RV_SALES_CSS: Asset = asset!(
    "/assets/css/rv_sales.css",
    AssetOptions::css().with_static_head(true)
);
const TERMS_CSS: Asset = asset!(
    "/assets/css/terms.css",
    AssetOptions::css().with_static_head(true)
);
const PARALLAX_JS: Asset = asset!("/assets/parallax.js");

fn main() {
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

#[derive(Clone, Copy)]
pub struct BookingLaunchRequest(pub Signal<bool>);

#[derive(Clone, Copy)]
pub struct AuthSession(pub Signal<Option<api::AuthUser>>);

pub fn booking_launch_requires_home(route: &Route) -> bool {
    !matches!(route, Route::Home {})
}

const SITE_URL: &str = match option_env!("VL_FRONTEND_BASE_URL") {
    Some(value) => value,
    None => "https://gaponovalexey.github.io/viktor_rv_front",
};

#[derive(Clone, PartialEq)]
struct SeoMetadata {
    title: String,
    description: String,
    canonical: String,
    robots: &'static str,
}

impl SeoMetadata {
    fn indexed(title: impl Into<String>, description: impl Into<String>, path: &str) -> Self {
        Self {
            title: title.into(),
            description: description.into(),
            canonical: format!("{SITE_URL}{path}"),
            robots: "index,follow,max-image-preview:large,max-snippet:-1,max-video-preview:-1",
        }
    }

    fn private(title: impl Into<String>, description: impl Into<String>, path: &str) -> Self {
        Self {
            title: title.into(),
            description: description.into(),
            canonical: format!("{SITE_URL}{path}"),
            robots: "noindex,nofollow",
        }
    }
}

fn seo_metadata(route: &Route) -> SeoMetadata {
    match route {
        Route::Home {} => SeoMetadata::indexed(
            "RV Rentals in Kelowna & Okanagan | VL Rental",
            "Rent clean, fully equipped RVs in Kelowna and the Okanagan. Delivery and setup are available across approved destinations. Browse trailers and book online.",
            "/",
        ),
        Route::Catalog {} => {
            let mut metadata = SeoMetadata::indexed(
                "RV Rentals in Kelowna & Okanagan | VL Rental",
                "Browse VL Rental travel trailers and fifth wheels available for delivery throughout the Okanagan.",
                "/",
            );
            metadata.robots = "noindex,follow";
            metadata
        }
        Route::RvDetail { slug } => data::rv_listings()
            .into_iter()
            .find(|listing| listing.slug == slug)
            .map(|listing| {
                SeoMetadata::indexed(
                    format!("{} Rental in Kelowna | VL Rental", listing.title),
                    format!(
                        "Rent the {} from VL Rental with delivery and setup available in Kelowna and approved Okanagan destinations. View details and request your dates.",
                        listing.title
                    ),
                    &format!("/rv/{}", listing.slug),
                )
            })
            .unwrap_or_else(|| {
                SeoMetadata::private(
                    "RV Not Found | VL Rental",
                    "The requested RV listing could not be found.",
                    "/rv",
                )
            }),
        Route::Contact {} => SeoMetadata::indexed(
            "Contact VL Rental | Kelowna RV Rentals",
            "Contact VL Rental for RV availability, delivery questions, and booking help in Kelowna and the Okanagan.",
            "/contact",
        ),
        Route::About {} => SeoMetadata::indexed(
            "About VL Rental | Okanagan RV Rentals",
            "Learn about VL Rental, a Kelowna-based RV rental service offering delivered and set-up travel trailers across approved Okanagan destinations.",
            "/about",
        ),
        Route::Attractions {} => SeoMetadata::indexed(
            "Okanagan Attractions for Your RV Trip | VL Rental",
            "Plan your Okanagan RV stay with ideas for family attractions, outdoor activities, and places to explore near Kelowna.",
            "/attractions",
        ),
        Route::Restaurants {} => SeoMetadata::indexed(
            "Okanagan Restaurants Near Your RV Stay | VL Rental",
            "Discover restaurants and local food options to enjoy during your RV stay in Kelowna and the Okanagan.",
            "/restaurants",
        ),
        Route::CoolerTrailers {} => SeoMetadata::indexed(
            "Cooler Trailer Rentals in Kelowna | VL Rental",
            "Explore cooler trailer rental options from VL Rental for events and trips in Kelowna and the Okanagan.",
            "/cooler-trailers",
        ),
        Route::Delivery {} => SeoMetadata::indexed(
            "RV Delivery & Setup in the Okanagan | VL Rental",
            "See how VL Rental delivers and sets up RVs across approved destinations within 150 km of the Kelowna base.",
            "/delivery",
        ),
        Route::RvSales {} => SeoMetadata::indexed(
            "RVs for Sale in Kelowna | VL Rental",
            "View RV sales information from VL Rental in Kelowna, British Columbia.",
            "/rv-sales",
        ),
        Route::Terms {} => SeoMetadata::indexed(
            "Rental Terms | VL Rental",
            "Read the terms and conditions for VL Rental RV bookings, delivery, payments, cancellations, and customer responsibilities.",
            "/terms",
        ),
        Route::Checkout {} => SeoMetadata::private(
            "Checkout | VL Rental",
            "Complete your VL Rental booking.",
            "/checkout",
        ),
        Route::Confirmed {} => SeoMetadata::private(
            "Booking Confirmation | VL Rental",
            "VL Rental booking confirmation.",
            "/confirmed",
        ),
        Route::Login {} => SeoMetadata::private(
            "Sign In | VL Rental",
            "Sign in to your VL Rental account.",
            "/login",
        ),
        Route::Register {} => SeoMetadata::private(
            "Create Account | VL Rental",
            "Create a VL Rental account.",
            "/register",
        ),
        Route::AuthCallback {} => SeoMetadata::private(
            "Signing In | VL Rental",
            "Completing secure sign in.",
            "/auth/callback",
        ),
        Route::Account {} => SeoMetadata::private(
            "My Account | VL Rental",
            "Manage your VL Rental account and bookings.",
            "/account",
        ),
        Route::Admin {} => SeoMetadata::private(
            "Administration | VL Rental",
            "VL Rental administration.",
            "/admin",
        ),
    }
}

#[component]
fn App() -> Element {
    // Keep the statically injected route styles reachable as a single bundle
    // dependency even though page components no longer create dynamic links.
    let _route_styles = [
        MAIN_CSS,
        ABOUT_CSS,
        ATTRACTIONS_CSS,
        CHECKOUT_CSS,
        CONFIRMED_CSS,
        CONTACT_CSS,
        COOLER_TRAILERS_CSS,
        DELIVERY_CSS,
        RESTAURANTS_CSS,
        RV_DETAIL_CSS,
        RV_SALES_CSS,
        TERMS_CSS,
    ];
    rsx! {
        document::Script { src: PARALLAX_JS }
        Router::<Route> {}
    }
}

#[component]
fn SiteShell() -> Element {
    let booking_launch_request = use_signal(|| false);
    let auth_session = use_signal(api::current_user);
    use_context_provider(|| BookingLaunchRequest(booking_launch_request));
    use_context_provider(|| AuthSession(auth_session));

    rsx! {
        SeoHead {}
        AuthSessionBridge {}
        div { class: "site-shell",
            Header {}
            main { class: "site-main", Outlet::<Route> {} }
            Footer {}
        }
    }
}

#[component]
fn SeoHead() -> Element {
    let route = use_route::<Route>();
    let metadata = seo_metadata(&route);

    use_effect(use_reactive((&metadata,), move |(metadata,)| {
        let Some(document) = web_sys::window().and_then(|window| window.document()) else {
            return;
        };

        document.set_title(&metadata.title);
        if let Ok(Some(root)) = document.query_selector("html") {
            let _ = root.set_attribute("lang", "en-CA");
        }

        let upsert_meta = |selector: &str, attribute: &str, key: &str, content: &str| {
            let element = document
                .query_selector(selector)
                .ok()
                .flatten()
                .or_else(|| {
                    let element = document.create_element("meta").ok()?;
                    element.set_attribute(attribute, key).ok()?;
                    let head = document.query_selector("head").ok().flatten()?;
                    head.append_child(&element).ok()?;
                    Some(element)
                });

            if let Some(element) = element {
                let _ = element.set_attribute("content", content);
            }
        };

        upsert_meta(
            "meta[name='description']",
            "name",
            "description",
            &metadata.description,
        );
        upsert_meta("meta[name='robots']", "name", "robots", metadata.robots);
        upsert_meta(
            "meta[property='og:title']",
            "property",
            "og:title",
            &metadata.title,
        );
        upsert_meta(
            "meta[property='og:description']",
            "property",
            "og:description",
            &metadata.description,
        );
        upsert_meta(
            "meta[property='og:url']",
            "property",
            "og:url",
            &metadata.canonical,
        );
        upsert_meta(
            "meta[name='twitter:title']",
            "name",
            "twitter:title",
            &metadata.title,
        );
        upsert_meta(
            "meta[name='twitter:description']",
            "name",
            "twitter:description",
            &metadata.description,
        );

        let canonical = document
            .query_selector("link[rel='canonical']")
            .ok()
            .flatten()
            .or_else(|| {
                let element = document.create_element("link").ok()?;
                element.set_attribute("rel", "canonical").ok()?;
                let head = document.query_selector("head").ok().flatten()?;
                head.append_child(&element).ok()?;
                Some(element)
            });
        if let Some(canonical) = canonical {
            let _ = canonical.set_attribute("href", &metadata.canonical);
        }
    }));

    rsx! {}
}

#[component]
fn AuthSessionBridge() -> Element {
    let compatibility_callback = matches!(use_route::<Route>(), Route::AuthCallback {});
    use_effect(move || {
        spawn(async move {
            let result = api::finish_google_sign_in().await;
            match result {
                Ok(Some(path)) => {
                    if let Some(window) = web_sys::window() {
                        let _ = window.location().replace(&api::auth_completion_url(&path));
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
                        let _ = window.location().replace(&api::auth_completion_url(&path));
                    }
                }
                Err(message) => {
                    let _ = api::take_google_auth_pending();
                    let path = api::take_auth_return().unwrap_or_else(|| "/".to_string());
                    api::request_inline_auth(false, Some(&message));
                    if let Some(window) = web_sys::window() {
                        let _ = window.location().replace(&api::auth_completion_url(&path));
                    }
                }
                Ok(None) => {}
            }
        });
    });
    rsx! {}
}

#[cfg(test)]
mod seo_tests {
    use super::*;

    #[test]
    fn public_rv_pages_have_unique_indexable_metadata() {
        for listing in data::rv_listings() {
            let metadata = seo_metadata(&Route::RvDetail {
                slug: listing.slug.to_string(),
            });

            assert!(metadata.robots.starts_with("index,follow"));
            assert_eq!(
                metadata.canonical,
                format!("{SITE_URL}/rv/{}", listing.slug)
            );
            assert!(metadata.title.contains(listing.title));
        }
    }

    #[test]
    fn account_and_transaction_pages_are_not_indexed() {
        let private_routes = [
            Route::Checkout {},
            Route::Confirmed {},
            Route::Login {},
            Route::Register {},
            Route::AuthCallback {},
            Route::Account {},
            Route::Admin {},
        ];

        for route in private_routes {
            assert_eq!(seo_metadata(&route).robots, "noindex,nofollow");
        }
    }

    #[test]
    fn global_booking_cta_stays_on_home_and_routes_every_other_page_home() {
        assert!(!booking_launch_requires_home(&Route::Home {}));

        let non_home_routes = [
            Route::Catalog {},
            Route::RvDetail {
                slug: "missing-rv".into(),
            },
            Route::Checkout {},
            Route::Confirmed {},
            Route::Contact {},
            Route::About {},
            Route::Attractions {},
            Route::Restaurants {},
            Route::CoolerTrailers {},
            Route::Delivery {},
            Route::RvSales {},
            Route::Terms {},
            Route::Login {},
            Route::Register {},
            Route::AuthCallback {},
            Route::Account {},
            Route::Admin {},
        ];

        for route in non_home_routes {
            assert!(booking_launch_requires_home(&route), "route: {route}");
        }
    }
}

use dioxus::prelude::*;

use crate::data::PHONE;
use crate::Route;
use crate::{
    api,
    components::{CookieConsent, CookieConsentContext, Icon},
};

const LOGO: Asset = asset!(
    "/assets/img/logo.png",
    AssetOptions::image().with_size(ImageSize::Manual {
        width: 120,
        height: 120,
    })
);

#[component]
pub fn Footer() -> Element {
    let rentals_href = api::frontend_path("/#home-rentals");
    let mut cookie_consent = use_context::<CookieConsentContext>();
    rsx! {
        footer { class: "site-footer",
            div { class: "footer-top",
                div { class: "f-brand",
                    Link { class: "brand", to: Route::Home {},
                        img { class: "brand-mark", src: LOGO, alt: "VL Rental" }
                        span { class: "brand-word", style: "color: var(--vl-white);", "VL Rental" }
                    }
                    p { class: "f-tagline",
                        "Travel with us. RVs and everything you need to explore Kelowna & the Okanagan."
                    }
                    div { class: "f-fact",
                        Icon { name: "map-pin", size: 16, color: "var(--vl-accent)" }
                        span { "Kelowna, British Columbia" }
                    }
                    div { class: "f-fact",
                        Icon { name: "phone", size: 16, color: "var(--vl-accent)" }
                        span { "{PHONE}" }
                    }
                }
                div { class: "f-col",
                    div { class: "f-head", "RENTALS" }
                    a { class: "f-link", href: rentals_href, "RV Rentals" }
                    Link { class: "f-link", to: Route::CoolerTrailers {}, "Cooler Trailers" }
                }
                div { class: "f-col",
                    div { class: "f-head", "SERVICES" }
                    Link { class: "f-link", to: Route::Delivery {}, "Delivery Services" }
                    Link { class: "f-link", to: Route::RvSales {}, "RV Sales" }
                    Link { class: "f-link", to: Route::ParksInOurRange {}, "Parks in Our Range" }
                }
                div { class: "f-col",
                    div { class: "f-head", "COMPANY" }
                    Link { class: "f-link", to: Route::About {}, "About Us" }
                    Link { class: "f-link", to: Route::Contact {}, "Contact" }
                    Link { class: "f-link", to: Route::Terms {}, "Terms & Conditions" }
                    Link { class: "f-link", to: Route::Privacy {}, "Privacy & Cookies" }
                    button {
                        class: "f-link f-cookie-link",
                        r#type: "button",
                        onclick: move |_| cookie_consent.0.set(CookieConsent::Undecided),
                        "Cookie choices"
                    }
                }
            }
            div { class: "footer-divider" }
            div { class: "footer-bottom",
                div { "© 2026 VL Rental. All rights reserved." }
            }
        }
    }
}

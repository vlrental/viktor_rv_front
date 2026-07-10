use dioxus::prelude::*;

use crate::components::Icon;
use crate::data::PHONE;
use crate::Route;

const LOGO: Asset = asset!("/assets/img/logo.png");

/// Шапка сайта. На Home — прозрачный overlay поверх hero, на остальных — белая с волосяной линией.
#[component]
pub fn Header() -> Element {
    let route = use_route::<Route>();
    let overlay = matches!(route, Route::Home {});
    let class = if overlay { "site-header overlay" } else { "site-header" };

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
                Link { class: "nav-cta", to: Route::Catalog {}, "Book now" }
            }
        }
    }
}

use dioxus::prelude::*;

use crate::data::PHONE;
use crate::Route;
use crate::{api, components::Icon};

const LOGO: Asset = asset!("/assets/img/logo.png");

#[component]
pub fn Footer() -> Element {
    let mut email = use_signal(String::new);
    let mut status = use_signal(String::new);
    let mut busy = use_signal(|| false);
    let subscribe = move |_| {
        let value = email.read().trim().to_string();
        async move {
            status.set(String::new());
            if !value.contains('@') || !value.contains('.') {
                status.set("Enter a valid email address.".into());
                return;
            }
            busy.set(true);
            match api::subscribe(&value).await {
                Ok(()) => {
                    status.set("Subscribed".into());
                    email.set(String::new());
                }
                Err(error) => status.set(error),
            }
            busy.set(false);
        }
    };
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
                    div { class: "f-social",
                        a {
                            class: "f-social-btn",
                            href: "https://www.facebook.com/people/VL-Pro-Trailer-Care/61576201770508/",
                            target: "_blank",
                            rel: "noopener noreferrer",
                            aria_label: "Facebook",
                            Icon { name: "facebook", size: 19, color: "var(--vl-white)" }
                        }
                        a {
                            class: "f-social-btn",
                            href: "https://www.instagram.com/lairichviktor/",
                            target: "_blank",
                            rel: "noopener noreferrer",
                            aria_label: "Instagram",
                            Icon { name: "instagram", size: 19, color: "var(--vl-white)" }
                        }
                    }
                }
                div { class: "f-col",
                    div { class: "f-head", "RENTALS" }
                    a { class: "f-link", href: "/#home-rentals", "RV Rentals" }
                    Link { class: "f-link", to: Route::CoolerTrailers {}, "Cooler Trailers" }
                }
                div { class: "f-col",
                    div { class: "f-head", "SERVICES" }
                    Link { class: "f-link", to: Route::Delivery {}, "Delivery Services" }
                    Link { class: "f-link", to: Route::RvSales {}, "RV Sales" }
                    Link { class: "f-link", to: Route::Attractions {}, "Attractions" }
                    Link { class: "f-link", to: Route::Restaurants {}, "Restaurants" }
                }
                div { class: "f-col",
                    div { class: "f-head", "COMPANY" }
                    Link { class: "f-link", to: Route::About {}, "About Us" }
                    Link { class: "f-link", to: Route::Contact {}, "Contact" }
                    Link { class: "f-link", to: Route::Terms {}, "Trailer & RV T&C" }
                    Link { class: "f-link", to: Route::Terms {}, "Terms" }
                }
                div { class: "f-newsletter",
                    div { class: "f-news-title", "Plan your trip" }
                    div { class: "f-news-desc", "Get seasonal deals and availability straight to your inbox." }
                    div { class: "f-news-input",
                        input { r#type: "email", placeholder: "you@email.com", value: "{email}", oninput: move |e| email.set(e.value()) }
                        button { class: "f-news-go", disabled: *busy.read(), onclick: subscribe, if *busy.read() { "Joining…" } else { "Subscribe" } }
                    }
                    if !status.read().is_empty() { div { class: "f-news-desc", role: "status", "{status}" } }
                }
            }
            div { class: "footer-divider" }
            div { class: "footer-bottom",
                div { "© 2026 VL Rental. All rights reserved." }
                div { class: "footer-handles",
                    span { "facebook.com/VL-Pro-Trailer-Care" }
                    span { "@lairichviktor" }
                }
            }
        }
    }
}

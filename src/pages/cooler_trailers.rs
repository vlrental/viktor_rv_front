use dioxus::prelude::*;

use crate::{api, components::Icon};

const IMG_COOLER_HERO: Asset = asset!(
    "/assets/img/cooler-trailer-refrigerated.webp",
    AssetOptions::image().with_jpg()
);

/// Карточка «фича» под hero.
struct Feature {
    icon: &'static str,
    title: &'static str,
    desc: &'static str,
}

const FEATURES: [Feature; 4] = [
    Feature {
        icon: "thermometer-snowflake",
        title: "Refrigerated storage",
        desc: "Insulated cargo space for keeping food and beverages chilled.",
    },
    Feature {
        icon: "snowflake",
        title: "Powered refrigeration",
        desc: "A cooling unit maintains cold storage without relying on ice alone.",
    },
    Feature {
        icon: "party-popper",
        title: "For events and campsites",
        desc: "Planned for group camping, outdoor gatherings and event support.",
    },
    Feature {
        icon: "hourglass",
        title: "Coming soon",
        desc: "Not available to reserve yet. Specifications and pricing will be published before launch.",
    },
];

const BULLETS: [&str; 3] = [
    "Chilled storage for food and beverages",
    "No sleeping or living space",
    "Specifications, pricing and availability coming soon",
];

#[component]
pub fn CoolerTrailers() -> Element {
    let rentals_href = api::frontend_path("/#home-rentals");
    rsx! {
        section { class: "clt-hero",
            div { class: "clt-hero-copy",
                div { class: "clt-soon",
                    Icon { name: "hourglass", size: 14, color: "var(--vl-white)" }
                    span { "COMING SOON" }
                }
                div { class: "eyebrow", "REFRIGERATED CARGO TRAILER" }
                h1 { class: "clt-title", "Cold storage for events and campsites" }
                p { class: "clt-sub",
                    "Cooler trailers are insulated cargo trailers with powered refrigeration for keeping food and drinks cold. They are not RVs and are not intended for sleeping. This service is coming soon and cannot be booked yet."
                }
                img {
                    class: "clt-hero-img clt-hero-img-mobile",
                    src: IMG_COOLER_HERO,
                    alt: "Open refrigerated cargo trailer with an insulated cold-storage interior",
                }
                div { class: "clt-bullets",
                    for (i, bullet) in BULLETS.iter().enumerate() {
                        div { key: "b-{i}", class: "clt-bullet",
                            Icon { name: "check-circle-2", size: 18, color: "var(--vl-forest)" }
                            span { {*bullet} }
                        }
                    }
                }
                a { class: "clt-btn", href: rentals_href,
                    span { "Explore available RV rentals" }
                    Icon { name: "arrow-right", size: 17, color: "var(--vl-white)" }
                }
            }
            img {
                class: "clt-hero-img clt-hero-img-desktop",
                src: IMG_COOLER_HERO,
                alt: "Open refrigerated cargo trailer with an insulated cold-storage interior",
            }
        }
        section { class: "clt-features",
            for (i, f) in FEATURES.iter().enumerate() {
                div { key: "f-{i}", class: "clt-card",
                    div { class: "clt-ib",
                        Icon { name: f.icon, size: 22, color: "var(--vl-white)" }
                    }
                    div { class: "clt-card-body",
                        div { class: "clt-card-title", {f.title} }
                        div { class: "clt-card-desc", {f.desc} }
                    }
                }
            }
        }
    }
}

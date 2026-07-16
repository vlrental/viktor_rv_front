use dioxus::prelude::*;

use crate::{api, components::Icon};

const IMG_COOLER_HERO: Asset = asset!(
    "/assets/img/cooler-hero.jpg",
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
        title: "Powered cooling",
        desc: "Consistent cold for days — no daily ice runs.",
    },
    Feature {
        icon: "package",
        title: "Big capacity",
        desc: "Room for a whole group's food, drinks and catch.",
    },
    Feature {
        icon: "truck",
        title: "Delivered chilled",
        desc: "We drop it off pre-cooled and ready at your site.",
    },
    Feature {
        icon: "link",
        title: "Add to any rental",
        desc: "Pairs with any RV booking in a click.",
    },
];

const BULLETS: [&str; 3] = [
    "Large, insulated capacity for groups",
    "Powered cooling — no melting ice runs",
    "Easy to tow or delivered to your site",
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
                div { class: "eyebrow", "COOLER TRAILERS" }
                h1 { class: "clt-title", "Keep it cold, wherever you roam" }
                p { class: "clt-sub",
                    "Exciting new cooler trailer rentals launching soon — perfect for events, camping trips and outdoor adventures. Stay tuned, something cool is coming your way!"
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
                    span { "Explore RV rentals" }
                    Icon { name: "arrow-right", size: 17, color: "var(--vl-white)" }
                }
            }
            div {
                class: "clt-hero-img",
                style: "background-image: url('{IMG_COOLER_HERO}');",
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

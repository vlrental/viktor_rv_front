use crate::components::Icon;
use crate::data::IMG_BULLET_OVERLOOK;
use dioxus::prelude::*;

const IMG_HOST: Asset = asset!(
    "/assets/img/host-viktor.webp",
    AssetOptions::image().with_jpg()
);

/// Ценности компании (правая колонка секции Story).
const VALUES: [(&str, &str, &str); 4] = [
    (
        "shield-check",
        "Safety first",
        "Meticulous maintenance and safety checks on every unit.",
    ),
    (
        "scale",
        "Fair & transparent",
        "Upfront pricing and clear policies — no hidden fees.",
    ),
    (
        "map-pin",
        "Local & personal",
        "Okanagan-based team that knows the lakes and the roads.",
    ),
    (
        "compass",
        "Adventure-ready",
        "Whatever your trip, we've got the gear to get you out there.",
    ),
];

/// Отзывы гостей.
const REVIEWS: [(&str, &str, &str); 3] = [
    (
        "“Viktor's trailer was very clean and well maintained…”",
        "Michael Coulter",
        "August 2025",
    ),
    (
        "“Excellent experience with Victor…”",
        "Sabrina Polga",
        "July 2025",
    ),
    (
        "“Viktor is a gracious and considerate host…”",
        "Karin Atkinson",
        "July 2025",
    ),
];

/// Цифры-факты (тёмная полоса Stats).
const STATS: [(&str, &str); 4] = [
    ("6", "RVs in the fleet"),
    ("Owner-run", "No middlemen, no fees"),
    ("Kelowna", "Local Okanagan team"),
    ("Hours", "Typical response time"),
];

/// Страница «About Us» — Pencil: desktop `DsUMN`, mobile `A0RyD`.
#[component]
pub fn About() -> Element {
    rsx! {
        // Hero с фото команды и градиентом
        section { class: "ab-hero",
            div { class: "ab-hero-img", style: "background-image: url('{IMG_BULLET_OVERLOOK}');" }
            div { class: "ab-hero-ov" }
            div { class: "ab-hero-copy",
                div { class: "eyebrow gold", "ABOUT VL RENTAL" }
                h1 { class: "ab-hero-title", "Welcome to adventure, made simple" }
                p { class: "ab-hero-sub",
                    "Book directly with the owner — no middlemen, no service fees, no hassle. Fully-equipped RVs for every kind of Okanagan trip, all in one place in Kelowna."
                }
            }
        }

        // Story: текст + хост и ценности
        section { class: "ab-story",
            div { class: "ab-story-l",
                h2 { class: "ab-story-h", "Who we are" }
                p { class: "ab-story-p",
                    "VL Rental was built around a simple idea: "
                    strong { "RV travel should be easy, comfortable, and enjoyable from the very beginning." }
                }
                p { class: "ab-story-p",
                    "Based in the Okanagan, we started VL Rental with a focus on providing clean, well-maintained RVs and a rental experience that makes your trip easier — not more complicated."
                }
                p { class: "ab-story-p",
                    "We know that planning an RV trip already takes enough work. Finding the right trailer, picking it up, setting everything up, connecting the utilities, and making sure everything is ready can take time and add unnecessary stress. That’s why we offer "
                    strong { "delivery and on-site setup" }
                    ". Our goal is simple: when you arrive, your RV is already there, set up, and ready for you to enjoy."
                }
                p { class: "ab-story-p",
                    "We take care of our trailers the way we would want someone to take care of our own. Every unit is inspected, cleaned, and prepared before each rental. We pay attention to the small details because we know those details can make a big difference during a trip."
                }
                p { class: "ab-story-p",
                    "We also believe that rental pricing should be straightforward. You should know what you’re paying for and what to expect before your vacation begins — without unnecessary surprises."
                }
                p { class: "ab-story-p",
                    "VL Rental is still a small, locally operated business, and we believe that’s one of our strengths. When you contact us, you’re dealing with people who know our RVs, know the Okanagan, and genuinely care about making sure your trip goes well."
                }
                p { class: "ab-story-p",
                    "Whether you're planning a weekend getaway, a family vacation, or your first RV adventure, our goal is the same:"
                }
                p { class: "ab-story-p",
                    strong { "You enjoy the trip. We take care of the RV." }
                }
                p { class: "ab-story-p",
                    "That’s what VL Rental is all about."
                }
            }
            div { class: "ab-story-r",
                div { class: "ab-host",
                    div { class: "ab-host-photo", style: "background-image: url('{IMG_HOST}');" }
                    div { class: "ab-host-c",
                        div { class: "ab-host-n", "Viktor — owner & host" }
                        div { class: "ab-host-d", "Book directly with the owner — no middlemen, no service fees." }
                    }
                }
                for (icon, title, desc) in VALUES.iter() {
                    div { key: "{title}", class: "ab-value",
                        div { class: "ab-value-ib",
                            Icon { name: icon, size: 20, color: "var(--vl-forest)" }
                        }
                        div { class: "ab-value-c",
                            div { class: "ab-value-t", {*title} }
                            div { class: "ab-value-d", {*desc} }
                        }
                    }
                }
            }
        }

        // Reviews
        section { class: "ab-reviews",
            div { class: "ab-reviews-head",
                div {
                    div { class: "eyebrow", "REVIEWS" }
                    h2 { class: "ab-reviews-title", "What guests say" }
                }
                div { class: "ab-host-chip",
                    div { class: "ab-host-av", style: "background-image: url('{IMG_HOST}');" }
                    div { class: "ab-host-chip-c",
                        span { class: "ab-host-chip-n", "Viktor — your host" }
                        span { class: "ab-host-chip-s", "Books every rental personally" }
                    }
                }
            }
            div { class: "card-row",
                for (quote, name, date) in REVIEWS.iter() {
                    div { key: "{name}", class: "ab-review",
                        div { class: "ab-stars",
                            for i in 0..5 {
                                span { key: "star-{i}",
                                    Icon { name: "star", size: 15, color: "var(--vl-accent)" }
                                }
                            }
                        }
                        p { class: "ab-quote", {*quote} }
                        div { class: "ab-who",
                            span { class: "ab-who-n", {*name} }
                            span { class: "ab-who-d", {*date} }
                        }
                    }
                }
            }
        }

        // Stats
        section { class: "ab-stats",
            for (num, label) in STATS.iter() {
                div { key: "{label}", class: "ab-stat",
                    div { class: "ab-stat-n", {*num} }
                    div { class: "ab-stat-l", {*label} }
                }
            }
        }
    }
}

use dioxus::prelude::*;

use crate::components::Icon;
use crate::data::PHONE;
use crate::Route;

const IMG_DELIVERY_HERO: Asset = asset!(
    "/assets/img/delivery-hero-v2.jpg",
    AssetOptions::image().with_jpg()
);

/// Шаг «как это работает» под hero.
struct Step {
    icon: &'static str,
    num: &'static str,
    title: &'static str,
    desc: &'static str,
}

const STEPS: [Step; 3] = [
    Step {
        icon: "calendar-check",
        num: "01",
        title: "Book & share your site",
        desc: "Reserve your rig and tell us where you're headed.",
    },
    Step {
        icon: "truck",
        num: "02",
        title: "We deliver & set up",
        desc: "We tow, position, level and prep everything for you.",
    },
    Step {
        icon: "tent-tree",
        num: "03",
        title: "You just enjoy",
        desc: "Arrive to a ready campsite. We collect the RV after your stay.",
    },
];

#[component]
pub fn Delivery() -> Element {
    rsx! {
        section { class: "dv-hero",
            div { class: "dv-hero-media",
                img {
                    class: "dv-hero-img",
                    src: IMG_DELIVERY_HERO,
                    alt: "Silver pickup delivering a Jayco fifth-wheel RV in the Okanagan",
                }
            }
            div { class: "dv-hero-copy",
                div { class: "eyebrow", "DELIVERY SERVICES" }
                h1 { class: "dv-title", "We deliver, level & set up" }
                p { class: "dv-sub",
                    "No truck? No problem. Every RV includes required delivery and setup within 150 km of Kelowna. CA$150 through 50 km, then CA$2 per additional kilometre, two way."
                }
                Link { class: "dv-btn", to: Route::Contact {},
                    span { "Request delivery" }
                    Icon { name: "arrow-right", size: 17, color: "var(--vl-white)" }
                }
            }
        }
        section { class: "dv-steps",
            for (i, step) in STEPS.iter().enumerate() {
                div { key: "s-{i}", class: "dv-step",
                    div { class: "dv-step-top",
                        div { class: "dv-ib",
                            Icon { name: step.icon, size: 22, color: "var(--vl-white)" }
                        }
                        div { class: "dv-step-num", {step.num} }
                    }
                    div { class: "dv-step-title", {step.title} }
                    div { class: "dv-step-desc", {step.desc} }
                }
            }
        }
        section { class: "dv-coverage",
            div { class: "dv-coverage-copy",
                div { class: "dv-coverage-loc",
                    Icon { name: "map-pin", size: 22, color: "var(--vl-accent)" }
                    div { class: "dv-coverage-title", "Delivery and Setup — real rates" }
                }
                p { class: "dv-coverage-sub",
                    "Minimum fee CA$150 through 50 km, then CA$2 per additional kilometre, two way, up to 150 km. Enter your destination and the server calculates the driving route and exact fee automatically."
                }
            }
            a { class: "dv-phone-btn", href: "tel:+12508785874",
                Icon { name: "phone", size: 17, color: "var(--vl-forest-2)" }
                span { {PHONE} }
            }
        }
    }
}

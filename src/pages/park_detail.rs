use dioxus::prelude::*;

use crate::{components::Icon, Route};

const IMG_BEAR_CREEK: Asset = asset!(
    "/assets/img/park-bear-creek.webp",
    AssetOptions::image().with_jpg()
);
const IMG_FINTRY: Asset = asset!(
    "/assets/img/park-fintry.webp",
    AssetOptions::image().with_jpg()
);
const IMG_ELLISON: Asset = asset!(
    "/assets/img/park-ellison.webp",
    AssetOptions::image().with_jpg()
);

#[derive(Clone, Copy)]
struct ParkGuide {
    name: &'static str,
    area: &'static str,
    introduction: &'static str,
    image: Asset,
    image_alt: &'static str,
    official_url: &'static str,
}

fn guide_for(slug: &str) -> Option<ParkGuide> {
    match slug {
        "bear-creek" => Some(ParkGuide {
            name: "Bear Creek Provincial Park",
            area: "West Kelowna",
            introduction: "Stay close to Kelowna beside Okanagan Lake. Reserve your campsite first, then choose an available VL Rental RV and we will arrange delivery and setup.",
            image: IMG_BEAR_CREEK,
            image_alt: "Okanagan Lake shoreline at Bear Creek Provincial Park",
            official_url: "https://bcparks.ca/bear-creek-park/",
        }),
        "fintry" => Some(ParkGuide {
            name: "Fintry Provincial Park",
            area: "Westside Road, Okanagan",
            introduction: "Plan a delivered RV stay at this waterfront park north of Kelowna. Reserve your campsite first, then choose an available VL Rental RV for your dates.",
            image: IMG_FINTRY,
            image_alt: "Waterfront and mountains at Fintry Provincial Park",
            official_url: "https://bcparks.ca/fintry-park/",
        }),
        "ellison" => Some(ParkGuide {
            name: "Ellison Provincial Park",
            area: "Vernon, Okanagan",
            introduction: "Enjoy a forested campsite near Okanagan Lake without towing an RV. Reserve your campsite first and VL Rental can deliver and set up an available trailer for your stay.",
            image: IMG_ELLISON,
            image_alt: "Rocky Okanagan Lake shoreline at Ellison Provincial Park",
            official_url: "https://bcparks.ca/ellison-park/",
        }),
        _ => None,
    }
}

#[component]
pub fn ParkDetail(slug: String) -> Element {
    let Some(park) = guide_for(&slug) else {
        return rsx! {
            section { class: "park-detail-missing",
                div { class: "parks-eyebrow", "PARK GUIDE" }
                h1 { "Park guide not found" }
                p { "Return to our campground guide to explore destinations in our delivery range." }
                Link { class: "parks-primary-link", to: Route::ParksInOurRange {},
                    "View parks in our range"
                    Icon { name: "arrow-right", size: 17, color: "currentColor" }
                }
            }
        };
    };

    rsx! {
        article { class: "park-detail-page",
            header { class: "park-detail-hero",
                div { class: "park-detail-hero-copy",
                    Link { class: "park-detail-back", to: Route::ParksInOurRange {},
                        Icon { name: "arrow-left", size: 16, color: "currentColor" }
                        "Parks in our range"
                    }
                    div { class: "parks-eyebrow", "DELIVERED RV CAMPING · {park.area}" }
                    h1 { "RV Delivery to {park.name}" }
                    p { "{park.introduction}" }
                    div { class: "park-detail-actions",
                        Link { class: "parks-primary-link", to: Route::Catalog {},
                            "Browse available RVs"
                            Icon { name: "arrow-right", size: 17, color: "currentColor" }
                        }
                        a {
                            class: "parks-secondary-link",
                            href: park.official_url,
                            target: "_blank",
                            rel: "noopener noreferrer",
                            "Check campground details"
                            Icon { name: "arrow-up-right", size: 17, color: "currentColor" }
                        }
                    }
                }
                img { src: park.image, alt: park.image_alt, decoding: "async" }
            }

            section { class: "park-detail-content",
                div { class: "park-detail-main",
                    div { class: "parks-eyebrow", "HOW IT WORKS" }
                    h2 { "Your campsite, our RV and setup" }
                    div { class: "park-steps",
                        div { class: "park-step",
                            span { "01" }
                            div {
                                h3 { "Reserve your campsite" }
                                p { "Book a suitable campsite directly with the campground and keep your reservation details handy." }
                            }
                        }
                        div { class: "park-step",
                            span { "02" }
                            div {
                                h3 { "Choose your RV" }
                                p { "Select an available travel trailer for your dates and enter the campground as your destination." }
                            }
                        }
                        div { class: "park-step",
                            span { "03" }
                            div {
                                h3 { "We deliver and set up" }
                                p { "We transport, position, level and prepare the RV at the approved campsite. Customer pickup is not required." }
                            }
                        }
                    }
                }
                aside { class: "park-delivery-card",
                    div { class: "parks-eyebrow", "DELIVERY DETAILS" }
                    h2 { "Know before you book" }
                    ul {
                        li { strong { "Delivery range" } span { "Up to 150 km one way from our Kelowna base" } }
                        li { strong { "Delivery fee" } span { "CA$150 through 40 km" } }
                        li { strong { "Beyond 40 km" } span { "CA$2.50/km in each direction" } }
                        li { strong { "Delivery & setup" } span { "2:00 PM" } }
                        li { strong { "Return" } span { "11:00 AM" } }
                    }
                    p { "Final eligibility and delivery pricing are confirmed using the actual road route to your campsite." }
                    Link { class: "park-detail-contact", to: Route::Contact {},
                        "Ask about this destination"
                        Icon { name: "arrow-right", size: 16, color: "currentColor" }
                    }
                }
            }
        }
    }
}

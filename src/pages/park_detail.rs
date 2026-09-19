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
const IMG_KEKULI_BAY: Asset = asset!(
    "/assets/img/park-kekuli-bay.webp",
    AssetOptions::image().with_jpg()
);
const IMG_OKANAGAN_LAKE: Asset = asset!(
    "/assets/img/park-okanagan-lake.webp",
    AssetOptions::image().with_jpg()
);
const IMG_OKANAGAN_FALLS: Asset = asset!(
    "/assets/img/park-okanagan-falls.webp",
    AssetOptions::image().with_jpg()
);
const IMG_VASEUX_LAKE: Asset = asset!(
    "/assets/img/park-vaseux-lake.webp",
    AssetOptions::image().with_jpg()
);
const IMG_HAYNES_POINT: Asset = asset!(
    "/assets/img/park-haynes-point.webp",
    AssetOptions::image().with_jpg()
);
const IMG_SHUSWAP_LAKE: Asset = asset!(
    "/assets/img/park-shuswap-lake.webp",
    AssetOptions::image().with_jpg()
);
const IMG_HERALD: Asset = asset!(
    "/assets/img/park-herald.webp",
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
    planning_title: &'static str,
    planning_copy: &'static str,
    access_note: &'static str,
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
            planning_title: "Lakeside camping close to Kelowna",
            planning_copy: "Bear Creek combines a sandy beach, shaded campsites and canyon trails on the west side of Okanagan Lake. It is a practical choice for families who want a campground near Kelowna while still arriving to a fully set-up RV.",
            access_note: "Summer camping requires a reservation. Send us your site number and the maximum trailer length shown for that site; we will check the Westside Road approach and the fit of your chosen RV.",
        }),
        "fintry" => Some(ParkGuide {
            name: "Fintry Provincial Park",
            area: "Westside Road, Okanagan",
            introduction: "Plan a delivered RV stay at this waterfront park north of Kelowna. Reserve your campsite first, then choose an available VL Rental RV for your dates.",
            image: IMG_FINTRY,
            image_alt: "Waterfront and mountains at Fintry Provincial Park",
            official_url: "https://bcparks.ca/fintry-park/",
            planning_title: "Waterfront camping on Westside Road",
            planning_copy: "Fintry offers a long Okanagan Lake shoreline, walking routes and access to the historic estate and nearby waterfall. Because the destination is reached along Westside Road, final delivery approval is based on the actual route and campsite details.",
            access_note: "Fintry is reached from Westside Road via Fintry Delta Road. Share the campground loop, site number and permitted trailer length so we can check both the route and the space available for setup.",
        }),
        "ellison" => Some(ParkGuide {
            name: "Ellison Provincial Park",
            area: "Vernon, Okanagan",
            introduction: "Enjoy a forested campsite near Okanagan Lake without towing an RV. Reserve your campsite first and VL Rental can deliver and set up an available trailer for your stay.",
            image: IMG_ELLISON,
            image_alt: "Rocky Okanagan Lake shoreline at Ellison Provincial Park",
            official_url: "https://bcparks.ca/ellison-park/",
            planning_title: "A forested base near Vernon",
            planning_copy: "Ellison is known for sheltered swimming coves and lakeside trails in a quieter forest setting near Vernon. It works well for guests who want an Okanagan Lake stay without collecting or towing a trailer themselves.",
            access_note: "The park approach runs from Vernon via 25th Avenue. Send the site number and its trailer-length limit; a campground reservation alone does not confirm that our RV fits or can be delivered safely.",
        }),
        "kekuli-bay" => Some(ParkGuide {
            name: "Kekuli Bay Provincial Park",
            area: "Kalamalka Lake, south of Vernon",
            introduction: "Camp above Kalamalka Lake and arrive to an RV already delivered and set up. The park is reached from Highway 97, about 11 km south of Vernon.",
            image: IMG_KEKULI_BAY,
            image_alt: "Kalamalka Lake viewed from Kekuli Bay Provincial Park",
            official_url: "https://bcparks.ca/kekuli-bay-park/",
            planning_title: "Sunny sites and Kalamalka Lake views",
            planning_copy: "Kekuli Bay has landscaped campsites, lake views and access to the Okanagan Rail Trail. The campground can have limited shade in summer, so guests should plan for warm daytime conditions and verify current park advisories before arrival.",
            access_note: "The campground is about 3 km beyond the Highway 97 turn-off. Tell us your site number and the site's trailer-length allowance so we can confirm the last part of the route and setup space.",
        }),
        "okanagan-lake" => Some(ParkGuide {
            name: "Okanagan Lake Provincial Park",
            area: "North of Summerland",
            introduction: "Choose the North or South campground, reserve a suitable site and let VL Rental deliver and set up your RV beside Okanagan Lake.",
            image: IMG_OKANAGAN_LAKE,
            image_alt: "Okanagan Lake Provincial Park campground near Summerland",
            official_url: "https://bcparks.ca/okanagan-lake-park/",
            planning_title: "Two campgrounds beside Okanagan Lake",
            planning_copy: "The park sits about 11 km north of Summerland on Highway 97 and offers two large campgrounds, beaches and panoramic lake views. Tell us whether your reservation is in the North or South campground so the delivery route and timing can be planned correctly.",
            access_note: "Check whether your reservation is for North or South, then send the loop, site number and trailer-length limit. We need the correct campground to confirm the delivery address and RV fit.",
        }),
        "okanagan-falls" => Some(ParkGuide {
            name: "sx̌ʷəx̌ʷnitkʷ Provincial Park",
            area: "Okanagan Falls",
            introduction: "Stay beside the Okanagan River in Okanagan Falls with a delivered RV ready at your reserved campsite.",
            image: IMG_OKANAGAN_FALLS,
            image_alt: "Entrance to sx̌ʷəx̌ʷnitkʷ Provincial Park in Okanagan Falls",
            official_url: "https://bcparks.ca/sxwexwnitkw-park/",
            planning_title: "A compact riverside campground in town",
            planning_copy: "The campground is on Green Lake Road, approximately 500 metres from Highway 97. It is managed by the Osoyoos Indian Band and is culturally significant; guests should review current notices and respect all posted rules and protected areas.",
            access_note: "BC Parks notes a narrow approach with a blind corner from Green Lake Road. Send your exact site and permitted trailer length before paying for the RV so we can assess safe access; check the park's current opening dates and advisories too.",
        }),
        "vaseux-lake" => Some(ParkGuide {
            name: "Vaseux Lake Provincial Park",
            area: "South Okanagan",
            introduction: "Plan a quiet South Okanagan stay between Okanagan Falls and Oliver, with your selected RV delivered directly to an approved campsite.",
            image: IMG_VASEUX_LAKE,
            image_alt: "Vaseux Lake Provincial Park in the South Okanagan",
            official_url: "https://bcparks.ca/vaseux-lake-park/",
            planning_title: "A small nature-focused lakeside stay",
            planning_copy: "Vaseux Lake is surrounded by dry-country scenery and important wildlife habitat. It suits guests looking for a quieter base for paddling, birdwatching and exploring the South Okanagan rather than a large resort-style campground.",
            access_note: "Check the specific site's trailer allowance and available services in the current reservation details. Send us the site number and RV length limit before assuming a trailer or its equipment will fit.",
        }),
        "swiws" => Some(ParkGuide {
            name: "sẁiẁs Provincial Park",
            area: "Osoyoos",
            introduction: "Explore a waterfront campsite on Osoyoos Lake, then ask us to verify the road route and site before committing to an RV rental at this outer-range destination.",
            image: IMG_HAYNES_POINT,
            image_alt: "Wetland boardwalk at sẁiẁs Provincial Park on Osoyoos Lake",
            official_url: "https://bcparks.ca/swiws-park/",
            planning_title: "Water on both sides in Osoyoos",
            planning_copy: "sẁiẁs extends into Osoyoos Lake and is valued for waterfront camping, swimming and wetland habitat. It sits near the outer edge of VL Rental's service range, so actual driving distance and route conditions must be approved before the booking is finalized.",
            access_note: "The park is south of Osoyoos off Highway 97. Send the exact campsite and its trailer-length allowance before committing to a reservation; delivery is available only if the actual one-way road route is within 150 km and access is safe.",
        }),
        "shuswap-lake" => Some(ParkGuide {
            name: "Shuswap Lake Provincial Park",
            area: "Scotch Creek, Shuswap",
            introduction: "Explore a family campground at Shuswap Lake and ask us to verify the road route and campsite before making RV rental plans.",
            image: IMG_SHUSWAP_LAKE,
            image_alt: "Beach and mountain view at Shuswap Lake Provincial Park",
            official_url: "https://bcparks.ca/shuswap-lake-park/",
            planning_title: "A family beach destination in the Shuswap",
            planning_copy: "This large campground in Scotch Creek is known for its broad beach, warm-water swimming and paved approach. As a longer delivery from Kelowna, it requires an exact route calculation and confirmation that the chosen site fits the RV.",
            access_note: "The campground is reached by a further 19 km of paved road after leaving Highway 1 at Squilax. Send us the loop, site number and trailer-length limit early; we can offer delivery only after confirming the full one-way road distance is within 150 km.",
        }),
        "herald" => Some(ParkGuide {
            name: "Herald Provincial Park",
            area: "Tappen, Shuswap",
            introduction: "Explore a Shuswap Lake campsite near Tappen and request a route and site check before committing to RV delivery.",
            image: IMG_HERALD,
            image_alt: "Margaret Falls trail at Herald Provincial Park",
            official_url: "https://bcparks.ca/herald-park/",
            planning_title: "Beach days and the Margaret Falls trail",
            planning_copy: "Herald is a large campground with a family beach and a shaded walking route to Margaret Falls. The drive from Kelowna is near the outer portion of our service area, so delivery depends on the actual road distance and safe access to the reserved site.",
            access_note: "Herald Park is 14 km east of Highway 1 at Tappen. Send the campground loop, site number and permitted trailer length before paying for an RV; the complete road route and campsite access must be approved first.",
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
                    section { class: "park-destination-intro",
                        div { class: "parks-eyebrow", "WHY CAMP HERE" }
                        h2 { "{park.planning_title}" }
                        p { "{park.planning_copy}" }
                        div { class: "park-access-note",
                            Icon { name: "info", size: 18, color: "currentColor" }
                            span { "{park.access_note}" }
                        }
                    }
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

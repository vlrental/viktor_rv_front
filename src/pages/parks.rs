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
struct Park {
    name: &'static str,
    area: &'static str,
    description: &'static str,
    highlights: &'static str,
    image: Asset,
    image_alt: &'static str,
    url: &'static str,
    detail_slug: Option<&'static str>,
}

#[derive(Clone, Copy)]
struct SourceLink {
    label: &'static str,
    url: &'static str,
}

#[derive(Clone, Copy)]
struct CampgroundRegion {
    area: &'static str,
    description: &'static str,
    campgrounds: &'static [&'static str],
    sources: &'static [SourceLink],
}

const PROVINCIAL_PARKS: [Park; 10] = [
    Park {
        name: "Bear Creek",
        area: "West Kelowna",
        description: "Stay beside Okanagan Lake with a sandy beach, shaded campsites and canyon trails, all within an easy drive of Kelowna.",
        highlights: "Beach · Trails · Close to Kelowna",
        image: IMG_BEAR_CREEK,
        image_alt: "Okanagan Lake shoreline at Bear Creek Park",
        url: "https://bcparks.ca/bear-creek-park/",
        detail_slug: Some("bear-creek"),
    },
    Park {
        name: "Fintry",
        area: "Westside Road",
        description: "A spacious waterfront park north of Kelowna, pairing long beaches with forest walks, waterfalls and the historic Fintry estate.",
        highlights: "Waterfront · Waterfalls · Heritage",
        image: IMG_FINTRY,
        image_alt: "Waterfront and mountains at Fintry Park",
        url: "https://bcparks.ca/fintry-park/",
        detail_slug: Some("fintry"),
    },
    Park {
        name: "Ellison",
        area: "Vernon",
        description: "A peaceful, forested campground just outside Vernon, with sheltered swimming coves and trails along the Okanagan Lake shoreline.",
        highlights: "Swimming · Forest · Lakeside trails",
        image: IMG_ELLISON,
        image_alt: "Rocky Okanagan Lake shoreline at Ellison Park",
        url: "https://bcparks.ca/ellison-park/",
        detail_slug: Some("ellison"),
    },
    Park {
        name: "Kekuli Bay",
        area: "South of Vernon",
        description: "Camp on a sunny hillside above the brilliant waters of Kalamalka Lake, with a beach, convenient lake access and direct highway access.",
        highlights: "Lake views · Swimming · Beach",
        image: IMG_KEKULI_BAY,
        image_alt: "Kalamalka Lake seen from Kekuli Bay Park",
        url: "https://bcparks.ca/kekuli-bay-park/",
        detail_slug: None,
    },
    Park {
        name: "Okanagan Lake Provincial Park",
        area: "Summerland",
        description: "Choose between the North and South campgrounds near Summerland for easy Highway 97 access, lake views and relaxed beach days.",
        highlights: "Two campgrounds · Beach · Hwy 97",
        image: IMG_OKANAGAN_LAKE,
        image_alt: "South campground entrance at Okanagan Lake Park",
        url: "https://bcparks.ca/okanagan-lake-park/",
        detail_slug: None,
    },
    Park {
        name: "sx̌ʷəx̌ʷnitkʷ / Okanagan Falls",
        area: "Okanagan Falls",
        description: "A compact riverside campground in Okanagan Falls, only about 500 metres from Highway 97 and close to South Okanagan wineries.",
        highlights: "Riverside · In town · Wine country",
        image: IMG_OKANAGAN_FALLS,
        image_alt: "Campground entrance at sx̌ʷəx̌ʷnitkʷ Park",
        url: "https://bcparks.ca/sxwexwnitkw-park/",
        detail_slug: None,
    },
    Park {
        name: "Vaseux Lake",
        area: "South Okanagan",
        description: "A quiet lakeside base between Okanagan Falls and Oliver, surrounded by dry-country scenery and exceptional bird and wildlife habitat.",
        highlights: "Wildlife · Paddling · Hwy 97",
        image: IMG_VASEUX_LAKE,
        image_alt: "Vaseux Lake Park campground entrance in the South Okanagan",
        url: "https://bcparks.ca/vaseux-lake-park/",
        detail_slug: None,
    },
    Park {
        name: "sẁiẁs / Haynes Point",
        area: "Osoyoos",
        description: "A narrow peninsula reaching into warm Osoyoos Lake, with water on both sides and campsites at the outer edge of our guide radius.",
        highlights: "Waterfront sites · Swimming · Osoyoos",
        image: IMG_HAYNES_POINT,
        image_alt: "Wetland boardwalk at sẁiẁs Park on Osoyoos Lake",
        url: "https://bcparks.ca/swiws-park/",
        detail_slug: None,
    },
    Park {
        name: "Shuswap Lake",
        area: "Scotch Creek",
        description: "A favourite family campground in Scotch Creek with a broad beach, warm-water swimming and a fully paved approach.",
        highlights: "Family beach · Swimming · Paved access",
        image: IMG_SHUSWAP_LAKE,
        image_alt: "Beach and mountain view at Shuswap Lake Park",
        url: "https://bcparks.ca/shuswap-lake-park/",
        detail_slug: None,
    },
    Park {
        name: "Herald",
        area: "Tappen / Shuswap",
        description: "A large Shuswap Lake campground near Tappen, known for its family beach and the shaded walk to Margaret Falls.",
        highlights: "Large campground · Beach · Waterfall trail",
        image: IMG_HERALD,
        image_alt: "Margaret Falls trail at Herald Park",
        url: "https://bcparks.ca/herald-park/",
        detail_slug: None,
    },
];

const LAKE_COUNTRY_CAMPGROUNDS: &[&str] = &[
    "Wood Lake RV Park & Marina",
    "Ley’s RV Park & Farm",
    "The Orchard RV Retreat",
    "Okanagan RV Park",
    "Apple Valley Orchard & RV Park",
    "Apple Orchard RV Park",
    "Orchard Hill RV Park",
    "Kelowna Urban Farm & RV Park",
    "West Bay Beach Resort",
    "Peachland RV Park",
];

const VERNON_CAMPGROUNDS: &[&str] = &[
    "Swan Lake RV Park & Campground",
    "Swan Lake RV Resort",
    "Lake Front RV Park",
    "Dutch’s Campground",
    "Cedar Falls Campground",
    "Silver Star Campground & RV Park",
    "Riverside RV Park & Campground",
    "Quilakwa RV Park",
];

const PENTICTON_CAMPGROUNDS: &[&str] = &[
    "Peach Orchard Campground",
    "Camp-Along Resort",
    "Wright’s Beach Camp",
    "South Beach Gardens RV Park",
    "Oxbow RV Resort",
    "Naramata Campground",
    "Twin Lakes Golf Course & RV Park",
    "Playa Okanagan RV Resort",
];

const OSOYOOS_CAMPGROUNDS: &[&str] = &[
    "Gallagher Lake Camping & RV Resort",
    "The Lakeside Resort",
    "Desert Lake RV Resort",
    "The Orchard at Oliver",
    "Nk’Mip RV Park & Campground (380+ sites)",
];

const LAKE_COUNTRY_SOURCES: &[SourceLink] = &[
    SourceLink {
        label: "Lake Country stays",
        url: "https://visitlakecountry.ca/where-to-stay/",
    },
    SourceLink {
        label: "Kelowna RV locations",
        url: "https://www.bccancer.bc.ca/centre-for-the-southern-interior-site/Documents/Travelling%20To%20Kelowna%20For%20Cancer%20Treatment%20Brochure.pdf",
    },
];

const VERNON_SOURCES: &[SourceLink] = &[SourceLink {
    label: "Tourism Vernon camping guide",
    url: "https://www.tourismvernon.com/places-to-stay/camping-rv",
}];

const PENTICTON_SOURCES: &[SourceLink] = &[SourceLink {
    label: "Visit Penticton camping guide",
    url: "https://visitpenticton.com/wp-content/uploads/2025/06/Camping-RV-Sani-Stations.pdf",
}];

const OSOYOOS_SOURCES: &[SourceLink] = &[
    SourceLink {
        label: "Nk’Mip campground",
        url: "https://www.destinationosoyoos.com/listing/nkmip-campground-rv-park",
    },
    SourceLink {
        label: "Gallagher Lake",
        url: "https://www.parkbridge.com/en/resort/resort-detail/gallagher-lake",
    },
];

const CAMPGROUND_REGIONS: [CampgroundRegion; 4] = [
    CampgroundRegion {
        area: "Lake Country / Kelowna",
        description:
            "The closest choices to our Kelowna base, with lakefront, farm and orchard settings.",
        campgrounds: LAKE_COUNTRY_CAMPGROUNDS,
        sources: LAKE_COUNTRY_SOURCES,
    },
    CampgroundRegion {
        area: "Vernon / Enderby",
        description:
            "Northern Okanagan options around Swan Lake, Silver Star and the Shuswap corridor.",
        campgrounds: VERNON_CAMPGROUNDS,
        sources: VERNON_SOURCES,
    },
    CampgroundRegion {
        area: "Summerland / Penticton / Kaleden",
        description: "Beachfront and resort-style stays around Okanagan Lake and Skaha Lake.",
        campgrounds: PENTICTON_CAMPGROUNDS,
        sources: PENTICTON_SOURCES,
    },
    CampgroundRegion {
        area: "Oliver / Osoyoos",
        description:
            "Warm-climate South Okanagan stays, including several large full-service RV resorts.",
        campgrounds: OSOYOOS_CAMPGROUNDS,
        sources: OSOYOOS_SOURCES,
    },
];

#[component]
pub fn ParksInOurRange() -> Element {
    rsx! {
        section { class: "parks-hero",
            div { class: "parks-hero-copy",
                div { class: "parks-eyebrow", "OKANAGAN & SHUSWAP CAMPING" }
                h1 { "Parks in Our Range" }
                p {
                    "Find a beautiful place to stay, then let us deliver and set up your RV at established campgrounds across the Okanagan and Shuswap."
                }
            }
            div { class: "parks-range-card",
                div { class: "parks-range-icon",
                    Icon { name: "truck", size: 22, color: "var(--vl-accent)" }
                }
                div {
                    strong { "Up to 150 km one way" }
                    span { "Final eligibility is confirmed by road distance when you request a quote." }
                }
            }
        }

        div { class: "parks-page",
            section { class: "parks-section",
                div { class: "parks-section-head",
                    div {
                        div { class: "parks-eyebrow", "PROVINCIAL PARKS" }
                        h2 { "Ten paved-road destinations" }
                    }
                    p {
                        "Ten established campgrounds selected for paved-road access, memorable scenery and a comfortable delivered-RV stay. Open any card for official site details and alerts."
                    }
                }
                div { class: "parks-grid",
                    for (index, park) in PROVINCIAL_PARKS.iter().enumerate() {
                        article {
                            key: "{park.url}",
                            class: "park-card",
                            div { class: "park-card-media",
                                img {
                                    src: park.image,
                                    alt: park.image_alt,
                                    loading: "lazy",
                                    decoding: "async",
                                }
                                span { class: "park-number", "{index + 1:02}" }
                                span { class: "park-area",
                                    Icon { name: "map-pin", size: 14, color: "currentColor" }
                                    "{park.area}"
                                }
                            }
                            div { class: "park-card-body",
                                h3 { "{park.name}" }
                                p { "{park.description}" }
                                span { class: "park-highlights", "{park.highlights}" }
                                div { class: "park-card-actions",
                                    if let Some(slug) = park.detail_slug {
                                        Link {
                                            class: "park-link park-guide-link",
                                            to: Route::ParkDetail { slug: slug.to_string() },
                                            "RV delivery guide"
                                            Icon { name: "arrow-right", size: 16, color: "currentColor" }
                                        }
                                    }
                                    a {
                                        class: "park-link",
                                        href: park.url,
                                        target: "_blank",
                                        rel: "noopener noreferrer",
                                        aria_label: "View {park.name} on BC Parks (opens in a new tab)",
                                        "BC Parks"
                                        Icon { name: "arrow-up-right", size: 16, color: "currentColor" }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            section { class: "parks-section campground-section",
                div { class: "parks-section-head",
                    div {
                        div { class: "parks-eyebrow", "MORE PLACES TO CAMP" }
                        h2 { "Private & municipal campgrounds" }
                    }
                    p {
                        "Looking for full hookups, marina access or resort-style facilities? Start with these regional shortlists, then confirm site length and availability directly with the campground."
                    }
                }
                div { class: "campground-grid",
                    for region in CAMPGROUND_REGIONS {
                        article { key: "{region.area}", class: "campground-card",
                            div { class: "campground-card-head",
                                span { class: "campground-card-icon",
                                    Icon { name: "map-pin", size: 15, color: "currentColor" }
                                }
                                div {
                                    h3 { "{region.area}" }
                                    p { "{region.description}" }
                                }
                            }
                            ul {
                                for campground in region.campgrounds {
                                    li { key: "{campground}", "{campground}" }
                                }
                            }
                            div { class: "campground-sources",
                                for source in region.sources {
                                    a {
                                        key: "{source.url}",
                                        href: source.url,
                                        target: "_blank",
                                        rel: "noopener noreferrer",
                                        "{source.label}"
                                        Icon { name: "arrow-up-right", size: 14, color: "currentColor" }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            section { class: "parks-planning",
                div { class: "parks-planning-copy",
                    div { class: "parks-eyebrow", "BEFORE YOU BOOK" }
                    h2 { "Check the route, the site and the latest alerts" }
                    p {
                        "This guide uses a 120 km straight-line radius from 155 Potterton Rd. Road distances can be longer, including trips toward Osoyoos and parts of the Shuswap. VL Rental confirms delivery by the actual road route and a maximum 150 km one-way distance."
                    }
                    p { class: "parks-exclusions",
                        strong { "Not included in this guide: " }
                        "Mabel Lake Provincial Park because of the approximately 1 km gravel section before the campground; Chute Lake, Darke Lake and forestry or recreation sites; and Willow Creek and Todd’s RV, which are closed."
                    }
                }
                div { class: "parks-planning-actions",
                    a {
                        class: "parks-primary-link",
                        href: "https://camping.bcparks.ca/",
                        target: "_blank",
                        rel: "noopener noreferrer",
                        "Check BC Parks reservations"
                        Icon { name: "arrow-up-right", size: 17, color: "currentColor" }
                    }
                    Link { class: "parks-secondary-link", to: Route::Delivery {},
                        "See how delivery works"
                        Icon { name: "arrow-right", size: 17, color: "currentColor" }
                    }
                }
            }
        }
    }
}

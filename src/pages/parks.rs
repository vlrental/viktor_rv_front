use dioxus::prelude::*;

use crate::{components::Icon, Route};

#[derive(Clone, Copy)]
struct Park {
    name: &'static str,
    area: &'static str,
    description: &'static str,
    url: &'static str,
}

#[derive(Clone, Copy)]
struct SourceLink {
    label: &'static str,
    url: &'static str,
}

#[derive(Clone, Copy)]
struct CampgroundRegion {
    area: &'static str,
    campgrounds: &'static [&'static str],
    sources: &'static [SourceLink],
}

const PROVINCIAL_PARKS: [Park; 10] = [
    Park {
        name: "Bear Creek",
        area: "West Kelowna",
        description: "Lakeside camping on the west side of Okanagan Lake, close to Kelowna.",
        url: "https://bcparks.ca/bear-creek-park/",
    },
    Park {
        name: "Fintry",
        area: "Westside Road",
        description: "North of Kelowna, with waterfront camping, beaches and nearby trails.",
        url: "https://bcparks.ca/fintry-park/",
    },
    Park {
        name: "Ellison",
        area: "Vernon",
        description: "A forested Okanagan Lake campground just outside Vernon.",
        url: "https://bcparks.ca/ellison-park/",
    },
    Park {
        name: "Kekuli Bay",
        area: "South of Vernon",
        description: "A hillside campground above Kalamalka Lake with direct highway access.",
        url: "https://bcparks.ca/kekuli-bay-park/",
    },
    Park {
        name: "Okanagan Lake Provincial Park",
        area: "Summerland",
        description: "Two large campgrounds, North and South, along Highway 97 near Summerland.",
        url: "https://bcparks.ca/okanagan-lake-park/",
    },
    Park {
        name: "sx̌ʷəx̌ʷnitkʷ / Okanagan Falls",
        area: "Okanagan Falls",
        description: "In Okanagan Falls, approximately 500 metres from Highway 97.",
        url: "https://bcparks.ca/sxwexwnitkw-park/",
    },
    Park {
        name: "Vaseux Lake",
        area: "South Okanagan",
        description: "On Highway 97 between Okanagan Falls and Oliver.",
        url: "https://bcparks.ca/vaseux-lake-park/",
    },
    Park {
        name: "sẁiẁs / Haynes Point",
        area: "Osoyoos",
        description: "A waterfront campground at the southern edge of this guide's map radius.",
        url: "https://bcparks.ca/swiws-park/",
    },
    Park {
        name: "Shuswap Lake",
        area: "Scotch Creek",
        description: "A popular Shuswap campground reached by paved roads.",
        url: "https://bcparks.ca/shuswap-lake-park/",
    },
    Park {
        name: "Herald",
        area: "Tappen / Shuswap",
        description: "A large beach campground on Shuswap Lake near Tappen.",
        url: "https://bcparks.ca/herald-park/",
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
        campgrounds: LAKE_COUNTRY_CAMPGROUNDS,
        sources: LAKE_COUNTRY_SOURCES,
    },
    CampgroundRegion {
        area: "Vernon / Enderby",
        campgrounds: VERNON_CAMPGROUNDS,
        sources: VERNON_SOURCES,
    },
    CampgroundRegion {
        area: "Summerland / Penticton / Kaleden",
        campgrounds: PENTICTON_CAMPGROUNDS,
        sources: PENTICTON_SOURCES,
    },
    CampgroundRegion {
        area: "Oliver / Osoyoos",
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
                    "Plan a delivered RV stay at established provincial, private and municipal campgrounds across the region."
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
                        "Use the official park links for campsite details, vehicle limits, seasonal dates and current advisories."
                    }
                }
                div { class: "parks-grid",
                    for (index, park) in PROVINCIAL_PARKS.iter().enumerate() {
                        a {
                            key: "{park.url}",
                            class: "park-card",
                            href: park.url,
                            target: "_blank",
                            rel: "noopener noreferrer",
                            aria_label: "View {park.name} on BC Parks (opens in a new tab)",
                            div { class: "park-card-top",
                                span { class: "park-number", "{index + 1:02}" }
                                span { class: "park-area",
                                    Icon { name: "map-pin", size: 14, color: "currentColor" }
                                    "{park.area}"
                                }
                            }
                            h3 { "{park.name}" }
                            p { "{park.description}" }
                            span { class: "park-link",
                                "View on BC Parks"
                                Icon { name: "arrow-up-right", size: 16, color: "currentColor" }
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
                        "A regional shortlist of larger RV parks and campgrounds with practical road access. Confirm site length and availability directly with each campground."
                    }
                }
                div { class: "campground-grid",
                    for region in CAMPGROUND_REGIONS {
                        article { key: "{region.area}", class: "campground-card",
                            h3 { "{region.area}" }
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

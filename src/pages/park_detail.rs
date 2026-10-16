use dioxus::prelude::*;
use std::{collections::HashMap, sync::OnceLock};

use crate::{api, components::Icon, Route};

const PARK_DETAIL_PREVIEWS_JSON: &str = include_str!("../../content/park_detail_previews.json");

#[derive(serde::Deserialize)]
struct ParkPreviewManifest {
    source_width: u32,
    variants: Vec<ParkPreviewVariant>,
}

#[derive(serde::Deserialize)]
struct ParkPreviewVariant {
    filename: String,
    width: u32,
}

fn park_preview_manifest() -> &'static HashMap<String, ParkPreviewManifest> {
    static PREVIEWS: OnceLock<HashMap<String, ParkPreviewManifest>> = OnceLock::new();
    PREVIEWS.get_or_init(|| {
        serde_json::from_str(PARK_DETAIL_PREVIEWS_JSON)
            .expect("content/park_detail_previews.json must remain valid")
    })
}

fn park_image_srcset_for_base(slug: &str, fallback: &str, base_url: &str) -> Option<String> {
    let preview = park_preview_manifest().get(slug)?;
    let mut candidates = preview
        .variants
        .iter()
        .map(|variant| {
            format!(
                "{}/park-previews/{} {}w",
                base_url.trim_end_matches('/'),
                variant.filename,
                variant.width
            )
        })
        .collect::<Vec<_>>();
    candidates.push(format!("{fallback} {}w", preview.source_width));
    Some(candidates.join(", "))
}

fn park_image_srcset(slug: &str, fallback: &str) -> Option<String> {
    park_image_srcset_for_base(slug, fallback, &api::frontend_base_url())
}

const PARK_IMAGE_SIZES: &str =
    "(max-width: 860px) max(570px, 100vw), (max-width: 960px) max(676px, 100vw), 1600px";

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
            introduction: "Plan a delivered RV stay on the west shore of Okanagan Lake, subject to approval of both your reserved campsite and the selected VL Rental RV.",
            image: IMG_BEAR_CREEK,
            image_alt: "Okanagan Lake shoreline at Bear Creek Provincial Park",
            official_url: "https://bcparks.ca/bear-creek-park/",
            planning_title: "Reserve the park and RV separately",
            planning_copy: "Bear Creek is reached from Kelowna via Westside Road and combines an Okanagan Lake shoreline with trails into Bear Creek Canyon. Summer camping requires a separate BC Parks reservation; reserving a VL Rental RV does not reserve a campsite, and a park reservation does not guarantee that every trailer will fit.",
            access_note: "Send us the site number, the full bumper-to-bumper trailer length permitted for that site and the hookups shown on your actual reservation. Confirm the services for your booked site. We will check the Westside Road access and RV fit. Air conditioning requires a compatible 30-amp or 50-amp hookup.",
        }),
        "fintry" => Some(ParkGuide {
            name: "Fintry Provincial Park",
            area: "Westside Road, Okanagan",
            introduction: "Plan a delivered RV stay at this waterfront park north of Kelowna. Arrange a suitable campsite directly with the campground, then choose an available VL Rental RV for your dates.",
            image: IMG_FINTRY,
            image_alt: "Waterfront and mountains at Fintry Provincial Park",
            official_url: "https://bcparks.ca/fintry-park/",
            planning_title: "Waterfront camping on Westside Road",
            planning_copy: "Fintry offers a long Okanagan Lake shoreline, walking routes and access to the historic estate and nearby waterfall. Because the destination is reached along Westside Road, final delivery approval is based on the actual route and campsite details.",
            access_note: "Fintry Delta Road leaves Westside Road about 34 km north of Kelowna and 49 km south of Vernon. Reserve the campsite separately, then send the loop, site number, full bumper-to-bumper trailer allowance and listed hookups for that booked site. We confirm route access and RV fit; air conditioning requires compatible 30-amp or 50-amp service.",
        }),
        "ellison" => Some(ParkGuide {
            name: "Ellison Provincial Park",
            area: "Vernon, Okanagan",
            introduction: "Enjoy a forested campsite near Okanagan Lake without towing an RV. Arrange a suitable campsite directly with the campground and VL Rental can deliver and set up an available trailer for your stay.",
            image: IMG_ELLISON,
            image_alt: "Rocky Okanagan Lake shoreline at Ellison Provincial Park",
            official_url: "https://bcparks.ca/ellison-park/",
            planning_title: "A forested base near Vernon",
            planning_copy: "Ellison is known for sheltered swimming coves and lakeside trails in a quieter forest setting near Vernon. It works well for guests who want an Okanagan Lake stay without collecting or towing a trailer themselves.",
            access_note: "From Vernon, follow 25th Avenue for about 16 km from the Highway 97 junction. Reserve the campsite separately, then send its number, full bumper-to-bumper trailer allowance and listed hookups. We confirm route access and RV fit; air conditioning requires compatible 30-amp or 50-amp service.",
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
            access_note: "The park is about 11 km south of Vernon on the west side of Kalamalka Lake; the campground is about 3 km beyond the Highway 97 turn-off, past two park-boundary signs and the campground sign. Reserve the campsite separately, then send its number, full bumper-to-bumper trailer allowance and listed hookups. We confirm access and RV fit; air conditioning requires compatible 30-amp or 50-amp service.",
        }),
        "okanagan-lake" => Some(ParkGuide {
            name: "Okanagan Lake Provincial Park",
            area: "North of Summerland",
            introduction: "Plan delivery to an approved site in the North or South campground, subject to confirmation that the reserved campsite and selected VL Rental RV are compatible.",
            image: IMG_OKANAGAN_LAKE,
            image_alt: "Okanagan Lake Provincial Park campground near Summerland",
            official_url: "https://bcparks.ca/okanagan-lake-park/",
            planning_title: "Two campgrounds beside Okanagan Lake",
            planning_copy: "Okanagan Lake Provincial Park is about 11 km north of Summerland on Highway 97 and has distinct North and South campgrounds. This guide is for those two park campgrounds, not every campground on Okanagan Lake and not Okanagan Mountain Provincial Park.",
            access_note: "Send whether the reservation is in the North or South campground, plus the loop, site number, full bumper-to-bumper trailer length permitted for that site and its listed hookups. We use those details to confirm the delivery address and RV fit; air conditioning requires a compatible 30-amp or 50-amp hookup.",
        }),
        "okanagan-falls" => Some(ParkGuide {
            name: "sx̌ʷəx̌ʷnitkʷ Provincial Park",
            area: "Okanagan Falls",
            introduction: "Stay beside the Okanagan River in Okanagan Falls with a delivered RV ready at your approved campsite.",
            image: IMG_OKANAGAN_FALLS,
            image_alt: "Entrance to sx̌ʷəx̌ʷnitkʷ Provincial Park in Okanagan Falls",
            official_url: "https://bcparks.ca/sxwexwnitkw-park/",
            planning_title: "A compact riverside campground in town",
            planning_copy: "The campground is on Green Lake Road, approximately 500 metres from Highway 97. It is managed by the Osoyoos Indian Band and is culturally significant; guests should review current notices and respect all posted rules and protected areas.",
            access_note: "BC Parks states that campsites are closed to the general public on the third weekend of September each year for the Salmon Feast. Check current park notices before planning your dates. The Green Lake Road entrance is about 500 metres from Highway 97; BC Parks notes a narrow approach with a blind corner. Reserve the campsite separately, then send its number, full bumper-to-bumper trailer allowance and listed hookups. We confirm safe access and RV fit; air conditioning requires compatible 30-amp or 50-amp service.",
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
            access_note: "Vaseux Lake is beside Highway 97 about 25 km south of Penticton. Reserve the campsite separately, then send its number, full bumper-to-bumper trailer allowance and listed hookups. We confirm route access and RV fit; air conditioning requires compatible 30-amp or 50-amp service.",
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
            access_note: "BC Parks limits camping at sẁiẁs to seven nights per calendar year. Check the current park rules before reserving. The park is about 2 km south of Osoyoos, reached from Highway 97 via 32nd Avenue. Reserve the campsite separately, then send its number, full bumper-to-bumper trailer allowance and listed hookups. Air conditioning requires compatible 30-amp or 50-amp service. Delivery is not guaranteed: we must confirm safe access, RV fit and an actual one-way road route within 150 km.",
        }),
        "shuswap-lake" => Some(ParkGuide {
            name: "Shuswap Lake Provincial Park",
            area: "Scotch Creek, Shuswap",
            introduction: "Explore a family campground at Shuswap Lake and ask us to verify the road route and campsite before making RV rental plans.",
            image: IMG_SHUSWAP_LAKE,
            image_alt: "Beach and mountain view at Shuswap Lake Provincial Park",
            official_url: "https://bcparks.ca/shuswap-lake-park/",
            planning_title: "A family beach destination in the Shuswap",
            planning_copy: "This large campground on the north side of Shuswap Lake's main arm is known for its broad beach, warm-water swimming and paved approach. Copper Island is a landmark offshore, not an RV destination. As a longer delivery from Kelowna, the park requires an exact route and campsite check.",
            access_note: "From Highway 1 at Squilax, the campground is reached by another 19 km of paved road. Reserve the campsite separately, then send the loop, site number, full bumper-to-bumper trailer allowance and listed hookups. Air conditioning requires compatible 30-amp or 50-amp service. Delivery is not guaranteed: we must confirm safe access, RV fit and an actual one-way road route within 150 km.",
        }),
        "herald" => Some(ParkGuide {
            name: "Herald Provincial Park",
            area: "Tappen, Shuswap",
            introduction: "Explore a Shuswap Lake campsite near Tappen and request a route and site check before committing to RV delivery.",
            image: IMG_HERALD,
            image_alt: "Margaret Falls trail at Herald Provincial Park",
            official_url: "https://bcparks.ca/herald-park/",
            planning_title: "Beach days and the Margaret Falls trail",
            planning_copy: "Herald is on the west shore of Shuswap Lake near Salmon Arm, with a family beach and a shaded walking route to Margaret Falls. The drive from Kelowna is near the outer portion of our service area, so delivery depends on the actual road distance and safe access to the reserved site.",
            access_note: "Herald Park is 14 km east of Highway 1 at Tappen. Reserve the campsite separately, then send the loop, site number, full bumper-to-bumper trailer allowance and listed hookups. Air conditioning requires compatible 30-amp or 50-amp service. Delivery is not guaranteed: we must confirm safe access, RV fit and an actual one-way road route within 150 km.",
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

    let rentals_href = api::frontend_path("/#home-rentals");
    let park_image = park.image.to_string();
    let park_image_srcset = park_image_srcset(slug.as_str(), &park_image);
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
                        a { class: "parks-primary-link", href: rentals_href,
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
                img {
                    src: park_image,
                    srcset: park_image_srcset.clone(),
                    sizes: park_image_srcset.as_ref().map(|_| PARK_IMAGE_SIZES),
                    alt: park.image_alt,
                    decoding: "async",
                    fetchpriority: "high"
                }
            }

            section { class: "park-detail-content",
                div { class: "park-detail-main",
                    section { class: "park-destination-intro",
                        div { class: "parks-eyebrow", "WHY CAMP HERE" }
                        h2 { "{park.planning_title}" }
                        p { "{park.planning_copy}" }
                        div { class: "park-access-note",
                            Icon { name: "info", size: 18, color: "currentColor" }
                            span {
                                "{park.access_note}"
                                br {}
                                small {
                                    "Source: "
                                    a { href: park.official_url, target: "_blank", rel: "noopener noreferrer", "BC Parks" }
                                    " · checked October 14, 2026"
                                }
                            }
                        }
                    }
                    div { class: "parks-eyebrow", "HOW IT WORKS" }
                    h2 { "Your campsite, our RV and setup" }
                    div { class: "park-steps",
                        div { class: "park-step",
                            span { "01" }
                            div {
                                h3 { "Arrange your campsite" }
                                p { "Arrange a suitable campsite directly with the campground and follow its current booking or site-allocation rules. Keep your campsite details handy." }
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
                    ParkRvComparison {}
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

#[component]
fn ParkRvComparison() -> Element {
    let links = [
        (
            "Rockwood · 21 ft · sleeps 4",
            "/rv/2014-forest-river-rockwood/",
        ),
        (
            "Keystone Bullet · 30 ft · sleeps 8",
            "/rv/2015-keystone-bullet/",
        ),
        ("Compare all RV rentals", "/#home-rentals"),
    ];
    rsx! {
        section { class: "park-destination-intro park-rv-comparison",
            h2 { "Compare RVs for your campsite" }
            p { "These are rental options, not a guarantee that they fit this park. Confirm the site’s permitted full trailer length and delivery route." }
            for (label, path) in links {
                p {
                    a {
                        class: "park-detail-contact",
                        href: crate::api::frontend_path(path),
                        style: "text-decoration: underline; text-underline-offset: 4px;",
                        "{label}"
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn researched_park_guides_keep_confirmed_access_and_site_requirements() {
        let expected = [
            ("fintry", "34 km north of Kelowna"),
            ("ellison", "25th Avenue for about 16 km"),
            ("kekuli-bay", "about 3 km beyond the Highway 97 turn-off"),
            ("okanagan-falls", "about 500 metres from Highway 97"),
            ("vaseux-lake", "about 25 km south of Penticton"),
            ("swiws", "about 2 km south of Osoyoos"),
            ("shuswap-lake", "another 19 km of paved road"),
            ("herald", "14 km east of Highway 1 at Tappen"),
        ];
        for (slug, access_fact) in expected {
            let guide = guide_for(slug).expect("researched park guide");
            assert!(guide.access_note.contains(access_fact));
            assert!(guide
                .access_note
                .contains("Reserve the campsite separately"));
            assert!(guide.access_note.contains("full bumper-to-bumper"));
            assert!(guide.access_note.contains("listed hookups"));
            assert!(guide.access_note.contains("30-amp or 50-amp"));
            assert!(guide.official_url.starts_with("https://bcparks.ca/"));
        }
        for slug in ["swiws", "shuswap-lake", "herald"] {
            let note = guide_for(slug).expect("outer-range guide").access_note;
            assert!(note.contains("Delivery is not guaranteed"));
            assert!(note.contains("actual one-way road route within 150 km"));
        }
        assert!(guide_for("shuswap-lake")
            .expect("Shuswap guide")
            .planning_copy
            .contains("Copper Island is a landmark offshore, not an RV destination"));
        let falls = guide_for("okanagan-falls").expect("Okanagan Falls guide");
        assert!(falls.access_note.starts_with(
            "BC Parks states that campsites are closed to the general public on the third weekend of September each year for the Salmon Feast."
        ));
        assert_eq!(falls.official_url, "https://bcparks.ca/sxwexwnitkw-park/");
        let swiws = guide_for("swiws").expect("swiws guide");
        assert!(swiws
            .access_note
            .starts_with("BC Parks limits camping at sẁiẁs to seven nights per calendar year."));
        assert_eq!(swiws.official_url, "https://bcparks.ca/swiws-park/");
    }

    #[test]
    fn only_bear_creek_uses_the_verified_responsive_hero_candidates() {
        let srcset = park_image_srcset_for_base(
            "bear-creek",
            "/assets/park-bear-creek.jpg",
            "https://vlrental.ca",
        )
        .expect("Bear Creek responsive hero");
        assert!(srcset.contains("park-bear-creek-800-9a21f3645d45295e.webp 800w"));
        assert!(srcset.contains("park-bear-creek-1161-86b723844c054826.webp 1161w"));
        assert!(srcset.ends_with("/assets/park-bear-creek.jpg 1600w"));
        assert_eq!(park_preview_manifest()["bear-creek"].source_width, 1600);
        assert!(park_image_srcset_for_base(
            "fintry",
            "/assets/park-fintry.jpg",
            "https://vlrental.ca"
        )
        .is_none());
        assert_eq!(
            PARK_IMAGE_SIZES,
            "(max-width: 860px) max(570px, 100vw), (max-width: 960px) max(676px, 100vw), 1600px"
        );
    }
}

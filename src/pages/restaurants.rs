use dioxus::prelude::*;

use crate::components::Icon;

const IMG_WATERFRONT: Asset = asset!(
    "/assets/img/rest-waterfront.webp",
    AssetOptions::image().with_jpg()
);
const IMG_BRICK: Asset = asset!(
    "/assets/img/rest-brick.webp",
    AssetOptions::image().with_jpg()
);
const IMG_JOEY: Asset = asset!(
    "/assets/img/rest-joey.webp",
    AssetOptions::image().with_jpg()
);
const IMG_BERNIES: Asset = asset!(
    "/assets/img/rest-bernies.webp",
    AssetOptions::image().with_jpg()
);
const IMG_ROMA: Asset = asset!(
    "/assets/img/rest-roma.webp",
    AssetOptions::image().with_jpg()
);
const IMG_OLDVINES: Asset = asset!(
    "/assets/img/rest-oldvines.webp",
    AssetOptions::image().with_jpg()
);
const IMG_TERRACE: Asset = asset!(
    "/assets/img/rest-terrace.webp",
    AssetOptions::image().with_jpg()
);
const IMG_KINGTAPS: Asset = asset!(
    "/assets/img/rest-kingtaps.webp",
    AssetOptions::image().with_jpg()
);
const IMG_HUMO: Asset = asset!(
    "/assets/img/rest-humo.webp",
    AssetOptions::image().with_jpg()
);
const IMG_FRANKIE: Asset = asset!(
    "/assets/img/rest-frankie.webp",
    AssetOptions::image().with_jpg()
);

/// Карточка ресторана (данные фрейма Restaurants).
struct RestaurantCard {
    id: &'static str,
    img: Asset,
    category: &'static str,
    title: &'static str,
    desc: &'static str,
    address: &'static str,
    phone: &'static str,
}

fn restaurant_cards() -> Vec<RestaurantCard> {
    vec![
        RestaurantCard {
            id: "brick",
            img: IMG_BRICK,
            category: "Wine bar & share plates",
            title: "Salt & Brick",
            desc: "Cozy Bernard Ave spot with an ever-changing seasonal menu.",
            address: "243 Bernard Ave, Kelowna",
            phone: "+1 778-484-3234",
        },
        RestaurantCard {
            id: "joey",
            img: IMG_JOEY,
            category: "Upscale casual",
            title: "JOEY Kelowna",
            desc: "Globally inspired menu, sleek space and a lively patio.",
            address: "2475 BC-97 #300, Kelowna",
            phone: "+1 250-860-8999",
        },
        RestaurantCard {
            id: "bernies",
            img: IMG_BERNIES,
            category: "Dinner & entertainment",
            title: "Bernie's Supper Club",
            desc: "Supper club, craft cocktails and a boutique cinema in one.",
            address: "353 Bernard Ave, Kelowna",
            phone: "+1 778-484-9836",
        },
        RestaurantCard {
            id: "roma",
            img: IMG_ROMA,
            category: "Italian",
            title: "Roma Nord Bistro",
            desc: "Traditional Italian built on fresh Okanagan produce.",
            address: "421 Cawston Ave, Kelowna",
            phone: "+1 236-420-0125",
        },
        RestaurantCard {
            id: "oldvines",
            img: IMG_OLDVINES,
            category: "Winery dining",
            title: "Old Vines Restaurant",
            desc: "Wine-led contemporary cuisine from seasonal BC farms.",
            address: "3303 Boucherie Rd, Kelowna",
            phone: "+1 250-769-2500",
        },
        RestaurantCard {
            id: "terrace",
            img: IMG_TERRACE,
            category: "Winery dining",
            title: "The Terrace at Mission Hill",
            desc: "Local, seasonal and simple — dining above the vines.",
            address: "1730 Mission Hill Rd, Kelowna",
            phone: "+1 250-768-6467",
        },
        RestaurantCard {
            id: "kingtaps",
            img: IMG_KINGTAPS,
            category: "Waterfront & taps",
            title: "King Taps Kelowna",
            desc: "Lakeside energy, big shareable menu and stunning views.",
            address: "1352 Water St, Kelowna",
            phone: "+1 778-738-3989",
        },
        RestaurantCard {
            id: "humo",
            img: IMG_HUMO,
            category: "Global & local",
            title: "HUMO Kelowna",
            desc: "Globally inspired, locally curated dining experience.",
            address: "210 Lawrence Ave, Kelowna",
            phone: "+1 250-826-8218",
        },
        RestaurantCard {
            id: "frankie",
            img: IMG_FRANKIE,
            category: "Plant-based",
            title: "Frankie We Salute You!",
            desc: "Plant-based plates, regional wine and craft beer.",
            address: "1717 Harvey Ave #6, Kelowna",
            phone: "+1 236-420-3338",
        },
    ]
}

#[component]
pub fn Restaurants() -> Element {
    let cards = restaurant_cards();
    rsx! {
        section { class: "rs-hero",
            div { class: "rs-hero-eyebrow", "RESTAURANTS" }
            h1 { class: "rs-hero-title", "Where to eat around the lake" }
            p { class: "rs-hero-sub",
                "Our local picks by vibe — from lakeside patios to craft breweries. Ask us for current favourites when you book."
            }
        }
        section { class: "rs-wrap",
            article { class: "rs-featured",
                div {
                    class: "rs-featured-img",
                    style: "background-image: url('{IMG_WATERFRONT}');",
                }
                div { class: "rs-featured-body",
                    div { class: "rs-featured-badge",
                        Icon { name: "award", size: 13, color: "var(--vl-forest)" }
                        span { "Best Okanagan Restaurant — 12 years running" }
                    }
                    h2 { class: "rs-featured-title", "Waterfront Wines" }
                    p { class: "rs-featured-desc",
                        "Chef-sommelier Mark Filatow's fresh, local plates — a Kelowna institution and award favourite."
                    }
                    div { class: "rs-featured-meta",
                        div { class: "rs-meta-row",
                            Icon { name: "map-pin", size: 13, color: "var(--vl-muted)" }
                            span { "1180 Sunset Dr #104, Kelowna" }
                        }
                        div { class: "rs-meta-row",
                            Icon { name: "phone", size: 13, color: "var(--vl-muted)" }
                            span { "+1 250-979-1222" }
                        }
                    }
                }
            }
            div { class: "rs-grid",
                for card in cards {
                    article { key: "{card.id}", class: "rs-card",
                        div {
                            class: "rs-card-img",
                            style: "background-image: url('{card.img}');",
                        }
                        div { class: "rs-card-body",
                            div { class: "rs-card-cat", "{card.category}" }
                            h2 { class: "rs-card-title", "{card.title}" }
                            p { class: "rs-card-desc", "{card.desc}" }
                            div { class: "rs-card-meta",
                                div { class: "rs-meta-row",
                                    Icon { name: "map-pin", size: 13, color: "var(--vl-muted)" }
                                    span { "{card.address}" }
                                }
                                div { class: "rs-meta-row",
                                    Icon { name: "phone", size: 13, color: "var(--vl-muted)" }
                                    span { "{card.phone}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

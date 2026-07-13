use dioxus::prelude::*;

const CSS: Asset = asset!("/assets/css/attractions.css");

const IMG_LAKE: Asset = asset!("/assets/img/attr-lake.webp");
const IMG_WINE: Asset = asset!("/assets/img/attr-wine.webp");
const IMG_HIKING: Asset = asset!("/assets/img/attr-hiking.webp");
const IMG_BEACH: Asset = asset!("/assets/img/attr-beach.webp");
const IMG_FAMILY: Asset = asset!("/assets/img/attr-family.webp");
const IMG_FISHING: Asset = asset!("/assets/img/attr-fishing.webp");

/// Карточка достопримечательности (данные фрейма Attractions).
struct AttractionCard {
    id: &'static str,
    img: Asset,
    category: &'static str,
    title: &'static str,
    desc: &'static str,
}

fn attraction_cards() -> Vec<AttractionCard> {
    vec![
        AttractionCard {
            id: "lake",
            img: IMG_LAKE,
            category: "Beaches & Water",
            title: "Okanagan Lake Adventures",
            desc: "Discover the beauty of Okanagan Lake — cruise, swim and play on 135 km of water.",
        },
        AttractionCard {
            id: "wine",
            img: IMG_WINE,
            category: "Food & Wine",
            title: "Wine Country Tours",
            desc: "Explore Kelowna's wineries with expert guides and lakeside tastings.",
        },
        AttractionCard {
            id: "hiking",
            img: IMG_HIKING,
            category: "Trails",
            title: "Hiking & Outdoor Fun",
            desc: "Kelowna is a hiker's paradise — trails for every level, minutes from town.",
        },
        AttractionCard {
            id: "beach",
            img: IMG_BEACH,
            category: "Beaches & Water",
            title: "Beaches & Parks",
            desc: "Some of the best beaches in British Columbia, perfect for family days.",
        },
        AttractionCard {
            id: "family",
            img: IMG_FAMILY,
            category: "Family",
            title: "Family-Friendly Attractions",
            desc: "Kelowna fun for every generation — activities the whole crew will love.",
        },
        AttractionCard {
            id: "fishing",
            img: IMG_FISHING,
            category: "On the Water",
            title: "Fishing & Water Fun",
            desc: "Catch the big one and enjoy lakeside recreation all season.",
        },
    ]
}

#[component]
pub fn Attractions() -> Element {
    let cards = attraction_cards();
    rsx! {
        document::Link { rel: "stylesheet", href: CSS }
        section { class: "at-hero",
            div { class: "at-hero-eyebrow", "ATTRACTIONS" }
            h1 { class: "at-hero-title", "Explore the Okanagan" }
            p { class: "at-hero-sub",
                "Local favourites for your trip — lakes, trails, views and vineyards, all within reach of your rental."
            }
        }
        section { class: "at-grid",
            for card in cards {
                article { key: "{card.id}", class: "at-card",
                    div {
                        class: "at-card-img",
                        style: "background-image: url('{card.img}');",
                    }
                    div { class: "at-card-body",
                        div { class: "at-card-cat", "{card.category}" }
                        h2 { class: "at-card-title", "{card.title}" }
                        p { class: "at-card-desc", "{card.desc}" }
                    }
                }
            }
        }
    }
}

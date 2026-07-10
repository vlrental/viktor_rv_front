use dioxus::prelude::*;

use crate::components::{Icon, ListingCard};
use crate::data::catalog_listings;

#[component]
pub fn Catalog() -> Element {
    let listings = catalog_listings();
    rsx! {
        section { class: "cat-header",
            h1 { class: "sec-title", "Explore the fleet" }
            p { class: "cat-sub", "RVs and trailers ready for your Okanagan adventure — book in minutes." }
            div { class: "cat-search",
                div { class: "cat-search-field",
                    div { class: "cat-search-label", "LOCATION" }
                    div { class: "cat-search-value",
                        Icon { name: "map-pin", size: 15, color: "var(--vl-forest)" }
                        span { "Kelowna, BC" }
                    }
                }
                div { class: "cat-search-divider" }
                div { class: "cat-search-field",
                    div { class: "cat-search-label", "DATES" }
                    div { class: "cat-search-value",
                        Icon { name: "calendar", size: 15, color: "var(--vl-forest)" }
                        span { "Jul 12 – 15" }
                    }
                }
                div { class: "cat-search-divider" }
                div { class: "cat-search-field",
                    div { class: "cat-search-label", "GUESTS" }
                    div { class: "cat-search-value",
                        Icon { name: "users", size: 15, color: "var(--vl-forest)" }
                        span { "2 guests" }
                    }
                }
                button { class: "cat-search-btn",
                    Icon { name: "search", size: 17, color: "var(--vl-white)" }
                    span { "Search" }
                }
            }
        }
        section { class: "cat-body",
            Filters {}
            div { class: "cat-results",
                div { class: "cat-results-head",
                    div { class: "cat-count", "6 stays in Kelowna" }
                    button { class: "cat-sort",
                        Icon { name: "arrow-up-down", size: 14, color: "var(--vl-ink)" }
                        span { "Sort: Recommended" }
                        Icon { name: "chevron-down", size: 14, color: "var(--vl-ink)" }
                    }
                }
                div { class: "cat-grid",
                    for l in listings {
                        ListingCard { key: "{l.title}", listing: l }
                    }
                }
            }
        }
    }
}

#[component]
fn Filters() -> Element {
    rsx! {
        aside { class: "cat-filters",
            div { class: "filter-group",
                div { class: "filter-head", "Type" }
                FilterCheck { label: "RVs", checked: true }
                FilterCheck { label: "Cooler trailers", checked: false }
            }
            div { class: "filter-divider" }
            div { class: "filter-group", style: "gap: 14px;",
                div { class: "filter-head", "Price / night" }
                div { class: "price-track",
                    div { class: "price-fill" }
                    div { class: "price-handle", style: "left: 26px;" }
                    div { class: "price-handle", style: "left: 170px;" }
                }
                div { class: "price-labels",
                    span { "$125" }
                    span { "$185+" }
                }
            }
            div { class: "filter-divider" }
            div { class: "filter-group",
                div { class: "filter-head", "Sleeps / capacity" }
                div { class: "sleep-chips",
                    for (label, active) in [("2", false), ("4", true), ("6", false), ("8", true), ("10+", false)] {
                        button {
                            key: "sleeps-{label}",
                            class: "sleep-chip",
                            style: if active {
                                "background-color: var(--vl-forest); border-color: var(--vl-forest); color: var(--vl-white);"
                            } else {
                                "background-color: var(--vl-white); border-color: var(--vl-hair); color: var(--vl-muted);"
                            },
                            "{label}"
                        }
                    }
                }
            }
            div { class: "filter-divider" }
            div { class: "filter-group",
                div { class: "filter-head", "Features" }
                FilterCheck { label: "Delivery available", checked: true }
                FilterCheck { label: "Air conditioning", checked: false }
                FilterCheck { label: "Pet friendly", checked: false }
            }
        }
    }
}

#[component]
fn FilterCheck(label: &'static str, checked: bool) -> Element {
    rsx! {
        div { class: "filter-check",
            div {
                class: "filter-box",
                style: if checked {
                    "background-color: var(--vl-forest); border-color: var(--vl-forest);"
                } else {
                    "background-color: var(--vl-white); border-color: var(--vl-hair);"
                },
                if checked {
                    Icon { name: "check", size: 13, color: "var(--vl-white)" }
                }
            }
            span { "{label}" }
        }
    }
}

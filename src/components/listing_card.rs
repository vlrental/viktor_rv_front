use dioxus::prelude::*;

use crate::components::Icon;
use crate::data::Listing;
use crate::Route;

/// Карточка листинга из дизайна «Listing Card» (bnVxN) — RV.
#[component]
pub fn ListingCard(listing: Listing) -> Element {
    let to = Route::RvDetail { slug: listing.slug.to_string() };
    rsx! {
        Link { class: "listing-card", to,
            div {
                class: "lc-image",
                style: "background-image: url('{listing.image}');",
                div { class: "lc-badge",
                    Icon { name: "star", size: 13, color: "var(--vl-accent)" }
                    span { "{listing.badge}" }
                }
                button { class: "lc-heart", aria_label: "Save to favourites",
                    Icon { name: "heart", size: 18, color: "var(--vl-forest)" }
                }
            }
            div { class: "lc-body",
                div { class: "lc-title-row",
                    div { class: "lc-title", "{listing.title}" }
                    div { class: "lc-rating",
                        Icon { name: "star", size: 14, color: "var(--vl-accent)" }
                        span { "{listing.rating}" }
                    }
                }
                div { class: "lc-meta", "{listing.meta}" }
                div { class: "lc-price-row",
                    span { class: "lc-price", "{listing.price}" }
                    span { class: "lc-per", "{listing.per}" }
                }
            }
        }
    }
}

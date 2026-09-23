use dioxus::prelude::*;

use super::Icon;

const GOOGLE_PROFILE: &str =
    "https://www.google.com/maps/place/VL+Rental.ca/data=!4m2!3m1!1s0x0:0x1cbc237dc1c1577c";
const WRITE_REVIEW: &str = "https://g.page/r/CXxXwcF9I7wcEBM/review";

// Public Google review excerpts, checked against VL Rental.ca on 2026-09-23.
// This is a dated editorial snapshot, not an automatically refreshed feed.
const REVIEWS: [(&str, &str, &str, &str); 3] = [
    ("Irene Rotnow", "Excellent service, friendly and respectful staff, and everything was well organized and on time.", "en", "https://www.google.com/maps/contrib/110810819511337150081/reviews"),
    ("Emilia Moor", "Toller Service! Sehr nettes Personal, wir haben uns sehr wohl gefühlt. Gerne wieder!", "de", "https://www.google.com/maps/contrib/100936381199612856103/reviews"),
    ("Melanie", "Perfect service, would do it again!!", "en", "https://www.google.com/maps/contrib/111717682799025743173/reviews"),
];

#[component]
pub fn GoogleReviews() -> Element {
    rsx! {
        section { class: "ab-reviews google-reviews", aria_labelledby: "google-reviews-title",
            div { class: "ab-reviews-head",
                div {
                    div { class: "eyebrow", "FROM OUR GUESTS" }
                    h2 { id: "google-reviews-title", class: "ab-reviews-title", "Google Reviews" }
                }
                a { class: "google-reviews-rating", href: GOOGLE_PROFILE, target: "_blank", rel: "noopener noreferrer",
                    "5.0 / 5 · 7 Google reviews"
                }
            }
            div { class: "google-reviews-grid",
                for (name, quote, language, profile) in REVIEWS {
                    article { key: "{name}", class: "ab-review",
                        div { class: "ab-stars", role: "img", aria_label: "5 out of 5 stars",
                            for i in 0..5 {
                                span { key: "{i}", aria_hidden: "true",
                                    Icon { name: "star", size: 15, color: "var(--vl-accent)" }
                                }
                            }
                        }
                        blockquote { class: "ab-quote", lang: language, "“{quote}”" }
                        div { class: "ab-who",
                            a { class: "ab-who-n", href: profile, target: "_blank", rel: "noopener noreferrer", "{name}" }
                            span { class: "ab-who-d", "Google" }
                        }
                    }
                }
            }
            p { class: "google-reviews-note",
                "Selected Google review excerpts · Rating checked "
                time { datetime: "2026-09-23", "September 23, 2026" }
            }
            div { class: "google-reviews-links",
                a { href: GOOGLE_PROFILE, target: "_blank", rel: "noopener noreferrer", "Read all reviews on Google" }
                a { href: WRITE_REVIEW, target: "_blank", rel: "noopener noreferrer", "Write a Google review" }
            }
        }
    }
}

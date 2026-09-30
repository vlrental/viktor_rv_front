use crate::api::reviews::{self, GoogleContent, GoogleReview};
use dioxus::prelude::*;

const GOOGLE_PROFILE: &str =
    "https://www.google.com/maps/place/VL+Rental.ca/data=!4m2!3m1!1s0x0:0x1cbc237dc1c1577c";
const WRITE_REVIEW: &str = "https://g.page/r/CXxXwcF9I7wcEBM/review";
const REVIEW_STAR_PATH: &str =
    "M12 2.75l2.84 5.75 6.35.92-4.59 4.48 1.08 6.32L12 18.23l-5.68 2.99 1.08-6.32-4.59-4.48 6.35-.92L12 2.75Z";

// Public Google review excerpts, checked against VL Rental.ca on 2026-09-23.
// This is a dated editorial snapshot, not an automatically refreshed feed.
const REVIEWS: [(&str, &str, &str, &str); 3] = [
    ("Irene Rotnow", "Excellent service, friendly and respectful staff, and everything was well organized and on time.", "en", "https://www.google.com/maps/contrib/110810819511337150081/reviews"),
    ("Emilia Moor", "Toller Service! Sehr nettes Personal, wir haben uns sehr wohl gefühlt. Gerne wieder!", "de", "https://www.google.com/maps/contrib/100936381199612856103/reviews"),
    ("Melanie", "Perfect service, would do it again!!", "en", "https://www.google.com/maps/contrib/111717682799025743173/reviews"),
];

pub fn default_content() -> GoogleContent {
    GoogleContent {
        rating: "5.0".into(),
        count: 7,
        checked_on: "2026-09-23".into(),
        profile_url: GOOGLE_PROFILE.into(),
        write_url: WRITE_REVIEW.into(),
        published: true,
        reviews: REVIEWS
            .iter()
            .map(|(name, body, language, url)| GoogleReview {
                name: (*name).into(),
                body: (*body).into(),
                language: (*language).into(),
                url: (*url).into(),
                rating: 5,
                published: true,
            })
            .collect(),
    }
}

#[component]
pub fn GoogleReviews() -> Element {
    let content = use_resource(|| async { reviews::public_google().await });
    let data = match &*content.read() {
        Some(Ok(Some(data))) => data.clone(),
        Some(Ok(None)) => default_content(),
        Some(Err(_)) => {
            return rsx! { p { class: "google-reviews-note", a { href: GOOGLE_PROFILE, target: "_blank", rel: "noopener noreferrer", "Read our reviews on Google" } } }
        }
        None => return rsx! {},
    };
    if !data.published {
        return rsx! {};
    }
    rsx! {
        section { class: "ab-reviews google-reviews", aria_labelledby: "google-reviews-title",
            div { class: "ab-reviews-head",
                div {
                    div { class: "eyebrow", "FROM OUR GUESTS" }
                    h2 { id: "google-reviews-title", class: "ab-reviews-title", "Google Reviews" }
                }
                a { class: "google-reviews-rating", href: data.profile_url.clone(), target: "_blank", rel: "noopener noreferrer",
                    "{data.rating} / 5 · {data.count} Google reviews"
                }
            }
            div { class: "google-reviews-grid",
                for (index, review) in data.reviews.iter().filter(|r| r.published).enumerate() {
                    article { key: "{index}", class: "ab-review",
                        div { class: "ab-stars", role: "img", aria_label: "{review.rating} out of 5 stars",
                            for i in 0..review.rating {
                                span { key: "{i}", aria_hidden: "true",
                                    svg {
                                        width: "15",
                                        height: "15",
                                        view_box: "0 0 24 24",
                                        fill: "var(--vl-accent)",
                                        path { d: REVIEW_STAR_PATH }
                                    }
                                }
                            }
                        }
                        blockquote { class: "ab-quote", lang: review.language.clone(), "“{review.body}”" }
                        div { class: "ab-who",
                            a { class: "ab-who-n", href: review.url.clone(), target: "_blank", rel: "noopener noreferrer", "{review.name}" }
                            span { class: "ab-who-d", "Google" }
                        }
                    }
                }
            }
            p { class: "google-reviews-note",
                "Selected Google review excerpts · Rating checked "
                time { datetime: data.checked_on.clone(), "{data.checked_on}" }
            }
            div { class: "google-reviews-links",
                a { href: data.profile_url.clone(), target: "_blank", rel: "noopener noreferrer", "Read all reviews on Google" }
                a { href: data.write_url.clone(), target: "_blank", rel: "noopener noreferrer", "Write a Google review" }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn review_star_is_a_closed_filled_shape() {
        assert!(REVIEW_STAR_PATH.ends_with('Z'));
        assert_eq!(REVIEWS.len(), 3);
    }
}

use crate::api::reviews::{self, GoogleContent, GoogleReview};
use dioxus::prelude::*;

const REVIEWS_PER_PAGE: usize = 6;

fn page_range(total: usize, page: usize) -> std::ops::Range<usize> {
    let last_page = total.saturating_sub(1) / REVIEWS_PER_PAGE;
    let start = page.min(last_page) * REVIEWS_PER_PAGE;
    start..(start + REVIEWS_PER_PAGE).min(total)
}

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
    let mut page = use_signal(|| 0_usize);
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
    let total = data.published_count();
    let range = page_range(total, page());
    let start = range.start;
    let end = range.end;
    rsx! {
        section { class: "ab-reviews google-reviews", aria_labelledby: "google-reviews-title",
            div { class: "ab-reviews-head",
                div {
                    div { class: "eyebrow", "FROM OUR GUESTS" }
                    h2 { id: "google-reviews-title", class: "ab-reviews-title", "Google Reviews" }
                }
                a { class: "google-reviews-rating", href: data.profile_url.clone(), target: "_blank", rel: "noopener noreferrer",
                    "{data.rating} / 5 · {total} Google reviews"
                }
            }
            div { class: "google-reviews-grid",
                for (index, review) in data.reviews.iter().filter(|r| r.published).enumerate().skip(start).take(REVIEWS_PER_PAGE) {
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
            if total > REVIEWS_PER_PAGE {
                nav { class: "google-reviews-pagination", aria_label: "Google review pages",
                    button { r#type: "button", aria_label: "Previous Google reviews", disabled: start == 0,
                        onclick: move |_| page.set((start / REVIEWS_PER_PAGE).saturating_sub(1)),
                        span { aria_hidden: "true", "←" }
                    }
                    span { role: "status", aria_live: "polite", aria_atomic: "true", "{start + 1}–{end} of {total}" }
                    button { r#type: "button", aria_label: "Next Google reviews", disabled: end == total,
                        onclick: move |_| page.set(start / REVIEWS_PER_PAGE + 1),
                        span { aria_hidden: "true", "→" }
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
    #[test]
    fn pagination_caps_pages_and_clamps_after_review_removal() {
        assert_eq!(page_range(0, 0), 0..0);
        assert_eq!(page_range(6, 0), 0..6);
        assert_eq!(page_range(9, 0), 0..6);
        assert_eq!(page_range(9, 1), 6..9);
        assert_eq!(page_range(12, 1), 6..12);
        assert_eq!(page_range(13, 2), 12..13);
        assert_eq!(page_range(5, 2), 0..5);
    }

    #[test]
    fn count_uses_published_reviews_instead_of_stale_manual_total() {
        let mut data = default_content();
        data.count = 7;
        assert_eq!(data.published_count(), 3);
        data.reviews[0].published = false;
        assert_eq!(data.published_count(), 2);
        data.reviews.clear();
        assert_eq!(data.published_count(), 0);
    }
}

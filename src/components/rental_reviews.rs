use std::collections::HashSet;

use dioxus::prelude::*;

use crate::api;

use super::ReviewForm;

fn set_like_count(reviews: &mut api::RentalReviewsResponse, review_id: &str, like_count: i64) {
    if let Some(review) = reviews
        .reviews
        .iter_mut()
        .find(|review| review.rental_review_id == review_id)
    {
        review.like_count = like_count.max(0);
    }
}

fn set_like_membership(context: &mut api::RentalReviewContext, review_id: &str, liked: bool) {
    context.liked_review_ids.retain(|id| id != review_id);
    if liked {
        context.liked_review_ids.push(review_id.to_string());
    }
}

fn like_disabled(can_like: bool, is_liked: bool, busy: bool) -> bool {
    (!can_like && !is_liked) || busy
}

fn refresh_reviews(
    slug: String,
    mut reviews: Signal<Option<api::RentalReviewsResponse>>,
    mut busy: Signal<bool>,
    mut error: Signal<String>,
    on_summary: EventHandler<api::RentalReviewSummary>,
) {
    busy.set(true);
    error.set(String::new());
    spawn(async move {
        match api::rental_reviews(&slug).await {
            Ok(value) => {
                on_summary.call(value.summary.clone());
                reviews.set(Some(value));
            }
            Err(message) => error.set(message),
        }
        busy.set(false);
    });
}

fn refresh_context(
    slug: String,
    mut context: Signal<Option<api::RentalReviewContext>>,
    mut busy: Signal<bool>,
    mut error: Signal<String>,
) {
    if api::access_token().is_none() {
        context.set(None);
        busy.set(false);
        error.set(String::new());
        return;
    }

    busy.set(true);
    error.set(String::new());
    spawn(async move {
        match api::rental_review_context(&slug).await {
            Ok(value) => context.set(Some(value)),
            Err(api_error) => {
                context.set(None);
                error.set(if api_error.status == 401 {
                    "Your session expired. Sign in again to review or like comments.".into()
                } else {
                    "Your review access could not be checked. Please try again.".into()
                });
            }
        }
        busy.set(false);
    });
}

#[component]
pub fn RentalReviewsSection(
    slug: String,
    rental_name: String,
    on_summary: EventHandler<api::RentalReviewSummary>,
) -> Element {
    let mut reviews = use_signal(|| None::<api::RentalReviewsResponse>);
    let reviews_busy = use_signal(|| true);
    let reviews_error = use_signal(String::new);
    let mut review_context = use_signal(|| None::<api::RentalReviewContext>);
    let context_busy = use_signal(|| api::access_token().is_some());
    let context_error = use_signal(String::new);
    let mut like_busy = use_signal(HashSet::<String>::new);
    let mut like_error = use_signal(String::new);

    use_effect({
        let slug = slug.clone();
        move || {
            refresh_reviews(
                slug.clone(),
                reviews,
                reviews_busy,
                reviews_error,
                on_summary,
            );
            refresh_context(slug.clone(), review_context, context_busy, context_error);
        }
    });

    let signed_in = api::access_token().is_some();
    let current_summary = reviews.read().as_ref().map(|value| value.summary.clone());
    let average_rating = current_summary
        .as_ref()
        .and_then(|summary| summary.average_rating.clone())
        .unwrap_or_else(|| "New".into());
    let review_count = current_summary
        .as_ref()
        .map(|summary| summary.review_count)
        .unwrap_or_default();
    let rounded_rating = average_rating
        .parse::<f64>()
        .ok()
        .map(|value| value.round() as i32)
        .unwrap_or_default();

    rsx! {
        section { id: "guest-reviews", class: "rvd-reviews", aria_label: "Guest reviews for {rental_name}",
            header { class: "rvd-reviews-head",
                div { class: "rvd-reviews-heading",
                    span { "VERIFIED GUEST EXPERIENCES" }
                    h2 { "Guest reviews" }
                    p { "Comments from customers who completed a paid stay in this RV." }
                }
                div { class: "rvd-reviews-summary", aria_label: "{average_rating} out of 5 from {review_count} verified reviews",
                    b { "{average_rating}" }
                    div {
                        ReviewStars { rating: rounded_rating }
                        small { "{review_count} verified reviews" }
                    }
                }
            }

            div { class: "rvd-reviews-policy",
                span { aria_hidden: "true", "✓" }
                p { "Write a review after return. Likes are available to customers with a paid booking." }
            }

            if *reviews_busy.read() {
                p { class: "rvd-reviews-state", role: "status", "Loading guest reviews…" }
            } else if !reviews_error.read().is_empty() {
                div { class: "rvd-reviews-state is-error", role: "alert",
                    p { "{reviews_error}" }
                    button { r#type: "button", onclick: { let slug = slug.clone(); move |_| refresh_reviews(slug.clone(), reviews, reviews_busy, reviews_error, on_summary) }, "Try again" }
                }
            } else if let Some(value) = reviews.read().as_ref() {
                div { class: "rvd-reviews-body",
                    aside { class: "rvd-reviews-form",
                        if *context_busy.read() {
                            p { class: "rvd-review-access", role: "status", "Checking your booking…" }
                        } else if !context_error.read().is_empty() {
                            div { class: "rvd-review-access is-error", role: "alert",
                                p { "{context_error}" }
                                button { r#type: "button", onclick: { let slug = slug.clone(); move |_| refresh_context(slug.clone(), review_context, context_busy, context_error) }, "Try again" }
                            }
                        } else if let Some(context) = review_context.read().as_ref() {
                            if let Some(booking_id) = context.reviewable_booking_id.as_ref() {
                                ReviewForm {
                                    booking_id: booking_id.clone(),
                                    rental_name: rental_name.clone(),
                                    on_published: {
                                        let slug = slug.clone();
                                        move |_| {
                                            refresh_reviews(slug.clone(), reviews, reviews_busy, reviews_error, on_summary);
                                            refresh_context(slug.clone(), review_context, context_busy, context_error);
                                        }
                                    }
                                }
                            } else {
                                p { class: "rvd-review-access",
                                    match context.review_state.as_str() {
                                        "used" => "Your review opportunity for this trip has already been used.",
                                        "waiting_for_return" => "You can write a review after the RV has been returned.",
                                        _ => "Reviews are available after a completed, paid RV trip.",
                                    }
                                }
                            }
                        } else if !signed_in {
                            p { class: "rvd-review-access", "Sign in to write a review after return. Likes are available to customers with a paid booking." }
                        }
                    }

                    div { class: "rvd-reviews-list",
                        if !like_error.read().is_empty() {
                            p { class: "rvd-review-like-error", role: "alert", "{like_error}" }
                        }
                        if value.reviews.is_empty() {
                            p { class: "rvd-reviews-empty", "No guest comments yet. A verified customer can be the first after their RV is returned." }
                        }
                        for review in value.reviews.iter() {
                            article { class: "rvd-review-item", key: "{review.rental_review_id}",
                                div { class: "rvd-review-meta",
                                    div { ReviewStars { rating: review.rating } b { "{review.rating}/5" } }
                                    time { datetime: "{review.created_at}", "{review.created_at.get(0..10).unwrap_or(&review.created_at)}" }
                                }
                                if !review.title.is_empty() { h3 { "{review.title}" } }
                                p { class: "rvd-review-comment", "{review.body}" }
                                div { class: "rvd-review-foot",
                                    small { "{review.reviewer_name} · Verified booking" }
                                    if let Some(context) = review_context.read().as_ref() {
                                        if context.own_review_ids.contains(&review.rental_review_id) {
                                            span { class: "rvd-review-own", "Your review · {review.like_count} likes" }
                                        } else {
                                            button {
                                                class: if context.liked_review_ids.contains(&review.rental_review_id) { "rvd-review-like active" } else { "rvd-review-like" },
                                                r#type: "button",
                                                aria_pressed: context.liked_review_ids.contains(&review.rental_review_id),
                                                aria_label: if context.liked_review_ids.contains(&review.rental_review_id) { "Unlike this review" } else { "Like this review" },
                                                title: if !context.can_like && !context.liked_review_ids.contains(&review.rental_review_id) { "Likes are available to customers with a paid booking" } else { "" },
                                                disabled: like_disabled(
                                                    context.can_like,
                                                    context.liked_review_ids.contains(&review.rental_review_id),
                                                    like_busy.read().contains(&review.rental_review_id),
                                                ),
                                                onclick: {
                                                    let review_id = review.rental_review_id.clone();
                                                    let was_liked = context.liked_review_ids.contains(&review.rental_review_id);
                                                    let displayed_like_count = review.like_count;
                                                    move |_| {
                                                        like_error.set(String::new());
                                                        let previous_like_count = reviews
                                                            .read()
                                                            .as_ref()
                                                            .and_then(|value| value.reviews.iter().find(|item| item.rental_review_id == review_id))
                                                            .map(|item| item.like_count)
                                                            .unwrap_or(displayed_like_count);

                                                        let optimistic_reviews = reviews.read().clone();
                                                        if let Some(mut current) = optimistic_reviews {
                                                            set_like_count(&mut current, &review_id, previous_like_count + if was_liked { -1 } else { 1 });
                                                            reviews.set(Some(current));
                                                        }
                                                        let optimistic_context = review_context.read().clone();
                                                        if let Some(mut current) = optimistic_context {
                                                            set_like_membership(&mut current, &review_id, !was_liked);
                                                            review_context.set(Some(current));
                                                        }
                                                        like_busy.write().insert(review_id.clone());

                                                        let review_id_for_request = review_id.clone();
                                                        spawn(async move {
                                                            match api::set_rental_review_like(&review_id_for_request, !was_liked).await {
                                                                Ok(result) => {
                                                                    let latest_reviews = reviews.read().clone();
                                                                    if let Some(mut current) = latest_reviews {
                                                                        set_like_count(&mut current, &result.rental_review_id, result.like_count);
                                                                        reviews.set(Some(current));
                                                                    }
                                                                    let latest_context = review_context.read().clone();
                                                                    if let Some(mut current) = latest_context {
                                                                        set_like_membership(&mut current, &result.rental_review_id, result.liked);
                                                                        review_context.set(Some(current));
                                                                    }
                                                                }
                                                                Err(_) => {
                                                                    let latest_reviews = reviews.read().clone();
                                                                    if let Some(mut current) = latest_reviews {
                                                                        set_like_count(&mut current, &review_id_for_request, previous_like_count);
                                                                        reviews.set(Some(current));
                                                                    }
                                                                    let latest_context = review_context.read().clone();
                                                                    if let Some(mut current) = latest_context {
                                                                        set_like_membership(&mut current, &review_id_for_request, was_liked);
                                                                        review_context.set(Some(current));
                                                                    }
                                                                    like_error.set("The like could not be updated. Please try again.".into());
                                                                }
                                                            }
                                                            like_busy.write().remove(&review_id_for_request);
                                                        });
                                                    }
                                                },
                                                span { aria_hidden: "true", "♥" }
                                                " {review.like_count}"
                                            }
                                        }
                                    } else {
                                        span { class: "rvd-review-own", "♥ {review.like_count}" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn ReviewStars(rating: i32) -> Element {
    rsx! {
        span { class: "rvd-review-stars", aria_label: "{rating} out of 5 stars",
            for value in 1..=5_i32 {
                span { key: "rvd-review-star-{value}", class: if value <= rating { "filled" } else { "" }, aria_hidden: "true", "★" }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::like_disabled;

    #[test]
    fn paid_customer_can_toggle_and_busy_request_is_locked() {
        assert!(!like_disabled(true, false, false));
        assert!(!like_disabled(true, true, false));
        assert!(like_disabled(true, false, true));
    }

    #[test]
    fn existing_like_can_be_removed_even_if_new_likes_are_not_allowed() {
        assert!(like_disabled(false, false, false));
        assert!(!like_disabled(false, true, false));
    }
}

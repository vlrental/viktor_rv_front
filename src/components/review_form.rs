use dioxus::prelude::*;

use crate::api;

#[component]
pub fn ReviewForm(
    booking_id: String,
    rental_name: String,
    on_published: EventHandler<api::RentalReview>,
    #[props(default)] on_cancel: Option<EventHandler<()>>,
) -> Element {
    let mut rating = use_signal(|| 5_i32);
    let mut title = use_signal(String::new);
    let mut body = use_signal(String::new);
    let mut error = use_signal(String::new);
    let mut busy = use_signal(|| false);
    let character_count = body.read().chars().count();

    rsx! {
        div { class: "review-form",
            p { class: "review-form-kicker", "Verified stay" }
            h3 { "Review {rental_name}" }
            p { "Your rating" }
            div { class: "review-form-stars", role: "group", aria_label: "Rating out of 5",
                for value in 1..=5_i32 {
                    button { key: "review-star-{value}", class: if value <= *rating.read() { "active" } else { "" }, r#type: "button", aria_label: "{value} out of 5 stars", disabled: *busy.read(), onclick: move |_| rating.set(value), "★" }
                }
            }
            input { maxlength: "80", value: "{title}", placeholder: "Short title (optional)", aria_label: "Review title", disabled: *busy.read(), oninput: move |event| title.set(event.value()) }
            textarea { maxlength: "2000", value: "{body}", placeholder: "Tell future guests about your experience", aria_label: "Review comment", disabled: *busy.read(), oninput: move |event| body.set(event.value()) }
            small { "{character_count}/2000 characters · minimum 10" }
            if !error.read().is_empty() { p { class: "auth-error", role: "alert", "{error}" } }
            div { class: "review-form-actions",
                if let Some(cancel) = on_cancel {
                    button { r#type: "button", disabled: *busy.read(), onclick: move |_| cancel.call(()), "Cancel" }
                }
                button { class: "btn-forest", r#type: "button", disabled: *busy.read(), onclick: move |_| { let booking_id = booking_id.clone(); let title_value = title.read().clone(); let body_value = body.read().clone(); async move { if !(10..=2000).contains(&body_value.trim().chars().count()) { error.set("Write between 10 and 2000 characters about your experience.".into()); return; } busy.set(true); error.set(String::new()); match api::create_rental_review(&booking_id, *rating.read(), &title_value, &body_value).await { Ok(review) => on_published.call(review), Err(api_error) => error.set(api_error.message) } busy.set(false); } },
                    if *busy.read() { "Publishing…" } else { "Publish review" }
                }
            }
        }
    }
}

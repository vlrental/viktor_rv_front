use crate::api::{
    self,
    reviews::{self, ExternalReview, GoogleDocument, GoogleReview},
};
use dioxus::prelude::*;

pub(super) fn confirm_discard() -> bool {
    web_sys::window()
        .and_then(|w| {
            w.confirm_with_message("Discard unsaved review changes?")
                .ok()
        })
        .unwrap_or(false)
}

#[component]
fn Field(
    label: String,
    value: String,
    onchange: EventHandler<String>,
    #[props(default="text".into())] kind: String,
    #[props(default = 2048)] max: i64,
    #[props(default = false)] multiline: bool,
) -> Element {
    rsx! { label { class: "review-field", span { "{label}" }
        if multiline { textarea { value, maxlength:max, rows:4, oninput:move |e|onchange.call(e.value()) } }
        else { input { r#type:kind, value, maxlength:max, step:"any", oninput:move |e|onchange.call(e.value()) } }
    } }
}

#[component]
pub(super) fn GoogleEditor(mut dirty: Signal<bool>, mut busy: Signal<bool>) -> Element {
    let mut draft = use_signal(|| GoogleDocument {
        content: crate::components::google_reviews::default_content(),
        revision: None,
    });
    let mut ready = use_signal(|| false);
    let mut message = use_signal(String::new);
    let mut retry = use_signal(|| 0u32);
    use_effect(move || {
        let _ = retry();
        ready.set(false);
        message.set(String::new());
        spawn(async move {
            match reviews::admin_get::<Option<GoogleDocument>>("google-reviews").await {
                Ok(value) => {
                    draft.set(value.unwrap_or_else(|| GoogleDocument {
                        content: crate::components::google_reviews::default_content(),
                        revision: None,
                    }));
                    ready.set(true);
                    dirty.set(false);
                }
                Err(e) => message.set(e.message),
            }
        });
    });
    rsx! { section { class:"admin-panel admin-full-panel review-manager",
            div { class:"admin-panel-head", div { h2 { "Google · Home & About" } p { "Update the rating and selected Google review excerpts shown on both pages." } } }
            if !message().is_empty() { p { class:"admin-inline-message", role:"status", "{message}" } }
            if !ready() { p { "Loading Google reviews…" } button { r#type:"button", onclick:move |_|retry.set(retry()+1), "Retry" } }
            else { form { onsubmit:move |e| { e.prevent_default(); if busy() {return;} busy.set(true); message.set(String::new()); let value=draft(); spawn(async move {
                match reviews::admin_save::<_,GoogleDocument>("google-reviews", &value).await { Ok(saved)=>{draft.set(saved);dirty.set(false);message.set("Saved. Home and About now use these Google reviews.".into());},Err(e)=>message.set(e.message) } busy.set(false);
            }); },
            fieldset { disabled:busy(), class:"review-editor-fields", oninput:move |_|dirty.set(true), onchange:move |_|dirty.set(true),
                label { class:"review-toggle", input { r#type:"checkbox", checked:draft.read().content.published, onchange:move |e|draft.write().content.published=e.checked() } "Show Google reviews on the website" }
                div { class:"review-fields-grid",
    Field { label:"Overall rating (0–5)",value:draft.read().content.rating.to_string(),kind:"number",max:10,onchange:move |value: String|draft.write().content.rating=value }
    Field { label:"Total Google reviews",value:draft.read().content.count.to_string(),kind:"number",max:10,onchange:move |value: String|draft.write().content.count=value.parse().unwrap_or(-1) }
    Field { label:"Last checked",value:draft.read().content.checked_on.to_string(),kind:"date",max:10,onchange:move |value: String|draft.write().content.checked_on=value }
    Field { label:"Google business profile link",value:draft.read().content.profile_url.to_string(),kind:"url",max:2048,onchange:move |value: String|draft.write().content.profile_url=value }
    Field { label:"Write a review link",value:draft.read().content.write_url.to_string(),kind:"url",max:2048,onchange:move |value: String|draft.write().content.write_url=value }
    }
                p { class:"review-help", "Copy the current rating and count from Google. Only publish real reviews, preserving the guest’s meaning and source link." }
                for (index,review) in draft.read().content.reviews.iter().enumerate() { article { class:"review-edit-card", key:"{index}",
                    h3 { "Review {index + 1}" }
                    div { class:"review-fields-grid",
    Field { label:"Guest name",value:review.name.to_string(),kind:"text",max:100,multiline:false,onchange:move |value: String|draft.write().content.reviews[index].name=value }
    Field { label:"Stars (1–5)",value:review.rating.to_string(),kind:"number",max:1,multiline:false,onchange:move |value: String|draft.write().content.reviews[index].rating=value.parse().unwrap_or(0) }
    Field { label:"Language (e.g. en or de)",value:review.language.to_string(),kind:"text",max:12,multiline:false,onchange:move |value: String|draft.write().content.reviews[index].language=value }
    Field { label:"Original Google review / author link",value:review.url.to_string(),kind:"url",max:2048,multiline:false,onchange:move |value: String|draft.write().content.reviews[index].url=value }
    Field { label:"Review excerpt",value:review.body.to_string(),kind:"text",max:4000,multiline:true,onchange:move |value: String|draft.write().content.reviews[index].body=value }
    }
                    div { class:"review-actions", label { class:"review-toggle", input { r#type:"checkbox",checked:review.published,onchange:move |e|draft.write().content.reviews[index].published=e.checked() } "Visible" }
                        button { r#type:"button", onclick:move |_| {draft.write().content.reviews.remove(index);dirty.set(true);}, "Remove from selection" }
                    }
                } }
                div { class:"review-actions",
                    button { r#type:"button", disabled:draft.read().content.reviews.len()>=20, onclick:move |_| {draft.write().content.reviews.push(GoogleReview {name:String::new(),body:String::new(),language:"en".into(),url:String::new(),rating:5,published:false});dirty.set(true);}, "+ Add Google review" }
                    button { class:"admin-primary-small",r#type:"submit",disabled:!dirty(), if busy() {"Saving…"} else {"Save Google reviews"} }
                    button { r#type:"button",onclick:move |_|if !dirty() || confirm_discard() {retry.set(retry()+1);}, "Reload saved version" }
                }
            } }
            }
        } }
}

#[component]
pub(super) fn ExternalEditor(
    id: String,
    initial: Option<ExternalReview>,
    rentals: Vec<api::AdminRentalSummary>,
    mut dirty: Signal<bool>,
    mut busy: Signal<bool>,
    on_saved: EventHandler<()>,
    on_close: EventHandler<()>,
) -> Element {
    use_effect(move || {
        spawn(async move {
            let _ = document::eval("document.getElementById('external-review-editor')?.scrollIntoView({block: 'start', behavior: 'smooth'});").await;
        });
    });
    let creating = initial.is_none();
    let mut draft = use_signal(|| {
        initial.unwrap_or_else(|| ExternalReview {
            rental_slug: rentals.first().map(|r| r.slug.clone()).unwrap_or_default(),
            source: "outdoorsy".into(),
            source_review_id: String::new(),
            source_url: String::new(),
            reviewer_name: String::new(),
            rating: "5".into(),
            title: String::new(),
            body: String::new(),
            reviewed_at: String::new(),
            reviewed_at_label: String::new(),
            is_published: false,
            revision: None,
        })
    });
    let mut message = use_signal(String::new);
    rsx! { section { id:"external-review-editor", class:"admin-panel admin-full-panel review-manager",
            h2 { if creating {"Add external RV review"} else {"Edit external RV review"} }
            p { "Copy a real review from Outdoorsy or RVezy and assign it to the matching RV." }
            if !message().is_empty() { p { class:"admin-error",role:"alert","{message}" } }
            form { onsubmit:move |e| {e.prevent_default();if busy(){return;} busy.set(true);message.set(String::new());let value=draft();let path=format!("external-reviews/{id}");spawn(async move {
                match reviews::admin_save::<_,serde_json::Value>(&path,&value).await {Ok(_)=>{dirty.set(false);on_saved.call(());},Err(e)=>message.set(e.message)} busy.set(false);
            });},
            fieldset { class:"review-editor-fields",disabled:busy(),oninput:move |_|dirty.set(true),onchange:move |_|dirty.set(true),
                div { class:"review-fields-grid",
                    label { class:"review-field", "RV" select { aria_label:"RV", value:draft.read().rental_slug.clone(),onchange:move |e|draft.write().rental_slug=e.value(),for rental in rentals.iter() {option {value:rental.slug.clone(),"{rental.name}"}} } }
                    label { class:"review-field", "Source" select { aria_label:"Source", value:draft.read().source.clone(),onchange:move |e|draft.write().source=e.value(), option {value:"outdoorsy","Outdoorsy"} option {value:"rvezy","RVezy"} } }
    Field {label:"Original review ID / unique reference",value:draft.read().source_review_id.clone(),kind:"text",max:200,multiline:false,onchange:move |value: String|draft.write().source_review_id=value}
    Field {label:"Original listing / review link",value:draft.read().source_url.clone(),kind:"url",max:2048,multiline:false,onchange:move |value: String|draft.write().source_url=value}
    Field {label:"Guest name",value:draft.read().reviewer_name.clone(),kind:"text",max:100,multiline:false,onchange:move |value: String|draft.write().reviewer_name=value}
    Field {label:"Rating (1–5)",value:draft.read().rating.clone(),kind:"number",max:8,multiline:false,onchange:move |value: String|draft.write().rating=value}
    Field {label:"Review date",value:draft.read().reviewed_at.clone(),kind:"date",max:10,multiline:false,onchange:move |value: String|draft.write().reviewed_at=value}
    Field {label:"Date shown to guests (e.g. September 2026)",value:draft.read().reviewed_at_label.clone(),kind:"text",max:50,multiline:false,onchange:move |value: String|draft.write().reviewed_at_label=value}
    Field {label:"Title (optional)",value:draft.read().title.clone(),kind:"text",max:200,multiline:false,onchange:move |value: String|draft.write().title=value}
    Field {label:"Review text",value:draft.read().body.clone(),kind:"text",max:4000,multiline:true,onchange:move |value: String|draft.write().body=value}
    }
                p { class:"review-help", "Use the provider’s review ID, or a stable reference such as guest-date-RV. Reusing a reference is blocked to prevent duplicates." }
                label { class:"review-toggle",input { r#type:"checkbox",checked:draft.read().is_published,onchange:move |e|draft.write().is_published=e.checked() } "Publish on this RV’s page" }
                div {class:"review-actions", button {class:"admin-primary-small",r#type:"submit", if busy(){"Saving…"}else{"Save review"}} button {r#type:"button",onclick:move |_|on_close.call(()),"Cancel"} }
            } }
        } }
}

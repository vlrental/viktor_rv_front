use crate::{
    api::{
        self,
        sales::{self, Listing},
    },
    components::Icon,
    Route,
};
use dioxus::prelude::*;

#[component]
pub fn RvSales() -> Element {
    let mut items = use_signal(Vec::<Listing>::new);
    let mut loading = use_signal(|| true);
    let mut load_error = use_signal(String::new);
    let mut retry = use_signal(|| 0_u32);
    use_effect(move || {
        let _ = retry();
        loading.set(true);
        load_error.set(String::new());
        spawn(async move {
            match sales::listings(false).await {
                Ok(values) => items.set(values),
                Err(e) => {
                    items.set(Vec::new());
                    load_error.set(e.message)
                }
            }
            loading.set(false);
        });
    });
    rsx! {
        section {class:"sale-header",
            div {class:"eyebrow","RV SALES"}
            h1 {class:"sale-title","Own your next adventure"}
            p {class:"sale-sub","Explore RVs for sale in Kelowna, British Columbia. View vehicle details and photos, then contact us to ask about a listing or arrange a viewing."}
        }
        section {class:"sale-grid",aria_label:"RVs for sale",
            if loading(){div {class:"sale-empty",role:"status","Loading RVs for sale…"}}
            else if !load_error().is_empty(){div {class:"sale-empty",role:"alert",h2 {"Listings are temporarily unavailable"}p {"Please try again or contact us about RV sales."}button {class:"btn-forest",r#type:"button",onclick:move|_|retry.set(retry().wrapping_add(1)),"Try again"}}}
            else if items.read().is_empty(){div {class:"sale-empty",h2 {"No RVs are listed for sale right now"}p {"Tell us what you're looking for below, or check back for new listings."}}}
            else {for item in items.read().iter(){article {key:"{item.sale_id}",class:"sale-card",
                if let Some(photo)=item.photos.first(){Link {class:"sale-photo-open",aria_label:"View {item.title}",to:Route::RvSaleDetail {sale_id:item.sale_id.clone()},img {class:"sale-img",src:"{photo.source_url}",alt:"{photo.alt_text}",loading:"lazy"}}}
                div {class:"sale-body",
                    h2 {class:"sale-card-title",Link {to:Route::RvSaleDetail {sale_id:item.sale_id.clone()},"{item.title}"}}
                    p {class:"sale-meta","{sale_meta(item)}"}
                    p {class:"sale-meta","{item.location}"}
                    p {class:"sale-summary","{item.summary}"}
                    div {class:"sale-price-row",span {class:"sale-price","{sales::price_label(item.price.as_deref())}"}}
                    Link {class:"btn-forest",to:Route::RvSaleDetail {sale_id:item.sale_id.clone()},"View listing"}
                }
            }}}
        }
        SaleInquiry { listing: None }
    }
}

#[component]
pub(super) fn SaleInquiry(listing: Option<Listing>) -> Element {
    let initial_requested = listing
        .as_ref()
        .map(|item| item.title.clone())
        .unwrap_or_default();
    let mut full_name = use_signal(String::new);
    let mut email = use_signal(String::new);
    let mut requested = use_signal(|| initial_requested);
    let mut message = use_signal(|| "Interested in an RV purchase".to_string());
    let mut status = use_signal(String::new);
    let mut busy = use_signal(|| false);
    let specific_listing = listing.is_some();
    rsx! {
        form {id:"sales-inquiry",class:"sale-cta",onsubmit:move|event| {
            event.prevent_default();if busy(){return}let unit=listing.as_ref().map(inquiry_listing_reference).unwrap_or_else(&*requested);let values=(full_name(),email(),unit,message());
            spawn(async move {
                status.set(String::new());
                if values.0.trim().len()<2||!values.1.contains('@')||values.2.trim().len()<2{status.set("Enter your name, a valid email, and the RV or budget you are looking for.".into());return}
                busy.set(true);
                match api::send_sales_inquiry(&values.0,&values.1,"",&values.2,&values.3).await{Ok(())=>{status.set("Thanks — your sales request has been saved.".into());if !specific_listing {requested.set(String::new());}},Err(e)=>status.set(e)}busy.set(false);
            });
        },
            div {class:"sale-cta-text",
                h2 {class:"sale-cta-title",if specific_listing {"Enquire about this RV"}else{"Interested in an RV?"}}
                p {class:"sale-cta-sub","Ask about a listing, arrange a viewing, or tell us your budget and what you're looking for."}
                div {class:"ct-row sale-inquiry-fields",
                    label {"Your name" input {class:"ct-input",required:true,minlength:"2",maxlength:"120",autocomplete:"name",value:"{full_name}",disabled:busy(),oninput:move|e|full_name.set(e.value())}}
                    label {"Email" input {class:"ct-input",r#type:"email",required:true,maxlength:"254",autocomplete:"email",value:"{email}",disabled:busy(),oninput:move|e|email.set(e.value())}}
                    label {if specific_listing {"RV listing"}else{"RV or budget"} input {class:"ct-input",readonly:specific_listing,required:true,minlength:"2",maxlength:"160",value:"{requested}",disabled:busy(),oninput:move|e|requested.set(e.value())}}
                }
                if specific_listing {label {class:"sale-inquiry-message","Message" textarea {class:"ct-input",rows:"4",maxlength:"4000",required:true,value:"{message}",disabled:busy(),oninput:move|e|message.set(e.value())}}}
                if !status().is_empty(){p {role:"status","{status}"}}
            }
            button {class:"sale-cta-btn",r#type:"submit",disabled:busy(),Icon {name:"send",size:16,color:"var(--vl-forest-2)"}if busy(){"Sending…"}else{"Send inquiry"}}
        }
    }
}

fn inquiry_listing_reference(item: &Listing) -> String {
    format!(
        "{} (listing {})",
        item.title.chars().take(100).collect::<String>(),
        item.sale_id
    )
}

pub(super) fn sale_meta(item: &Listing) -> String {
    let mut parts = Vec::new();
    if let Some(year) = item.model_year {
        parts.push(year.to_string());
    }
    parts.push(
        if item.condition == "new" {
            "New"
        } else {
            "Used"
        }
        .into(),
    );
    if let Some(length) = &item.length_ft {
        parts.push(format!("{length} ft"));
    }
    if let Some(sleeps) = item.sleeps {
        parts.push(format!("Sleeps {sleeps}"));
    }
    parts.join(" · ")
}

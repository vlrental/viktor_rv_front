use crate::{
    api::{
        self,
        sales::{self, Listing},
    },
    components::Icon,
};
use dioxus::prelude::*;

#[component]
pub fn RvSales() -> Element {
    let mut items = use_signal(Vec::<Listing>::new);
    let mut loading = use_signal(|| true);
    let mut load_error = use_signal(String::new);
    let mut retry = use_signal(|| 0_u32);
    let mut selected = use_signal(|| None::<Listing>);
    let mut full_name = use_signal(String::new);
    let mut email = use_signal(String::new);
    let mut requested = use_signal(String::new);
    let mut status = use_signal(String::new);
    let mut busy = use_signal(|| false);
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
                if let Some(photo)=item.photos.first(){button {class:"sale-photo-open",r#type:"button",aria_label:"View {item.title}",onclick:{let item=item.clone();move|_|selected.set(Some(item.clone()))},img {class:"sale-img",src:"{photo.source_url}",alt:"{photo.alt_text}",loading:"lazy"}}}
                div {class:"sale-body",
                    h2 {class:"sale-card-title","{item.title}"}
                    p {class:"sale-meta","{sale_meta(item)}"}
                    p {class:"sale-meta","{item.location}"}
                    p {class:"sale-summary","{item.summary}"}
                    div {class:"sale-price-row",span {class:"sale-price","{sales::price_label(item.price.as_deref())}"}}
                    button {class:"btn-forest",r#type:"button",onclick:{let item=item.clone();move|_|selected.set(Some(item.clone()))},"View listing"}
                }
            }}}
        }
        form {id:"sales-inquiry",class:"sale-cta",onsubmit:move|event| {
            event.prevent_default();if busy(){return}let values=(full_name(),email(),requested());
            spawn(async move {
                status.set(String::new());
                if values.0.trim().len()<2||!values.1.contains('@')||values.2.trim().len()<2{status.set("Enter your name, a valid email, and the RV or budget you are looking for.".into());return}
                busy.set(true);
                match api::send_sales_inquiry(&values.0,&values.1,"",&values.2,"Interested in an RV purchase").await{Ok(())=>{status.set("Thanks — your sales request has been saved.".into());requested.set(String::new());},Err(e)=>status.set(e)}busy.set(false);
            });
        },
            div {class:"sale-cta-text",
                h2 {class:"sale-cta-title","Interested in an RV?"}
                p {class:"sale-cta-sub","Ask about a listing, arrange a viewing, or tell us your budget and what you're looking for."}
                div {class:"ct-row sale-inquiry-fields",
                    label {"Your name" input {class:"ct-input",required:true,minlength:"2",maxlength:"200",autocomplete:"name",value:"{full_name}",disabled:busy(),oninput:move|e|full_name.set(e.value())}}
                    label {"Email" input {class:"ct-input",r#type:"email",required:true,maxlength:"254",autocomplete:"email",value:"{email}",disabled:busy(),oninput:move|e|email.set(e.value())}}
                    label {"RV or budget" input {class:"ct-input",required:true,minlength:"2",maxlength:"500",value:"{requested}",disabled:busy(),oninput:move|e|requested.set(e.value())}}
                }
                if !status().is_empty(){p {role:"status","{status}"}}
            }
            button {class:"sale-cta-btn",r#type:"submit",disabled:busy(),Icon {name:"send",size:16,color:"var(--vl-forest-2)"}if busy(){"Sending…"}else{"Send inquiry"}}
        }
        if let Some(item)=selected(){SaleDetails {key:"{item.sale_id}",item,on_close:move|_|selected.set(None),on_inquire:move|item:Listing|{requested.set(format!("{} (listing {})",item.title,item.sale_id));selected.set(None);let _=document::eval("document.getElementById('sales-inquiry')?.scrollIntoView({behavior:'smooth',block:'center'}); document.querySelector('#sales-inquiry input')?.focus({preventScroll:true});");}}}
    }
}

fn sale_meta(item: &Listing) -> String {
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

#[component]
fn SaleDetails(
    item: Listing,
    on_close: EventHandler<()>,
    on_inquire: EventHandler<Listing>,
) -> Element {
    let mut index = use_signal(|| 0_usize);
    let count = item.photos.len();
    rsx! {div {class:"sale-dialog-backdrop",onclick:move|_|on_close.call(()),
        section {class:"sale-dialog",role:"dialog",aria_modal:"true",aria_label:"{item.title}",tabindex:"-1",autofocus:true,onclick:move|e|e.stop_propagation(),onkeydown:move|e|{if e.key()==Key::Escape {e.stop_propagation();on_close.call(());}else if count>1&&e.key()==Key::ArrowRight {index.set((index()+1)%count);}else if count>1&&e.key()==Key::ArrowLeft {index.set((index()+count-1)%count);}},
            button {class:"sale-dialog-close",r#type:"button",aria_label:"Close sale listing",onclick:move|_|on_close.call(()),"×"}
            if let Some(photo)=item.photos.get(index()){img {class:"sale-detail-photo",src:"{photo.source_url}",alt:"{photo.alt_text}"}}
            if count>1{div {class:"sale-gallery-controls",button {r#type:"button",aria_label:"Previous photo",onclick:move|_|index.set((index()+count-1)%count),"←"}span {"Photo {index()+1} of {count}"}button {r#type:"button",aria_label:"Next photo",onclick:move|_|index.set((index()+1)%count),"→"}}}
            div {class:"sale-detail-copy",h2 {"{item.title}"}p {class:"sale-meta","{sale_meta(&item)}"}p {"{item.location}"}strong {class:"sale-price","{sales::price_label(item.price.as_deref())}"}p {class:"sale-description","{item.description}"}
                button {class:"btn-forest",r#type:"button",onclick:move|_|on_inquire.call(item.clone()),"Enquire about this RV"}
            }
        }
    }}
}

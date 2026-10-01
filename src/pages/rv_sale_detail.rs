//! Individual sales pages — Pencil IaceU (desktop), b9EOQ (mobile).

use super::rv_sales::SaleInquiry;
use crate::{
    api::sales::{self, Listing},
    Route, SaleSeoContext,
};
use dioxus::prelude::*;

#[component]
pub fn RvSaleDetail(sale_id: String) -> Element {
    let mut listing = use_signal(|| None::<Listing>);
    let mut loading = use_signal(|| true);
    let mut error = use_signal(String::new);
    let mut retry = use_signal(|| 0_u32);
    let mut generation = use_signal(|| 0_u64);
    let mut seo = use_context::<SaleSeoContext>().0;
    use_effect(use_reactive((&sale_id,), move |(id,)| {
        let _ = retry();
        let request = generation.peek().wrapping_add(1);
        generation.set(request);
        listing.set(None);
        error.set(String::new());
        seo.set(None);
        if uuid::Uuid::parse_str(&id).is_err() {
            loading.set(false);
            return;
        }
        loading.set(true);
        spawn(async move {
            let result = sales::listings(false).await;
            if *generation.peek() != request {
                return;
            }
            match result {
                Ok(items) => {
                    let item = items
                        .into_iter()
                        .find(|item| item.sale_id == id && item.status == "published");
                    seo.set(item.clone().map(|item| (id, item)));
                    listing.set(item);
                }
                Err(e) => error.set(e.message),
            }
            loading.set(false);
        });
    }));
    rsx! {
        if loading() {
            section {class:"sale-page-state",role:"status",h1 {"Loading RV listing…"}}
        } else if !error().is_empty() {
            section {class:"sale-page-state",role:"alert",h1 {"This listing is temporarily unavailable"}p {"Please try again or contact VL Rental about RV sales."}
                button {class:"btn-forest",r#type:"button",onclick:move|_|retry.set(retry().wrapping_add(1)),"Try again"}
                Link {class:"sale-back",to:Route::RvSales {},"← All RV Sales"}
            }
        } else if let Some(item) = listing() {
            SalePage {key:"{item.sale_id}",item}
        } else {
            section {class:"sale-page-state",h1 {"This RV is no longer listed for sale"}p {"The listing may have been sold or taken offline. Browse our current RVs for sale or contact us for help."}
                Link {class:"btn-forest",to:Route::RvSales {},"Browse RV Sales"}
            }
        }
    }
}

#[component]
fn SalePage(item: Listing) -> Element {
    let mut index = use_signal(|| 0_usize);
    let count = item.photos.len();
    let specs = sale_specs(&item);
    rsx! {
        section {class:"sale-header sale-page-header",
            Link {class:"sale-back",to:Route::RvSales {},"← All RV Sales"}
            div {class:"eyebrow","RV FOR SALE"}
            h1 {class:"sale-title","{item.title}"}
            p {class:"sale-meta","{super::rv_sales::sale_meta(&item)} · {item.location}"}
            p {class:"sale-page-price",span {"Asking price"}strong {"{sales::price_label(item.price.as_deref())}"}}
        }
        div {class:"sale-page-content",
            section {class:"sale-page-gallery",role:"region",aria_label:"RV photo gallery",tabindex:"0",onkeydown:move|event|{
                if count>1 && (event.key()==Key::ArrowRight || event.key()==Key::ArrowLeft) {
                    event.prevent_default();
                    index.set(if event.key()==Key::ArrowRight {(index()+1)%count}else{(index()+count-1)%count});
                }
            },
                if let Some(photo)=item.photos.get(index()) {
                    figure {class:"sale-page-photo",img {src:"{photo.source_url}",alt:"{photo.alt_text}",decoding:"async"}figcaption {"{photo.alt_text}"}}
                }
                if count>1 {
                    div {class:"sale-gallery-controls",
                        button {r#type:"button",aria_label:"Previous photo",onclick:move|_|index.set((index()+count-1)%count),"← Previous"}
                        span {aria_live:"polite","Photo {index()+1} of {count}"}
                        button {r#type:"button",aria_label:"Next photo",onclick:move|_|index.set((index()+1)%count),"Next →"}
                    }
                    ol {class:"sale-page-thumbnails",aria_label:"Choose an RV photo",
                        for (photo_index,photo) in item.photos.iter().enumerate() {
                            li {key:"{photo.photo_id}",button {r#type:"button",aria_label:"Show photo {photo_index+1}: {photo.alt_text}",aria_pressed:index()==photo_index,onclick:move|_|index.set(photo_index),
                                img {src:"{photo.source_url}",alt:"",loading:"lazy",decoding:"async"}
                            }}
                        }
                    }
                }
            }
            section {class:"sale-page-specs",aria_label:"Vehicle details",
                h2 {"Vehicle details"}
                dl {for (label,value) in specs {div {dt {"{label}"}dd {"{value}"}}}}
            }
            section {class:"sale-page-description",
                h2 {"About this RV"}
                p {class:"sale-description","{item.description}"}
            }
            a {class:"btn-forest sale-page-enquire",href:"#sales-inquiry","Enquire about this RV"}
        }
        SaleInquiry {listing:Some(item)}
    }
}

fn sale_specs(item: &Listing) -> Vec<(&'static str, String)> {
    let mut specs = vec![
        (
            "RV type",
            match item.rv_type.as_str() {
                "travel_trailer" => "Travel trailer",
                "fifth_wheel" => "Fifth wheel",
                "motorhome" => "Motorhome",
                "camper" => "Camper",
                _ => "RV",
            }
            .into(),
        ),
        (
            "Condition",
            if item.condition == "new" {
                "New"
            } else {
                "Used"
            }
            .into(),
        ),
        ("Location", item.location.clone()),
    ];
    if let Some(year) = item.model_year {
        specs.insert(0, ("Model year", year.to_string()));
    }
    for (label, value) in [("Manufacturer", &item.manufacturer), ("Model", &item.model)] {
        if !value.is_empty() {
            specs.push((label, value.clone()));
        }
    }
    if let Some(length) = &item.length_ft {
        specs.push(("Length", format!("{length} ft")));
    }
    if let Some(sleeps) = item.sleeps {
        specs.push(("Sleeps", sleeps.to_string()));
    }
    specs
}

//! Individual sales pages — Pencil IaceU (desktop), b9EOQ (mobile).

use super::rv_sales::SaleInquiry;
use crate::{
    api::sales::{self, Listing},
    components::Icon,
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
            SaleGallery {item:item.clone()}
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

#[component]
fn SaleGallery(item: Listing) -> Element {
    let mut selected = use_signal(|| None::<usize>);
    let mut opener = use_signal(|| 0_usize);
    let count = item.photos.len();
    let preview_count = count.saturating_sub(1).min(6);
    rsx! {
        section {class:"sale-page-gallery",aria_label:"RV photo gallery",
            if let Some(photo)=item.photos.first() {
                div {class:if count>1 {"sale-gallery-mosaic"}else{"sale-gallery-mosaic is-single"},
                    button {id:"sale-gallery-photo-0",class:"sale-gallery-main",r#type:"button",aria_label:"Open photo 1 of {count}",onclick:move|_|{opener.set(0);selected.set(Some(0));},
                        img {src:"{photo.source_url}",alt:"{photo.alt_text}",decoding:"async",draggable:"false"}
                    }
                    if preview_count>0 {
                        div {class:"sale-gallery-previews","data-count":"{preview_count}",style:"--sale-preview-rows: {preview_count.div_ceil(2)}",
                            for (photo_index,photo) in item.photos.iter().enumerate().skip(1).take(6) {
                                button {key:"{photo.photo_id}",id:"sale-gallery-photo-{photo_index}",class:"sale-gallery-tile",r#type:"button",aria_label:if photo_index==preview_count {format!("Show all {count} photos")}else{format!("Open photo {} of {count}",photo_index+1)},onclick:move|_|{opener.set(photo_index);selected.set(Some(photo_index));},
                                    img {src:"{photo.source_url}",alt:"{photo.alt_text}",loading:"lazy",decoding:"async",draggable:"false"}
                                    if photo_index==preview_count {span {class:"sale-gallery-more","Show all {count} photos"}}
                                }
                            }
                        }
                    }
                }
                if count>1 {
                    button {id:"sale-gallery-all",class:"sale-gallery-all",r#type:"button",onclick:move|_|{opener.set(usize::MAX);selected.set(Some(0));},Icon {name:"images",size:18,color:"var(--vl-forest)"}"Show all {count} photos"}
                }
            }else{
                div {class:"sale-gallery-empty",Icon {name:"image",size:32,color:"var(--vl-muted)"}"Photos are being prepared"}
            }
        }
        if let Some(index)=selected() {
            SalePhotoViewer {item,index,selected,opener:opener()}
        }
    }
}

#[component]
fn SalePhotoViewer(
    item: Listing,
    index: usize,
    mut selected: Signal<Option<usize>>,
    opener: usize,
) -> Element {
    let count = item.photos.len();
    let opener_id = if opener == usize::MAX {
        "sale-gallery-all".to_string()
    } else {
        format!("sale-gallery-photo-{opener}")
    };
    use_drop(|| {
        document::eval("window.__vlSaleGalleryCleanup?.(); window.__vlSaleGalleryCleanup = null;");
    });
    rsx! {
        div {id:"sale-photo-viewer",class:"sale-photo-viewer",role:"dialog",aria_modal:"true",aria_label:"{item.title} photo gallery",tabindex:"-1",onmounted:move|_|{
            document::eval(&format!(r#"
                const overlay = document.getElementById('sale-photo-viewer');
                if (overlay) {{
                    const opener = document.getElementById('{opener_id}');
                    const previousOverflow = document.body.style.overflow;
                    const trapFocus = (event) => {{
                        if (event.key !== 'Tab') return;
                        const buttons = [...overlay.querySelectorAll('button')];
                        const first = buttons[0], last = buttons[buttons.length - 1];
                        if (event.shiftKey && (document.activeElement === first || document.activeElement === overlay)) {{
                            event.preventDefault(); last?.focus();
                        }} else if (!event.shiftKey && document.activeElement === last) {{
                            event.preventDefault(); first?.focus();
                        }}
                    }};
                    let startX = null, startY = null;
                    const touchStart = (event) => {{
                        startX = event.touches[0]?.clientX ?? null;
                        startY = event.touches[0]?.clientY ?? null;
                    }};
                    const touchEnd = (event) => {{
                        if (startX === null || startY === null) return;
                        const dx = (event.changedTouches[0]?.clientX ?? startX) - startX;
                        const dy = (event.changedTouches[0]?.clientY ?? startY) - startY;
                        if (Math.abs(dx) > 50 && Math.abs(dx) > Math.abs(dy)) {{
                            document.getElementById(dx > 0 ? 'sale-photo-prev' : 'sale-photo-next')?.click();
                        }}
                        startX = startY = null;
                    }};
                    document.body.style.overflow = 'hidden';
                    overlay.addEventListener('keydown', trapFocus);
                    overlay.addEventListener('touchstart', touchStart, {{passive:true}});
                    overlay.addEventListener('touchend', touchEnd, {{passive:true}});
                    overlay.focus({{preventScroll:true}});
                    window.__vlSaleGalleryCleanup = () => {{
                        document.body.style.overflow = previousOverflow;
                        overlay.removeEventListener('keydown', trapFocus);
                        overlay.removeEventListener('touchstart', touchStart);
                        overlay.removeEventListener('touchend', touchEnd);
                        if (opener?.isConnected) opener.focus({{preventScroll:true}});
                    }};
                }}
            "#));
        },onkeydown:move|event|{
            match event.key() {
                Key::Escape=>{event.prevent_default();event.stop_propagation();selected.set(None);},
                Key::ArrowLeft if count>1=>{event.prevent_default();event.stop_propagation();selected.set(Some((index+count-1)%count));},
                Key::ArrowRight if count>1=>{event.prevent_default();event.stop_propagation();selected.set(Some((index+1)%count));},
                _=>{}
            }
        },onclick:move|_|selected.set(None),
            div {class:"sale-photo-viewer-content",onclick:move|event|event.stop_propagation(),
                img {class:"sale-photo-viewer-image",src:"{item.photos[index].source_url}",alt:"{item.photos[index].alt_text}",draggable:"false"}
                button {class:"sale-photo-viewer-close",r#type:"button",aria_label:"Close gallery",onclick:move|_|selected.set(None),Icon {name:"x",size:24,color:"var(--vl-white)"}}
                if count>1 {
                    button {id:"sale-photo-prev",class:"sale-photo-viewer-nav prev",r#type:"button",aria_label:"Previous photo",onclick:move|_|selected.set(Some((index+count-1)%count)),Icon {name:"chevron-left",size:30,color:"var(--vl-white)"}}
                    button {id:"sale-photo-next",class:"sale-photo-viewer-nav next",r#type:"button",aria_label:"Next photo",onclick:move|_|selected.set(Some((index+1)%count)),Icon {name:"chevron-right",size:30,color:"var(--vl-white)"}}
                }
                p {class:"sale-photo-viewer-caption",aria_live:"polite",span {"Photo {index+1} of {count}"}span {"{item.photos[index].alt_text}"}}
            }
        }
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

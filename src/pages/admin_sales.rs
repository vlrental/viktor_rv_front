use crate::{
    api::sales::{self, Listing, Payload},
    components::Icon,
};
use dioxus::prelude::*;

#[component]
pub(super) fn SalesTab(mut editor_dirty: Signal<bool>, mut editor_busy: Signal<bool>) -> Element {
    let mut listings = use_signal(Vec::<Listing>::new);
    let mut loading = use_signal(|| true);
    let mut error = use_signal(String::new);
    let mut action_error = use_signal(String::new);
    let mut notice = use_signal(String::new);
    let mut retry = use_signal(|| 0_u32);
    let mut selected = use_signal(|| None::<Listing>);
    let mut new_id = use_signal(String::new);
    let mut search = use_signal(String::new);
    let mut filter = use_signal(|| "all".to_string());
    use_effect(move || {
        let _ = retry();
        loading.set(true);
        error.set(String::new());
        spawn(async move {
            match sales::listings(true).await {
                Ok(rows) => listings.set(rows),
                Err(e) => error.set(e.message),
            }
            loading.set(false);
        });
    });
    let close = move |_| {
        if editor_busy() {
            return;
        }
        if editor_dirty() && !confirm("Discard unsaved sale listing changes?") {
            return;
        }
        selected.set(None);
        new_id.set(String::new());
        editor_dirty.set(false);
    };
    let query = search().trim().to_lowercase();
    let visible = listings()
        .into_iter()
        .filter(|v| filter() == "all" || v.status == filter())
        .filter(|v| {
            format!("{} {} {}", v.title, v.manufacturer, v.model)
                .to_lowercase()
                .contains(&query)
        })
        .collect::<Vec<_>>();
    let editor_key = selected
        .read()
        .as_ref()
        .map(|v| format!("{}-{}", v.sale_id, v.updated_at))
        .unwrap_or_else(&*new_id);
    rsx! {
        if selected.read().is_some() || !new_id().is_empty() {
            SaleEditor {
                key: "{editor_key}",
                id: selected.read().as_ref().map(|v|v.sale_id.clone()).unwrap_or_else(&*new_id),
                initial: selected(), editor_dirty, editor_busy,
                on_close: close,
                on_saved: move |v:Listing| { let mut rows=listings.write(); if let Some(row)=rows.iter_mut().find(|row|row.sale_id==v.sale_id){*row=v.clone()}else{rows.insert(0,v.clone())} selected.set(Some(v));new_id.set(String::new());editor_dirty.set(false); }
            }
        } else {
            section { class: "admin-panel admin-full-panel admin-rvs-panel",
                div { class: "admin-panel-head admin-list-head",
                    div { h2 { "RV Sales" } p { "Manage sale listings, asking prices and photos." } }
                    div { class: "admin-inline-filters",
                        label { class: "admin-search", Icon { name: "search", size: 16, color: "var(--vl-muted)" } input { r#type: "search", aria_label:"Search sale listings", placeholder: "Name or model", value: "{search}", oninput:move|e|search.set(e.value()) } }
                        select { aria_label:"Listing status", value:"{filter}", disabled:editor_busy(), onchange:move|e|filter.set(e.value()),
                            option { value:"all", "All listings" } option { value:"draft", "Draft" } option { value:"published", "Published" } option { value:"sold", "Sold" } option { value:"archived", "Inactive" }
                        }
                        button { class:"admin-primary-small", r#type:"button", disabled:editor_busy(), onclick:move|_|{editor_dirty.set(false);new_id.set(uuid::Uuid::new_v4().to_string());}, Icon { name:"plus",size:15,color:"currentColor" } "Add sale listing" }
                    }
                }
                if !action_error().is_empty() { p {class:"admin-error",role:"alert","{action_error}" button {r#type:"button",disabled:editor_busy(),onclick:move|_|{action_error.set(String::new());retry.set(retry().wrapping_add(1));},"Refresh listings"}} }
                if !notice().is_empty() { p {class:"admin-success",role:"status","{notice}"} }
                if loading() { div { class:"admin-empty", role:"status", "Loading sale listings…" } }
                else if !error().is_empty() { div { class:"admin-error",role:"alert", "{error}" button {r#type:"button",onclick:move|_|retry.set(retry().wrapping_add(1)),"Retry"} } }
                else if visible.is_empty() { div {class:"admin-empty","No sale listings yet. Add a listing, upload its photos, then publish it to RV Sales."} }
                else { div {class:"admin-rv-grid",
                    for item in visible {
                        article { key:"{item.sale_id}",class:"admin-rv-card admin-sale-card",
                            button {class:"admin-sale-card-open",r#type:"button",disabled:editor_busy(),aria_label:"Edit {item.title}",onclick:{let item=item.clone();move|_|selected.set(Some(item.clone()))},
                            if let Some(photo)=item.photos.first() {SalePhoto { photo: photo.clone() }} else {span {class:"admin-rv-placeholder",Icon {name:"image-plus",size:28,color:"currentColor"}}}
                            div {class:"admin-rv-card-copy",span {class:"admin-status", "{status_label(&item.status)}"} h3 {"{item.title}"} p {"{item.location}"}
                                dl { div {dt {"Photos"}dd {"{item.photos.len()}"}} div {dt {"Asking price"}dd {"{sales::price_label(item.price.as_deref())}"}} }
                            }
                            }
                            div {class:"admin-sale-card-actions",
                                button {class:"admin-sale-deactivate",r#type:"button",disabled:editor_busy()||item.status!="published",aria_label:"Deactivate {item.title}",onclick:{let item=item.clone();move|_|{
                                    if editor_busy()||item.status!="published"||!confirm_deactivation(&item.title){return}
                                    let item=item.clone();editor_busy.set(true);action_error.set(String::new());notice.set(String::new());
                                    spawn(async move {match sales::deactivate(&item).await {
                                        Ok(saved)=>{if let Some(row)=listings.write().iter_mut().find(|row|row.sale_id==saved.sale_id){*row=saved;}notice.set(format!("{} is inactive and hidden from RV Sales. Details and photos are saved.",item.title));},
                                        Err(e)=>action_error.set(e.message),
                                    }editor_busy.set(false);});
                                }},"Deactivate"}
                            }
                        }
                    }
                } }
            }
        }
    }
}

fn status_label(status: &str) -> &str {
    match status {
        "draft" => "Draft",
        "published" => "Published",
        "sold" => "Sold",
        "archived" => "Inactive",
        _ => status,
    }
}

fn confirm_deactivation(title: &str) -> bool {
    confirm(&format!(
        "Deactivate \"{title}\"? It will be hidden from RV Sales. Details and photos will be kept, and you can publish it again later."
    ))
}

fn confirm(message: &str) -> bool {
    web_sys::window()
        .and_then(|w| w.confirm_with_message(message).ok())
        .unwrap_or(false)
}

#[component]
fn SaleEditor(
    id: String,
    initial: Option<Listing>,
    mut editor_dirty: Signal<bool>,
    mut editor_busy: Signal<bool>,
    on_close: EventHandler<()>,
    on_saved: EventHandler<Listing>,
) -> Element {
    let baseline = initial.as_ref().map(Payload::from).unwrap_or_default();
    let original = baseline.clone();
    let mut draft = use_signal(|| baseline);
    let mut message = use_signal(String::new);
    let mut file = use_signal(|| None::<web_sys::File>);
    let mut caption = use_signal(String::new);
    use_effect(move || editor_dirty.set(draft() != original || file.read().is_some()));
    let changed = editor_dirty();
    let busy = editor_busy();
    let fields_locked = busy || file.read().is_some();
    let photos = initial
        .as_ref()
        .map(|v| v.photos.clone())
        .unwrap_or_default();
    let saved = initial.is_some();
    let save_id = id.clone();
    let upload_id = id.clone();
    let published = initial.as_ref().is_some_and(|v| v.status == "published");
    let deactivate_initial = initial.clone();
    rsx! {
        section {class:"admin-panel admin-rv-editor-panel",role:"region",aria_label:"Sale listing editor",tabindex:"-1",autofocus:true,onmounted:move|event|async move {let _=event.set_focus(true).await;},onkeydown:move|e|if e.key()==Key::Escape {e.stop_propagation();on_close.call(());},
            div {class:"admin-panel-head",div {h2 {if saved {"Edit sale listing"}else{"Add sale listing"}} p {"Vehicle details, sale price and photos."}} button {r#type:"button",aria_label:"Close sale listing editor",disabled:busy,onclick:move|_|on_close.call(()),"×"} }
            if !message().is_empty(){p {class:"admin-error",role:"status","{message}"}}
            if published {
                div {class:"admin-sale-publication",p {"Published on RV Sales"}
                    button {class:"admin-sale-deactivate",r#type:"button",disabled:fields_locked||changed,onclick:move|_|{
                        if editor_busy()||editor_dirty(){return}let Some(item)=deactivate_initial.clone()else{return};if !confirm_deactivation(&item.title){return}
                        editor_busy.set(true);message.set(String::new());spawn(async move{match sales::deactivate(&item).await{Ok(saved)=>on_saved.call(saved),Err(e)=>message.set(e.message)}editor_busy.set(false);});
                    },"Deactivate"}
                    if changed {small {"Save or discard your changes before deactivating."}}
                }
            } else if initial.as_ref().is_some_and(|v|v.status=="archived") {p {class:"admin-sale-publication","Inactive — hidden from RV Sales. Select Published and save to reactivate."}}
            form {onsubmit:move|e|{
                e.prevent_default();if editor_busy(){return}let payload=draft();
                if payload.status!="draft" && initial.as_ref().is_none_or(|v|v.status!=payload.status) && !confirm("Save this listing status? Published listings appear on RV Sales; draft, sold and inactive listings are hidden."){return}
                let id=save_id.clone();editor_busy.set(true);message.set(String::new());spawn(async move {match sales::save(&id,&payload).await {Ok(v)=>on_saved.call(v),Err(e)=>message.set(e.message)}editor_busy.set(false);});
            },
                section {class:"admin-drawer-section",h3 {"Listing details"}
                    div {class:"admin-drawer-fields",
                        for (field,label,value,limit) in [("title","Listing title",draft().title,160),("manufacturer","Manufacturer",draft().manufacturer,120),("model","Model",draft().model,120),("location","Location",draft().location,160)] {
                            label {"{label}" input {required:field=="title",maxlength:"{limit}",value,disabled:fields_locked,oninput:move|e|{let value=e.value();draft.with_mut(|d|match field {"title"=>d.title=value,"manufacturer"=>d.manufacturer=value,"model"=>d.model=value,_=>d.location=value});}}}
                        }
                        label {"Model year" input {r#type:"number",min:"1900",max:"2100",value:draft().model_year.map(|v|v.to_string()).unwrap_or_default(),disabled:fields_locked,oninput:move|e|draft.with_mut(|d|d.model_year=e.value().parse().ok())}}
                        label {"RV type" select {value:draft().rv_type,disabled:fields_locked,onchange:move|e|draft.with_mut(|d|d.rv_type=e.value()),option {value:"travel_trailer","Travel trailer"}option {value:"fifth_wheel","Fifth wheel"}option {value:"motorhome","Motorhome"}option {value:"camper","Camper"}}}
                        label {"Length (ft)" input {r#type:"number",min:"0.1",max:"100",step:"0.1",value:draft().length_ft.unwrap_or_default(),disabled:fields_locked,oninput:move|e|draft.with_mut(|d|d.length_ft=if e.value().is_empty(){None}else{Some(e.value())})}}
                        label {"Sleeps" input {r#type:"number",min:"1",max:"20",value:draft().sleeps.map(|v|v.to_string()).unwrap_or_default(),disabled:fields_locked,oninput:move|e|draft.with_mut(|d|d.sleeps=e.value().parse().ok())}}
                        label {"Condition" select {value:draft().condition,disabled:fields_locked,onchange:move|e|draft.with_mut(|d|d.condition=e.value()),option {value:"used","Used"}option {value:"new","New"}}}
                        label {"Asking price (CAD)" input {r#type:"number",min:"0.01",max:"9999999999.99",step:"0.01",placeholder:"Leave blank for Enquire for price",value:draft().price.unwrap_or_default(),disabled:fields_locked,oninput:move|e|draft.with_mut(|d|d.price=if e.value().is_empty(){None}else{Some(e.value())})}}
                        label {"Status" select {value:draft().status,disabled:fields_locked,onchange:move|e|draft.with_mut(|d|d.status=e.value()),option {value:"draft","Draft"}option {value:"published",disabled:photos.is_empty(),"Published"}option {value:"sold","Sold"}option {value:"archived","Inactive"}}}
                        label {"Display order" input {r#type:"number",value:"{draft().sort_order}",disabled:fields_locked,oninput:move|e|draft.with_mut(|d|d.sort_order=e.value().parse().unwrap_or(0))}}
                    }
                    label {"Short summary" textarea {maxlength:"500",value:draft().summary,disabled:fields_locked,oninput:move|e|draft.with_mut(|d|d.summary=e.value())}}
                    label {"Sale description" textarea {maxlength:"10000",rows:"6",value:draft().description,disabled:fields_locked,oninput:move|e|draft.with_mut(|d|d.description=e.value())}}
                    small {"To publish, add a summary, description, location and at least one photo. A blank asking price displays “Enquire for price”."}
                }
                div {class:"admin-drawer-actions",button {r#type:"button",disabled:fields_locked,onclick:move|_|on_close.call(()),"Back to listings"}button {class:"primary",r#type:"submit",disabled:fields_locked,if busy {"Saving…"}else{"Save listing"}}}
            }
            section {class:"admin-drawer-section",h3 {"Photos ({photos.len()}/40)"}
                if !saved {p {"Save the listing as a draft to upload photos."}}
                else {
                    if changed && file.read().is_none(){p {"Save your listing changes before editing photos."}}
                    div {class:"admin-rv-photo-upload",
                        label {"Image file" input {r#type:"file",accept:"image/jpeg,image/png,image/webp",disabled:busy||photos.len()>=40||(changed&&file.read().is_none()),oninput:move|e|{if let Some(chosen)=e.files().into_iter().next(){if chosen.size()>10*1024*1024{message.set("Choose an image up to 10 MB.".into());return}file.set(chosen.inner().downcast_ref::<web_sys::File>().cloned());caption.set(format!("{} — exterior",draft().title));}}}if let Some(chosen)=file.read().as_ref(){small {"{chosen.name()}"}}}
                        label {"Photo caption / alt text" input {value:"{caption}",maxlength:"500",disabled:busy,oninput:move|e|caption.set(e.value())}}
                        button {r#type:"button",disabled:busy||file.read().is_none()||caption().trim().is_empty(),onclick:move|_|{let Some(chosen)=file.read().clone()else{return};let id=upload_id.clone();let alt=caption();editor_busy.set(true);spawn(async move{match sales::upload(&id,&chosen,&alt).await{Ok(v)=>{file.set(None);on_saved.call(v)},Err(e)=>message.set(e.message)}editor_busy.set(false);});},"Upload photo"}
                        if file.read().is_some(){button {r#type:"button",disabled:busy,onclick:move|_|{file.set(None);caption.set(String::new());},"Clear selection"}}
                    }
                    div {class:"admin-rv-media-grid",for (index,photo) in photos.iter().enumerate(){article {key:"{photo.photo_id}",class:"admin-rv-media-card",SalePhoto { photo: photo.clone() }p {"{photo.alt_text}"}if index==0 {span {class:"admin-status","Cover photo"}}
                        button {r#type:"button",disabled:busy||changed||index==0,onclick:{let id=id.clone();let photo=photo.clone();move|_|{let id=id.clone();let photo=photo.clone();editor_busy.set(true);spawn(async move{match sales::edit_photo(&id,&photo,true).await{Ok(v)=>on_saved.call(v),Err(e)=>message.set(e.message)}editor_busy.set(false);});}},"Use as cover"}
                        button {r#type:"button",disabled:busy||changed,onclick:{let id=id.clone();let photo=photo.clone();move|_|{if !confirm("Remove this photo from the sale listing?"){return}let id=id.clone();let photo=photo.clone();editor_busy.set(true);spawn(async move{match sales::delete_photo(&id,&photo.photo_id).await{Ok(v)=>on_saved.call(v),Err(e)=>message.set(e.message)}editor_busy.set(false);});}},"Remove photo"}
                    }}}
                }
            }
        }
    }
}

#[component]
fn SalePhoto(photo: sales::Photo) -> Element {
    let mut url = use_signal(String::new);
    let mut error = use_signal(String::new);
    let alt = photo.alt_text.clone();
    use_future(move || {
        let photo = photo.clone();
        async move {
            match sales::admin_photo_url(&photo).await {
                Ok(value) => url.set(value),
                Err(e) => error.set(e.message),
            }
        }
    });
    use_drop(move || {
        let value = url.peek();
        if !value.is_empty() {
            sales::revokeSaleImageUrl(&value);
        }
    });
    rsx! {if !url().is_empty(){img{src:"{url}",alt:"{alt}"}}else if !error().is_empty(){span{role:"status","Photo unavailable"}}else{span{class:"admin-rv-placeholder","Loading photo…"}}}
}

use dioxus::prelude::*;

use crate::{api, components::Icon};

#[component]
pub(super) fn CouponsTab() -> Element {
    let mut coupons = use_signal(Vec::<api::Coupon>::new);
    let mut loading = use_signal(|| true);
    let mut load_error = use_signal(String::new);
    let mut retry = use_signal(|| 0_u32);
    let mut editor_open = use_signal(|| false);
    let mut coupon_id = use_signal(|| None::<String>);
    let mut code = use_signal(String::new);
    let mut percent = use_signal(|| "10".to_string());
    let mut active = use_signal(|| true);
    let mut busy = use_signal(|| false);
    let mut confirmation = use_signal(|| false);
    let mut message = use_signal(String::new);
    let mut error = use_signal(String::new);

    use_effect(move || {
        let _ = retry();
        loading.set(true);
        load_error.set(String::new());
        spawn(async move {
            match api::admin_coupons().await {
                Ok(values) => coupons.set(values),
                Err(value) => load_error.set(value.message),
            }
            loading.set(false);
        });
    });

    rsx! {
        section { class: "admin-panel admin-coupons",
            div { class: "admin-panel-head",
                div {
                    h2 { "Coupons" }
                    p { "Percentage discounts on the amount before tax, excluding Stationary Plus Protection. Existing bookings keep their agreed price." }
                }
                button { class: "admin-primary-small", r#type: "button", disabled: busy() || editor_open(), onclick: move |_| {
                    coupon_id.set(None); code.set(String::new()); percent.set("10".into()); active.set(true);
                    error.set(String::new()); message.set(String::new()); editor_open.set(true);
                }, Icon { name: "plus", size: 15, color: "currentColor" } "New coupon" }
            }
            if !message().is_empty() { p { class: "admin-success", role: "status", "{message}" } }
            if editor_open() {
                form { class: "admin-coupon-editor", onsubmit: move |event| {
                    event.prevent_default();
                    if busy() { return; }
                    let normalized = code().trim().to_ascii_uppercase();
                    let valid_code = (3..=40).contains(&normalized.len()) && normalized.bytes().all(|value| value.is_ascii_alphanumeric() || matches!(value, b'-' | b'_'));
                    if !valid_code { error.set("Use 3–40 letters, numbers, hyphens or underscores for the code.".into()); return; }
                    let valid_percent = percent().parse::<f64>().ok().is_some_and(|value| value.is_finite() && value > 0.0 && value <= 100.0 && (value * 100.0 - (value * 100.0).round()).abs() < 0.000001);
                    if !valid_percent { error.set("Enter a discount greater than 0 and at most 100%, with up to two decimal places.".into()); return; }
                    code.set(normalized); error.set(String::new()); confirmation.set(true);
                },
                    h3 { if coupon_id().is_some() { "Edit coupon" } else { "New coupon" } }
                    div { class: "admin-drawer-fields",
                        label { "Coupon code" input { required: true, value: "{code}", maxlength: "40", autocomplete: "off", disabled: busy(), oninput: move |event| code.set(event.value()) } }
                        label { "Discount (%)" input { required: true, r#type: "number", min: "0.01", max: "100", step: "0.01", value: "{percent}", disabled: busy(), oninput: move |event| percent.set(event.value()) } }
                    }
                    label { class: "admin-coupon-active", input { r#type: "checkbox", checked: active(), disabled: busy(), onchange: move |event| active.set(event.checked()) } "Active — customers can apply this code" }
                    if !error().is_empty() { p { class: "admin-error", role: "alert", "{error}" } }
                    div { class: "admin-drawer-actions",
                        button { r#type: "button", disabled: busy(), onclick: move |_| { editor_open.set(false); error.set(String::new()); }, "Cancel" }
                        button { class: "primary", r#type: "submit", disabled: busy(), "Review and save" }
                    }
                }
            }
            if loading() { p { role: "status", "Loading coupons…" } }
            else if !load_error().is_empty() {
                div { class: "admin-error", role: "alert", "{load_error}" button { r#type: "button", onclick: move |_| retry.set(retry().wrapping_add(1)), "Retry" } }
            } else if coupons.read().is_empty() {
                div { class: "admin-empty", "No coupons yet. Create your first percentage discount." }
            } else {
                div { class: "admin-coupon-list",
                    for coupon in coupons.read().iter().cloned() {
                        article { key: "{coupon.coupon_id}", class: "admin-coupon-row",
                            div { strong { "{coupon.code}" } span { "{coupon.percent_off}% off" } }
                            span { class: if coupon.is_active { "admin-coupon-badge is-active" } else { "admin-coupon-badge" }, if coupon.is_active { "Active" } else { "Inactive" } }
                            div { class: "admin-coupon-actions",
                                button { r#type: "button", disabled: busy() || editor_open(), onclick: {
                                    let value = coupon.clone(); move |_| {
                                        coupon_id.set(Some(value.coupon_id.clone())); code.set(value.code.clone()); percent.set(value.percent_off.clone()); active.set(value.is_active);
                                        message.set(String::new()); error.set(String::new()); editor_open.set(true);
                                    }
                                }, "Edit" }
                                button { r#type: "button", disabled: busy() || editor_open(), onclick: {
                                    let value = coupon.clone(); move |_| {
                                        coupon_id.set(Some(value.coupon_id.clone())); code.set(value.code.clone()); percent.set(value.percent_off.clone()); active.set(!value.is_active);
                                        error.set(String::new()); confirmation.set(true);
                                    }
                                }, if coupon.is_active { "Deactivate" } else { "Activate" } }
                            }
                        }
                    }
                }
            }
        }
        if confirmation() {
            div { class: "admin-confirm-layer", onclick: move |_| if !busy() { confirmation.set(false); },
                section { class: "admin-confirm-modal", role: "alertdialog", aria_modal: "true", aria_label: "Confirm coupon changes", tabindex: "-1", onmounted: move |event| async move { let _ = event.set_focus(true).await; }, onclick: move |event| event.stop_propagation(), onkeydown: move |event| if event.key() == Key::Escape { event.stop_propagation(); if !busy() { confirmation.set(false); } },
                    header { h3 { "Save coupon?" } button { r#type: "button", aria_label: "Close confirmation", disabled: busy(), onclick: move |_| confirmation.set(false), Icon { name: "x", size: 16, color: "currentColor" } } }
                    p { "{code} · {percent}% off · " if active() { "Active" } else { "Inactive" } }
                    p { if active() { "Customers can use this code on new bookings. The discount excludes taxes, protection and the refundable damage deposit." } else { "Customers will no longer be able to use this code for new bookings. Existing reservations keep their agreed price." } }
                    if !error().is_empty() { p { class: "admin-error", role: "alert", "{error}" } }
                    footer {
                        button { r#type: "button", disabled: busy(), onclick: move |_| confirmation.set(false), "Cancel" }
                        button { class: "primary", r#type: "button", disabled: busy(), onclick: move |_| async move {
                            if busy() { return; }
                            busy.set(true); error.set(String::new());
                            let id = coupon_id();
                            let payload = api::CouponPayload { code: code(), percent_off: percent(), is_active: active() };
                            match api::save_admin_coupon(id.as_deref(), payload).await {
                                Ok(saved) => {
                                    let status = if saved.is_active { "active" } else { "inactive" };
                                    message.set(format!("Coupon {} saved — {status}.", saved.code));
                                    let mut values = coupons();
                                    if let Some(index) = values.iter().position(|value| value.coupon_id == saved.coupon_id) { values[index] = saved; }
                                    else { values.insert(0, saved); }
                                    coupons.set(values); confirmation.set(false); editor_open.set(false);
                                }
                                Err(value) => error.set(if value.is_conflict() { "This coupon code already exists or was changed. Reload and try again.".into() } else { value.message }),
                            }
                            busy.set(false);
                        }, if busy() { "Saving…" } else { "Save coupon" } }
                    }
                }
            }
        }
    }
}

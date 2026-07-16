use crate::data::PHONE;
use crate::{api, components::Icon};
use dioxus::prelude::*;

/// Страница «Contact» — Pencil: desktop `v1lkS`, mobile `Kzp7R`.
#[component]
pub fn Contact() -> Element {
    let mut full_name = use_signal(String::new);
    let mut email = use_signal(String::new);
    let mut phone = use_signal(String::new);
    let mut interest = use_signal(|| "rv".to_string());
    let mut message = use_signal(String::new);
    let mut status = use_signal(String::new);
    let mut busy = use_signal(|| false);
    let submit = move |_| {
        let values = (
            full_name.read().clone(),
            email.read().clone(),
            phone.read().clone(),
            interest.read().clone(),
            message.read().clone(),
        );
        async move {
            status.set(String::new());
            if values.0.trim().len() < 2 || !values.1.contains('@') || values.4.trim().len() < 5 {
                status.set("Enter your name, a valid email, and a short message.".into());
                return;
            }
            busy.set(true);
            match api::send_contact(&values.0, &values.1, &values.2, &values.3, &values.4).await {
                Ok(()) => {
                    status.set(
                        "Thanks — your message has been saved. We'll get back to you soon.".into(),
                    );
                    message.set(String::new());
                }
                Err(error) => status.set(error),
            }
            busy.set(false);
        }
    };
    rsx! {
        // Тёмная шапка страницы
        section { class: "ct-header",
            div { class: "eyebrow gold", "WE'RE HERE TO HELP" }
            h1 { class: "ct-title", "Get in touch" }
            p { class: "ct-sub",
                "Questions about a rental, delivery or dates? Send us a message and our local Okanagan team will get right back to you."
            }
        }

        // Форма + инфо-карточки
        section { class: "ct-body",
            div { class: "ct-form",
                h2 { class: "ct-form-title", "Send us a message" }
                div { class: "ct-row",
                    label { class: "ct-field",
                        span { class: "ct-label", "Full name" }
                        input { class: "ct-input", r#type: "text", placeholder: "Your name", value: "{full_name}", oninput: move |e| full_name.set(e.value()) }
                    }
                    label { class: "ct-field",
                        span { class: "ct-label", "Email" }
                        input { class: "ct-input", r#type: "email", placeholder: "you@email.com", value: "{email}", oninput: move |e| email.set(e.value()) }
                    }
                }
                div { class: "ct-row",
                    label { class: "ct-field",
                        span { class: "ct-label", "Phone" }
                        input { class: "ct-input", r#type: "tel", placeholder: "+1 (250) 000 0000", value: "{phone}", oninput: move |e| phone.set(e.value()) }
                    }
                    label { class: "ct-field",
                        span { class: "ct-label", "Interested in" }
                        div { class: "ct-select-wrap",
                            select { class: "ct-select", value: "{interest}", onchange: move |e| interest.set(e.value()),
                                option { value: "rv", "RV Rental" }
                                option { value: "cooler", "Cooler Trailer" }
                                option { value: "other", "Something else" }
                            }
                            span { class: "ct-select-chevron",
                                Icon { name: "chevron-down", size: 16, color: "var(--vl-muted)" }
                            }
                        }
                    }
                }
                label { class: "ct-field ct-field-full",
                    span { class: "ct-label", "Message" }
                    textarea {
                        class: "ct-textarea",
                        placeholder: "Tell us your dates and what you're looking for…",
                        value: "{message}",
                        oninput: move |e| message.set(e.value()),
                    }
                }
                if !status.read().is_empty() { p { role: "status", "{status}" } }
                button { class: "ct-send", disabled: *busy.read(), onclick: submit,
                    if *busy.read() { "Sending…" } else { "Send message" }
                    Icon { name: "send", size: 16, color: "var(--vl-white)" }
                }
            }

            div { class: "ct-info",
                div { class: "ct-info-card",
                    div { class: "ct-ib",
                        Icon { name: "phone", size: 20, color: "var(--vl-white)" }
                    }
                    div { class: "ct-info-c",
                        span { class: "ct-info-l", "Call or text" }
                        span { class: "ct-info-v", {PHONE} }
                    }
                }
                div { class: "ct-info-card",
                    div { class: "ct-ib",
                        Icon { name: "map-pin", size: 20, color: "var(--vl-white)" }
                    }
                    div { class: "ct-info-c",
                        span { class: "ct-info-l", "Based in" }
                        span { class: "ct-info-v", "Kelowna, British Columbia" }
                    }
                }
                div { class: "ct-info-card",
                    div { class: "ct-ib",
                        Icon { name: "clock", size: 20, color: "var(--vl-white)" }
                    }
                    div { class: "ct-info-c",
                        span { class: "ct-info-l", "Response time" }
                        span { class: "ct-info-v", "Usually within a few hours" }
                    }
                }
                div { class: "ct-info-card",
                    div { class: "ct-info-c ct-info-grow",
                        span { class: "ct-info-l", "Follow us" }
                        span { class: "ct-info-v", "@lairichviktor" }
                    }
                    div { class: "ct-social-btns",
                        a { class: "ct-social-btn", href: "https://www.facebook.com/people/VL-Pro-Trailer-Care/61576201770508/", target: "_blank", rel: "noopener noreferrer", aria_label: "VL Rental on Facebook",
                            Icon { name: "facebook", size: 18, color: "var(--vl-white)" }
                        }
                        a { class: "ct-social-btn", href: "https://www.instagram.com/lairichviktor/", target: "_blank", rel: "noopener noreferrer", aria_label: "VL Rental on Instagram",
                            Icon { name: "instagram", size: 18, color: "var(--vl-white)" }
                        }
                    }
                }
            }
        }
    }
}

use crate::components::Icon;
use crate::data::PHONE;
use dioxus::prelude::*;

const CSS: Asset = asset!("/assets/css/contact.css");

/// Страница «Contact» — Pencil: desktop `v1lkS`, mobile `Kzp7R`.
#[component]
pub fn Contact() -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: CSS }

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
                        input { class: "ct-input", r#type: "text", placeholder: "Your name" }
                    }
                    label { class: "ct-field",
                        span { class: "ct-label", "Email" }
                        input { class: "ct-input", r#type: "email", placeholder: "you@email.com" }
                    }
                }
                div { class: "ct-row",
                    label { class: "ct-field",
                        span { class: "ct-label", "Phone" }
                        input { class: "ct-input", r#type: "tel", placeholder: "+1 (250) 000 0000" }
                    }
                    label { class: "ct-field",
                        span { class: "ct-label", "Interested in" }
                        div { class: "ct-select-wrap",
                            select { class: "ct-select",
                                option { disabled: true, selected: true, value: "", "Select a rental…" }
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
                    }
                }
                button { class: "ct-send",
                    "Send message"
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
                        a { class: "ct-social-btn", href: "#",
                            Icon { name: "facebook", size: 18, color: "var(--vl-white)" }
                        }
                        a { class: "ct-social-btn", href: "#",
                            Icon { name: "instagram", size: 18, color: "var(--vl-white)" }
                        }
                    }
                }
            }
        }
    }
}

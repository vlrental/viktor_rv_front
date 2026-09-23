use crate::faq_content::faq_document;
use crate::Route;
use dioxus::prelude::*;

#[component]
pub fn Faq() -> Element {
    let document = faq_document();

    rsx! {
        section { class: "faq-hero",
            div { class: "faq-hero-inner",
                p { class: "eyebrow", "PLAN YOUR STAY" }
                h1 { "RV rental questions, answered" }
                p {
                    "Clear information about booking, delivery, campsite setup, paid extras and using your RV in Kelowna and the Okanagan."
                }
            }
        }

        nav { class: "faq-jump", aria_label: "FAQ categories",
            for category in document.categories.iter() {
                a { key: "jump-{category.id}", href: "#{category.id}", "{category.title}" }
            }
        }

        div { class: "faq-main",
            for category in document.categories.iter() {
                section { key: "category-{category.id}", id: "{category.id}", class: "faq-category",
                    header {
                        p { class: "faq-category-number", "{category.items.len()} QUESTIONS" }
                        h2 { "{category.title}" }
                    }
                    div { class: "faq-list",
                        for item in category.items.iter() {
                            details { key: "faq-{item.id}", id: "{item.id}", class: "faq-item",
                                summary {
                                    span { "{item.question}" }
                                    span { class: "faq-toggle", aria_hidden: "true" }
                                }
                                div { class: "faq-answer",
                                    p { "{item.answer}" }
                                    if !item.bullets.is_empty() {
                                        ul {
                                            for bullet in item.bullets.iter() {
                                                li { key: "{item.id}-{bullet}", "{bullet}" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            aside { class: "faq-contact",
                div {
                    p { class: "eyebrow gold", "STILL HAVE A QUESTION?" }
                    h2 { "Talk with the local VL Rental team" }
                    p { "Ask about an RV, your campsite or an existing booking." }
                }
                div { class: "faq-contact-actions",
                    Link { class: "btn-gold", to: Route::Contact {}, "Contact us" }
                    Link { class: "faq-terms-link", to: Route::Terms {}, "Read rental terms" }
                }
            }
        }
    }
}

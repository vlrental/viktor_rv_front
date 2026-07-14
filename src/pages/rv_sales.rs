use dioxus::prelude::*;

use crate::data::{IMG_BULLET, IMG_OPENRANGE, IMG_ROCKWOOD};
use crate::{api, components::Icon};

const CSS: Asset = asset!("/assets/css/rv_sales.css");

/// Юнит будущей продажи (пока «Coming soon» — цена по запросу).
struct SaleItem {
    title: &'static str,
    meta: &'static str,
    image: Asset,
}

fn sale_items() -> Vec<SaleItem> {
    vec![
        SaleItem {
            title: "Keystone Bullet 272BHS",
            meta: "2015 · Sleeps 10",
            image: IMG_BULLET,
        },
        SaleItem {
            title: "Forest River Rockwood",
            meta: "2014 · Sleeps 4",
            image: IMG_ROCKWOOD,
        },
        SaleItem {
            title: "Open Range 26BHS",
            meta: "2025 · Sleeps 8",
            image: IMG_OPENRANGE,
        },
    ]
}

#[component]
pub fn RvSales() -> Element {
    let items = sale_items();
    let mut full_name = use_signal(String::new);
    let mut email = use_signal(String::new);
    let mut requested = use_signal(String::new);
    let mut status = use_signal(String::new);
    let mut busy = use_signal(|| false);
    let send = move |_| {
        let values = (
            full_name.read().clone(),
            email.read().clone(),
            requested.read().clone(),
        );
        async move {
            status.set(String::new());
            if values.0.trim().len() < 2 || !values.1.contains('@') || values.2.trim().len() < 2 {
                status.set(
                    "Enter your name, a valid email, and the RV or budget you are looking for."
                        .into(),
                );
                return;
            }
            busy.set(true);
            match api::send_sales_inquiry(
                &values.0,
                &values.1,
                "",
                &values.2,
                "Interested in an RV purchase",
            )
            .await
            {
                Ok(()) => {
                    status.set("Thanks — your sales request has been saved.".into());
                    requested.set(String::new());
                }
                Err(error) => status.set(error),
            }
            busy.set(false);
        }
    };
    rsx! {
        document::Link { rel: "stylesheet", href: CSS }

        // Заголовок страницы
        section { class: "sale-header",
            div { class: "sale-soon",
                Icon { name: "hourglass", size: 13, color: "var(--vl-white)" }
                span { "SALES COMING SOON" }
            }
            div { class: "eyebrow", "RV SALES" }
            h1 { class: "sale-title", "Own your next adventure" }
            p { class: "sale-sub",
                "RV and cooler trailer sales are coming soon! In the meantime, explore our rental fleet — and tell us what you're looking to buy so we can reach out first."
            }
        }

        // Сетка юнитов на продажу
        section { class: "sale-grid",
            for item in items {
                div { key: "{item.title}", class: "sale-card",
                    div {
                        class: "sale-img",
                        style: "background-image: url('{item.image}');",
                        div { class: "sale-badge",
                            Icon { name: "star", size: 12, color: "var(--vl-accent)" }
                            span { "Coming soon" }
                        }
                        div { class: "sale-heart",
                            Icon { name: "heart", size: 16, color: "var(--vl-ink)" }
                        }
                    }
                    div { class: "sale-body",
                        div { class: "sale-card-title", "{item.title}" }
                        div { class: "sale-meta", "{item.meta}" }
                        div { class: "sale-price-row",
                            span { class: "sale-price", "Enquire" }
                            span { class: "sale-per", "for price" }
                        }
                    }
                }
            }
        }

        // CTA-полоса с телефоном
        section { class: "sale-cta",
            div { class: "sale-cta-text",
                div { class: "sale-cta-title", "Looking for something specific?" }
                p { class: "sale-cta-sub",
                    "Tell us your budget and how you like to travel — we'll help you find the perfect rig."
                }
                div { class: "ct-row",
                    input { class: "ct-input", placeholder: "Your name", value: "{full_name}", oninput: move |e| full_name.set(e.value()) }
                    input { class: "ct-input", r#type: "email", placeholder: "you@email.com", value: "{email}", oninput: move |e| email.set(e.value()) }
                    input { class: "ct-input", placeholder: "RV or budget", value: "{requested}", oninput: move |e| requested.set(e.value()) }
                }
                if !status.read().is_empty() { p { role: "status", "{status}" } }
            }
            button { class: "sale-cta-btn", disabled: *busy.read(), onclick: send,
                Icon { name: "send", size: 16, color: "var(--vl-forest-2)" }
                span { if *busy.read() { "Sending…" } else { "Send inquiry" } }
            }
        }
    }
}

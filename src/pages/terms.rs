use dioxus::prelude::*;
use serde::Deserialize;
use std::collections::HashSet;

const TERMS_JSON: &str = include_str!("../../content/terms.json");
const EXPECTED_TERM_SECTIONS: usize = 20;

#[derive(Clone, Debug, Deserialize, PartialEq)]
struct TermsDocument {
    sections: Vec<TermSection>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
struct TermSection {
    id: String,
    title: String,
    paragraphs: Vec<String>,
    #[serde(default)]
    bullets: Vec<String>,
}

fn parse_terms_document(source: &str) -> Result<TermsDocument, String> {
    let document: TermsDocument =
        serde_json::from_str(source).map_err(|error| format!("invalid terms JSON: {error}"))?;
    if document.sections.len() != EXPECTED_TERM_SECTIONS {
        return Err(format!(
            "terms content must contain exactly {EXPECTED_TERM_SECTIONS} sections"
        ));
    }

    let mut ids = HashSet::new();
    for section in &document.sections {
        if section.id.trim().is_empty()
            || section.title.trim().is_empty()
            || section.paragraphs.is_empty()
            || section.paragraphs.iter().any(|text| text.trim().is_empty())
            || section.bullets.iter().any(|text| text.trim().is_empty())
        {
            return Err("terms sections require an id, title, and non-empty content".to_string());
        }
        if !ids.insert(section.id.as_str()) {
            return Err(format!("duplicate terms section id: {}", section.id));
        }
    }

    Ok(document)
}

fn terms_document() -> TermsDocument {
    parse_terms_document(TERMS_JSON)
        .expect("content/terms.json must contain complete terms content")
}

#[component]
pub(crate) fn TermsAgreementContent() -> Element {
    let document = terms_document();
    rsx! {
        div { class: "tm-notice",
            strong { "Booking-specific details matter." }
            span { " Your accepted dates, RV, delivery address, itemized price, payment schedule, and selected extras are part of the Agreement." }
        }
        for section in document.sections.iter() {
            section { key: "{section.id}", id: "terms-{section.id}", class: "tm-sec",
                h2 { class: "tm-sec-h", "{section.title}" }
                for (index, paragraph) in section.paragraphs.iter().enumerate() {
                    p { key: "{section.id}-p-{index}", class: "tm-sec-p", "{paragraph}" }
                }
                if !section.bullets.is_empty() {
                    ul { class: "tm-list",
                        for (index, bullet) in section.bullets.iter().enumerate() {
                            li { key: "{section.id}-b-{index}", "{bullet}" }
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn Terms() -> Element {
    let document = terms_document();
    rsx! {
        section { class: "tm-header",
            div { class: "tm-kicker", "VL RENTAL · KELOWNA, BC" }
            h1 { class: "tm-title", "RV Rental Terms & Conditions" }
            p { class: "tm-sub",
                "These Terms govern delivery-only RV bookings with VL Rental in Kelowna and approved Okanagan destinations. Review them together with your itemized booking summary before accepting."
            }
            div { class: "tm-updated", "Effective September 23, 2026" }
        }

        section { class: "tm-body",
            nav { class: "tm-toc", aria_label: "Terms sections",
                div { class: "tm-toc-h", "ON THIS PAGE" }
                for section in document.sections.iter() {
                    a { key: "toc-{section.id}", class: "tm-toc-link", href: "#terms-{section.id}", "{section.title}" }
                }
                a { class: "tm-toc-link", href: "#terms-contact", "21. Contact and notices" }
            }
            article { class: "tm-content",
                TermsAgreementContent {}
                div { class: "tm-contact", id: "terms-contact",
                    div {
                        h2 { "21. Contact and notices" }
                        p { "VL Rental · Kelowna, British Columbia, Canada" }
                        p { "Email Vlrental.ca@gmail.com · Phone +1 (250) 878-5874" }
                    }
                    Link { class: "tm-help-btn", to: crate::Route::Contact {}, "Contact VL Rental" }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_terms_have_exactly_twenty_complete_sections() {
        let document = terms_document();
        assert_eq!(document.sections.len(), EXPECTED_TERM_SECTIONS);
        assert_eq!(document.sections.first().unwrap().id, "agreement");
        assert_eq!(document.sections.last().unwrap().id, "general");
    }

    #[test]
    fn canonical_terms_retain_approved_policy_phrases() {
        let source = TERMS_JSON;
        for phrase in [
            "RV Preparation Fee: CA$97 once per booking.",
            "Stationary Plus Protection: CA$150 fixed per trip",
            "separate refundable CA$1,000 damage deposit",
            "100% of the trip price when the request is received 14 days or less before delivery",
        ] {
            assert!(source.contains(phrase), "missing policy phrase: {phrase}");
        }
    }

    #[test]
    fn malformed_or_empty_terms_fail_closed() {
        assert!(parse_terms_document("not json").is_err());
        assert!(parse_terms_document(r#"{"sections": []}"#).is_err());
    }
}

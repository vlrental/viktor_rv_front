use serde::{Deserialize, Serialize};

const FAQ_JSON: &str = include_str!("../content/faq.json");

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct FaqDocument {
    pub categories: Vec<FaqCategory>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct FaqCategory {
    pub id: String,
    pub title: String,
    pub items: Vec<FaqItem>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct FaqItem {
    pub id: String,
    pub question: String,
    pub answer: String,
    #[serde(default)]
    pub bullets: Vec<String>,
    #[serde(default)]
    pub featured_home: Option<u8>,
}

pub fn faq_document() -> FaqDocument {
    serde_json::from_str(FAQ_JSON).expect("content/faq.json must contain valid FAQ content")
}

pub fn featured_faq_items() -> Vec<FaqItem> {
    let mut items = faq_document()
        .categories
        .into_iter()
        .flat_map(|category| category.items)
        .filter(|item| item.featured_home.is_some())
        .collect::<Vec<_>>();
    items.sort_by_key(|item| item.featured_home);
    items
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn faq_content_has_five_groups_and_unique_questions() {
        let document = faq_document();
        assert_eq!(document.categories.len(), 5);
        let items = document
            .categories
            .iter()
            .flat_map(|category| category.items.iter())
            .collect::<Vec<_>>();
        assert_eq!(items.len(), 50);
        assert_eq!(
            items
                .iter()
                .map(|item| item.id.as_str())
                .collect::<HashSet<_>>()
                .len(),
            items.len()
        );
        assert!(items
            .iter()
            .all(|item| { !item.question.trim().is_empty() && !item.answer.trim().is_empty() }));
    }

    #[test]
    fn home_has_exactly_six_ordered_faqs() {
        let items = featured_faq_items();
        assert_eq!(items.len(), 6);
        assert_eq!(
            items
                .iter()
                .filter_map(|item| item.featured_home)
                .collect::<Vec<_>>(),
            vec![1, 2, 3, 4, 5, 6]
        );
    }

    #[test]
    fn approved_customer_policies_are_present() {
        let serialized = serde_json::to_string(&faq_document()).unwrap();
        for approved in [
            "non-refundable CA$100 pet fee",
            "optional CA$50 extra",
            "Bedding sets are available for CA$40 per selected bed",
            "Wastewater emptying costs CA$45",
            "30-amp or 50-amp hookup",
            "dry camping under normal use",
            "9:00 AM to 8:00 PM",
        ] {
            assert!(serialized.contains(approved), "missing policy: {approved}");
        }
        assert!(faq_document().categories.iter().all(|category| category
            .items
            .iter()
            .all(|item| !item.answer.to_ascii_lowercase().contains("fully equipped"))));
    }
}

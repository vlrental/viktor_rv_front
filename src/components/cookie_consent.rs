use dioxus::prelude::*;

use crate::Route;

const COOKIE_CONSENT_KEY: &str = "vl_cookie_consent_v1";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CookieConsent {
    Undecided,
    NecessaryOnly,
    All,
}

impl CookieConsent {
    fn from_stored(value: Option<&str>) -> Self {
        match value {
            Some("necessary") => Self::NecessaryOnly,
            Some("all") => Self::All,
            _ => Self::Undecided,
        }
    }

    fn stored_value(self) -> Option<&'static str> {
        match self {
            Self::Undecided => None,
            Self::NecessaryOnly => Some("necessary"),
            Self::All => Some("all"),
        }
    }
}

#[derive(Clone, Copy)]
pub struct CookieConsentContext(pub Signal<CookieConsent>);

pub fn saved_cookie_consent() -> CookieConsent {
    let value = web_sys::window()
        .and_then(|window| window.local_storage().ok().flatten())
        .and_then(|storage| storage.get_item(COOKIE_CONSENT_KEY).ok().flatten());
    CookieConsent::from_stored(value.as_deref())
}

fn save_cookie_consent(consent: CookieConsent) {
    let Some(value) = consent.stored_value() else {
        return;
    };
    if let Some(storage) =
        web_sys::window().and_then(|window| window.local_storage().ok().flatten())
    {
        let _ = storage.set_item(COOKIE_CONSENT_KEY, value);
    }
}

#[component]
pub fn CookieConsentBanner() -> Element {
    let consent = use_context::<CookieConsentContext>();
    if *consent.0.read() != CookieConsent::Undecided {
        return rsx! {};
    }

    let mut necessary_consent = consent;
    let mut all_consent = consent;

    rsx! {
        section {
            class: "cookie-consent",
            role: "region",
            aria_labelledby: "cookie-consent-title",
            aria_describedby: "cookie-consent-description",
            div { class: "cookie-consent-copy",
                h2 { id: "cookie-consent-title", "Your privacy choices" }
                p { id: "cookie-consent-description",
                    "Necessary browser storage keeps sign-in and booking progress working. With your permission, we also load optional YouTube media, which may use cookies."
                }
                Link { class: "cookie-consent-link", to: Route::Privacy {}, "Read the Privacy & Cookie Policy" }
            }
            div { class: "cookie-consent-actions",
                button {
                    class: "cookie-consent-no",
                    r#type: "button",
                    onclick: move |_| {
                        save_cookie_consent(CookieConsent::NecessaryOnly);
                        necessary_consent.0.set(CookieConsent::NecessaryOnly);
                    },
                    "No, necessary only"
                }
                button {
                    class: "cookie-consent-yes",
                    r#type: "button",
                    onclick: move |_| {
                        save_cookie_consent(CookieConsent::All);
                        all_consent.0.set(CookieConsent::All);
                    },
                    "Yes, allow"
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_known_consent_values_are_accepted() {
        assert_eq!(
            CookieConsent::from_stored(Some("necessary")),
            CookieConsent::NecessaryOnly
        );
        assert_eq!(CookieConsent::from_stored(Some("all")), CookieConsent::All);
        assert_eq!(
            CookieConsent::from_stored(Some("unexpected")),
            CookieConsent::Undecided
        );
        assert_eq!(CookieConsent::from_stored(None), CookieConsent::Undecided);
    }
}

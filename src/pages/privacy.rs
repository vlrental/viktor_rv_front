use dioxus::prelude::*;

#[derive(Clone, Copy)]
struct PrivacySection {
    id: &'static str,
    title: &'static str,
    paragraphs: &'static [&'static str],
    bullets: &'static [&'static str],
}

const PRIVACY_SECTIONS: &[PrivacySection] = &[
    PrivacySection {
        id: "privacy-scope",
        title: "1. Scope and accountability",
        paragraphs: &[
            "This Privacy & Cookie Policy explains how VL Rental collects, uses, discloses, retains, and protects personal information through vlrental.ca, customer accounts, quotes, delivery estimates, RV bookings, payments, reviews, contact requests, newsletters, and support. It does not govern an independent third party's website or service after you leave VL Rental.",
            "VL Rental is responsible for personal information under its control. The designated Privacy Officer is the point of contact for questions, access or correction requests, withdrawal of optional consent, and complaints. Depending on the transaction, British Columbia's Personal Information Protection Act (PIPA), the federal Personal Information Protection and Electronic Documents Act (PIPEDA), Canada's anti-spam law, and other applicable laws may apply.",
        ],
        bullets: &[],
    },
    PrivacySection {
        id: "privacy-information",
        title: "2. Personal information we collect",
        paragraphs: &[
            "We collect only information reasonably needed for the purposes described in this Policy. The information varies with the features you use and may include:",
        ],
        bullets: &[
            "Identity and contact information: name, email address, telephone number, account identifier, and authenticated profile details.",
            "Booking information: selected RV, dates, guest count, delivery address and distance, campsite details, extras, notes, timezone, quote, booking number, status, and communications.",
            "Payment information: amount, currency, payment schedule, Stripe customer, Checkout, PaymentIntent, invoice or refund references, payment status, and limited transaction metadata. Full card numbers and security codes are entered directly into Stripe and are not stored by VL Rental.",
            "Safety, damage, and claim information: inspection notes, incident details, reasons for a deposit decision, private photographs, repair or cleaning records, and related correspondence.",
            "Account and security information: password hash held by the backend, sign-in provider identifiers, session records, one-time-code hashes, authentication events, and fraud or abuse indicators.",
            "Messages and public content: contact or sales inquiries, newsletter subscription, reviews, ratings, review likes, and any information you choose to include.",
            "Device and technical information: IP address, request method and path, browser or device information made available by standard web requests, timestamps, error and security events, consent choice, local/session storage, and push-notification registration status.",
        ],
    },
    PrivacySection {
        id: "privacy-sources",
        title: "3. Sources of information",
        paragraphs: &[
            "Most information comes directly from you when you search, request a quote, calculate delivery, create an account, sign in, book, pay, contact us, subscribe, enable notifications, or submit a review. We also receive information from service providers when needed to complete the action you requested, including payment status from Stripe, basic authenticated profile information from Google when you choose Google sign-in, address or route results from mapping providers, and delivery or security status from communications and hosting providers.",
            "We may create records derived from those sources, such as a distance estimate, immutable quote, payment obligation, booking status, fraud alert, audit event, or eligibility to submit a review. We do not purchase consumer profiles or data-broker lists.",
        ],
        bullets: &[],
    },
    PrivacySection {
        id: "privacy-purposes",
        title: "4. Why we collect, use, and disclose information",
        paragraphs: &[
            "VL Rental handles personal information for purposes a reasonable person would consider appropriate, including to:",
        ],
        bullets: &[
            "provide RV availability, address suggestions, delivery estimates, quotes, bookings, delivery, support, account functions, reviews, and requested communications;",
            "authenticate users, protect accounts, rotate sessions, prevent duplicate or fraudulent bookings, enforce rate limits, investigate incidents, and maintain audit records;",
            "process provider-confirmed payments, scheduled balances, refundable deposits, refunds, disputes, and financial reconciliation;",
            "send confirmations, due-payment links, safety or service notices, refund results, and other operational email or push notifications;",
            "administer damage evidence, inspection, cleaning, repair, insurance or protection matters, complaints, and legal claims;",
            "maintain, troubleshoot, secure, and improve the service using minimized operational logs and aggregate information;",
            "comply with tax, accounting, consumer-protection, privacy, court, regulatory, and other legal obligations; and",
            "send marketing only where consent or another lawful basis exists and honour unsubscribe requests.",
        ],
    },
    PrivacySection {
        id: "privacy-consent",
        title: "5. Consent and other lawful handling",
        paragraphs: &[
            "When you voluntarily provide information for an obvious purpose—such as giving an address for a delivery estimate or an email for a requested booking—we use it for that purpose and closely related service needs. We ask for an affirmative choice before optional YouTube media is loaded. We do not make optional media consent a condition of receiving a quote or booking an RV.",
            "You may withdraw optional consent on reasonable notice by changing Cookie choices, unsubscribing, disabling push notifications, or contacting the Privacy Officer. Withdrawal does not affect handling already lawfully completed and cannot prevent handling required to fulfil a booking, collect or refund an amount, preserve a legal record, investigate fraud, or meet another legal obligation. We will explain a material consequence when applicable.",
        ],
        bullets: &[],
    },
    PrivacySection {
        id: "privacy-sharing",
        title: "6. Service providers and disclosures",
        paragraphs: &[
            "VL Rental does not sell or rent personal information. We disclose limited information to service providers only for the service they perform, or as otherwise permitted or required by law. Key provider categories currently include:",
        ],
        bullets: &[
            "Stripe for secure payment entry, Checkout, invoices, payment status, deposits, and refunds;",
            "Google for sign-in when selected and YouTube media only after optional-cookie consent;",
            "Amazon Web Services for operational email and notification infrastructure, and Firebase Cloud Messaging for push notifications enabled by the user;",
            "Supabase and other controlled database, private storage, hosting, backup, and infrastructure providers;",
            "GitHub Pages and related delivery infrastructure for the public frontend;",
            "OpenStreetMap, Photon/Komoot, Nominatim, and OSRM for map tiles, address search, and route or distance calculation; and",
            "professional advisers, insurers, protection providers, repairers, law enforcement, courts, or regulators when reasonably necessary and legally permitted.",
        ],
    },
    PrivacySection {
        id: "privacy-transfers",
        title: "7. Processing outside British Columbia",
        paragraphs: &[
            "Some providers may process or store information in another Canadian province, the United States, or another country. While information is there, it may be subject to lawful access under that jurisdiction's laws. VL Rental remains accountable for personal information under its control and uses reasonable contractual, technical, and organizational measures appropriate to the sensitivity and provider relationship.",
            "Contact the Privacy Officer for more information about a material service-provider location or VL Rental's use of providers outside Canada.",
        ],
        bullets: &[],
    },
    PrivacySection {
        id: "privacy-cookies",
        title: "8. Cookies and browser storage",
        paragraphs: &[
            "The site uses cookies and similar browser technologies, including local storage and session storage. Necessary storage is used without optional-media consent because the site cannot safely provide requested authentication, booking, payment, security, and preference functions without it. Rejecting optional media does not disable those necessary functions.",
        ],
        bullets: &[
            "Consent preference: vl_cookie_consent_v1 stores “all” or “necessary” in local storage until you change it or clear browser data.",
            "Authentication and private booking state: access/refresh tokens, the signed-in profile, one-time return state, pending Checkout secret, and private booking access token use session storage and are cleared on sign-out, consumption, expiry, or the end of the browser session as applicable.",
            "Booking convenience: search choices, non-sensitive trip drafts, saved RVs, and recent delivery addresses may use local storage until removed or browser data is cleared.",
            "OAuth security: the API sets a short-lived, HttpOnly, SameSite=Lax state cookie during a Google sign-in attempt to prevent forged callbacks.",
            "Optional YouTube media: after “Yes, allow,” the site may load the YouTube API and embedded video. Google may receive IP address and device/request data and may set or read its own cookies or identifiers under Google's policies.",
            "Requested third-party functions: Stripe, maps, Google sign-in, and push notifications connect to their providers only when needed for the feature you request. Their necessary storage and data transfers are governed by the applicable provider and this Policy.",
        ],
    },
    PrivacySection {
        id: "privacy-no-ad-cookies",
        title: "9. No VL Rental analytics or advertising cookies",
        paragraphs: &[
            "As of the effective date, VL Rental does not use third-party advertising pixels, cross-site behavioural advertising, or analytics cookies. The “Yes” choice currently permits optional YouTube media only. If VL Rental later introduces a new optional purpose, this Policy and the consent interface will be updated before that purpose is enabled where consent is required.",
        ],
        bullets: &[],
    },
    PrivacySection {
        id: "privacy-choices",
        title: "10. Your cookie and communication choices",
        paragraphs: &[
            "On a first visit, the bottom banner offers “Yes, allow” and “No, necessary only.” The buttons are equally available and no optional media is loaded before “Yes.” You can reopen the banner at any time with Cookie choices in the footer. Selecting “No” or later withdrawing consent removes VL Rental's permission to load new optional YouTube media; clearing third-party cookies already stored is controlled through your browser settings.",
            "Newsletter sign-up is voluntary and separate from accepting these Terms or booking an RV. Commercial electronic messages identify the sender and provide a working unsubscribe method. Operational booking, payment, security, safety, and requested-response messages are not disabled by a marketing unsubscribe. Browser push notifications require both a user action in the account panel and browser permission and can be disabled there or in browser settings.",
        ],
        bullets: &[],
    },
    PrivacySection {
        id: "privacy-retention",
        title: "11. Retention and deletion",
        paragraphs: &[
            "VL Rental retains personal information only as long as reasonably necessary for the stated purposes or a legal requirement. Retention depends on the record: one-time authentication codes are short-lived; browser session secrets last only for the applicable session or workflow; inquiries and support records are kept while needed to respond and manage follow-up; damage evidence is kept for the claim, dispute, limitation, and legally required record period; and financial, booking, tax, refund, and audit records are generally kept for at least six years from the end of the last tax year to which they relate.",
            "Backups and security logs are deleted or overwritten on controlled schedules. When information is no longer required, VL Rental deletes, securely destroys, or anonymizes it, subject to technical backup cycles and lawful preservation holds.",
        ],
        bullets: &[],
    },
    PrivacySection {
        id: "privacy-security",
        title: "12. Safeguards",
        paragraphs: &[
            "VL Rental uses safeguards appropriate to the sensitivity of the information, including encrypted transport, password hashing, short-lived one-time codes stored only as hashes, rotating sessions, session-only storage for private browser tokens, role-based administration, private damage-evidence storage, short-lived signed access, restricted database privileges, payment processing through Stripe, request limits, audit records, and minimized HTTP tracing that excludes query strings and headers.",
            "No internet service can guarantee absolute security. If you believe an account, booking link, or personal information has been compromised, contact the Privacy Officer promptly and do not send passwords, full card numbers, or private access tokens by email.",
        ],
        bullets: &[],
    },
    PrivacySection {
        id: "privacy-rights",
        title: "13. Access, correction, and complaints",
        paragraphs: &[
            "Subject to lawful exceptions, you may make a written request for access to personal information under VL Rental's control, information about how it has been used or disclosed, or correction of an error or omission. VL Rental may need to verify identity and clarify the scope before responding. A response, fee if lawfully permitted, or extension will be handled within the time required by applicable law.",
            "Start with the Privacy Officer so VL Rental can investigate and respond. If a concern is not resolved, you may contact the Office of the Information and Privacy Commissioner for British Columbia or, where applicable, the Office of the Privacy Commissioner of Canada. Exercising a privacy right will not result in retaliation or denial of service, except where the requested information is necessary to provide that service or must be retained by law.",
        ],
        bullets: &[],
    },
    PrivacySection {
        id: "privacy-children",
        title: "14. Children",
        paragraphs: &[
            "Bookings and customer accounts are intended for adults who can enter a binding contract. VL Rental does not knowingly invite a child to create an account or provide personal information independently. A responsible adult may provide the guest count and information reasonably necessary for a family booking but should not include unnecessary details about children in free-text fields.",
        ],
        bullets: &[],
    },
    PrivacySection {
        id: "privacy-updates",
        title: "15. Policy updates",
        paragraphs: &[
            "VL Rental may update this Policy when practices, providers, or legal requirements change. The effective date will be updated, and a material change will be highlighted on the site or communicated through an appropriate channel. New optional collection, use, or disclosure will not be treated as consented to merely because an older version of this Policy was accepted.",
        ],
        bullets: &[],
    },
];

#[component]
pub fn Privacy() -> Element {
    rsx! {
        section { class: "tm-header",
            div { class: "tm-kicker", "VL RENTAL · PRIVACY" }
            h1 { class: "tm-title", "Privacy & Cookie Policy" }
            p { class: "tm-sub",
                "How VL Rental handles personal information, cookies, browser storage, service providers, and your choices for RV bookings in Canada."
            }
            div { class: "tm-updated", "Effective August 4, 2026" }
        }

        section { class: "tm-body",
            nav { class: "tm-toc", aria_label: "Privacy policy sections",
                div { class: "tm-toc-h", "ON THIS PAGE" }
                for section in PRIVACY_SECTIONS.iter() {
                    a { key: "toc-{section.id}", class: "tm-toc-link", href: "#{section.id}", "{section.title}" }
                }
            }
            article { class: "tm-content",
                div { class: "tm-notice",
                    strong { "Plain-language summary:" }
                    span { " VL Rental does not sell personal information and currently uses no analytics or advertising cookies. Optional YouTube media loads only after you choose Yes." }
                }
                for section in PRIVACY_SECTIONS.iter() {
                    section { key: "{section.id}", id: "{section.id}", class: "tm-sec",
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
                div { class: "tm-contact", id: "privacy-contact",
                    div {
                        h2 { "Privacy Officer" }
                        p { "Privacy Officer, VL Rental · Kelowna, British Columbia, Canada" }
                        p { "Email Vlrental.ca@gmail.com · Phone +1 (250) 878-5874" }
                    }
                    div { class: "tm-contact-actions",
                        a { class: "tm-secondary-btn", href: "mailto:Vlrental.ca@gmail.com?subject=Privacy%20request", "Email Privacy Officer" }
                        Link { class: "tm-help-btn", to: crate::Route::Contact {}, "Contact form" }
                    }
                }
                p { class: "tm-legal-links",
                    a { href: "https://www.oipc.bc.ca/", target: "_blank", rel: "noopener noreferrer", "OIPC British Columbia" }
                    span { " · " }
                    a { href: "https://www.priv.gc.ca/", target: "_blank", rel: "noopener noreferrer", "Office of the Privacy Commissioner of Canada" }
                }
            }
        }
    }
}

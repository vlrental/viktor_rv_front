use dioxus::prelude::*;

#[derive(Clone, Copy)]
struct TermSection {
    id: &'static str,
    title: &'static str,
    paragraphs: &'static [&'static str],
    bullets: &'static [&'static str],
}

const TERM_SECTIONS: &[TermSection] = &[
    TermSection {
        id: "agreement",
        title: "1. Agreement and operator",
        paragraphs: &[
            "These RV Rental Terms and Conditions (the “Terms”), the booking summary you review and accept, and any written addendum accepted by both parties form the rental agreement (the “Agreement”) between you and VL Rental. VL Rental is a Kelowna-based recreational vehicle rental business. It rents its fleet directly and is not a peer-to-peer marketplace, travel agent, insurer, or representative of a third-party host.",
            "By creating a booking, making a payment, or accepting delivery of an RV, you confirm that you have read and accepted the Agreement and have authority to bind every guest in your party. If a booking-specific term conflicts with these general Terms, the booking-specific term controls. Nothing in the Agreement limits a right or remedy that cannot lawfully be waived under applicable consumer-protection law.",
        ],
        bullets: &[],
    },
    TermSection {
        id: "eligibility",
        title: "2. Eligibility, accounts, and customer information",
        paragraphs: &[
            "The person making the booking must be at least 18 years old, have legal capacity to enter a contract, provide accurate and current contact information, and remain responsible for the booking and all guests. Accounts and booking access links may not be sold, transferred, or shared with an unauthorized person.",
            "VL Rental may reasonably verify identity, contact details, payment status, campsite information, and the authority to use the delivery location. A booking may be refused or cancelled for material misrepresentation, fraud risk, prohibited use, non-payment, an unsafe delivery site, or a material breach of the Agreement, subject to applicable law and any refund required by it.",
        ],
        bullets: &[],
    },
    TermSection {
        id: "booking",
        title: "3. Booking formation and confirmation",
        paragraphs: &[
            "Website availability and a quote are invitations to book, not guarantees. A pending Stripe Checkout may temporarily reserve dates until its displayed expiry. A booking is confirmed only when VL Rental's system records the required provider-confirmed payment, or when the no-card test flow expressly returns a confirmed test booking. A browser success screen, email, or bank authorization by itself does not override the booking status recorded by VL Rental.",
            "Before accepting an online booking, you can review and correct dates, guests, RV, delivery address, extras, itemized price, payment timing, and contact details. The confirmation email and account record provide an electronic copy of the accepted booking details. Requested changes remain subject to availability, written approval, and a new server quote.",
        ],
        bullets: &[],
    },
    TermSection {
        id: "dates",
        title: "4. Rental dates, times, and minimum pricing",
        paragraphs: &[
            "All delivery, return, payment-due, and availability rules are calculated by the backend in America/Vancouver time. Standard delivery and setup is at 2:00 PM, and standard return access is at 11:00 AM, on the dates in the booking summary. The RV must be vacant and ready for pickup at the return time.",
            "Customers may select one or more nights. For a one- or two-night stay, the selected delivery and return dates remain unchanged, but the booking is priced at the three-night minimum and the RV remains unavailable through the full protected three-night window. The customer may not use or occupy the RV after the selected return time even when the protected availability window continues longer.",
        ],
        bullets: &[],
    },
    TermSection {
        id: "price",
        title: "5. Trip price, mandatory charges, and taxes",
        paragraphs: &[
            "The immutable backend quote is the authoritative price. Amounts are in Canadian dollars unless the booking summary says otherwise. The itemized trip price includes the applicable nightly charge, delivery and setup, selected extras, applicable GST and PST, and these mandatory charges:",
        ],
        bullets: &[
            "RV Preparation Fee: CA$97 once per booking.",
            "Stationary Plus Protection: CA$150 for the first three booked nights, plus CA$30 for each additional booked night.",
        ],
    },
    TermSection {
        id: "delivery-fee",
        title: "6. Delivery area and delivery fee",
        paragraphs: &[
            "VL Rental is delivery-only; customer pickup and customer towing are not offered. Delivery is limited to approved locations no more than 150 km one way from the Kelowna base. The fee is CA$150 for a destination through 40 km one way, then CA$2.50 per kilometre in each direction (CA$5 total for each additional one-way kilometre). The accepted server estimate and resolved delivery address control.",
            "The customer is responsible for reserving and paying for a lawful campsite, obtaining permissions, providing an accurate address and access instructions, and ensuring the site is accessible to the delivery vehicle and RV. Extra work or a second delivery attempt caused by inaccurate information, blocked access, site restrictions, unsafe ground, or the customer's absence may be charged at the reasonable disclosed cost. VL Rental may decline an unsafe or unlawful setup location.",
        ],
        bullets: &[],
    },
    TermSection {
        id: "payments",
        title: "7. Trip-price payment schedule",
        paragraphs: &[
            "If delivery is more than 30 days away when the booking is made, 30% of the trip price is due immediately and the remaining 70% is due exactly 30 days before delivery. If delivery is 30 days away or less, 100% of the trip price is due immediately. The refundable damage deposit is separate and is never included when calculating the 30% payment.",
            "VL Rental sends a secure payment link when a scheduled amount becomes due. Failure to pay by the stated deadline may prevent delivery and may result in cancellation under the applicable cancellation terms. Payment processing is provided by Stripe. You must not dispute a valid charge fraudulently; this does not restrict any lawful chargeback or consumer remedy.",
        ],
        bullets: &[],
    },
    TermSection {
        id: "deposit",
        title: "8. Refundable CA$1,000 damage deposit",
        paragraphs: &[
            "Every booking requires a separate refundable CA$1,000 damage deposit. It is not charged through Stripe. The customer must send it by Interac e-Transfer to protrailercare@gmail.com no later than 48 hours before delivery and include the booking number in the transfer message. Delivery cannot proceed until VL Rental verifies receipt and records the deposit as paid.",
            "After return and inspection, VL Rental will return the deposit by e-Transfer when there is no valid claim. VL Rental may retain only an amount supported by documented damage, missing items, extraordinary cleaning, unauthorized use, unpaid charges, or another customer obligation under the Agreement. Any retention decision requires a stated reason and private photographic evidence. The unused balance is returned as soon as practical and the decision is made no later than seven days after return.",
            "The deposit is security and is not a cap on responsibility for loss exceeding CA$1,000. The customer is responsible for keeping an accurate email address and any information reasonably needed to receive the returned e-Transfer.",
        ],
        bullets: &[],
    },
    TermSection {
        id: "cancellations",
        title: "9. Customer cancellations and refunds",
        paragraphs: &[
            "Send a cancellation request to vlrental.ca@gmail.com and include the booking number. Unless a booking-specific policy or a mandatory legal right is more favourable, the following schedule applies to the trip price:",
        ],
        bullets: &[
            "No cancellation charge when the request is sent within five calendar days after booking and delivery was more than 30 days away when the booking was made.",
            "CA$100 when the request is received 30 or more days before delivery and the free-cancellation rule above does not apply.",
            "CA$500 when the request is received 15 to 29 days before delivery.",
            "100% of the trip price when the request is received 14 days or less before delivery, or for a no-show.",
        ],
    },
    TermSection {
        id: "statutory-rights",
        title: "10. British Columbia consumer rights",
        paragraphs: &[
            "The cancellation schedule does not replace statutory rights. The Business Practices and Consumer Protection Act may give a British Columbia consumer additional cancellation and refund remedies if required information is not disclosed, a required copy of a distance contract is not provided, or the service is not supplied as required. When a statutory right applies, VL Rental will follow the legally required notice, refund, and timing rules.",
            "A customer may give a legally permitted cancellation notice by email or another method allowed by law that enables the customer to keep proof. The customer should keep the booking confirmation and sent notice.",
        ],
        bullets: &[],
    },
    TermSection {
        id: "vl-cancellation",
        title: "11. VL Rental cancellation, substitution, and delay",
        paragraphs: &[
            "Safety, wildfire or evacuation conditions, road closures, severe weather, mechanical failure, campsite inaccessibility, government orders, or another event outside reasonable control may delay or prevent delivery. VL Rental will communicate promptly and may offer a reasonably comparable RV, different safe delivery arrangement, credit, rebooking, or refund appropriate to the affected service and applicable law.",
            "If VL Rental cannot supply the booked RV for a reason within its control and no acceptable substitute is agreed, VL Rental will refund amounts paid for the unavailable rental service, including the refundable deposit. VL Rental does not reimburse separate campground, travel, food, hotel, or vacation costs except where required by law.",
        ],
        bullets: &[],
    },
    TermSection {
        id: "use",
        title: "12. Delivery-only and stationary use",
        paragraphs: &[
            "Only VL Rental or its authorized contractor may tow, move, level, connect, disconnect, or retrieve the RV. The customer must not hitch, tow, relocate, or permit another person to move it after setup. Unauthorized movement may make protection unavailable and makes the customer responsible for resulting loss, damage, towing, recovery, and third-party claims.",
            "Follow the orientation, posted instructions, campground rules, occupancy limit, and all applicable laws. The RV may not be sublet, used for an unlawful purpose, used for an undisclosed commercial event, intentionally overloaded, altered, or occupied by more guests than stated in the booking. Children and guests requiring supervision must be supervised by a responsible adult.",
        ],
        bullets: &[],
    },
    TermSection {
        id: "care",
        title: "13. Care, cleaning, smoking, pets, and prohibited conditions",
        paragraphs: &[
            "Keep the RV reasonably clean and secure. Before return, wash and put away dishes and utensils, wipe sinks, counters, tables, appliances, refrigerator, stove, microwave, and shower surfaces, sweep floors, remove personal property and garbage, and leave the exterior free from avoidable damage. A CA$100 cleaning charge applies when ordinary required cleaning is not completed; extreme soiling, biohazards, odours, stains, or remediation may be charged at the reasonable documented cost.",
            "Smoking, vaping, and burning cannabis or tobacco inside the RV are prohibited. A CA$300 deodorizing charge applies when smoke odour is detected, plus documented remediation above that amount when reasonably necessary. Pets are allowed only when the listing and booking expressly allow them; disclosed pet fees and cleaning requirements apply. Never leave a pet unattended when doing so risks damage or distress.",
        ],
        bullets: &[],
    },
    TermSection {
        id: "equipment",
        title: "14. Equipment, utilities, and awnings",
        paragraphs: &[
            "The customer must review the supplied inventory and promptly report a missing, damaged, or malfunctioning item. On a site without services, fresh water and battery power are limited. Air conditioning, microwave ovens, coffee makers, toasters, and similar 120-volt equipment require an adequate approved electrical connection or generator. The customer is responsible for campground utility compatibility and ordinary conservation.",
            "Awnings are vulnerable to weather. Retract the awning when it is unattended and whenever wind, rain, pooling water, or other conditions could cause damage. The customer is responsible for awning damage caused by misuse or failure to follow instructions, including any booking-specific deductible or uncovered amount disclosed before booking.",
        ],
        bullets: &[],
    },
    TermSection {
        id: "incidents",
        title: "15. Damage, incidents, and repairs",
        paragraphs: &[
            "Immediately stop using unsafe equipment and contact VL Rental after damage, water intrusion, fire, theft, injury, utility failure, or another material incident. In an emergency, contact emergency services first. Take reasonable steps to prevent further loss, preserve relevant evidence, and cooperate with an insurer, protection provider, campground, payment provider, or lawful investigation.",
            "Do not arrange a non-emergency repair, replacement, towing, or alteration without VL Rental's prior approval. Authorized emergency expenses require receipts and are reimbursed only to the extent agreed or required by law. The customer is responsible for damage caused by the customer, guests, pets, invitees, unauthorized movement, misuse, negligence, or breach of the Agreement, excluding ordinary wear and tear.",
        ],
        bullets: &[],
    },
    TermSection {
        id: "risk-liability",
        title: "16. Personal property, outdoor risks, and liability",
        paragraphs: &[
            "Camping and outdoor activities involve risks, including weather, wildlife, fire, terrain, campground conditions, utility interruption, and acts of third parties. Customers remain responsible for choosing a suitable destination, obeying emergency orders, supervising guests, and securing personal property. VL Rental is not responsible for lost, stolen, or damaged personal property unless the loss results from VL Rental's breach of a duty that cannot lawfully be excluded.",
            "To the maximum extent permitted by law, VL Rental is not liable for indirect, incidental, special, or consequential loss, including lost enjoyment, lost vacation time, or third-party travel costs. This limitation does not exclude liability for gross negligence, wilful misconduct, personal injury caused by a legally non-excludable breach, or any statutory warranty, condition, right, or remedy that cannot be waived.",
        ],
        bullets: &[],
    },
    TermSection {
        id: "website",
        title: "17. Website, reviews, and acceptable use",
        paragraphs: &[
            "VL Rental owns or licenses the website content, branding, photographs, and software. You may use the site only for lawful personal booking and account purposes. Do not interfere with security, scrape protected data, upload malicious content, impersonate another person, or attempt unauthorized access.",
            "A review must reflect a genuine eligible booking and must not contain unlawful, threatening, private, infringing, or knowingly false content. VL Rental may moderate content for those reasons but will not prohibit an honest consumer review or exercise of a legal right.",
        ],
        bullets: &[],
    },
    TermSection {
        id: "privacy-communications",
        title: "18. Privacy and electronic communications",
        paragraphs: &[
            "The Privacy & Cookie Policy explains how VL Rental handles personal information and browser storage. Operational messages about authentication, quotes, bookings, payments, delivery, safety, refunds, and account security are part of providing the requested service. Marketing messages are sent only with a lawful basis and include a way to unsubscribe. Withdrawing marketing or optional-cookie consent does not stop necessary booking communications.",
        ],
        bullets: &[],
    },
    TermSection {
        id: "law-disputes",
        title: "19. Governing law and disputes",
        paragraphs: &[
            "The Agreement is governed by the laws of British Columbia and the federal laws of Canada applicable there, without limiting mandatory consumer law that applies based on the customer's residence. Please contact VL Rental first so the parties can try to resolve a concern promptly. If it is not resolved, the courts of British Columbia have jurisdiction, and the parties submit to a court located in or serving Kelowna, unless applicable law requires another forum.",
            "The Agreement does not require private arbitration, waive a right to bring a claim in court, or prohibit participation in a remedy that applicable law makes available.",
        ],
        bullets: &[],
    },
    TermSection {
        id: "general",
        title: "20. Changes, severability, and entire agreement",
        paragraphs: &[
            "VL Rental may update these website Terms prospectively by posting a new effective date. A material change will not alter an already accepted booking unless required by law or agreed by both parties. If a provision is unenforceable, it will be limited or severed only as necessary, and the remaining provisions continue. A delay in enforcing a provision is not a waiver.",
            "The accepted booking summary, these Terms, and written addenda are the entire Agreement about the booking and replace earlier inconsistent discussions. Headings are for convenience. Electronic acceptance, records, and notices may be used to the extent permitted by law.",
        ],
        bullets: &[],
    },
];

#[component]
pub(crate) fn TermsAgreementContent() -> Element {
    rsx! {
        div { class: "tm-notice",
            strong { "Booking-specific details matter." }
            span { " Your accepted dates, RV, delivery address, itemized price, payment schedule, and selected extras are part of the Agreement." }
        }
        for section in TERM_SECTIONS.iter() {
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
    rsx! {
        section { class: "tm-header",
            div { class: "tm-kicker", "VL RENTAL · KELOWNA, BC" }
            h1 { class: "tm-title", "RV Rental Terms & Conditions" }
            p { class: "tm-sub",
                "These Terms govern delivery-only RV bookings with VL Rental in Kelowna and approved Okanagan destinations. Review them together with your itemized booking summary before accepting."
            }
            div { class: "tm-updated", "Effective August 4, 2026" }
        }

        section { class: "tm-body",
            nav { class: "tm-toc", aria_label: "Terms sections",
                div { class: "tm-toc-h", "ON THIS PAGE" }
                for section in TERM_SECTIONS.iter() {
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

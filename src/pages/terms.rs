use dioxus::prelude::*;

const CSS: Asset = asset!("/assets/css/terms.css");

/// Раздел условий аренды: якорь + заголовок + текст.
struct TermSection {
    id: &'static str,
    title: &'static str,
    body: &'static str,
}

fn term_sections() -> Vec<TermSection> {
    vec![
        TermSection {
            id: "tm-s1",
            title: "1. Pricing, Booking Payments & Damage Deposit",
            body: "The trip price is shown separately from the refundable damage deposit. It includes the nightly rental, the mandatory one-time CA$97 RV Preparation Fee, mandatory Stationary Plus Protection at CA$50 per booked night, delivery and setup, selected extras, and applicable GST and PST. For trips booked more than 30 days ahead, 30% of the trip price is due to confirm and the remaining balance is due 30 days before delivery. For trips booked within 30 days, the full trip price is due when booking. A separate refundable CA$1,000 damage deposit is due 48 hours before delivery and is not included in the 30% calculation. Customers receive an email with a direct payment link whenever a payment becomes due. For the Gold option, the refundable damage deposit is held for seven days after the RV is returned and then refunded without interest, less any valid damage charges.",
        },
        TermSection {
            id: "tm-s2",
            title: "2. Cancellations (RVs)",
            body: "Cancel within 5 days of booking at no charge if departure is more than 30 days away. After that: $100 fee if cancelled 30+ days before departure, $500 if 16–30 days before, and the full amount within 14 days of departure or for no-shows.",
        },
        TermSection {
            id: "tm-s3",
            title: "3. Equipment & Cleaning",
            body: "Units come equipped with dishes and a coffeemaker. Return the RV thoroughly cleaned — dishes washed, surfaces disinfected, floors swept, appliances wiped. A $100 cleaning fee applies to unclean returns (more for extreme dirt). Smoking incurs a $300 deodorizing fee.",
        },
        TermSection {
            id: "tm-s4",
            title: "4. Awnings",
            body: "Renters are fully responsible for awning damage. Awnings must be retracted in wind or rain — manufacturers do not warranty weather damage. A $300 insurance deductible applies per incident.",
        },
        TermSection {
            id: "tm-s5",
            title: "5. Campsites Without Services",
            body: "On unserviced sites, conserve fresh water and use lights sparingly — battery power is limited. Air conditioning, microwaves and similar appliances require external 120-volt power or a generator.",
        },
        TermSection {
            id: "tm-s6",
            title: "6. Off-Road & Fuel",
            body: "Trailers must stay on maintained roads — a $200 fee applies if a unit is taken off-road. Rentals are quoted plus GST (5%) and PST (7%).",
        },
    ]
}

#[component]
pub fn Terms() -> Element {
    let sections = term_sections();
    let toc = term_sections();
    rsx! {
        document::Link { rel: "stylesheet", href: CSS }

        // Заголовок страницы: тайтл, сабтайтл, дата обновления
        section { class: "tm-header",
            h1 { class: "tm-title", "Terms & Conditions" }
            p { class: "tm-sub",
                "Please review the rental terms below before booking. These terms apply to RV rentals from VL Rental."
            }
            div { class: "tm-updated",
                "VL Rental RV Terms & Conditions · Last updated July 2026"
            }
        }

        // Тело: оглавление слева + разделы справа
        section { class: "tm-body",
            nav { class: "tm-toc",
                div { class: "tm-toc-h", "ON THIS PAGE" }
                for (i , s) in toc.into_iter().enumerate() {
                    a {
                        key: "{s.id}",
                        class: if i == 0 { "tm-toc-link tm-toc-active" } else { "tm-toc-link" },
                        href: "#{s.id}",
                        "{s.title}"
                    }
                }
            }
            div { class: "tm-content",
                for s in sections {
                    div { key: "{s.id}", id: "{s.id}", class: "tm-sec",
                        h2 { class: "tm-sec-h", "{s.title}" }
                        p { class: "tm-sec-p", "{s.body}" }
                    }
                }
                div { class: "tm-help",
                    p { class: "tm-help-t",
                        "Questions about these terms? Our team is happy to walk you through anything before you book."
                    }
                    Link { class: "tm-help-btn", to: crate::Route::Contact {}, "Contact us" }
                }
            }
        }
    }
}

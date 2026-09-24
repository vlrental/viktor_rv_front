use dioxus::prelude::*;

use super::Icon;

#[derive(Clone, Copy, PartialEq)]
pub enum PriceInfoKind {
    Preparation,
    StationaryPlus,
}

impl PriceInfoKind {
    fn title(self) -> &'static str {
        match self {
            Self::Preparation => "RV Preparation Fee",
            Self::StationaryPlus => "Stationary Plus Protection",
        }
    }
}

fn close_and_restore(mut open: Signal<Option<PriceInfoKind>>, trigger_id: String) {
    open.set(None);
    spawn(async move {
        document::eval(&format!(
            "requestAnimationFrame(() => document.getElementById('{trigger_id}')?.focus());"
        ));
    });
}

#[component]
pub fn PriceInfoPopover(
    id: String,
    kind: PriceInfoKind,
    mut open: Signal<Option<PriceInfoKind>>,
) -> Element {
    let expanded = *open.read() == Some(kind);
    let title = kind.title();
    let trigger_id = format!("{id}-trigger");
    let trigger_for_button = trigger_id.clone();
    let trigger_for_layer = trigger_id.clone();
    let trigger_for_dialog = trigger_id.clone();
    let trigger_for_close = trigger_id.clone();
    let panel_id = id.clone();
    rsx! {
        span { class: "price-info-anchor",
            button {
                id: trigger_for_button.clone(),
                class: "price-info-trigger",
                r#type: "button",
                aria_label: "About {title}",
                aria_expanded: expanded,
                aria_controls: "{id}",
                onclick: move |event| {
                    event.stop_propagation();
                    open.set(if expanded { None } else { Some(kind) });
                },
                onkeydown: move |event| {
                    if event.key() == Key::Escape && expanded {
                        event.stop_propagation();
                        close_and_restore(open, trigger_for_button.clone());
                    }
                },
                span { class: "price-info-question", aria_hidden: "true", "?" }
            }
            if expanded {
                span {
                    class: "price-info-dismiss-layer",
                    aria_hidden: "true",
                    onclick: move |event| {
                        event.stop_propagation();
                        close_and_restore(open, trigger_for_layer.clone());
                    },
                }
                section {
                    id: panel_id,
                    class: "price-info-popover",
                    role: "dialog",
                    aria_label: "About {title}",
                    tabindex: "-1",
                    autofocus: true,
                    onclick: move |event| event.stop_propagation(),
                    onkeydown: move |event| {
                        if event.key() == Key::Escape {
                            event.stop_propagation();
                            close_and_restore(open, trigger_for_dialog.clone());
                        }
                    },
                    header {
                        strong { "{title}" }
                        button {
                            class: "price-info-close",
                            r#type: "button",
                            aria_label: "Close {title} information",
                            onclick: move |_| close_and_restore(open, trigger_for_close.clone()),
                            Icon { name: "x", size: 16, color: "currentColor" }
                        }
                    }
                    match kind {
                        PriceInfoKind::Preparation => rsx! {
                            p { "A mandatory CA$97 one-time fee for preparing the RV for your booking." }
                        },
                        PriceInfoKind::StationaryPlus => rsx! {
                            ul {
                                li { strong { "Delivered & parked: " } "For stationary rentals delivered to a campground or property; the guest does not drive or tow the RV." }
                                li { strong { "Reduced rate: " } "Costs less than driving protection plans because public-road transit and collision exposure are excluded." }
                                li { strong { "Site protection: " } "Covers eligible physical damage, comprehensive incidents, and site liability while the RV is parked and occupied or left stationary." }
                            }
                            p { "Coverage is subject to the rental agreement and applicable policy terms." }
                        },
                    }
                }
            }
        }
    }
}

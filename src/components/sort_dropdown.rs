use dioxus::prelude::*;

use super::Icon;

#[component]
pub fn SortDropdown(
    value: String,
    #[props(default)] show_date_fit: bool,
    on_change: EventHandler<String>,
) -> Element {
    let mut open = use_signal(|| false);
    let options = if show_date_fit {
        vec![
            ("recommended", "Recommended"),
            ("date-fit", "Best fit for your dates"),
            ("price-low", "Price: low to high"),
            ("price-high", "Price: high to low"),
            ("capacity", "Most sleeping space"),
        ]
    } else {
        vec![
            ("recommended", "Recommended"),
            ("price-low", "Price: low to high"),
            ("price-high", "Price: high to low"),
            ("capacity", "Most sleeping space"),
        ]
    };
    let selected_label = options
        .iter()
        .find_map(|(key, label)| (*key == value).then_some(*label))
        .unwrap_or("Recommended");

    rsx! {
        div {
            class: "sort-dropdown",
            onkeydown: move |event| if event.key() == Key::Escape && *open.read() {
                event.stop_propagation();
                open.set(false);
            },
            if *open.read() {
                button {
                    class: "sort-dropdown-dismiss",
                    r#type: "button",
                    tabindex: "-1",
                    aria_hidden: "true",
                    onclick: move |_| open.set(false),
                }
            }
            button {
                class: if *open.read() { "cat-sort is-open" } else { "cat-sort" },
                r#type: "button",
                aria_label: "Sort RVs",
                aria_haspopup: "listbox",
                aria_expanded: *open.read(),
                aria_controls: "rv-sort-options",
                onclick: move |_| {
                    let next = !*open.read();
                    open.set(next);
                },
                Icon { name: "arrow-up-down", size: 14, color: "var(--vl-ink)" }
                span { class: "cat-sort-prefix", "Sort" }
                strong { class: "cat-sort-value", "{selected_label}" }
                span { class: "cat-sort-chevron",
                    Icon { name: "chevron-down", size: 14, color: "var(--vl-ink)" }
                }
            }
            if *open.read() {
                div {
                    id: "rv-sort-options",
                    class: "sort-dropdown-menu",
                    role: "listbox",
                    aria_label: "Sort RVs",
                    for (key, label) in options {
                        {
                            let selected = key == value;
                            let option_value = key.to_string();
                            rsx! {
                                button {
                                    key: "{key}",
                                    class: if selected { "sort-dropdown-option is-selected" } else { "sort-dropdown-option" },
                                    r#type: "button",
                                    role: "option",
                                    aria_selected: selected,
                                    onclick: move |_| {
                                        on_change.call(option_value.clone());
                                        open.set(false);
                                    },
                                    span { "{label}" }
                                    span { class: "sort-dropdown-check",
                                        if selected {
                                            Icon { name: "check", size: 16, color: "var(--vl-white)" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

use std::sync::atomic::{AtomicUsize, Ordering};

use dioxus::prelude::*;

use super::Icon;

static NEXT_SORT_DROPDOWN_ID: AtomicUsize = AtomicUsize::new(1);

const SORT_OPTIONS: [(&str, &str); 4] = [
    ("recommended", "Recommended"),
    ("price-low", "Price: low to high"),
    ("price-high", "Price: high to low"),
    ("capacity", "Most sleeping space"),
];

fn sort_options(show_date_fit: bool) -> Vec<(&'static str, &'static str)> {
    let mut options = SORT_OPTIONS.to_vec();
    if show_date_fit {
        options.insert(1, ("date-fit", "Best fit for your dates"));
    }
    options
}

fn selected_option_index(options: &[(&str, &str)], value: &str) -> usize {
    options
        .iter()
        .position(|(key, _)| *key == value)
        .unwrap_or(0)
}

fn activates_sort_option(key: &Key) -> bool {
    key == &Key::Enter || matches!(key, Key::Character(value) if value == " ")
}

fn focus_sort_element(element_id: &str) {
    document::eval(&format!(
        r#"requestAnimationFrame(() => document.getElementById("{element_id}")?.focus({{ preventScroll: true }}));"#
    ));
}

fn focus_sort_trigger(instance_id: usize) {
    focus_sort_element(&format!("rv-sort-trigger-{instance_id}"));
}

#[component]
pub fn SortDropdown(
    value: String,
    #[props(default)] show_date_fit: bool,
    on_change: EventHandler<String>,
) -> Element {
    let mut open = use_signal(|| false);
    let instance_id = use_hook(|| NEXT_SORT_DROPDOWN_ID.fetch_add(1, Ordering::Relaxed));
    let options = sort_options(show_date_fit);
    let selected_index = selected_option_index(&options, &value);
    let mut active_index = use_signal(|| selected_index);
    let selected_label = options[selected_index].1;
    let trigger_id = format!("rv-sort-trigger-{instance_id}");
    let menu_id = format!("rv-sort-options-{instance_id}");

    rsx! {
        div {
            class: "sort-dropdown",
            onkeydown: move |event| match event.key() {
                Key::Escape if *open.read() => {
                    event.prevent_default();
                    event.stop_propagation();
                    open.set(false);
                    focus_sort_trigger(instance_id);
                }
                Key::Tab if *open.read() => open.set(false),
                key @ (Key::ArrowDown | Key::ArrowUp | Key::Home | Key::End) => {
                    event.prevent_default();
                    let was_open = *open.read();
                    let current = *active_index.read();
                    let last = options.len() - 1;
                    let next = match key {
                        Key::ArrowDown if was_open => (current + 1) % options.len(),
                        Key::ArrowUp if was_open => current.checked_sub(1).unwrap_or(last),
                        Key::Home => 0,
                        Key::End => last,
                        _ => selected_index,
                    };
                    active_index.set(next);
                    if !was_open {
                        open.set(true);
                    }
                    focus_sort_element(&format!("rv-sort-option-{instance_id}-{next}"));
                }
                _ => {}
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
                id: "{trigger_id}",
                class: if *open.read() { "cat-sort is-open" } else { "cat-sort" },
                r#type: "button",
                aria_label: "Sort RVs, current: {selected_label}",
                aria_haspopup: "listbox",
                aria_expanded: *open.read(),
                aria_controls: "{menu_id}",
                onclick: move |_| {
                    let next = !*open.read();
                    open.set(next);
                    if next {
                        active_index.set(selected_index);
                        focus_sort_element(&format!("rv-sort-option-{instance_id}-{selected_index}"));
                    }
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
                    id: "{menu_id}",
                    class: "sort-dropdown-menu",
                    role: "listbox",
                    aria_label: "Sort RVs",
                    for (index, (key, label)) in options.iter().copied().enumerate() {
                        {
                            let selected = key == value;
                            let option_value = key.to_string();
                            let keyboard_value = option_value.clone();
                            rsx! {
                                button {
                                    id: "rv-sort-option-{instance_id}-{index}",
                                    key: "{key}",
                                    class: if selected { "sort-dropdown-option is-selected" } else { "sort-dropdown-option" },
                                    r#type: "button",
                                    role: "option",
                                    aria_selected: selected,
                                    tabindex: if index == *active_index.read() { "0" } else { "-1" },
                                    onfocus: move |_| active_index.set(index),
                                    onkeydown: move |event| {
                                        let key = event.key();
                                        if activates_sort_option(&key) {
                                            event.prevent_default();
                                            event.stop_propagation();
                                            on_change.call(keyboard_value.clone());
                                            open.set(false);
                                            focus_sort_trigger(instance_id);
                                        }
                                    },
                                    onclick: move |_| {
                                        on_change.call(option_value.clone());
                                        open.set(false);
                                        focus_sort_trigger(instance_id);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn date_fit_is_only_added_when_dates_exist() {
        assert_eq!(sort_options(false).len(), 4);
        assert_eq!(sort_options(true)[1].0, "date-fit");
    }

    #[test]
    fn unknown_sort_value_falls_back_to_recommended() {
        let options = sort_options(false);

        assert_eq!(selected_option_index(&options, "unknown"), 0);
        assert_eq!(options[0].0, "recommended");
    }

    #[test]
    fn enter_and_space_activate_a_keyboard_option() {
        assert!(activates_sort_option(&Key::Enter));
        assert!(activates_sort_option(&Key::Character(" ".into())));
        assert!(!activates_sort_option(&Key::ArrowDown));
    }
}

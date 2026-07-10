use dioxus::prelude::*;

/// Иконка lucide через icon-font (класс `icon-{name}`).
#[component]
pub fn Icon(name: &'static str, size: u32, color: &'static str) -> Element {
    rsx! {
        i {
            class: "icon-{name}",
            style: "font-size: {size}px; color: {color};",
        }
    }
}

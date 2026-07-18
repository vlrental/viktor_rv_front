use dioxus::prelude::*;

/// Иконка Lucide через icon-font (класс `icon-{name}`).
///
/// Брендовые иконки не входят в Lucide, поэтому социальные сети рисуем
/// встроенными SVG и не зависим от внешних шрифтов или изображений.
#[component]
pub fn Icon(name: &'static str, size: u32, color: &'static str) -> Element {
    match name {
        "facebook" => rsx! {
            svg {
                width: "{size}",
                height: "{size}",
                view_box: "0 0 24 24",
                fill: "currentColor",
                "aria-hidden": "true",
                style: "display: block; color: {color};",
                path { d: "M13.5 21v-8h2.75l.41-3.2H13.5V7.76c0-.93.26-1.56 1.59-1.56h1.7V3.34a22.8 22.8 0 0 0-2.47-.13c-2.45 0-4.13 1.5-4.13 4.25V9.8H7.42V13h2.77v8h3.31Z" }
            }
        },
        "instagram" => rsx! {
            svg {
                width: "{size}",
                height: "{size}",
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "2",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                "aria-hidden": "true",
                style: "display: block; color: {color};",
                rect { x: "3", y: "3", width: "18", height: "18", rx: "5" }
                circle { cx: "12", cy: "12", r: "4" }
                circle { cx: "17.5", cy: "6.5", r: "1", fill: "currentColor", stroke: "none" }
            }
        },
        _ => rsx! {
            i {
                class: "icon-{name}",
                style: "font-size: {size}px; color: {color};",
            }
        },
    }
}

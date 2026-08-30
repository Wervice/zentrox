use dioxus::prelude::*;
use dioxus_free_icons::{icons::bs_icons, Icon};

#[css_module("/src/components/checkbox/style.css")]
struct Style;

#[component]
pub fn Checkbox(value: bool, on_value_change: Callback<bool, ()>, disabled: Option<bool>, id: Option<String>) -> Element {
    rsx! {
        input {
            class: Style::checkbox,
            r#type: "checkbox",
            onchange: move |v: Event<FormData>| {
                if !disabled.unwrap_or_default() {
                    on_value_change.call(v.value() == "true")
                }
            },
            value: "{value}",
            "aria-checked": "{value}",
            "data-checked": "{value}",
            "aria-disabled": "{disabled.unwrap_or(false)}",
            "data-disabled": "{disabled.unwrap_or(false)}",
            id,
        }
    }
}

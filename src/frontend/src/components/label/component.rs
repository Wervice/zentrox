use dioxus::prelude::*;

#[component]
pub fn Label(r#for: String, children: Element) -> Element {
    rsx! {
        label { r#for, class: "select-none opacity-85 font-medium", {children} }
    }
}

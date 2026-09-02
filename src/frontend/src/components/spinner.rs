use dioxus::prelude::*;

#[component]
pub fn Spinner(class: Option<String>, w: Option<usize>, h: Option<usize>) -> Element {
    rsx! {
        svg {
            version: "1.1",
            view_box: "0 0 1 1",
            xmlns: "http://www.w3.org/2000/svg",
            class: class.unwrap_or_default(),
            fill: "currentColor",
            width: format!("{}", w.unwrap_or(20)),
            height: format!("{}", h.unwrap_or(20)),
            path {
                d: "m0.5 0a0.5 0.5 0 0 0-0.46289 0.30859 0.5 0.5 0 0 0 0.10938 0.54492 0.5 0.5 0 0 0 0.54492 0.10938 0.5 0.5 0 0 0 0.30859-0.46289h-0.125a0.375 0.375 0 0 1-0.375 0.375 0.375 0.375 0 0 1-0.375-0.375 0.375 0.375 0 0 1 0.375-0.375z",
                stroke_dashoffset: "11.339",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                stroke_width: "2",
                style: "paint-order:stroke markers fill;transform-box:fill-box;transform-origin:center",
            }
            animateTransform {
                attribute_name: "transform",
                attribute_type: "XML",
                dur: "1000ms",
                from: "0 0 0",
                r#type: "rotate",
                repeat_count: "indefinite",
                to: "360 0 0",
            }
        }
    }
}

use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct PieCartProps {
    pub percentage: f64,
    pub height: u32,
    pub width: u32,
}

#[component]
pub fn PieChart(props: PieCartProps) -> Element {
    let pretty_percentage = (props.percentage * 100.0).round();

    rsx! {
        div { class: "block relative w-min",
            div {
                class: "piechart",
                style: "--percentage: {props.percentage}turn",
                width: "{props.width}px",
                height: "{props.height}px",
            }
            div { class: "absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2",
                "{pretty_percentage}%"
            }
        }
    }
}

use chrono::{self, DateTime, Local};
use dioxus::prelude::*;

#[component]
pub fn Date(duration: DateTime<Local>) -> Element {
    let loc = chrono::Local::now();
    let off = loc.offset();

    let str = duration
        .with_timezone(off)
        .format("%d.%m.%y %H:%M")
        .to_string();
    rsx! {
        span { title: "Time is shown in your timezone: UTC{off}.", {str} }
    }
}

use dioxus::prelude::*;
use dioxus_free_icons::icons::bs_icons;
use dioxus_free_icons::{IconShape, Icon};
use crate::states::ButtonState;
use crate::components::spinner::Spinner;

#[component]
pub fn StatefulIcon<T>(icon: T, button_state: Option<Signal<ButtonState>>) -> Element where T: IconShape + Clone + PartialEq + 'static {
    rsx! {
        span { class: "relative block",
            Icon { icon }
            if let Some(signal) = button_state {
                match *signal.read() {
                    ButtonState::Default => rsx! {},
                    ButtonState::InProgress => rsx! {
                        span { class: "block absolute h-2/3 w-2/3 bottom-[-2px] right-[-2px] rounded-full bg-white/60 dark:bg-black/50",
                            Spinner { class: "block w-full h-full" }
                        }
                    },
                    ButtonState::Failed => rsx! {
                        span { class: "block text-red-500 absolute h-2/3 w-2/3 bottom-[-2px] right-[-2px] rounded-full bg-white dark:bg-black",
                            Icon { icon: bs_icons::BsExclamationCircleFill, class: "block w-full h-full" }
                        }
                    },
                }
            }
        }
    }
}

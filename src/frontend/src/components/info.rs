use crate::components::tooltip::*;
use dioxus::prelude::*;
use dioxus_free_icons::{Icon, icons::bs_icons};
use dioxus_primitives::ContentSide;

#[component]
pub fn Info(children: Element, side: ContentSide) -> Element {
    rsx! {
        Tooltip {
            TooltipTrigger {
                Icon {
                    icon: bs_icons::BsQuestionCircle,
                    class: "w-5 h-5 text-white/50 hover:text-white transition ease-in duration-200 cursor-pointer inline-block mx-1",
                }
            }
            TooltipContent { side, style: "width: 130px",
                div { {children} }
            }
        }
    }
}

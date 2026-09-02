use dioxus::prelude::*;
use dioxus_free_icons::Icon;

use crate::pages::admin_panel;

#[component]
pub fn ViewportTabs(children: Element) -> Element {
    let mut sidebar_toggle = use_context::<admin_panel::SidebarToggle>();

    rsx! {
        div { class: "flex items-center justify-between h-16 min-h-16 max-h-16 dark:bg-black bg-neutral-200 border-b dark:border-neutral-800 border-neutral-300",
            div { class: "flex items-center px-4 gap-2",
                button {
                    onclick: move |_| {
                        sidebar_toggle.0.toggle();
                    },
                    Icon {
                        icon: dioxus_free_icons::icons::bs_icons::BsList,
                        class: "dark:text-neutral-300 text-neutral-950 dark:hover:text-white hover:text-black transition-all duration-200 cursor-pointer",
                    }
                }
                span { class: "text-xl font-bold", {children} }
            }
        }
    }
}

#[component]
pub fn ViewportContents(children: Element) -> Element {
    rsx! {
        div { class: "p-4 overflow-y-scroll grow", {children} }
    }
}

#[component]
pub fn Viewport(children: Element) -> Element {
    rsx! {
        div { class: "flex flex-col flex-1 overflow-y-auto overflow-x-hidden h-screen",
            {children}
        }
    }
}

use crate::components::spinner::Spinner;
use dioxus::prelude::*;
use dioxus_free_icons::{Icon, icons::bs_icons};

const COLORS: &str = "dark:border-neutral-800/80 border-black/20 dark:bg-neutral-900/80 dark:hover:border-neutral-700/70 dark:hover:bg-neutral-800/70 transition-color dark:hover:text-white duration-200";
const LAYOUT: &str = "border  rounded flex flex-col grow rounded gap-2";

#[component]
pub fn BentoContainer(children: Element) -> Element {
    rsx! {
        div { class: "flex gap-4 flex-col", {children} }
    }
}

#[component]
pub fn BentoRow(children: Element) -> Element {
    rsx! {
        div { class: "h-48 min-h-48 flex gap-4 overflow-x-scroll overflow-y-hidden w-full max-w-screen",
            {children}
        }
    }
}

#[component]
pub fn BentoBoxSquare(children: Element) -> Element {
    rsx! {
        div { class: "h-48 min-w-48 w-48 max-w-48 {COLORS} {LAYOUT}", {children} }
    }
}

#[component]
pub fn BentoBoxFill(children: Element) -> Element {
    rsx! {
        div { class: "h-48 {COLORS} {LAYOUT}", {children} }
    }
}

#[component]
pub fn BentoBoxDouble(children: Element) -> Element {
    rsx! {
        div { class: "min-w-128 w-96 max-w-96 h-48 {COLORS} {LAYOUT}", {children} }
    }
}

#[component]
pub fn BentoTitle(children: Element) -> Element {
    rsx! {
        h3 { class: "text-xl w-full dark:bg-white/2 border-b dark:border-white/5 border-black/10 p-2 rounded-t font-medium dark:text-white select-none",
            {children}
        }
    }
}

#[component]
pub fn BentoContent(children: Element) -> Element {
    rsx! {
        div { class: "w-full px-2 font-regular grid text-lg overflow-y-scroll", {children} }
    }
}

#[component]
pub fn BentoPlaceholder() -> Element {
    rsx! {
        div { class: "flex items-center justify-center h-full",
            Spinner { class: "w-8 h-8 opacity-50" }
        }
    }
}

#[component]
pub fn BentoError(children: Element) -> Element {
    rsx! {
        div { class: "m-2 overflow-y-scroll flex items-center flex-col h-full text-red-500",
            Icon {
                icon: bs_icons::BsExclamationCircle,
                class: "text-red-500 h-8",
            }
            {children}
        }
    }
}

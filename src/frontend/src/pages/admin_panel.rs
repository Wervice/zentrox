use crate::{HTTP_PREFIX, pages};
use api::account::AccountDetailsRes;
use dioxus_free_icons::IconShape;
use crate::prelude::*;
use inflector::Inflector;
use web_sys::window;

#[derive(Clone)]
struct Level(usize);

#[component]
pub fn SidebarButton<I>(icon: I, caption: String, select: bool, onclick: EventHandler) -> Element where I: IconShape + Clone + PartialEq + 'static {
    let colors = if select {
        "dark:bg-neutral-800 bg-neutral-100 dark:hover:bg-neutral-700 hover:bg-neutral-200 font-medium dark:text-white text-black"
    } else {
        "dark:bg-neutral-900 bg-neutral-300 dark:hover:bg-neutral-800 hover:bg-neutral-200"
    };

    rsx! {
        button {
            class: "flex items-center mb-2 p-2 rounded {colors} transition-color duration-200 ease-in-out gap-2 w-full",
            onclick: move |_| onclick.call(()),
            title: caption,
            span {
                class: "flex gap-1 items-center justify-center md:justify-start grow",
                Icon {
                    icon
                }
                span {class: "hidden md:inline-block", "{caption}" }
            }
        }
    }
}

#[component]
pub fn SidebarHeader(children: Element) -> Element {
    rsx! {
        div { class: "hidden md:flex gap-2 text-xl font-medium items-center p-4 h-16 dark:bg-neutral-950 bg-neutral-200 border-b dark:border-neutral-800 border-neutral-300",
            {children}
        }
    }
}

#[component]
pub fn SidebarFooter(children: Element) -> Element {
    rsx! {
        div { class: "hidden md:flex gap-2 items-center p-4 h-12 dark:bg-neutral-950 bg-neutral-200 border-t dark:border-neutral-800 border-neutral-300",
            {children}
        }
    }
}

#[component]
pub fn SidebarEntries(children: Element, padding: Option<bool>) -> Element {
    let level = use_context::<Level>();

    let bg = {
        if level.0 > 1 {
            "dark:bg-white/2 bg-black/5"
        } else {
            "dark:bg-neutral-950 bg-neutral-300"
        }
    };

    let pad_class = if let Some(p) = padding
        && !p
    {
        ""
    } else {
        "px-2 py-4"
    };

    rsx! {
        div { class: "flex flex-col flex-1 overflow-y-auto",
            nav { class: "flex-1 {pad_class} {bg} gap-2", {children} }
        }
    }
}

#[component]
pub fn Sidebar(children: Element, level: usize) -> Element {
    use_context_provider(|| Level(level));

    let border = {
        if level > 1 {
            "dark:border-neutral-900 border-neutral-300"
        } else {
            "dark:border-neutral-800 border-neutral-300"
        }
    };

    rsx! {
        div { class: "flex flex-col md:min-w-64 md:max-w-64 border-r {border}",
            {children}
        }
    }
}

#[derive(Clone)]
pub struct SidebarToggle(pub Signal<bool>);

#[derive(PartialEq, Eq, Debug)]
pub enum Page {
    Dashboard,
    Drives,
}

#[component]
pub fn AdminPanel() -> Element {
    let sidebar_toggle_signal = use_signal(|| true);
    let mut page_signal = use_signal(|| Page::Dashboard);
    use_context_provider(|| SidebarToggle(sidebar_toggle_signal));
    use_context_provider(|| page_signal);
    let account_details = use_context::<Signal<Option<AccountDetailsRes>>>();

    rsx! {
        div { class: "flex h-screen",
            if *sidebar_toggle_signal.read() {
                Sidebar { level: 1,
                    SidebarHeader {
                        img {
                            src: asset!("/assets/emblem.svg"),
                            class: "h-8 w-8 inline-block invert dark:invert-0",
                        }
                        "Zentrox"
                    }
                    SidebarEntries {
                        SidebarButton {
                            select: *page_signal.read() == Page::Dashboard,
                            onclick: move |_| { page_signal.set(Page::Dashboard) },
                            icon: bs_icons::BsSpeedometer,
                            caption: "Dashboard"
                        }
                        SidebarButton {
                            select: *page_signal.read() == Page::Drives,
                            onclick: move |_| { page_signal.set(Page::Drives) },
                            icon: bs_icons::BsHdd,
                            caption: "Drives"
                        }
                    }
                    SidebarFooter {
                        object {
                            data: format!("{HTTP_PREFIX}/private/account/profilePicture"),
                            class: "h-8 w-8 rounded-full",
                            img {
                                src: asset!("/assets/account.svg"),
                                class: "h-8 w-8 rounded-full",
                            }
                        }
                        {
                            match &*account_details.read() {
                                Some(val) => val.username.clone().to_title_case(),
                                None => "No username".to_string(),
                            }
                        }
                        div { class: "w-full items-center justify-end flex gap-2",
                            button { class: "w-min",
                                Icon {
                                    icon: bs_icons::BsGear,
                                    class: "dark:text-neutral-300 dark:hover:text-white text-neutral-950 hover-black transition-color duration-200 cursor-pointer",
                                }
                            }

                            button {
                                class: "w-min",
                                onclick: move |_| {
                                    window()
                                        .unwrap()
                                        .location()
                                        .set_href(format!("{HTTP_PREFIX}/private/auth/logout").as_str());
                                },
                                Icon {
                                    icon: bs_icons::BsBoxArrowRight,
                                    class: "dark:text-neutral-300 dark:hover:text-white text-neutral-950 hover-black transition-color duration-200 cursor-pointer",
                                }
                            }
                        }
                    }
                }
            }
            div {
                class: "w-full",
                hidden: *page_signal.read() != Page::Dashboard,
                pages::dashboard::Dashboard {}
            }
            div { class: "w-full", hidden: *page_signal.read() != Page::Drives,
                pages::drives::page::Drives {}
            }
        }
    }
}

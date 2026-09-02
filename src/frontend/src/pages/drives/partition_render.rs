use std::rc::Rc;

use api::drives::{Drive, FileSystem, WithFilesystem, WithPrettyName};
use uuid::Uuid;

use chrono::{DateTime, Local};
use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::bs_icons;

use crate::{
    components::{
        dialog::*,
        dropdown::{Dropdown, DropdownContents, DropdownTrigger},
        spinner::Spinner,
    },
};
use crate::pages::drives::{check::CheckButton, mount::MountButton, repair::RepairButton};

#[derive(Clone)]
struct FileSystemTask {
    title: String,
    message: String,
    started: DateTime<Local>,
    id: Uuid,
}

#[derive(Clone)]
pub struct FileSystemActiveTasks(Vec<FileSystemTask>);

impl FileSystemActiveTasks {
    pub fn append(&mut self, title: String, message: String) -> Uuid {
        let id = uuid::Uuid::new_v4();

        self.0.push(FileSystemTask {
            title,
            message,
            started: Local::now(),
            id,
        });

        id
    }

    pub fn remove(&mut self, uuid: Uuid) {
        self.0.retain(|x| x.id != uuid);
    }
}

#[css_module("/src/components/dialog/style.css")]
struct DialogStyle;

#[component]
fn FileSystemRender(
    fs: FileSystem,
    mount_needs_password: Signal<Option<bool>>,
    refetch: EventHandler,
    drive: Rc<Drive>
) -> Element {
    let task_signal = use_signal(|| FileSystemActiveTasks(vec![]));
    let tasks = task_signal.read();

    let mut tasks_dialog_open = use_signal(|| false);
    use_context_provider(|| task_signal);

    rsx! {
        Dialog {
            open: tasks_dialog_open(),
            on_open_change: move |v| tasks_dialog_open.set(v),
            button {
                class: DialogStyle::dx_dialog_close,
                r#type: "button",
                aria_label: "Close",
                tabindex: if tasks_dialog_open() { "0" } else { "-1" },
                onclick: move |_| tasks_dialog_open.set(false),
                "×"
            }
            DialogTitle { "Active tasks for {fs.pretty_name()}" }
            DialogDescription {
                span { class: "max-h-64 overflow-scroll gap-2 flex flex-col",
                    if tasks.0.is_empty() {
                        span { class: "text-center opacity-50 text-xl", "No active tasks" }
                    }
                    {
                        tasks
                            .0
                            .iter()
                            .map(|task| {
                                rsx! {
                                    span { class: "block rounded p-2 bg-neutral-200 dark:bg-neutral-900 text-black dark:text-white",
                                        strong { class: "block", "{task.title}" }
                                        small { class: "block", "Started {task.started}" }
                                        "{task.message}"
                                    }
                                }
                            })
                    }
                }
            }
        }
        span {
            class: "grow bg-no-repeat dark:bg-neutral-800 bg-neutral-300 rounded",
            background_image: {
                if let Some(size) = fs.size && let Some(used) = fs.used {
                    let ratio = 100.0 - ((used.0 as f64 / size.0 as f64) * 100.0).floor();
                    format!(
                        "linear-gradient(180deg,rgba(255, 255, 255, 0) {ratio}%, oklch(68.5% 0.169 237.323 / 0.2) {ratio}%)",
                    )
                } else {
                    "".to_string()
                }
            },
            span { class: "flex p-2 border-b border-neutral-400 dark:border-neutral-700 items-center",
                strong { class: "grow truncate", "{fs.name}" }
                button {
                    class: "p-1 hover:bg-neutral-200 hover:dark:bg-neutral-900 transition-all duration-100 rounded-full cursor-pointer",
                    title: "Click to see active tasks on this filesystem",
                    hidden: task_signal.read().0.is_empty(),
                    onclick: move |_| {
                        tasks_dialog_open.set(true);
                    },
                    Spinner { class: "text-neutral-600 dark:text-neutral-300" }
                }
                Dropdown {
                    DropdownTrigger { class: "p-1 hover:bg-neutral-200 hover:dark:bg-neutral-900 transition-all duration-100 rounded-full",
                        Icon {
                            class: "cursor-pointer",
                            icon: bs_icons::BsThreeDotsVertical,
                        }
                    }
                    DropdownContents {
                        MountButton {
                            fs: Rc::new(fs.clone()),
                            drive: drive.clone(),
                            needs_password: mount_needs_password,
                            refetch,
                        }
                        CheckButton {
                            fs: Rc::new(fs.clone()),
                            drive: drive.clone(),
                            refetch,
                        }
                        RepairButton {
                            fs: Rc::new(fs.clone()),
                            drive: drive.clone(),
                            refetch,
                        }
                    }
                }
            }
            span { class: "block p-2",
                if let Some(ref kind) = fs.kind {
                    "{kind}"
                    if let Some(ref version) = fs.version {
                        " ({version})"
                    }
                } else {
                    "Unknown filesystem"
                }
                br {}
                {fs.size.map(|s| s.to_string()).unwrap_or("Unknown size".to_string())}
                {fs.available.map(|s| format!(" ({s} free)"))}
                br {}
                br {}
                i { "Mountpoints" }
                br {}
                {
                    if fs.mountpoints.is_empty() {
                        rsx! {
                            i { "Not mounted" }
                        }
                    } else {
                        rsx! {}
                    }
                }
                {fs.mountpoints.iter().map(|p| rsx! {
                    code { {p.to_string_lossy().to_string()} }
                    br {}
                })}
            }
        }
        {
            fs.children
                .iter()
                .map(|inner_fs| {
                    rsx! {
                        FileSystemRender {
                            fs: inner_fs.clone(),
                            mount_needs_password,
                            key: "{inner_fs.id()}",
                            refetch,
                            drive: drive.clone(),
                        }
                    }
                })
        }
    }
}

#[component]
pub fn PartitionRender<F: WithFilesystem + WithPrettyName + Clone + PartialEq + 'static>(
    element: Rc<F>,
    mount_needs_password: Signal<Option<bool>>,
    refetch: EventHandler,
    drive: Rc<Drive>
) -> Element {

    rsx! {
        span {
            class: "flex h-full min-w-64",
            width: format!(
                "{}%",
                ((element.size().0 as f64 / drive.size.0 as f64) * 100.0_f64).floor(),
            ),
            span { class: "p-4 flex flex-col gap-4 w-full",
                span { class: "flex whitespace-nowrap overflow-none",
                    span { class: "truncate grow font-bold", {element.pretty_name()} }
                    span { {element.size().to_string()} }
                }
                {
                    element
                        .fs()
                        .map(|fs| {
                            rsx! {
                                FileSystemRender {
                                    fs,
                                    key: "{fs.id()}",
                                    mount_needs_password,
                                    refetch,
                                    drive: drive.clone(),
                                }
                            }
                        })
                }
            }
        }
    }
}

use std::rc::Rc;
use std::sync::Arc;

use crate::pages::drives::{poweroff::PowerOffButton, benchmark::BenchmarkButton, eject::EjectButton};
use crate::pages::drives::partition_render::PartitionRender;
use api::drives::{WithFilesystem, WithPrettyName};
use api::{drives::Drive, units::Bytes};
use dioxus::prelude::*;
use dioxus::web::WebEventExt;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::bs_icons;

use crate::{
    components::{
        date::Date,
        tooltip::{Tooltip, TooltipContent, TooltipTrigger},
        viewport::*,
    },
    pages::admin_panel::{Page, Sidebar, SidebarEntries},
    request,
};

// TODO Better error handling for wrong passwords

#[component]
fn DriveButton(children: Element, select: bool, onclick: EventHandler) -> Element {
    let colors = if select {
        "dark:border-neutral-900 border-neutral-200 dark:bg-white/5 bg-white"
    } else {
        "dark:border-neutral-900 border-neutral-300 dark:bg-black bg-neutral-200"
    };

    rsx! {
        button {
            class: "p-2 border-b {colors} w-full text-left cursor-pointer transition-color duration-200 ease-in-out",
            onclick: move |_| onclick.call(()),
            {children}
        }
    }
}

#[component]
fn Badge(description: String, title: String, class: String) -> Element {
    rsx! {
        Tooltip {
            TooltipTrigger {
                span { class: "px-2 py-1 rounded-full text-black dark:text-white text-xs font-medium cursor-pointer select-none {class}",
                    {title}
                }
            }
            TooltipContent { side: dioxus_primitives::ContentSide::Bottom, {description} }
        }
    }
}

#[component]
pub fn DriveInformation(
    drive: Drive,
    poweroff_needs_password: Signal<Option<bool>>,
    mount_needs_password: Signal<Option<bool>>,
    eject_needs_password: Signal<Option<bool>>,
    benchmark_needs_password: Signal<Option<bool>>,
    refetch: EventHandler,
) -> Element {
    let mut show_serial = use_signal(|| false);
    let drive_rc = Rc::new(drive.clone());
    
    let mut show_details = use_signal(|| false);

    let FIELD_CLASS: &str = "font-bold dark:text-neutral-300 text-neutral-800";
    let VALUE_CLASS: &str = "text-right w-112 max-w-112 truncate";

    rsx! {
        details {
            open: show_details,
            ontoggle: move |_| {
                show_details.toggle();
            },
            summary { class: "text-2xl font-bold items-center inline-flex gap-2 cursor-pointer mb-1",
                if *show_details.read() {
                    dioxus_free_icons::Icon { icon: bs_icons::BsChevronDown }
                } else {
                    dioxus_free_icons::Icon { icon: bs_icons::BsChevronRight }
                }
                {drive_rc.pretty_name()}
                if drive_rc.hint_system {
                    Badge {
                        title: "System internal",
                        description: "Udisks2 reports this device to be a system device. Additional permissions would be required to access this device.",
                        class: "bg-blue-400 dark:bg-sky-800",
                    }
                }
                if drive_rc.removable {
                    Badge {
                        title: "Removable",
                        description: "This device is likely removable and not permanently installed.",
                        class: "bg-green-500 dark:bg-green-800",
                    }
                }
                if drive_rc.read_only {
                    Badge {
                        title: "Read-only",
                        description: "The device can only be read from, but not written to.",
                        class: "bg-red-500 dark:bg-red-800",
                    }
                }
            }
            table {
                tr {
                    td { class: FIELD_CLASS, "Model" }
                    td { class: VALUE_CLASS,
                        {drive_rc.model.as_deref().unwrap_or("Unknown model")}
                        if let Some(r) = drive_rc.revision.as_deref() {
                            " (Revision: {r})"
                        } else {
                            ""
                        }
                    }
                }
                tr {
                    td { class: FIELD_CLASS, "Vendor" }
                    td { class: VALUE_CLASS,
                        {drive_rc.vendor.as_deref().unwrap_or("Unknown vendor")}
                    }
                }
                tr {
                    td { class: FIELD_CLASS, "Serial" }
                    td { class: VALUE_CLASS,
                        span {
                            onclick: move |_| { show_serial.toggle() },
                            title: "Click to show serial",
                            class: "cursor-pointer",
                            if *show_serial.read() {
                                {drive_rc.serial.as_deref().unwrap_or("Unknown serial.")}
                            } else {
                                "#######"
                            }
                        }
                    }
                }
                tr {
                    td { class: FIELD_CLASS, "Time detected" }
                    td { class: VALUE_CLASS,
                        Date { duration: chrono::DateTime::from_timestamp_millis(drive_rc.time_detected).unwrap().into() }
                    }
                }
                tr {
                    td { class: FIELD_CLASS, "Device node" }
                    td { class: VALUE_CLASS, "{drive_rc.path.to_string_lossy()}" }
                }
            }
        }
        span { class: "mt-1 rounded-t dark:bg-neutral-900 bg-neutral-200 border-b dark:border-b-neutral-800 border-b-neutral-300 max-w-[calc(100%-2em)] block",
            PowerOffButton {
                needs_password: poweroff_needs_password,
                drive: drive_rc.clone(),
                refetch,
            }
            EjectButton {
                needs_password: eject_needs_password,
                drive: drive_rc.clone(),
                refetch,
            }
            BenchmarkButton {
                needs_password: benchmark_needs_password,
                drive: drive_rc.clone(),
                refetch,
            }
        }
        span { class: "flex dark:bg-neutral-900 bg-neutral-200 divide-x divide-solid dark:divide-neutral-800 divide-neutral-300 grow max-w-[calc(100%-2em)] h-128 rounded-b",
            {drive_rc.partitions.iter().map(|partition| rsx! {
                PartitionRender {
                    element: Rc::new(partition.clone()),
                    key: "partition-{partition.id()}-{drive_rc.id()}",
                    mount_needs_password,
                    refetch,
                    drive: drive_rc.clone(),
                }
            })}
            if drive_rc.partitions.is_empty() && drive_rc.filesystem.is_some() {
                PartitionRender {
                    element: drive_rc.clone(),
                    key: "partition-{drive_rc.id()}-{drive_rc.id()}",
                    mount_needs_password,
                    refetch,
                    drive: drive_rc.clone(),
                }
            } else if drive_rc.partitions.iter().fold(0, |acc, p| acc + p.size().0) < drive_rc.size().0 {
                span { class: "flex h-full min-w-32 grow justify-center text-center items-center hatched",
                    {
                        format!(
                            "{}",
                            drive_rc.size()
                                - drive_rc.partitions.iter().fold(Bytes(0), |acc, p| acc + p.size()),
                        )
                    }
                }
            }
        }
    }
}

#[component]
pub fn Contents() -> Element {
    let page_ctx = use_context::<Signal<Page>>();
    let mut refetch_signal = use_signal(|| false);

    let mut mount_needs_password = use_signal(|| None);
    let mut poweroff_needs_password = use_signal(|| None);
    let mut eject_needs_password = use_signal(|| None);
    let benchmark_needs_password = use_signal(|| Some(true));

    let drives_req = use_resource(move || async move {
        refetch_signal.read();
        request::get::<api::drives::DrivesRes>("/private/drives/list")
            .await
            .map(|mut res| {
                res.drives
                    .sort_by_key(|r| std::cmp::Reverse(r.time_detected));
                res.drives
                    .iter()
                    .filter(|drive| !drive.hint_ignore)
                    .cloned()
                    .collect::<Arc<[Drive]>>()
            })
    });

    use_future(move || async move {
        let mount =
            request::get::<api::polkit::NeedsPasswordRes>("/private/drives/can_mount").await;
        let poweroff =
            request::get::<api::polkit::NeedsPasswordRes>("/private/drives/can_poweroff").await;
        let eject =
            request::get::<api::polkit::NeedsPasswordRes>("/private/drives/can_eject").await;

        if let Ok(inner) = mount {
            mount_needs_password.set(Some(inner.needs_password));
        }

        if let Ok(inner) = poweroff {
            poweroff_needs_password.set(Some(inner.needs_password));
        }

        if let Ok(inner) = eject {
            eject_needs_password.set(Some(inner.needs_password));
        }
    });

    let mut selected_drive = use_signal::<Option<api::drives::Drive>>(|| None);
    let mut active_drive = use_signal::<Option<Drive>>(|| None);
    use_memo(move || {
        let selected_drive_read = &*selected_drive.read();
        if let Some(user_selection) = selected_drive_read
            && let Some(Ok(list)) = &*drives_req.read()
        {
            active_drive.set(
                list.iter()
                    .find(|new_drive| {
                        new_drive.time_detected == user_selection.time_detected
                            && new_drive.path == user_selection.path
                    })
                    .cloned(),
            );
        } else {
            active_drive.set(None)
        }
    });

    use_future(move || async move {
        // Resources are re-fetched in a certain interval, but only if no other resources are
        // currently not loaded.
        loop {
            if !drives_req.pending() && *page_ctx.read() == Page::Drives {
                refetch_signal.toggle();
            }
            gloo_timers::future::sleep(std::time::Duration::from_millis(2000)).await;
        }
    });

    rsx! {
        div { class: "flex grow",
            Sidebar { level: 2,
                SidebarEntries { padding: false,
                    match &*drives_req.read() {
                        Some(Ok(drives_res)) => rsx! {
                            span { class: "block border-b dark:border-neutral-900 border-neutral-300 dark:bg-black bg-neutral-200 p-2",
                                {
                                    format!(
                                        "{} connected drive{}.",
                                        drives_res.len(),
                                        if drives_res.len() == 1 { "" } else { "s" },
                                    )
                                }
                            }
                            for d in drives_res {
                                DriveButton {
                                    select: active_drive
                                        .read()
                                        .clone()
                                        .is_some_and(|act| act.time_detected == d.time_detected && act.path == d.path),
                                    key: "{d.id()}",
                                    onclick: {
                                        let c = d.clone();
                                        move |_| {
                                            selected_drive.set(Some(c.clone()));
                                        }
                                    },
                                    span { class: "flex items-center gap-1",
                                        span { class: "min-w-8 w-8",
                                            if d.bus_type == "usb" {
                                                Icon { icon: bs_icons::BsUsbDriveFill, class: "w-full" }
                                            } else {
                                                if d.rotating {
                                                    Icon { icon: bs_icons::BsDeviceHddFill, class: "w-full" }
                                                } else {
                                                    Icon { icon: bs_icons::BsDeviceSsdFill, class: "w-full" }
                                                }
                                            }
                                        }
                                        span { class: "grow",
                                            span { class: "flex",
                                                strong { class: "truncate grow max-w-32", {d.pretty_name()} }
                                                span { class: "min-w-8 grow text-right", "{d.size()}" }
                                            }
                                            small { class: "truncate", {d.model.as_deref().unwrap_or("Unknown model")} }
                                        }
                                    }
                                }
                            }
                        },
                        Some(Err(e)) => rsx! {
                            span { class: "p-2 block text-red-500", "{e}" }
                        },
                        None => rsx! {},
                    }
                }
            }
            ViewportContents {
                if let Some(active_drive) = active_drive() && let Some(Ok(ref res)) = drives_req() {
                    for available_drive in res {
                        span {
                            hidden: active_drive != *available_drive,
                            key: "drive-information-span-{available_drive.id()}",
                            DriveInformation {
                                drive: available_drive.clone(),
                                poweroff_needs_password,
                                mount_needs_password,
                                eject_needs_password,
                                benchmark_needs_password,
                                refetch: move || {
                                    refetch_signal.toggle();
                                },
                            }
                        }
                    }
                } else {
                    span { class: "flex w-full h-full items-center justify-center text-xl opacity-50",
                        "Select drive"
                    }
                }
            }
        }
    }
}

#[component]
pub fn Drives() -> Element {
    rsx! {
        Viewport {
            ViewportTabs { "Drives" }
            SuspenseBoundary { fallback: |_| rsx! {}, Contents {} }
        }
    }
}

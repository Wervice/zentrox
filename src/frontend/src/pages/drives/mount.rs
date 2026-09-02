use crate::prelude::*;

use api::{
    drives::{
        Drive, FileSystem, FileSystemActionReq, MountRes, WithPrettyName
    }, error::MessageRes
};

use crate::{
    pages::drives::partition_render::FileSystemActiveTasks,
};

async fn attempt_mount(
    fs: &FileSystem,
    drive: &Drive,
    password: Option<String>,
) -> Result<MountRes, String> {
    crate::request::post::<MountRes>(
        "/private/drives/mount",
        FileSystemActionReq {
            drive: api::drives::DriveSignature {
                time_detected: drive.time_detected,
                device_node: drive.path.clone(),
                serial: drive.serial.clone(),
                udisks_id: drive.udisks_id.clone()
            },
            fs: fs.path.clone(),
            password,
        },
    )
    .await
}

async fn attempt_unmount(fs: &FileSystem, drive: &Drive) -> Result<MessageRes, String> {
    crate::request::post::<MessageRes>(
        "/private/drives/unmount",
        FileSystemActionReq {
            drive: api::drives::DriveSignature {
                time_detected: drive.time_detected,
                device_node: drive.path.clone(),
                serial: drive.serial.clone(),
                udisks_id: drive.udisks_id.clone()
            },
            fs: fs.path.clone(),
            password: None,
        },
    )
    .await
}
#[component]
pub fn MountButton(
    fs: Rc<FileSystem>,
    drive: Rc<Drive>,
    needs_password: Signal<Option<bool>>,
    refetch: EventHandler,
) -> Element {
    let mut icon_state = use_signal::<ButtonState>(ButtonState::default);
    let toast_api = consume_toast();
    let has_mountpoints = !fs.mountpoints.is_empty();
    let mut active_tasks = consume_context::<Signal<FileSystemActiveTasks>>();

    rsx! {
        AuthButton {
            disabled: drive.hint_system || icon_state.read().is_in_progress(),
            title: if fs.mountpoints.is_empty() { "Mount" } else { "Unmount" },
            needs_password: Signal::new((*needs_password.read()).map(|b| b && !has_mountpoints)),
            variant: ButtonVariant::Ghost,
            size: crate::components::button::ButtonSize::Sm,
            class: "button w-full",
            onclick: move |password| {
                let fs_copy = fs.clone();
                let drive_copy = drive.clone();
                icon_state.set(ButtonState::InProgress);
                if !has_mountpoints {
                    spawn({
                        let id = active_tasks
                            .with_mut(|s| {
                                s
                                    .append(
                                    "Mounting".to_string(),
                                    "Requesting for the filesystem to be mounted.".to_string(),
                                )
                            });
                        async move {
                            let req = attempt_mount(&fs_copy, &drive_copy, password).await;
                            refetch.call(());
                            match req {
                                Ok(m) => {
                                    active_tasks.with_mut(|s| s.remove(id));
                                    icon_state.set(ButtonState::Default);
                                    toast_api
                                        .info(
                                            "Mounted filesystem".to_string(),
                                            ToastOptions::new()
                                                .description(
                                                    format!(
                                                        "{} has been mounted to {}.",
                                                        fs_copy.pretty_name(),
                                                        m.mountpoint.to_string_lossy(),
                                                    ),
                                                )
                                                .permanent(false),
                                        );
                                }
                                Err(err) => {
                                    active_tasks.with_mut(|s| s.remove(id));
                                    icon_state.set(ButtonState::Failed);
                                    toast_api
                                        .error(
                                            format!("Failed to mount {}", fs_copy.pretty_name()),
                                            ToastOptions::new().description(err).permanent(true),
                                        );
                                }
                            }
                        }
                    });
                } else {
                    let id = active_tasks
                        .with_mut(|s| {
                            s
                                .append(
                                "Unmounting".to_string(),
                                "Requesting for the filesystem to be unmounted.".to_string(),
                            )
                        });
                    spawn({
                        async move {
                            match attempt_unmount(&fs_copy, &drive_copy).await {
                                Ok(_) => {
                                    icon_state.set(ButtonState::Default);
                                    active_tasks.with_mut(|s| s.remove(id));
                                    toast_api
                                        .info(
                                            "Unmounted filesystem".to_string(),
                                            ToastOptions::new()
                                                .description(
                                                    format!("{} has been unmounted.", fs_copy.pretty_name()),
                                                )
                                                .permanent(false),
                                        );
                                }
                                Err(err) => {
                                    icon_state.set(ButtonState::Failed);
                                    active_tasks.with_mut(|s| s.remove(id));
                                    toast_api
                                        .error(
                                            format!("Failed to unmount {}", fs_copy.pretty_name()),
                                            ToastOptions::new().description(err).permanent(true),
                                        );
                                }
                            }
                        }
                    });
                }
            },
            match *icon_state.read() {
                ButtonState::Default | ButtonState::Failed => {
                    rsx! {
                        if !has_mountpoints {
                            "Mount"
                        } else {
                            "Unmount"
                        }
                    }
                }
                ButtonState::InProgress => rsx! { "Working..." },
            }
        }
    }
}


use crate::prelude::*;

use api::{
    EmptyRes, drives::{
        Drive, FileSystem, FileSystemActionReq, RepairRes, WithPrettyName
    }, jobs::JobRes
};

use crate::{
    pages::drives::partition_render::FileSystemActiveTasks,
    request::{self, Job},
};

async fn attempt_repair(fs: &FileSystem, drive: &Drive) -> Result<JobRes, String> {
    request::post::<JobRes>(
        "/private/drives/repair",
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
pub fn RepairButton(drive: Rc<Drive>, fs: Rc<FileSystem>, refetch: EventHandler) -> Element {
    let mut dialog_open = use_signal(|| false);
    let mut button_state = use_signal(ButtonState::default);
    let mut active_tasks = consume_context::<Signal<FileSystemActiveTasks>>();

    let toast_api = consume_toast();

    let drive_for_closure = drive.clone();
    let fs_for_closure = fs.clone();

    let onclick = move |_| {
        let drive_copy = drive_for_closure.clone();
        let fs_copy = fs_for_closure.clone();
        dialog_open.set(false);
        spawn(async move {
            let user_task_id = active_tasks
                .with_mut(|s| {
                    s
                        .append(
                            "Repair filesystem".to_string(),
                            "Trying to repair the filesystem. This may take several minutes to complete."
                            .to_string(),
                    )
                });
            button_state.set(ButtonState::InProgress);
            let res = attempt_repair(&fs_copy, &drive_copy).await;
            match res {
                Ok(job) => {
                    let uuid = job.uuid;
                    loop {
                        match request::get_job_status::<RepairRes, EmptyRes>(uuid).await {
                            Ok(Job::Ongoing | Job::Update(_)) => {}
                            Ok(Job::Ok(res)) => {
                                if res.success {
                                    toast_api
                                        .success(
                                            format!("Repaired {}.", fs_copy.pretty_name()),
                                            ToastOptions::default()
                                                .description(
                                                    "The filesystem has been repaired.".to_string(),
                                                )
                                                .permanent(true),
                                        );
                                    button_state.set(ButtonState::Default);
                                } else {
                                    toast_api
                                        .error(
                                            format!("Failed to repair {}!", fs_copy.pretty_name()),
                                            ToastOptions::default()
                                                .description(
                                                    "Repairing the filesystem was not possible.".to_string(),
                                                )
                                                .permanent(true),
                                        );
                                    button_state.set(ButtonState::Failed);
                                }
                                active_tasks.with_mut(|s| s.remove(user_task_id));
                                refetch.call(());
                                break;
                            }
                            Err(err) | Ok(Job::Failed(err)) => {
                                toast_api
                                    .error(
                                        format!("Failed to repair {}!", fs_copy.pretty_name()),
                                        ToastOptions::default().description(err).permanent(true),
                                    );
                                button_state.set(ButtonState::Failed);
                                active_tasks.with_mut(|s| s.remove(user_task_id));
                                refetch.call(());
                                break;
                            }
                        }
                        gloo_timers::future::sleep(
                                std::time::Duration::from_millis(1000),
                            )
                            .await;
                    }
                }
                Err(err) => {
                    toast_api
                        .error(
                            format!("Failed to repair {}!", fs_copy.pretty_name()),
                            ToastOptions::default().description(err).permanent(true),
                        );
                    active_tasks.with_mut(|s| s.remove(user_task_id));
                    button_state.set(ButtonState::Failed);
                }
            }
        });
    };

    rsx! {
        AlertDialog { open: dialog_open(), on_open_change: move |v| dialog_open.set(v),
            AlertDialogTitle { "Do you want to attempt to repair {fs.pretty_name()}?" }
            AlertDialogDescription {
                "Tries to repair a filesystem. It may take several minutes to complete. Attempt to check the filesystem for inconsistencies before repairing to safe time. Click "
                a {
                    href: "https://storaged.org/udisks/docs/gdbus-org.freedesktop.UDisks2.Filesystem.html#gdbus-method-org-freedesktop-UDisks2-Filesystem.Repair",
                    target: "_blank",
                    class: "underline text-blue-500",
                    "here"
                }
                " for further details."
            }
            AlertDialogActions {
                AlertDialogCancel { "Cancel" }
                Button { size: ButtonSize::Lg, onclick, "Start repair" }
            }
        }
        Button {
            disabled: drive.hint_system || !fs.can_repair || button_state.read().is_in_progress(),
            title: "Attempt to repair filesystem",
            variant: ButtonVariant::Ghost,
            size: crate::components::button::ButtonSize::Sm,
            class: "button w-full",
            onclick: move |_| {
                dialog_open.set(true);
            },
            match *button_state.read() {
                ButtonState::Default | ButtonState::Failed => rsx! { "Repair" },
                ButtonState::InProgress => rsx! { "Repairing..." },
            }
        }
    }
}

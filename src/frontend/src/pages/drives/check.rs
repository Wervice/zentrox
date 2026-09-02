use crate::{onunload, pages::drives::partition_render::FileSystemActiveTasks, prelude::*, request::Job};

use api::{
    EmptyRes, drives::{
        CheckRes, Drive, FileSystem, FileSystemActionReq, WithPrettyName
    },  jobs::JobRes
};

async fn attempt_check(fs: &FileSystem, drive: &Drive) -> Result<JobRes, String> {
    crate::request::post::<JobRes>(
        "/private/drives/check",
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
pub fn CheckButton(fs: Rc<FileSystem>, drive: Rc<Drive>, refetch: EventHandler) -> Element {
    let mut button_state = use_signal(ButtonState::default);
    let toast_api = consume_toast();
    let mut open_signal = use_signal(|| false);
    let mut active_tasks = consume_context::<Signal<FileSystemActiveTasks>>();

    let drive_for_closure = drive.clone();
    let fs_for_closure = fs.clone();

    let onclick = move |_| {
        let fs_copy = fs_for_closure.clone();
        let drive_copy = drive_for_closure.clone();
        let user_task_id = active_tasks
            .with_mut(|s| {
                s
                .append(
                    "Checking for consistency".to_string(),
                    "A check is running to examine if the filesystem is consistent. This may take several minutes to complete."
                    .to_string(),
                )
        });
        button_state.set(ButtonState::InProgress);
        spawn(async move {
            open_signal.set(false);
            let cb = onunload::warn();
            match attempt_check(&fs_copy, &drive_copy).await {
                Ok(job) => {
                    let uuid = job.uuid;
                    loop {
                        match crate::request::get_job_status::<CheckRes, EmptyRes>(uuid).await {
                            Ok(Job::Ongoing | Job::Update(_)) => {}
                            Ok(Job::Ok(res)) => {
                                onunload::unwarn(cb);
                                if res.consistent {
                                    toast_api
                                        .success(
                                            format!(
                                                "The filesystem {} is consistent.",
                                                fs_copy.pretty_name(),
                                            ),
                                            ToastOptions::default()
                                                .description(
                                                    "The consistency check completed, and the filesystem appears to be consistent and not damaged.",
                                                )
                                                .permanent(true),
                                        );
                                } else {
                                    toast_api
                                        .warning(
                                            format!(
                                                "The filesystem {} is damaged!",
                                                fs_copy.pretty_name(),
                                            ),
                                            ToastOptions::default()
                                                .description(
                                                    "The consistency check completed, and the filesystem appears to be inconsistent and thus damaged.
                                                    You may want to consider using the \"Repair\" feature to attempt to repair the filesystem.",
                                                )
                                                .permanent(true),
                                        );
                                }
                                active_tasks.with_mut(|s| s.remove(user_task_id));
                                button_state.set(ButtonState::Default);
                                refetch.call(());
                                break;
                            }
                            Err(err) | Ok(Job::Failed(err)) => {
                                onunload::unwarn(cb);
                                toast_api
                                    .error(
                                        format!(
                                            "Failed to check {} for consistency!",
                                            fs_copy.pretty_name(),
                                        ),
                                        ToastOptions::default().description(err).permanent(true),
                                    );
                                button_state.set(ButtonState::Failed);
                                active_tasks.with_mut(|s| s.remove(user_task_id));
                                refetch.call(());
                                break;
                            }
                        }
                        gloo_timers::future::sleep(std::time::Duration::from_millis(1000)).await;
                    }
                }
                Err(err) => {
                    toast_api.error(
                        format!("Failed to check {} for consistency!", fs_copy.pretty_name(),),
                        ToastOptions::default().description(err).permanent(true),
                    );
                    button_state.set(ButtonState::Failed);
                    active_tasks.with_mut(|s| s.remove(user_task_id));
                }
            }
        });
    };

    rsx! {
        AlertDialog { open: open_signal(), on_open_change: move |v| open_signal.set(v),
            AlertDialogTitle { "Do you want to check {fs.pretty_name()} for consistency?" }
            AlertDialogDescription {
                "Checking a drive for consistency scans for filesystem errors. It may take several minutes to complete."
            }
            AlertDialogActions {
                AlertDialogCancel { "Cancel" }
                Button { size: ButtonSize::Lg, onclick, "Start check" }
            }
        }
        Button {
            disabled: drive.hint_system || !fs.can_check || button_state.read().is_in_progress(),
            title: "Check filesystem for consistency",
            variant: ButtonVariant::Ghost,
            size: crate::components::button::ButtonSize::Sm,
            class: "button w-full",
            onclick: move |_| {
                open_signal.set(true);
            },
            match *button_state.read() {
                ButtonState::Default | ButtonState::Failed => rsx! { "Check" },
                ButtonState::InProgress => rsx! { "Checking..." },
            }
        }
    }
}

use crate::prelude::*;

use std::rc::Rc;
use api::{
    drives::{
        Drive, WithPrettyName
    }, error::MessageRes,
};

async fn attempt_poweroff(drive: &Drive, password: Option<String>) -> Result<MessageRes, String> {
    crate::request::post::<MessageRes>(
        "/private/drives/poweroff",
        api::drives::DriveActionReq {
            drive: api::drives::DriveSignature {
                time_detected: drive.time_detected,
                device_node: drive.path.clone(),
                serial: drive.serial.clone(),
                udisks_id: drive.udisks_id.clone(),
            },
            password,
        },
    )
    .await
}

#[component]
pub fn PowerOffButton(
    needs_password: Signal<Option<bool>>,
    refetch: EventHandler,
    drive: Rc<Drive>
) -> Element {
    let mut icon_state = use_signal::<ButtonState>(ButtonState::default);
    let mut dialog_open = use_signal(|| false);

    let drive_for_closure = drive.clone();

    rsx! {
        AlertDialog { open: dialog_open(), on_open_change: move |v| dialog_open.set(v),
            AlertDialogTitle { "Do you want to remove {drive.pretty_name()}?" }
            AlertDialogDescription {
                p {
                    "Are you sure, that you want to remove this drive?"
                    br {}
                    "Click here for "
                    a {
                        href: "https://storaged.org/udisks/docs/gdbus-org.freedesktop.UDisks2.Drive.html#gdbus-method-org-freedesktop-UDisks2-Drive.PowerOff",
                        target: "_blank",
                        class: "underline text-blue-500",
                        "further technical information"
                    }
                    " on safe removal."
                }
            }
            AlertDialogActions {
                AlertDialogCancel { "Cancel" }
                AuthButton {
                    needs_password,
                    variant: ButtonVariant::Destructive,
                    size: ButtonSize::Lg,
                    onclick: move |password| {
                        dialog_open.set(false);
                        let drive_copy = drive_for_closure.clone();
                        icon_state.set(ButtonState::InProgress);
                        spawn(async move {
                            let toast_api = consume_toast();
                            let req = attempt_poweroff(&drive_copy, password).await;
                            refetch.call(());
                            match req {
                                Ok(_) => {
                                    toast_api
                                        .success(
                                            format!(
                                                "{} can be safely removed.",
                                                drive_copy.pretty_name(),
                                            ),
                                            ToastOptions::default(),
                                        );
                                }
                                Err(err) => {
                                    toast_api
                                        .error(
                                            format!("Failed to remove {}.", drive_copy.pretty_name()),
                                            ToastOptions::new().description(err).permanent(true),
                                        );
                                }
                            }
                            icon_state.set(ButtonState::Default);
                        });
                    },
                    "Confirm"
                }
            }
        }
        button {
            class: "p-2 border-r border-neutral-300 dark:border-neutral-800 cursor-pointer disabled:cursor-not-allowed disabled:opacity-50 transition-all duration-200",
            disabled: drive.hint_system || !drive.can_power_off || icon_state.read().is_in_progress(),
            title: "Power off the drive and attempt to remove safely.",
            onclick: move |_| {
                dialog_open.set(true);
            },
            StatefulIcon { icon: bs_icons::BsPower, button_state: icon_state }
        }
    }
}


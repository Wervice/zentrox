use crate::prelude::*;
use std::rc::Rc;

use api::{
    drives::{
        Drive, WithPrettyName
    }, error::MessageRes
};

async fn attempt_eject(drive: &Drive, password: Option<String>) -> Result<MessageRes, String> {
    crate::request::post::<MessageRes>(
        "/private/drives/eject",
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
pub fn EjectButton(
    needs_password: Signal<Option<bool>>,
    refetch: EventHandler,
    drive: Rc<Drive>
) -> Element {
    let mut icon_state = use_signal::<ButtonState>(ButtonState::default);
    let mut dialog_open = use_signal(|| false);

    let drive_for_closure = drive.clone();

    rsx! {
        AlertDialog { open: dialog_open(), on_open_change: move |v| dialog_open.set(v),
            AlertDialogTitle { "Do you want to eject {drive.pretty_name()}?" }
            AlertDialogDescription {
                p {
                    "Are you sure, that you want to eject this drive? Depending on the hardware this may cause physical action, for example a disk drive opening."
                    br {}
                    "Click here for "
                    a {
                        href: "https://storaged.org/udisks/docs/gdbus-org.freedesktop.UDisks2.Drive.html#gdbus-method-org-freedesktop-UDisks2-Drive.Eject",
                        target: "_blank",
                        class: "underline text-blue-500",
                        "further technical information"
                    }
                    " on ejection."
                }
            }
            AlertDialogActions {
                AlertDialogCancel { "Cancel" }
                AuthButton {
                    variant: ButtonVariant::Destructive,
                    size: ButtonSize::Lg,
                    needs_password,
                    onclick: move |password| {
                        dialog_open.set(false);
                        let drive_copy = drive_for_closure.clone();
                        icon_state.set(ButtonState::InProgress);
                        spawn(async move {
                            let toast_api = consume_toast();
                            let req = attempt_eject(&drive_copy, password).await;
                            refetch.call(());
                            match req {
                                Ok(_) => {
                                    toast_api
                                        .success(
                                            format!("{} has been ejected.", drive_copy.pretty_name()),
                                            ToastOptions::default(),
                                        );
                                }
                                Err(err) => {
                                    toast_api
                                        .error(
                                            format!("Failed to eject {}.", drive_copy.pretty_name()),
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
            disabled: drive.hint_system || !drive.ejectable || icon_state.read().is_in_progress(),
            title: "Eject the drive.",
            onclick: move |_| {
                dialog_open.set(true);
            },
            StatefulIcon { icon: bs_icons::BsEject, button_state: icon_state }
        }
    }
}

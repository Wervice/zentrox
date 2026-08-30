/*
*	Copyright github.com/DioxusLabs
*	SPDX-License-Identifier: Apache-2.0 OR MIT
*
*	Modified by Constantin Volke
*	SPDX-License-Identifier: Apache-2.0
*	See further details in the `legal` directory under `dioxus-components.txt`.
*/

use dioxus::prelude::*;
use dioxus_primitives::alert_dialog::{
    self, AlertDialogActionProps, AlertDialogActionsProps, AlertDialogCancelProps,
    AlertDialogDescriptionProps, AlertDialogRootProps, AlertDialogTitleProps,
};

#[css_module("/src/components/alert_dialog/style.css")]
struct Styles;

#[component]
pub fn AlertDialog(props: AlertDialogRootProps) -> Element {
    rsx! {
        alert_dialog::AlertDialogRoot {
            class: Styles::alert_dialog_backdrop,
            id: props.id,
            default_open: props.default_open,
            open: props.open,
            on_open_change: props.on_open_change,
            attributes: props.attributes,
            alert_dialog::AlertDialogContent { class: Styles::alert_dialog.to_string(), {props.children} }
        }
    }
}

#[component]
pub fn AlertDialogTitle(props: AlertDialogTitleProps) -> Element {
    rsx! {
        alert_dialog::AlertDialogTitle { class: Styles::alert_dialog_title, attributes: props.attributes, {props.children} }
    }
}

#[component]
pub fn AlertDialogDescription(props: AlertDialogDescriptionProps) -> Element {
    rsx! {
        alert_dialog::AlertDialogDescription {
            class: Styles::alert_dialog_description,
            attributes: props.attributes,
            {props.children}
        }
    }
}

#[component]
pub fn AlertDialogActions(props: AlertDialogActionsProps) -> Element {
    rsx! {
        alert_dialog::AlertDialogActions { class: Styles::alert_dialog_actions, attributes: props.attributes, {props.children} }
    }
}

#[component]
pub fn AlertDialogCancel(props: AlertDialogCancelProps) -> Element {
    rsx! {
        alert_dialog::AlertDialogCancel {
            on_click: props.on_click,
            class: Styles::alert_dialog_cancel,
            attributes: props.attributes,
            {props.children}
        }
    }
}

#[component]
pub fn AlertDialogAction(props: AlertDialogActionProps) -> Element {
    rsx! {
        alert_dialog::AlertDialogAction {
            class: Styles::alert_dialog_action,
            on_click: props.on_click,
            attributes: props.attributes,
            {props.children}
        }
    }
}

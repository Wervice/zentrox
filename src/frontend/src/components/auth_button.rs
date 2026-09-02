use dioxus::{html::Key::Enter, prelude::*};
use dioxus_primitives::{toast::{ToastOptions, consume_toast}};

use crate::components::{button::*, alert_dialog::*, input::*};

// TODO Make it more customizable

#[component]
pub fn AuthButton(
    children: Element,
    onclick: Callback<Option<String>>,
    needs_password: Signal<Option<bool>>,
    disabled: Option<bool>,
    title: Option<String>,
    class: Option<String>,
    variant: Option<ButtonVariant>,
    size: Option<ButtonSize>,
) -> Element {
    let mut dialog_open = use_signal(|| false);
    let mut password: Signal<Option<String>> = use_signal(|| None);

    let toast_api = consume_toast();

    let mut require_password = move || {
        dialog_open.set(true);
    };

    let mut submit = move || {
        match &*password.read() {
            Some(password) => onclick.call(Some(password.clone())),
            None => {
                toast_api.warning(
                    "A password is required.".to_string(),
                    ToastOptions::default(),
                );
                return;
            }
        }
        dialog_open.set(false);
        password.set(None);
    };

    rsx! {
        AlertDialog { open: dialog_open(), on_open_change: move |v| dialog_open.set(v),
            AlertDialogTitle { "A password is required." }
            AlertDialogDescription {
                "For this action to be executed, you need to verify your identity with your password. Please input it below."
            }
            Input {
                placeholder: "Password",
                r#type: "password",
                value: password,
                oninput: move |evt: Event<FormData>| {
                    password.set(Some(evt.value()));
                },
                onkeypress: move |evt: Event<KeyboardData>| {
                    if evt.key() == Enter {
                        submit();
                    }
                },
            }
            AlertDialogActions {
                AlertDialogCancel { "Cancel" }
                Button {
                    variant: crate::components::button::ButtonVariant::Primary,
                    onclick: move |_| submit(),
                    "Confirm"
                }
            }
        }
        Button {
            onclick: move |_evt| {
                if (*needs_password.read()).unwrap() {
                    require_password();
                } else {
                    onclick.call(None);
                }
            },
            disabled: disabled.unwrap_or(false) || needs_password().is_none(),
            title,
            class: class.unwrap_or("button".to_string()),
            variant: variant.unwrap_or_default(),
            size: size.unwrap_or_default(),
            {children}
        }
    }
}


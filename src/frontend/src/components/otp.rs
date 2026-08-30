use dioxus::prelude::*;
use web_sys::{HtmlElement, HtmlInputElement, InputEvent, wasm_bindgen::JsCast};

use crate::components::input::Input;

#[derive(Props, Clone, PartialEq)]
pub struct OtpInputProps {
    set_value: Signal<Vec<Option<usize>>>,
    size: usize,
}

#[component]
pub fn OtpInput(mut props: OtpInputProps) -> Element {
    let otp_input_field_class = "input flex w-full text-center";
    let size: usize = props.size;
    let empty_digits = {
        let mut v = vec![];
        (0..size).for_each(|_| {
            v.push(None::<usize>);
        });
        v
    };
    let mut digits = use_signal(|| empty_digits);
    let allowed_digits = ["0", "1", "2", "3", "4", "5", "6", "7", "8", "9"];

    use_effect(move || {
        props.set_value.set(digits.read().to_vec());
    });

    fn go_to_digit(k: usize, max: usize) {
        if k > max - 1 {
            return;
        }
        let ele: HtmlInputElement = web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .get_element_by_id(format!("otp_input_{}", k).as_str())
            .unwrap()
            .dyn_into()
            .unwrap();
        ele.select();
        let _ = ele.focus();
    }

    let mut on_keydown = move |evt: KeyboardEvent, k: usize| {
        let key = evt.key();
        let digit = key.to_string();

        // Tab || Ctrl+V
        if key.legacy_keycode() == 9 || evt.modifiers().ctrl() && digit == "v" {
            return;
        }
        if !allowed_digits.contains(&digit.as_str()) {
            evt.prevent_default();
            if key.legacy_keycode() == 8 {
                // Backspace
                let mut digits_copy = digits.read().clone();
                digits_copy[k] = None;
                digits.set(digits_copy);
                if k > 0 && k != size {
                    go_to_digit(k - 1, size);
                }
            }
        } else {
            evt.prevent_default();
            let mut digits_copy = digits.read().clone();
            digits_copy[k] = digit.parse::<usize>().ok();
            digits.set(digits_copy);
            go_to_digit(k + 1, size);
        }
    };

    rsx! {
        div { class: "block overflow-none flex gap-2 whitespace-nowrap",
            {
                (0..size)
                    .map(|k| {
                        rsx! {
                            Input {
                                r#type: "text",
                                id: "otp_input_{k}",
                                onkeydown: move |evt: KeyboardEvent| {
                                    on_keydown(evt, k);
                                },
                                oninput: move |evt: FormEvent| {
                                    let value = evt.value();
                                    if value.len() > 1 {
                                        let mut parsed_digits: Vec<Option<usize>> = value
                                            .chars()
                                            .map(|e| e.to_string().parse::<usize>().ok())
                                            .collect();
                                        parsed_digits.shrink_to(size);
                                        while parsed_digits.len() != size {
                                            parsed_digits.push(None);
                                        }
                                        digits.set(parsed_digits);
                                    }
                                },
                                value: digits.read()[k],
                                class: otp_input_field_class,
                                display: "flex",
                                autocomplete: "off",
                            }
                        }
                    })
            }
        }
    }
}

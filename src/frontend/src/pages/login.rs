use api::login::LoginReq;
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, consume_toast};
use web_sys::HtmlInputElement;
use web_sys::wasm_bindgen::JsCast;

use crate::HTTP_PREFIX;

use crate::components::button::Button;
use crate::components::card::*;
use crate::components::input::Input;
use crate::components::label::Label;
use crate::components::otp::OtpInput;
use crate::components::spinner::Spinner;

use dioxus::prelude::GlobalAttributesExtension;

const FIELD: Asset = asset!("/assets/field.jpg", ImageAssetOptions::new().with_avif());

#[derive(PartialEq, Eq)]
enum LoginStatus {
    Default,
    Pending,
}

#[component]
pub fn Login(logged_in: Signal<Option<bool>>, login_trigger: Signal<bool>) -> Element {
    let mut username = use_signal(|| None::<String>);
    let mut password = use_signal(|| None::<String>);
    let mut needs_otp = use_signal(|| false);
    let mut login_status = use_signal(|| LoginStatus::Default);
    let otp = use_signal(Vec::<Option<usize>>::new);
    let toast_api = consume_toast();

    use_effect(move || {
        let ele: HtmlInputElement = web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .get_element_by_id("username")
            .unwrap()
            .dyn_into()
            .unwrap();
        ele.focus();
    });

    let submit_login = move || async move {
        let username_derived = username.read().clone().unwrap_or("".to_string());
        let password_derived = password.read().clone().unwrap_or("".to_string());
        login_status.set(LoginStatus::Pending);
        let req =
            gloo_net::http::Request::post(format!("{HTTP_PREFIX}/public/auth/login").as_str())
                .credentials(web_sys::RequestCredentials::Include)
                .json(&LoginReq {
                    username: username_derived,
                    password: password_derived,
                    otp: if *needs_otp.read() {
                        Some(
                            (*otp.read())
                                .iter()
                                .map(|e| match e {
                                    Some(v) => format!("{v}"),
                                    None => "".to_string(),
                                })
                                .collect::<String>(),
                        )
                    } else {
                        None
                    },
                })
                .unwrap()
                .send()
                .await;

        if let Ok(res) = req {
            let status = res.status();
            if status == 200 {
                logged_in.set(Some(true));
                login_trigger.set(true);
                toast_api.success(
                    "Login successfull".to_string(),
                    ToastOptions::new()
                        .duration(std::time::Duration::from_secs(5))
                        .description("You will be redirected shortly."),
                );
                login_status.set(LoginStatus::Default);
            } else if status == 403 {
                toast_api.error(
                    "Wrong credentials provided".to_string(),
                    ToastOptions::new()
                        .duration(std::time::Duration::from_secs(10))
                        .description("Check your username and password."),
                );
                login_status.set(LoginStatus::Default);
            } else if status == 422 {
                toast_api.info(
                    "OTP code required".to_string(),
                    ToastOptions::new()
                        .duration(std::time::Duration::from_secs(10))
                        .description("Please provide your OTP code."),
                );
                login_status.set(LoginStatus::Default);
                needs_otp.set(true);
            } else if status == 429 {
                toast_api.warning("Too many requests".to_string(), ToastOptions::new()
                    .duration(std::time::Duration::from_secs(20))
                    .description("You have been temporarily blocked from log-in in order to prevent brute-force attacks."));
                login_status.set(LoginStatus::Default);
            }
        }
    };

    rsx! {
        div {
            class: "bg-cover bg-bottom bg-white dark:bg-black",
            background_image: "url('{FIELD}')",
            div { class: "block w-screen h-screen flex justify-center items-center bg-black/75",
                div { class: "absolute bottom-0 right-0 opacity-75 text-sm p-1 text-white",
                    a {
                        class: "underline",
                        href: "https://unsplash.com/photos/photo-of-green-grass-field-at-sunrise-4miBe6zg5r0",
                        "Image"
                    }
                    " by "
                    a {
                        class: "underline",
                        href: "https://unsplash.com/@aleskrivec",
                        "Ales Krivec"
                    }
                    " on "
                    a { class: "underline", href: "https://unsplash.com", "Unsplash" }
                }
                div { class: "w-96 inline-block",
                    Card {
                        class: "card duration-500 transition-[height] h-min",
                        gap: "1rem",
                        CardHeader {
                            CardTitle { "Login to your account" }
                            CardDescription { "Enter your credentials to get access to your interface." }
                        }
                        CardContent {
                            div { class: "grid gap-2",
                                div { class: "grid gap-2",
                                    Label { r#for: "username", "Username" }
                                    Input {
                                        id: "username",
                                        autocomplete: "off",
                                        value: username,
                                        oninput: move |evt: FormEvent| {
                                            username.set(Some(evt.value()));
                                        },
                                    }
                                }
                                div { class: "grid gap-2",
                                    Label { r#for: "password", "Password" }
                                    Input {
                                        r#type: "password",
                                        id: "password",
                                        value: password,
                                        oninput: move |evt: FormEvent| {
                                            password.set(Some(evt.value()));
                                        },
                                        onkeydown: move |evt: KeyboardEvent| async move {
                                            if evt.key().legacy_keycode() == 13 {
                                                let _ = submit_login().await;
                                            }
                                        },
                                    }
                                }
                                if *needs_otp.read() {
                                    div { class: "grid gap-2",
                                        Label { r#for: "otp", "OTP token" }
                                        OtpInput { set_value: otp, size: 8 }
                                    }
                                }
                            }
                        }
                        CardFooter {
                            span { class: "flex items-center justify-end grow",
                                Button {
                                    class: "button transition-all w-32 duration-200",
                                    onclick: move |_| { submit_login() },
                                    disabled: *login_status.read() == LoginStatus::Pending,
                                    {
                                        match *login_status.read() {
                                            LoginStatus::Pending => rsx! {
                                                Spinner {}
                                            },
                                            _ => rsx! { "Login" },
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

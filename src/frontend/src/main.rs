use crate::components::toast::ToastProvider;
use api::account::AccountDetailsRes;
use dioxus::prelude::*;
use gloo_net::http::Request;

use crate::pages::{admin_panel::AdminPanel, login::Login};
mod components;
mod pages;
mod request;
mod states;
mod prelude;
mod onunload;

const FAVICON: Asset = asset!("/assets/favicon.png");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");
const COMPONENTS_CSS: Asset = asset!("/assets/dx-components-theme.css");
const DEFAULT_CSS: Asset = asset!(
    "/assets/default.css",
    CssAssetOptions::new().with_preload(true)
);
const PIECHART_CSS: Asset = asset!("/assets/piechart.css");

#[cfg(debug_assertions)]
pub const HTTP_PREFIX: &str = "https://localhost:8080/api";

#[cfg(not(debug_assertions))]
pub const HTTP_PREFIX: &str = "/api";

fn main() {
    dioxus::launch(App);
}

#[derive(Clone, Copy)]
pub struct LoggedInCtx(Signal<Option<bool>>);

#[derive(Clone, Copy)]
pub struct LoadingCtx(Signal<bool>);

#[derive(Clone, Copy)]
pub struct LoginTriggerCtx(Signal<bool>);

#[component]
fn App() -> Element {
    let mut logged_in = use_signal::<Option<bool>>(|| None);
    let mut user = use_signal::<Option<AccountDetailsRes>>(|| None);
    let mut loading = use_signal(|| true);
    let login_trigger = use_signal(|| false);

    use_context_provider(|| user);
    use_context_provider(|| LoggedInCtx(logged_in));
    use_context_provider(|| LoadingCtx(loading));
    use_context_provider(|| LoginTriggerCtx(login_trigger));
    use_resource(move || async move {
        let _ = *login_trigger.read();

        let req = Request::get(format!("{HTTP_PREFIX}/private/account/details").as_str())
            .credentials(web_sys::RequestCredentials::Include)
            .send()
            .await;

        if let Ok(res) = req {
            if res.status() == 200 {
                user.set(res.json::<api::account::AccountDetailsRes>().await.ok());
                logged_in.set(Some(true));
            } else {
                logged_in.set(Some(false));
            }
        }
        loading.set(false);
    });

    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }
        document::Link { rel: "stylesheet", href: COMPONENTS_CSS }
        document::Link { rel: "stylesheet", href: DEFAULT_CSS }
        document::Link { rel: "stylesheet", href: PIECHART_CSS }

            ToastProvider {
                if !*loading.read() {
                    if let Some(login_status) = &*logged_in.read() {
                        if *login_status {
                            AdminPanel {}
                        } else {
                            Login { logged_in, login_trigger }
                        }
                    }
                }
            }
        }
    }

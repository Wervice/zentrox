/*
*	Copyright github.com/DioxusLabs
*	SPDX-License-Identifier: Apache-2.0 OR MIT
*
*	Modified by Constantin Volke
*	SPDX-License-Identifier: Apache-2.0
*	See further details in the `legal` directory under `dioxus-components.txt`.
*/

use dioxus::prelude::*;
use dioxus_primitives::progress::{self, ProgressIndicatorProps, ProgressProps};

#[component]
pub fn Progress(props: ProgressProps) -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./style.css") }
        progress::Progress {
            class: "progress",
            value: props.value,
            max: props.max,
            attributes: props.attributes,
            {props.children}
        }
    }
}

#[component]
pub fn ProgressIndicator(props: ProgressIndicatorProps) -> Element {
    rsx! {
        progress::ProgressIndicator { class: "progress-indicator", attributes: props.attributes, {props.children} }
    }
}

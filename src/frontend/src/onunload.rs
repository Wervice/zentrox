use crate::prelude::*;

use web_sys::{Event, wasm_bindgen::{JsCast, closure::Closure}, window};

fn cb() -> Closure<dyn FnMut(Event)> {
    Closure::wrap(Box::new(|evt: Event| {
        evt.prevent_default();
    }) as Box<dyn FnMut(_)>)
}

pub fn warn() -> Closure<dyn FnMut(Event)> {
    let cb = cb();
    let _ = window().unwrap().add_event_listener_with_callback("beforeunload", &cb.as_ref().unchecked_ref());
    cb
}

pub fn unwarn(cb: Closure<dyn FnMut(Event)>) {
    tracing::warn!("Unwarned");
    window().unwrap().remove_event_listener_with_callback("beforeunload", &cb.as_ref().unchecked_ref());
    cb.forget();
}

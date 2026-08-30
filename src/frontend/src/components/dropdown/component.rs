use dioxus::prelude::*;

#[derive(Clone)]
struct DropdownOpen(Signal<bool>);

#[derive(Clone)]
struct DropdownActivated(Signal<bool>);

#[component]
pub fn Dropdown(children: Element) -> Element {
    let open_signal = use_signal(|| false);
    let activated_signal = use_signal(|| false);

    use_context_provider(|| DropdownOpen(open_signal));
    use_context_provider(|| DropdownActivated(activated_signal));

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./style.css") }
        document::Link { rel: "stylesheet", href: asset!("../button/style.css") }
        span { class: "relative flex items-center justify-center", {children} }
    }
}

#[component]
pub fn DropdownTrigger(class: Option<String>, children: Element) -> Element {
    let mut open_signal = use_context::<DropdownOpen>().0;
    let mut activated_signal = use_context::<DropdownActivated>().0;

    rsx! {
        button {
            class: class.unwrap_or("button".to_string()),
            onclick: move |_| {
                open_signal.toggle();
                activated_signal.set(true);
            },
            {children}
        }
    }
}

#[component]
pub fn DropdownContents(class: Option<String>, children: Element) -> Element {
    let mut open_sig = use_context::<DropdownOpen>().0;
    let mut activated_sig = use_context::<DropdownActivated>().0;

    rsx! {
        span {
            class: "fixed top-0 left-0 w-screen h-screen z-[500]",
            hidden: !open_sig(),
            onclick: move |_| async move {
                open_sig.set(false);
                gloo_timers::future::sleep(std::time::Duration::from_millis(150)).await;
                activated_sig.set(false);
            },
        }
        span {
            class: "absolute p-2 w-48 rounded z-[1000] bg-white dark:bg-neutral-900 border border-neutral-300 dark:border-neutral-800 shadow-md top-[125%] gap-1 flex-col dropdown-contents",
            "data-state": if open_sig() { "open" } else { "close" },
            display: if activated_sig() { "flex" } else { "none" },
            {children}
        }
    }
}

use crate::{onunload, prelude::*};

use std::time::Duration;

use api::{
    EmptyRes, drives::{
        BenchmarkHistory, BenchmarkReq, BenchmarkRes, BenchmarkRetrievalReq, BenchmarkUpdate, Drive, DriveActionReq, DriveSignature, HistoricalBenchmarkRes, WithPrettyName
    }, jobs::JobRes, units::Bytes
};
use crate::components::chart::*;
use chrono::TimeZone;
use plotters::style::RGBColor;

use crate::{
    request::{self, Job},
};

#[derive(Eq, PartialEq)]
enum Screens {
    ModeSelection,
    History,
    Configuration,
    Progress,
    Results
}

async fn attempt_benchmark(req: BenchmarkReq) -> Result<JobRes, String> {
    request::post::<JobRes>(
        "/private/drives/benchmark",
        req
    )
    .await
}

async fn attempt_store_benchmark(req: &BenchmarkRes) -> Result<EmptyRes, String> {
    request::post::<EmptyRes>(
        "/private/drives/store_benchmark",
        req
    )
    .await
}

#[component]
fn BenchmarkChart(write: Option<Vec<Duration>>, read: Vec<Duration>, sample_size: usize) -> Element {
    fn formatter(f: &f64) -> String {
        format!("{}MiB/s", short_decimals(f))
    }

    rsx! {
        Chart { formatter,
            Line {
                data: read
                                                    .iter()
                    .map(|sample| (sample_size as f64 / sample.as_secs_f64()) / 1024_f64.powf(2.0))
                    .collect(),
                label: "Read",
                color: RGBColor(251, 44, 54),
            }
            if let Some(inner_write_vec) = &write {
                Line {
                    data: inner_write_vec
                        .iter()
                        .map(|sample| (sample_size as f64 / sample.as_secs_f64()) / 1024_f64.powf(2.0))
                        .collect(),
                    label: "Write",
                    color: RGBColor(43, 127, 255),
                }
            }
        }
    }
}

#[component]
fn ConfigurationForm(drive: Rc<Drive>, on_start: Callback<BenchmarkReq, ()>, on_cancel: Callback<(), ()>, needs_password: Signal<Option<bool>>) -> Element {
    let mut do_write_benchmark = use_signal(|| false);
    let mut do_random_benchmark = use_signal(|| false);
    let mut sample_size = use_signal(|| 4 * 1024_usize * 1024_usize); // In bytes
    let mut iterations = use_signal(|| 64_usize);

    let drive_for_closure = drive.clone();

    rsx! {
        span { class: "text-black dark:text-white flex gap-2 flex-col",
            p { "Benchmarking measures the data throughput of the device." }
            span {
                span { class: "flex gap-1 items-center",
                    Checkbox {
                        value: do_random_benchmark(),
                        on_value_change: move |v| do_random_benchmark.set(v),
                        id: "checkbox-do-random",
                    }
                    Label { r#for: "checkbox-do-random", "Do random seeks" }
                }
                small { class: "block",
                    "Random seeks request data on random positions on the storage device.
                    Depending on the devices technology this may have different effects.
                    Sequential seeks are used by default."
                }
            }
            span {
                span { class: "flex gap-1 items-center",
                    Checkbox {
                        disabled: drive.read_only,
                        value: do_write_benchmark() && !drive.read_only,
                        on_value_change: move |v| do_write_benchmark.set(v),
                        id: "checkbox-do-write",
                    }
                    Label { r#for: "checkbox-do-write", "Perform write benchmark" }
                }

                span {
                    class: "text-red-500 text-sm",
                    hidden: !do_write_benchmark(),
                    details { open: "true",
                        summary { class: "font-bold cursor-pointer select-none",
                            "Disclaimer for write benchmarks"
                        }
                        small { class: "block",
                            "To perform a write benchmark, Zentrox needs exclusive access to the device."
                            br {}
                            br {}
                            "A write benchmark involves reading data from the device and writing it back to the device.
                            This process carries a risk of data loss, corruption, or physical damage to your device.
                            You are solely responsible for backing up all relevant data before proceeding.
                            The developers and contributors of this software disclaim all liability for any data loss, corruption or physical damage to your device,
                            to the fullest extent permitted by applicable law.
                            By initiating this benchmark, you acknowledge these risks, confirm you have created a backup, and accept full responsibility for any consequences."
                        }
                    }
                }
            }
            span { class: "flex gap-2",
                span { class: "grow flex-1 flex-col",
                    Label { r#for: "sample-size-input", "Sampel size (KiB)" }
                    ConstrainedInput::<usize> {
                        r#type: InputType::Number {
                            min: Some(1),
                            max: Some(1024 * 1024),
                        },
                        id: "sample-size-input",
                        value: sample_size() / 1024,
                        onchange: move |v: usize| { sample_size.set(v * 1024) },
                    }
                }

                span { class: "grow flex-1 flex-col",
                    Label { r#for: "iterations-input", "Iterations" }
                    ConstrainedInput::<usize> {
                        r#type: InputType::Number {
                            min: Some(2),
                            max: Some(1024),
                        },
                        id: "iterations-input",
                        value: iterations(),
                        onchange: move |v: usize| { iterations.set(v) },
                    }
                }
            }
            span { class: "flex",
                span { class: "flex-1 grow",
                    Button {
                        variant: ButtonVariant::Outline,
                        onclick: move |_| { on_cancel.call(()) },
                        span { class: "flex items-center gap-1",
                            Icon { icon: bs_icons::BsArrowLeft }
                            "Back"
                        }
                    }
                }
                span { class: "flex flex-1 grow justify-end",
                    AuthButton {
                        needs_password,
                        onclick: move |password: Option<String>| {
                            let drive_copy = drive_for_closure.clone();
                            let req = BenchmarkReq {
                                sample_size: *sample_size.read(),
                                iterations: *iterations.read(),
                                write: *do_write_benchmark.read() && !drive.read_only,
                                random: *do_random_benchmark.read(),
                                drive: DriveSignature {
                                    time_detected: drive_copy.time_detected,
                                    device_node: drive_copy.path.clone(),
                                    serial: drive_copy.serial.clone(),
                                    udisks_id: drive_copy.udisks_id.clone(),
                                },
                                password: password.expect("A password should have been provided."),
                            };
                            on_start.call(req);
                        },
                        "Start benchmark"
                    }
                }
            }
        }
    }
}

#[component]
pub fn BenchmarkHistoryDisplay(entry: api::drives::BenchmarkHistoryEntry) -> Element {
    let mut opened = use_signal(|| false);

    let mut benchmark: Signal<Option<HistoricalBenchmarkRes>> = use_signal(|| None);

    async fn attempt_get_benchmark(uuid: String) -> Result<HistoricalBenchmarkRes, String> {
        request::post::<HistoricalBenchmarkRes>("/private/drives/past_benchmark", BenchmarkRetrievalReq { id: uuid }).await
    }

    rsx! {
        span {
            class: "border-b border-neutral-200 dark:border-neutral-700 text-black dark:text-white hover:bg-neutral-100 dark:hover:bg-neutral-800 cursor-pointer select-none block p-2",
            onclick: move |_| {
                if !opened() && benchmark().is_none() {
                    let toast_api = consume_toast();
                    let uuid = entry.uuid.clone();
                    spawn(async move {
                        let res = attempt_get_benchmark(uuid).await;
                        match res {
                            Ok(inner) => benchmark.set(Some(inner)),
                            Err(err) => {
                                toast_api
                                    .error(
                                        "Failed to get historical benchmark.".to_string(),
                                        ToastOptions::default()
                                            .permanent(true)
                                            .description(err.to_string()),
                                    );
                            }
                        }
                    });
                }
                opened.toggle()
            },
            span { class: "flex items-center gap-1",
                if opened() {
                    Icon { icon: bs_icons::BsChevronDown }
                } else {
                    Icon { icon: bs_icons::BsChevronRight }
                }
                {chrono::Local.timestamp_millis_opt(entry.time).unwrap().to_string()}
            }
        }
        if opened() {
            span { class: "block text-black dark:text-white p-2 bg-neutral-200 dark:bg-neutral-900",
                if benchmark().is_none() {
                    span { class: "block items-center justify-center text-center", Spinner {} }
                }

                if let Some(inner_results) = benchmark() {
                    "Sample size: {Bytes(inner_results.sample_size as u64)}"
                    br {}
                    "Random seeks: "
                    if inner_results.random {
                        "yes"
                    } else {
                        "no"
                    }

                    span { class: "flex flex-col max-w-full mt-2",
                        BenchmarkChart {
                            sample_size: entry.sample_size,
                            read: inner_results.read.clone(),
                            write: inner_results.write.clone(),
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn Results(results: BenchmarkRes, on_quit: Callback<(), ()>) -> Element {
    let mut stored = use_signal(|| false);

    let toast_api = consume_toast();

    let read = results.read.clone();
    let write = results.write.clone();
    let sample_size = results.sample_size;

    let results_for_closure = Signal::new(results);
    let store_results = move |_| {
        spawn(async move {
            let req = attempt_store_benchmark(&results_for_closure()).await;
            match req {
                Ok(_) => {
                    toast_api
                        .success(
                            "Stored successfully.".to_string(),
                            ToastOptions::default()
                                .permanent(true)
                                .description(
                                    "The benchmark results have been stored successfully.
                                    You may now exit the results display if you wish."
                                        .to_string(),
                                ),
                        );
                        stored.set(true);
                }
                Err(err) => {
                    toast_api
                        .error(
                            "Failed to store results.".to_string(),
                            ToastOptions::default().permanent(true).description(err),
                        );
                }
            }
        });
    };

    rsx! {
        span { class: "text-black dark:text-white flex flex-col gap-2",
            span { class: "block",
                {
                    let avg = read.iter().fold(0.0, |acc, x| { acc + x.as_secs_f64() })
                        / read.iter().len() as f64;
                    let rate = sample_size as f64 / avg;
                    let rate_bytes = Bytes(rate as u64);
                    format!("Read avg. {rate_bytes}/s")
                }
            }

            if let Some(inner) = write.clone() {
                span { class: "block",
                    {
                        let avg = inner.iter().fold(0.0, |acc, x| { acc + x.as_secs_f64() })
                            / inner.iter().len() as f64;
                        let rate = sample_size as f64 / avg;
                        let rate_bytes = Bytes(rate as u64);
                        format!("Write avg. {rate_bytes}/s")
                    }
                }
            }

            BenchmarkChart { sample_size, read: read.clone(), write: write.clone() }

            span { class: "flex",
                span { class: "grow flex-1",
                    Button {
                        onclick: move |_| {
                            on_quit(());
                        },
                        variant: ButtonVariant::Destructive,
                        Icon { icon: bs_icons::BsArrowLeft }
                        "Discard"
                    }
                }
                span { class: "grow flex-1 flex gap-2 justify-end",
                    Button { disabled: stored(), onclick: store_results,
                        if stored() {
                            span { class: "flex gap-2 items-center",
                                Icon { icon: bs_icons::BsCheckCircle }
                                "Stored"
                            }
                        } else {
                            "Store in history"
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn BenchmarkButton(
    needs_password: Signal<Option<bool>>,
    drive: Rc<Drive>,
    refetch: EventHandler,
) -> Element {
    let toast_api = consume_toast();

    let mut button_state = use_signal(ButtonState::default);
    let mut dialog_open = use_signal(|| false);

    let mut screen = use_signal(|| Screens::ModeSelection);

    let mut sample_size = use_signal(|| 0);
    let mut started_at = use_signal(chrono::Local::now);
    let mut now = use_signal(|| chrono::Local::now().timestamp());
    let mut progress = use_signal(|| 0.0_f64);
    let mut results = use_signal(|| None);
    let mut error = use_signal(|| None);

    let mut stored = use_signal(|| false);

    let mut refresh_history = use_signal(|| false);

    let mut reset = move || {
        screen.set(Screens::ModeSelection);
        started_at.set(chrono::Local::now());
        progress.set(0.0);
        results.set(None);
        error.set(None);
        stored.set(false);
        sample_size.set(0);
    };

    use_resource(move || async move {
        if *screen.read() == Screens::Progress && *dialog_open.read() {
            loop {
                now.set(chrono::Local::now().timestamp());
                gloo_timers::future::sleep(std::time::Duration::from_millis(100)).await;
            }
        }
    });

    use_resource(move || async move {
        if *dialog_open.read() {
            loop {
                refresh_history.toggle();
                gloo_timers::future::sleep(std::time::Duration::from_millis(1000)).await;
            }
        }
    });

    let drive_for_closure = drive.clone();
    
    let drive_for_resource = drive.clone();
    let benchmark_history = use_resource(move || {
        if dialog_open() && *screen.read() == Screens::History { refresh_history.read(); }
        request::post::<BenchmarkHistory>("/private/drives/benchmark_history", DriveActionReq { drive:
            DriveSignature {
                time_detected: drive_for_resource.time_detected,
                device_node: drive_for_resource.path.clone(),
                serial: drive_for_resource.serial.clone(),
                udisks_id: drive_for_resource.udisks_id.clone()
            },
            password: None
        })
    });

    let on_start = move |req: BenchmarkReq| {
        screen.set(Screens::Progress);
        button_state.set(ButtonState::InProgress);
        started_at.set(chrono::Local::now());
        now.set(chrono::Local::now().timestamp());
        sample_size.set(req.sample_size);
        let drive_copy = drive_for_closure.clone();
        let cb = onunload::warn();
        spawn(async move {
            match attempt_benchmark(req).await {
                Ok(job) => {
                    let uuid = job.uuid;
                    loop {
                        match request::get_job_status::<
                            BenchmarkRes,
                            BenchmarkUpdate,
                        >(uuid)
                            .await
                        {
                            Ok(Job::Ongoing) => {}
                            Ok(Job::Ok(res)) => {
                                button_state.set(ButtonState::Default);
                                progress.set(100.0);
                                results.set(Some(res));
                                screen.set(Screens::Results);
                                onunload::unwarn(cb);
                                toast_api
                                    .success(
                                        format!(
                                            "Benchmark for {} completed!",
                                            drive_copy.pretty_name(),
                                        ),
                                        ToastOptions::default().permanent(true),
                                    );
                                break;
                            }
                            Ok(Job::Update(update)) => {
                                progress.set(update.progress);
                            }
                            Err(err) | Ok(Job::Failed(err)) => {
                                toast_api
                                    .error(
                                        format!(
                                            "Failed to benchmark {}!",
                                            drive_copy.pretty_name(),
                                        ),
                                        ToastOptions::default().description(&err).permanent(true),
                                    );
                                button_state.set(ButtonState::Failed);
                                onunload::unwarn(cb);
                                error.set(Some(err));
                                break;
                            }
                        }
                        gloo_timers::future::sleep(
                                std::time::Duration::from_millis(1000),
                            )
                            .await;
                    }
                }
                Err(err) => {
                    toast_api
                        .error(
                            format!("Failed to benchmark {}!", drive_copy.pretty_name()),
                            ToastOptions::default().description(&err).permanent(true),
                        );
                    error.set(Some(err));
                    onunload::unwarn(cb);
                    button_state.set(ButtonState::Failed);
                }
            }
        });
    };


    rsx! {
        AlertDialog {
            open: dialog_open(),
            on_open_change: move |v| {
                dialog_open.set(v);
                spawn(async move {
                    gloo_timers::future::sleep(std::time::Duration::from_millis(200)).await;
                });
            },

            AlertDialogTitle {
                match &*screen.read() {
                    Screens::ModeSelection => rsx! { "Benchmarking" },
                    Screens::History => rsx! { "Benchmark history" },
                    Screens::Configuration => rsx! { "Configure benchmark" },
                    Screens::Progress => rsx! { "Benchmarking..." },
                    Screens::Results => rsx! { "Benchmark results" },
                }
            }
            AlertDialogDescription {
                match &*screen.read() {
                    Screens::ModeSelection => rsx! {
                        span { class: "w-full h-64 items-center justify-center gap-2 flex",
                            Button {
                                variant: ButtonVariant::Outline,
                                onclick: move |_| { screen.set(Screens::History) },
                                class: "w-32 block",
                                "History"
                            }
                            Button {
                                variant: ButtonVariant::Outline,
                                class: "w-32 block",
                                onclick: move |_| { screen.set(Screens::Configuration) },
                                "New benchmark"
                            }
                        }
                        span { class: "flex grow",
                            Button {
                                variant: ButtonVariant::Outline,
                                onclick: move |_| {
                                    reset();
                                    dialog_open.set(false);
                                },
                                span { class: "flex items-center gap-1", "Close" }
                            }
                        }
                    },
                    Screens::History => rsx! {
                        span { class: "block max-h-72 overflow-scroll mb-2 scrollable",
                            if let Some(resource) = benchmark_history() {
                                match resource {
                                    Ok(inner) => rsx! {
                                        for benchmark in inner.history.clone() {
                                            BenchmarkHistoryDisplay { entry: benchmark }
                                        }
                                        if inner.history.is_empty() {
                                            span { class: "text-xl opacity-75 text-center font-medium text-black dark:text-white select-none block",
                                                "No past benchmarks"
                                            }
                                        }
                                    },
                                    Err(err) => rsx! {
                                        span { class: "text-red-500 font-sm", {err} }
                                    },
                                }
                            }
                        }
                        span { class: "block",
                            Button {
                                variant: ButtonVariant::Outline,
                                onclick: move |_| { screen.set(Screens::ModeSelection) },
                                span { class: "flex items-center gap-1",
                                    Icon { icon: bs_icons::BsArrowLeft }
                                    "Back"
                                }
                            }
                        }
                    },
                    Screens::Configuration => rsx! {
                        ConfigurationForm {
                            on_cancel: move |_| { screen.set(Screens::ModeSelection) },
                            on_start,
                            needs_password,
                            drive: drive.clone(),
                        }
                    },
                    Screens::Progress => rsx! {
                        span { class: "min-h-64 flex gap-2 text-black dark:text-white items-center justify-center",
                            span { class: "flex flex-col items-center gap-2",
                                if let Some(err_desc) = error() {
                                    span { class: "block text-red-500 max-w-[75%] text-center", {err_desc} }
                                    Button {
                                        variant: ButtonVariant::Outline,
                                        onclick: move |_| {
                                            reset();
                                            screen.set(Screens::Configuration);
                                        },
                                        span { class: "flex items-center gap-1",
                                            Icon { icon: bs_icons::BsArrowLeft }
                                            "Back"
                                        }
                                    }
                                } else {
                                    {format!("Running for {}s.", *now.read() - started_at.read().timestamp())}
                                    br {}
                                    span { class: "flex flex-row gap-2 items-center",
                                        {format!("{:.2}% ", progress())}
                                        Progress { value: progress(), max: 100.0, ProgressIndicator {} }
                                    }
                                }
                            }
                        }
                    },
                    Screens::Results => rsx! {
                        if let Some(inner) = results() {
                            Results { results: inner, on_quit: reset }
                        } else {
                            span { class: "text-red-500 font-medium", "Failed to get results for unknown reason." }
                        }
                    },
                }
            }
        }
        button {
            class: "p-2 border-r border-neutral-300 dark:border-neutral-800 cursor-pointer disabled:cursor-not-allowed disabled:opacity-50",
            disabled: drive.hint_system,
            title: "Measure the drives performance.",
            onclick: move |_| {
                dialog_open.set(true);
            },
            StatefulIcon { icon: bs_icons::BsSpeedometer2, button_state }
        }
    }
}

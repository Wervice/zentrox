use std::ops::Sub;
use std::time::Duration;

use api::docker::{Container, ContainerState};
use api::network::Interface;
use chrono::{DateTime, Local};
use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::bs_icons;
use inflector::Inflector;

use crate::components::bento_box::{
    BentoBoxDouble, BentoBoxSquare, BentoContainer, BentoContent, BentoError, BentoPlaceholder,
    BentoRow, BentoTitle,
};
use crate::components::date::Date;
use crate::components::piechart::PieChart;
use crate::components::viewport::{Viewport, ViewportContents, ViewportTabs};
use crate::pages::admin_panel::Page;
use crate::request;
use api::units::Hertz;

#[component]
fn Contents() -> Element {
    let mut refetch_signal = use_signal(|| false);
    let mut docker_refetch_signal = use_signal(|| false);
    let mut package_statistics_refetch_signal = use_signal(|| false);
    let mut server_name = use_signal(String::new);

    let page_ctx = use_context::<Signal<Page>>();

    let user = use_context::<Signal<Option<api::account::AccountDetailsRes>>>();
    let dashboard_information_req = use_resource(move || async move {
        refetch_signal.read();
        request::get::<api::dashboard::DeviceInformationRes>("/private/dashboard/information").await
    });

    let hostname_req = use_resource(move || async move {
        refetch_signal.read();
        request::get::<api::network::HostnameRes>("/private/network/hostname").await
    });

    let uptime_req = use_resource(move || async move {
        refetch_signal.read();
        request::get::<api::processes::UptimeRes>("/private/processes/uptime").await
    });

    let docker_containers_req = use_resource(move || async move {
        docker_refetch_signal.read();
        request::get::<api::docker::ContainersRes>("/private/docker/containers").await
    });

    let package_statistics_req = use_resource(move || async move {
        package_statistics_refetch_signal.read();
        request::get::<api::packages::StatisticsRes>("/private/packages/statistics").await
    });

    let cpu_req = use_resource(move || async move {
        refetch_signal.read();
        request::get::<api::processes::CpuStatsRes>("/private/processes/cpu").await
    });

    let memory_stats_req = use_resource(move || async move {
        refetch_signal.read();
        request::get::<api::processes::MemoryStatsRes>("/private/processes/memory").await
    });

    let interfaces_req = use_resource(move || async move {
        refetch_signal.read();
        request::get::<api::network::InterfacesRes>("/private/network/interfaces").await
    });

    let thermometers_req = use_resource(move || async move {
        refetch_signal.read();
        match request::get::<api::processes::ThermometersRes>("/private/processes/thermometers")
            .await
        {
            Ok(v) => {
                let mut clone = v.thermometers.clone();
                clone.sort_by(|l, r| r.critical.is_some().cmp(&l.critical.is_some()));
                Ok(clone)
            }
            Err(e) => Err(e),
        }
    });

    let drives_req = use_resource(move || async move {
        refetch_signal.read();
        request::get::<api::drives::DrivesRes>("/private/drives/list").await
    });

    use_future(move || async move {
        // Resources are re-fetched in a certain interval, but only if no other resources are
        // currently not loaded.
        loop {
            if !dashboard_information_req.pending()
                && !hostname_req.pending()
                && !uptime_req.pending()
                && !docker_containers_req.pending()
                && !package_statistics_req.pending()
                && !cpu_req.pending()
                && !memory_stats_req.pending()
                && !interfaces_req.pending()
                && !thermometers_req.pending()
                && !drives_req.pending()
                && *page_ctx.read() == Page::Dashboard
            {
                refetch_signal.toggle();
            }
            gloo_timers::future::sleep(std::time::Duration::from_millis(1500)).await;
        }
    });

    use_future(move || async move {
        loop {
            if !docker_containers_req.pending()
                && !package_statistics_req.pending()
                && *page_ctx.read() == Page::Dashboard
            {
                // Some requests cause heavy loads on the backend, in order to not DoS ourself,
                // these requests are started with a higher delay.
                docker_refetch_signal.toggle();
                package_statistics_refetch_signal.toggle();
            }
            gloo_timers::future::sleep(std::time::Duration::from_millis(10000)).await;
        }
    });

    use_effect(move || {
        if let Some(Ok(system_info)) = &*dashboard_information_req.read() {
            server_name.set(system_info.server_name.clone());
        }
    });

    let username = user.read().clone().unwrap_or_default().username;

    let GeneralInformation = move || {
        rsx! {
            BentoBoxDouble {
                if let Some(Ok(hostname)) = &*hostname_req.read()
                    && let Some(Ok(dashboard_information)) = &*dashboard_information_req.read()
                    && let Some(Ok(uptime)) = &*uptime_req.read()
                {
                    BentoTitle { {hostname.hostname.clone()} }
                    BentoContent {
                        div { class: "grid grid-cols-2 gap-1.5",
                            div {
                                class: "flex gap-1 items-center",
                                title: "Distribution name",
                                Icon { icon: bs_icons::BsDisc, height: 20 }
                                {
                                    dashboard_information
                                        .os_name
                                        .clone()
                                        .map(|v| v.split(' ').take(2).collect::<Vec<&str>>().join(" "))
                                }
                            }
                            div {
                                class: "flex gap-1 items-center",
                                title: "Last boot",
                                Icon { icon: bs_icons::BsMoonStars, height: 20 }
                                Date { duration: chrono::Local::now().sub(Duration::from_secs(uptime.uptime)) }
                            }
                            div {
                                class: "flex gap-1 items-center",
                                title: "Kernel version",
                                Icon { icon: bs_icons::BsTag, height: 20 }
                                {dashboard_information.kernel_version.clone()}
                            }
                            div {
                                class: "flex gap-1 items-center",
                                title: "Remote timezone relative to UTC",
                                Icon { icon: bs_icons::BsClock, height: 20 }
                                "Server timezone "
                                {
                                    let offset = (dashboard_information.utc_offset as f32 / 360.0).round() / 10.0;
                                    let prefix = if offset < 0.0 { "-" } else { "+" };
                                    format!("UTC{}{}", prefix, offset)
                                }
                            }
                            {dashboard_information.relevant_services.iter().take(4).map(|e| rsx! {
                                div {
                                    title: if e.1 { "Daemon \"{e.0}\" is installed and active." } else { "Daemon \"{e.0}\" is installed but inactive." },
                                    class: "flex gap-1 items-center",
                                    if e.1 {
                                        Icon {
                                            icon: bs_icons::BsCheckCircle,
                                            class: "text-green-500",
                                            height: 20,
                                        }
                                    } else {
                                        span {
                                            Icon {
                                                icon: bs_icons::BsXCircle,
                                                class: "text-red-500",
                                                height: 20,
                                            }
                                        }
                                    }
                                    {e.0.clone()}
                                }
                            })}
                        }
                    }
                } else {
                    BentoTitle { "General information" }
                    BentoPlaceholder {}
                }
            }

        }
    };

    let DockerContainers = move || {
        let container_information = move |container: &Container| {
            let mut open = use_signal(|| false);
            let image = container
                .image
                .clone()
                .unwrap_or("Unknown image".to_string());
            let name = container
                .names
                .clone()
                .unwrap_or_default()
                .first()
                .unwrap_or(&"Unknown name".to_string())
                .strip_prefix('/')
                .unwrap_or_default()
                .to_string()
                .clone();
            let since = container.created;
            let state = match &container.state {
                Some(v) => v.to_string().to_title_case(),
                None => "N/A".to_string(),
            };
            let state_color = match container.state.clone() {
                Some(ContainerState::EMPTY) => "neutral",
                Some(ContainerState::CREATED) => "blue",
                Some(ContainerState::RUNNING) => "green",
                Some(ContainerState::PAUSED) => "yellow",
                Some(ContainerState::RESTARTING) => "neutral",
                Some(ContainerState::REMOVING) => "purple",
                Some(ContainerState::EXITED) => "black",
                Some(ContainerState::DEAD) => "red",
                None => "purple",
            };
            let mut id = container.id.clone();
            id.truncate(12);
            rsx! {
                div { class: "p-2 grow rounded dark:bg-white/5 bg-black/5 snap-always snap-center",
                    div {
                        span { class: "text-{state_color}-500 pr-2 mr-2 border-r dark:border-white/20 border-black/20",
                            "{state}"
                        }
                        strong { title: "Container name", "{name}" }
                        span {
                            class: "inline-block float-end",
                            onclick: move |_| { open.toggle() },
                            if *open.read() {
                                Icon { icon: bs_icons::BsChevronUp }
                            } else {
                                Icon { icon: bs_icons::BsChevronDown }
                            }
                        }
                    }
                    if *open.read() {
                        div { class: "grid grid-cols grid-cols-2",
                            "Container ID"
                            code { {id} }
                            "Image name"
                            code { class: "block overflow-x-scroll whitespace-nowrap",
                                "{image}"
                            }
                            "Created at"
                            if let Some(datetime) = since {
                                Date {
                                    duration: DateTime::from_timestamp_secs(datetime)
                                        .unwrap()
                                        .with_timezone(&Local::now().timezone()),
                                }
                            } else {
                                "Unknown time"
                            }
                        }
                    }
                }
            }
        };

        rsx! {
            BentoBoxDouble {
                BentoTitle { "Docker" }
                if let Some(Err(err)) = &*docker_containers_req.read() {
                    BentoError { "{err}" }
                } else {
                    if let Some(Ok(docker_containers)) = &*docker_containers_req.read() {
                        BentoContent {
                            if docker_containers.containers.is_empty() {
                                div { class: "text-center opacity-75", "No active containers" }
                            } else {
                                div { class: "grid gap-2 mb-2 snap-y snap-proximity overflow-y-scroll",
                                    div { class: "block snap-center snap-always",
                                        "Total containers:"
                                        {docker_containers.containers.len().to_string()}
                                    }
                                    {docker_containers.containers.iter().map(container_information)}
                                }
                            }
                        }
                    } else {
                        BentoPlaceholder {}
                    }
                }
            }
        }
    };

    let Packages = move || {
        rsx! {
            BentoBoxSquare {
                BentoTitle { "Packages" }
                match &*package_statistics_req.read() {
                    Some(Ok(package_statistics)) => {
                        rsx! {
                            BentoContent {
                                div { class: "grid gap-1.5",
                                    div {
                                        title: "Package manager backend used by Zentrox",
                                        class: "flex items-center gap-1",
                                        Icon { icon: bs_icons::BsBoxSeam, height: 20 }
                                        if let Some(name) = &package_statistics.package_manager {
                                            "{name}"
                                        } else {
                                            "Unknown package manager"
                                        }
                                    }
                                    div { title: "Available updates", class: "flex items-center gap-1",
                                        if let Some(u) = package_statistics.updates && u > 0 {
                                            if u < 10 {
                                                Icon { icon: bs_icons::BsArrowRepeat, height: 20 }
                                                span { "{u} updates" }
                                            } else {
                                                Icon { icon: bs_icons::BsExclamationCircle, height: 20 }
                                                span { class: "text-orange-500", "{u} updates" }
                                            }
                                        } else {
                                            Icon { icon: bs_icons::BsArrowRepeat, height: 20 }
                                            "No updates"
                                        }
                                    }
                                    if let Some(stamp) = package_statistics.last_database_update {
                                        div {
                                            title: "Last time the local package database was refreshed",
                                            class: "flex items-center gap-1",
                                            Icon { icon: bs_icons::BsClockHistory, height: 20 }
                                            Date {
                                                duration: DateTime::from_timestamp_secs(stamp)
                                                    .unwrap()
                                                    .with_timezone(&Local::now().timezone()),
                                            }
                                        }
                                    }
                                    div { title: "Installed packages", class: "flex items-center gap-1",
                                        Icon { icon: bs_icons::BsCollection, height: 20 }
                                        "{package_statistics.installed} installed"
                                    }
                                }
                            }
                        }
                    }
                    Some(Err(e)) => {
                        rsx! {
                            BentoError { "{e}" }
                        }
                    }
                    None => {
                        rsx! {
                            BentoPlaceholder {}
                        }
                    }
                }
            }
        }
    };

    let Cpu = move || {
        let percentage = use_memo(move || {
            if let Some(Ok(cpu_stats)) = &*cpu_req.read() {
                let mut sum = 0_f32;
                cpu_stats.cpus.iter().for_each(|e| sum += e.usage);
                Some(sum / cpu_stats.cpus.len() as f32 / 100.0)
            } else {
                None
            }
        });

        let frequency = use_memo(move || {
            if let Some(Ok(cpu_stats)) = &*cpu_req.read() {
                let mut sum = 0_f64;
                cpu_stats.cpus.iter().for_each(|e| sum += e.frequency.0);
                Some(sum / cpu_stats.cpus.len() as f64)
            } else {
                None
            }
        });

        let cpu_count = use_memo(move || {
            if let Some(Ok(cpu_stats)) = &*cpu_req.read() {
                Some(cpu_stats.cpus.len())
            } else {
                None
            }
        });

        rsx! {
            BentoBoxSquare {
                BentoTitle { "Processor" }
                if let Some(percentage_value) = *percentage.read()
                    && let Some(frequency_value) = &*frequency.read()
                    && let Some(cpu_count_value) = *cpu_count.read()
                {
                    BentoContent {
                        div { class: "flex flex-col gap-2 align-center justify-center mb-1 items-center",
                            PieChart {
                                percentage: percentage_value.into(),
                                height: 80,
                                width: 80,
                            }
                            div { class: "grid grid-cols-2 text-center text-sm w-full grow",
                                div { title: "Average frequency over all cores",
                                    strong { class: "block", "Clock rate" }
                                    "{Hertz(*frequency_value)}"
                                }
                                div { title: "Number of installed CPUs",
                                    strong { class: "block", "Cores" }
                                    "{cpu_count_value}"
                                }
                            }
                        }
                    }
                } else {
                    match &*cpu_req.read() {
                        Some(Err(e)) => rsx! {
                            BentoError { {e.clone()} }
                        },
                        _ => rsx! {
                            BentoPlaceholder {}
                        },
                    }
                }
            }
        }
    };

    let Memory = move || {
        rsx! {
            BentoBoxSquare {
                BentoTitle { "Memory" }
                match &*memory_stats_req.read() {
                    Some(Ok(memory_stats)) => {
                        rsx! {
                            BentoContent {
                                div {
                                    class: "flex flex-col gap-2 align-center justify-center mb-1 items-center",
                                    title: "Total memory usage",
                                    PieChart {
                                        percentage: (memory_stats.total.0 - memory_stats.free.0) as f64 / memory_stats.total.0 as f64,
                                        height: 80,
                                        width: 80,
                                    }
                                    div { class: "grid grid-cols-2 text-center text-sm w-full grow",
                                        div { title: "",
                                            strong { class: "block", "Installed" }
                                            "{memory_stats.total}"
                                        }
                                        div { title: "",
                                            strong { class: "block", "Swap" }
                                            "{memory_stats.swap_total}"
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Some(Err(e)) => rsx! {
                        BentoError { {e.to_string()} }
                    },
                    None => rsx! {
                        BentoPlaceholder {}
                    },
                }
            }
        }
    };

    let Network = move || {
        let mut show_mac = use_signal(|| false);

        let relevant_interface = use_memo(move || match &*interfaces_req.read() {
            Some(Ok(interfaces)) => {
                let list = interfaces.interfaces.clone();
                let mut relevant_interface: Option<Interface> = None;
                list.iter().for_each(|ele| {
                    if ele.link_type == "loopback" {
                        return;
                    };
                    let ele_sum = ele.delta_up_per_five_s + ele.delta_down_per_five_s;
                    if let Some(ref curr) = relevant_interface {
                        let curr_sum = curr.delta_up_per_five_s + curr.delta_down_per_five_s;
                        if ele_sum > curr_sum {
                            relevant_interface = Some(ele.clone())
                        }
                    } else {
                        relevant_interface = Some(ele.clone())
                    }
                });
                Some(Ok(relevant_interface))
            }
            Some(Err(err)) => Some(Err(err.clone())),
            None => None,
        });

        rsx! {
            BentoBoxSquare {
                BentoTitle { "Network" }
                match &*relevant_interface.read() {
                    Some(Ok(Some(interface))) => {
                        rsx! {
                            BentoContent {
                                div { class: "gap-1 flex items-center",
                                    if interface.name.starts_with("wlan") {
                                        Icon { icon: bs_icons::BsWifi }
                                    } else {
                                        Icon { icon: bs_icons::BsEthernet }
                                    }
                                    "{interface.name}"
                                }
                                div { class: "gap-1 flex items-center",
                                    Icon { icon: bs_icons::BsGeo }
                                    if let Some(ip) = interface.ips.first() {
                                        "{ip}"
                                    } else {
                                        "Unknown IP"
                                    }
                                }
                                div { class: "gap-1 flex items-center mb-2",
                                    Icon { icon: bs_icons::BsHash }
                                    span {
                                        class: "font-mono text-sm select-none",
                                        onclick: move |_| { show_mac.toggle() },
                                        title: "Click to toggle MAC address visibility",
                                        if *show_mac.read() {
                                            {interface.address.clone().unwrap_or("Unknown MAC".to_string())}
                                        } else {
                                            "##:##:##:##:##:##"
                                        }
                                    }
                                }
                                div { class: "grid grid-cols-2 text-center text-sm w-full grow mt-1",
                                    div { title: "Data sent up",
                                        strong { class: "block", "Up" }
                                        "{interface.delta_up_per_five_s}/s"
                                    }
                                    div { title: "Data downloaded",
                                        strong { class: "block", "Down" }
                                        "{interface.delta_down_per_five_s}/s"
                                    }
                                }
                            }
                        }
                    }
                    Some(Err(err)) => {
                        rsx! {
                            BentoError { "{err}" }
                        }
                    }
                    _ => {
                        rsx! {
                            BentoPlaceholder {}
                        }
                    }
                }
            }
        }
    };

    let Thermometers = move || {
        rsx! {
            BentoBoxSquare {
                BentoTitle { "Thermometers" }
                match &*thermometers_req.read() {
                    Some(Ok(thermometers)) => rsx! {
                        BentoContent {
                            div { class: "block mb-2 overflow-y-scroll grid gap-2",
                                {
                                    thermometers
                                        .iter()
                                        .filter(|v| v.reading.is_some())
                                        .map(|v| {
                                            rsx! {
                                                div { class: "p-2 grow rounded dark:bg-white/5 bg-black/5",
                                                    div {
                                                        strong { title: "Thermometer name", "{v.label}" }
                                                    }
                                                    if let Some(limit) = v.critical {
                                                        {
                                                            let ratio = v.reading.unwrap().0 / limit.0;
                                                            let color = if ratio < 0.6 {
                                                                "green"
                                                            } else if ratio < 0.7 {
                                                                "orange"
                                                            } else if ratio < 0.9 {
                                                                "red"
                                                            } else {
                                                                "purple"
                                                            };
                                                            rsx! {
                                                                span { class: "text-{color}-500", "{v.reading.unwrap()}" }
                                                                br {}
                                                                "Critical: {limit}"
                                                            }
                                                        }
                                                    } else {
                                                        div { "{v.reading.unwrap()}" }
                                                    }
                                                }
                                            }
                                        })
                                }
                            }
                        }
                    },
                    Some(Err(e)) => rsx! {
                        BentoError { "{e}" }
                    },
                    None => rsx! {
                        BentoPlaceholder {}
                    },
                }
            }
        }
    };

    let Drives = move || {
        rsx! {
            BentoBoxDouble {
                BentoTitle { "Drives" }
                match &*drives_req.read() {
                    Some(Ok(drives)) => {
                        rsx! {
                            BentoContent {
                                div { class: "grid gap-2 grid-cols-2",
                                    {
                                        drives
                                            .drives
                                            .iter()
                                            .map(|d| {
                                                rsx! {
                                                    div { class: "rounded dark:bg-white/5 bg-black/5 p-2 grid grid-cols-2 items-center gap-2 grid-cols-[min-content_1fr]",
                                                        if d.bus_type == "usb" {
                                                            Icon { icon: bs_icons::BsUsbDriveFill }
                                                        } else {
                                                            if d.rotating {
                                                                Icon { icon: bs_icons::BsDeviceHddFill }
                                                            } else {
                                                                Icon { icon: bs_icons::BsDeviceSsdFill }
                                                            }
                                                        }
                                                        div { class: "grow grid grid-cols-2",
                                                            span { class: "whitespace-nowrap truncate text-ellipsis max-w-32",
                                                                {d.label.clone().unwrap_or(d.name.clone())}
                                                            }
                                                            span { class: "text-right", title: "Total storage capacity", "{d.size}" }
                                                        }
                                                    }
                                                }
                                            })
                                    }
                                }
                            }
                        }
                    }
                    Some(Err(err)) => rsx! {
                        BentoError { "{err}" }
                    },
                    None => rsx! {
                        BentoPlaceholder {
                        }
                    },
                }

            }
        }
    };

    rsx! {
        div { class: "flex flex-col gap-2",
            h1 { class: "text-5xl font-bold", "Welcome, {username.to_title_case()}!" }
            h2 { class: "font-regular italic", "Connected with {server_name}" }
            div { class: "2xl:pr-32",
                BentoContainer {
                    BentoRow {
                        GeneralInformation {}
                        {
                            if let Some(Ok(info)) = &*dashboard_information_req.read()
                                && info.relevant_services.contains(&("docker".to_string(), true))
                            {
                                rsx! {
                                    DockerContainers {}
                                }
                            } else {
                                rsx! {}
                            }
                        }
                        Packages {}
                    }
                    BentoRow {
                        Cpu {}
                        Memory {}
                        Network {}
                        Thermometers {}
                    }
                    BentoRow { Drives {} }
                }
            }
        }
    }
}

#[component]
pub fn Dashboard() -> Element {
    rsx! {
        Viewport {
            ViewportTabs { "Dashboard" }
            ViewportContents {
                SuspenseBoundary { fallback: |_| rsx! {}, Contents {} }
            }
        }
    }
}

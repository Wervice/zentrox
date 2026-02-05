use actix_web::HttpResponse;
use actix_web::web::Path;
use serde::Serialize;
use std::{ffi::OsString, fs, ops::Div, path::PathBuf};
use sysinfo::{MemoryRefreshKind, Pid, ProcessRefreshKind, RefreshKind, UpdateKind};
use utils::status_com::ErrorCode;
use utils::{status_com::MessageRes, users::NativeUser};
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
struct ListProcessesRes {
    processes: Vec<SerializableProcess>,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
struct SerializableProcess {
    name: Option<String>,
    cpu_usage: f32,
    memory_usage_bytes: u64,
    username: Option<String>,
    executable_path: Option<String>,
    pid: u32,
}

fn os_string_array_to_string_vector(s: &[OsString]) -> Vec<String> {
    s.iter()
        .map(|x| {
            let b = x.as_encoded_bytes();
            String::from_utf8(b.to_vec()).unwrap()
        })
        .collect::<Vec<String>>()
}

#[utoipa::path(get, path = "/private/processes/list", responses((status = 200, body = ListProcessesRes)), tags = ["processes", "private"])]
/// List of processes
pub async fn list() -> HttpResponse {
    let process_refresh = ProcessRefreshKind::nothing()
        .without_tasks()
        .with_cpu()
        .with_memory()
        .with_user(UpdateKind::OnlyIfNotSet)
        .with_exe(UpdateKind::OnlyIfNotSet);

    let mut system = sysinfo::System::new_with_specifics(
        RefreshKind::nothing()
            .with_memory(MemoryRefreshKind::everything())
            .with_processes(process_refresh),
    );
    std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
    system.refresh_processes_specifics(sysinfo::ProcessesToUpdate::All, false, process_refresh);
    let processes = system.processes();
    let mut processes_for_response = Vec::new();

    processes.iter().for_each(|x| {
        let memory_usage_bytes = x.1.memory();
        let cpu_usage = x.1.cpu_usage() / 100_f32;

        let executable_path = x.1.exe().map(|x| x.to_str().unwrap().to_string());
        let mut username: Option<String> = None;
        if let Some(uid) = x.1.user_id() {
            username = Some(NativeUser::from_uid(uid.div(1)).unwrap().username);
        }

        let pid = x.1.pid().as_u32();
        let name = Some(x.1.name().to_string_lossy().to_string());
        processes_for_response.push(SerializableProcess {
            name,
            cpu_usage,
            memory_usage_bytes,
            executable_path,
            username,
            pid,
        });
    });

    HttpResponse::Ok().json(ListProcessesRes {
        processes: processes_for_response,
    })
}

/// Kill process by PID.
#[utoipa::path(post,
    path = "/private/processes/kill/{pid}",
    params(("pid" = u32, Path)),
    responses((status = 200), (status = 404, description = "The pid was not found.")),
    tags = ["processes", "private"]
)]
pub async fn kill(path: Path<u32>) -> HttpResponse {
    let process_refresh = ProcessRefreshKind::nothing()
        .without_tasks()
        .with_cpu()
        .with_memory()
        .with_user(UpdateKind::OnlyIfNotSet)
        .with_exe(UpdateKind::OnlyIfNotSet);
    let mut system = sysinfo::System::new_with_specifics(
        RefreshKind::nothing()
            .with_memory(MemoryRefreshKind::everything())
            .with_processes(process_refresh),
    );
    std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
    system.refresh_processes_specifics(sysinfo::ProcessesToUpdate::All, false, process_refresh);
    let processes = system.processes();

    match processes.get(&Pid::from_u32(path.into_inner())) {
        Some(p) => {
            if p.kill() {
                HttpResponse::Ok().json(MessageRes::from("The singal has been sent successfully."))
            } else {
                HttpResponse::InternalServerError().json(ErrorCode::SignalError.as_error_message())
            }
        }
        None => HttpResponse::NotFound().json(ErrorCode::UnknownPid.as_error_message()),
    }
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
struct ProcessDetailsRes {
    name: String,
    pid: u32,
    user: NativeUser,
    uid: u32,
    memory_usage_bytes: u64,
    cpu_usage: f32,
    run_time: u64,
    command_line: Vec<String>,
    executable_path: Option<String>,
    priority: isize,
    threads: isize,
    parent: String,
}

#[utoipa::path(get,
    path = "/private/processes/details/{pid}",
    params(("pid" = u32, Path)),
    responses((status = 200, body = ProcessDetailsRes), (status = 404, description = "The pid was not found.")),
    tags = ["processes", "private"]
)]
/// Details about process
pub async fn details(path: Path<u32>) -> HttpResponse {
    let process_refresh = ProcessRefreshKind::nothing()
        .without_tasks()
        .with_cpu()
        .with_memory()
        .with_cmd(UpdateKind::OnlyIfNotSet)
        .with_user(UpdateKind::OnlyIfNotSet)
        .with_exe(UpdateKind::OnlyIfNotSet);

    let mut system = sysinfo::System::new_with_specifics(
        RefreshKind::nothing()
            .with_memory(MemoryRefreshKind::everything())
            .with_processes(process_refresh),
    );

    std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
    system.refresh_processes_specifics(sysinfo::ProcessesToUpdate::All, false, process_refresh);

    let processes = system.processes();
    let selected_process = processes.get(&Pid::from_u32(path.into_inner())).unwrap();

    let name = selected_process.name();
    let pid = selected_process.pid();
    let uid = selected_process.user_id().unwrap();
    let user = NativeUser::from_uid(uid.div(1)).unwrap();
    let memory_usage_bytes = selected_process.memory();
    let cpu_usage = selected_process.cpu_usage() / 100_f32;
    let run_time = selected_process.run_time();
    let command_line = os_string_array_to_string_vector(selected_process.cmd());
    let executable_path_determination = selected_process.exe();

    let parent = selected_process
        .parent()
        .expect("The process has no parrent id.");
    let parent_name = system
        .process(parent)
        .expect("The process can not be accessed.")
        .name()
        .to_str()
        .unwrap()
        .to_string();

    let executable_path =
        executable_path_determination.map(|det| det.to_str().unwrap().to_string());

    let stat_file = fs::read_to_string(
        PathBuf::from("/")
            .join("proc")
            .join(pid.to_string())
            .join("stat"),
    )
    .unwrap();

    let stat_file_split = stat_file.split(" ").collect::<Vec<&str>>();
    let priority = stat_file_split[18].parse::<isize>().unwrap_or(-1);

    let thread_count = stat_file_split[19].parse::<isize>().unwrap_or(-1);

    HttpResponse::Ok().json(ProcessDetailsRes {
        name: name.to_str().unwrap().to_string(),
        pid: pid.as_u32(),
        user,
        uid: uid.div(1),
        memory_usage_bytes,
        cpu_usage,
        command_line,
        threads: thread_count,
        priority,
        run_time,
        executable_path,
        parent: parent_name,
    })
}

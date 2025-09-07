#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod logger;
mod timeUtil;

use timeUtil::{utc_to_sh_rfc3339,now_sh_rfc3339,now_utc_ms};
use chrono::Local;
use log::{error, info, LevelFilter};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    time::Duration,
};
use tauri::{AppHandle, Emitter, State};

use logger::{global_log_buffer, init_logger, LogEntry};

#[derive(Debug, Deserialize, Clone)]
pub struct TaskRequest {
    pub user_id: String,
    pub user_token: String,
    pub run_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct StartResponse {
    pub run_id: String,
    pub message: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct TaskStatus {
    pub run_id: String,
    pub state: String,      // queued|running|success|failed|canceled
    pub progress: u8,       // 0..=100
    pub started_at: String, // RFC3339
    pub finished_at: Option<String>,
    pub last_message: Option<String>,
}

#[derive(Debug, Default)]
struct AppState {
    // 如需允许多任务并发，可删除 is_running，仅靠 map 维护状态即可
    is_running: AtomicBool,
    cancels: Mutex<HashMap<String, Arc<AtomicBool>>>,
    statuses: Mutex<HashMap<String, TaskStatus>>,
}

// ========== 命令：启动长任务（立即返回 run_id） ==========
#[tauri::command]
async fn start_run_task(
    payload: TaskRequest,
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
) -> Result<StartResponse, String> {
    // 单任务互斥（如不需要可删）
    if state.is_running.swap(true, Ordering::SeqCst) {
        return Err("已有任务在运行中，请稍后再试。".into());
    }

    let run_id = payload
        .run_id
        .clone()
        .unwrap_or_else(|| format!("{}", now_utc_ms().to_string()));

    {
        let mut st = state.statuses.lock().unwrap();
        st.insert(
            run_id.clone(),
            TaskStatus {
                run_id: run_id.clone(),
                state: "running".into(),
                progress: 0,
                started_at: now_sh_rfc3339(),
                finished_at: None,
                last_message: Some("任务开始…".into()),
            },
        );
    }

    let cancel_flag = Arc::new(AtomicBool::new(false));
    {
        let mut cs = state.cancels.lock().unwrap();
        cs.insert(run_id.clone(), cancel_flag.clone());
    }

    let _ = app.emit(
        "run_progress",
        serde_json::json!({
            "runId": run_id,
            "progress": 0u8,
            "message": "任务开始…",
            "ts": now_sh_rfc3339(),
        }),
    );

    let app_cloned = app.clone();
    let state_cloned = state.inner().clone();
    let run_id_cloned = run_id.clone();

    // 真正的长任务在后台线程执行 —— 不要 await！
    tauri::async_runtime::spawn_blocking(move || {
        info!("长任务启动: run_id={}", run_id_cloned);

        // TODO: 这里替换成你的 jbpmain() 逻辑。此处用循环模拟超长执行。
        for pct in (0..=100).step_by(2) {
            if cancel_flag.load(Ordering::Relaxed) {
                info!("任务被取消: {}", run_id_cloned);
                {
                    let mut st = state_cloned.statuses.lock().unwrap();
                    if let Some(s) = st.get_mut(&run_id_cloned) {
                        s.state = "canceled".into();
                        s.progress = pct.min(100);
                        s.finished_at = Some(now_sh_rfc3339());
                        s.last_message = Some("任务已取消".into());
                    }
                }
                let _ = app_cloned.emit(
                    "run_done",
                    serde_json::json!({
                        "runId": run_id_cloned,
                        "state": "canceled",
                        "ts": now_sh_rfc3339(),
                    }),
                );
                state_cloned.is_running.store(false, Ordering::SeqCst);
                state_cloned.cancels.lock().unwrap().remove(&run_id_cloned);
                return;
            }

            // 模拟耗时步骤
            std::thread::sleep(Duration::from_millis(500));

            // 中间进度事件
            let msg = format!("正在处理… {}%", pct);
            let _ = app_cloned.emit(
                "run_progress",
                serde_json::json!({
                    "runId": run_id_cloned,
                    "progress": pct,
                    "message": msg,
                    "ts": now_sh_rfc3339(),
                }),
            );

            // 写入状态表
            {
                let mut st = state_cloned.statuses.lock().unwrap();
                if let Some(s) = st.get_mut(&run_id_cloned) {
                    s.progress = pct as u8;
                    s.last_message = Some(format!("步骤推进至 {pct}%"));
                }
            }
        }

        // 正常结束
        info!("长任务完成: {}", run_id_cloned);
        {
            let mut st = state_cloned.statuses.lock().unwrap();
            if let Some(s) = st.get_mut(&run_id_cloned) {
                s.state = "success".into();
                s.progress = 100;
                s.finished_at = Some(now_sh_rfc3339());
                s.last_message = Some("任务完成 ✅".into());
            }
        }
        let _ = app_cloned.emit(
            "run_done",
            serde_json::json!({
                "runId": run_id_cloned,
                "state": "success",
                "ts": now_sh_rfc3339(),
            }),
        );

        state_cloned.is_running.store(false, Ordering::SeqCst);
        state_cloned.cancels.lock().unwrap().remove(&run_id_cloned);
    });

    Ok(StartResponse {
        run_id,
        message: "任务已启动".into(),
    })
}

#[derive(Debug, Deserialize)]
struct GetTaskStatusArgs {
    // 统一我们在 Rust 里使用 `run_id` 变量名，
    // 但兼容前端传 runId / runid / run_id 三种写法
    #[serde(alias = "runId", alias = "runid", alias = "run_id")]
    run_id: String,
}

#[tauri::command]
fn get_task_status(args: GetTaskStatusArgs, state: State<'_, Arc<AppState>>) -> Option<TaskStatus> {
		let run_id = args.run_id;
    state.statuses.lock().unwrap().get(&run_id).cloned()
}

#[derive(Debug, Deserialize)]
struct CancelArgs {
    // 统一我们在 Rust 里使用 `run_id` 变量名，
    // 但兼容前端传 runId / runid / run_id 三种写法
    #[serde(alias = "runId", alias = "runid", alias = "run_id")]
    run_id: String,
}

#[tauri::command]
fn cancel_task(args: CancelArgs, state: State<'_, Arc<AppState>>) -> bool {
    let run_id = args.run_id;
    if let Some(flag) = state.cancels.lock().unwrap().get(&run_id) {
        flag.store(true, Ordering::SeqCst);
        true
    } else {
        false
    }
}

// ===== 日志相关（与你之前版本保持一致） =====

#[derive(Deserialize)]
struct GetLogsArgs {
    since_id: Option<u64>,
    limit: Option<usize>,
}

#[tauri::command]
fn get_logs(args: GetLogsArgs) -> (Vec<LogEntry>, u64) {
    global_log_buffer().get_since(args.since_id, args.limit)
}

#[tauri::command]
fn clear_logs() {
    global_log_buffer().clear();
}

#[tauri::command]
fn set_log_level(level: String) -> Result<(), String> {
    use log::LevelFilter;
    let level_filter = match level.to_lowercase().as_str() {
        "trace" => LevelFilter::Trace,
        "debug" => LevelFilter::Debug,
        "info" => LevelFilter::Info,
        "warn" => LevelFilter::Warn,
        "error" => LevelFilter::Error,
        "off" => LevelFilter::Off,
        _ => return Err("invalid level".into()),
    };
    log::set_max_level(level_filter);
    global_log_buffer().set_level(level_filter);
    Ok(())
}

fn main() {
    tauri::Builder::default()
    		.plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            init_logger(&app.handle(), 5000, LevelFilter::Info).expect("init logger");
            Ok(())
        })
        .manage(Arc::new(AppState::default()))
        .invoke_handler(tauri::generate_handler![
            start_run_task,
            get_task_status,
            cancel_task,
            get_logs,
            clear_logs,
            set_log_level
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

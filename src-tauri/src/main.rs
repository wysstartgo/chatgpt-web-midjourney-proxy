#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod logger; // <== 新增

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, State};
use log::{info, warn, error, LevelFilter};

use logger::{global_log_buffer, LogEntry, init_logger};

#[derive(Debug, Deserialize)]
pub struct TaskRequest {
    pub user_id: String,
    pub user_token: String,
    pub run_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct TaskResponse {
    pub message: String,
}

#[derive(Debug)]
struct AppState {
    is_running: AtomicBool,
}

impl AppState {
    fn new() -> Self {
        Self { is_running: AtomicBool::new(false) }
    }
}

// === 现有 run_task 命令（略微把 println 换成 log::info）===

#[tauri::command]
async fn run_task(
    payload: TaskRequest,
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
) -> Result<TaskResponse, String> {
    info!("run_task called: user_id={} (token_len={})",
        payload.user_id, payload.user_token.len());

    if state.is_running.swap(true, Ordering::SeqCst) {
        warn!("run_task ignored: already running");
        return Ok(TaskResponse { message: "任务正在运行中，请稍后再试。".into() });
    }

    let run_id = payload.run_id.clone().unwrap_or_else(|| format!("{}", chrono::Local::now().timestamp_millis()));
    let app_cloned = app.clone();
    let run_id_cloned = run_id.clone();

    // 事件：开始
    let _ = app.emit_all("run_log", serde_json::json!({
        "runId": run_id,
        "line": "任务开始…",
        "ts": chrono::Local::now().to_rfc3339(),
    }));

    let handle = tauri::async_runtime::spawn_blocking(move || {
        info!("开始执行耗时操作…");
        for step in 1..=3 {
            std::thread::sleep(std::time::Duration::from_millis(500));
            info!("步骤 {step}/3 完成");
        }
        // TODO: 调你的 jbpmain()
        info!("jbpmain(): 正在处理数据…");
        std::thread::sleep(std::time::Duration::from_millis(600));
        info!("jbpmain(): 数据处理结束");
        info!("耗时操作完成！");
        let _ = app_cloned.emit_all("run_log", serde_json::json!({
            "runId": run_id_cloned,
            "line": "任务成功完成 ✅",
            "ts": chrono::Local::now().to_rfc3339(),
        }));
        Ok::<_, anyhow::Error>(())
    });

    let result = handle.await.map_err(|e| e.to_string()).and_then(|r| r.map_err(|e| e.to_string()));
    state.is_running.store(false, Ordering::SeqCst);

    match result {
        Ok(_) => Ok(TaskResponse {
            message: format!("任务已完成: 用户ID: {}, 用户令牌: {}!", payload.user_id, payload.user_token),
        }),
        Err(err) => {
            error!("run_task failed: {err}");
            Err(format!("执行失败：{err}"))
        }
    }
}

// === 新增：日志相关命令 ===

#[derive(Deserialize)]
struct GetLogsArgs {
    since_id: Option<u64>,
    limit: Option<usize>,
}

/// 增量获取日志
#[tauri::command]
fn get_logs(args: GetLogsArgs) -> (Vec<LogEntry>, u64) {
    global_log_buffer().get_since(args.since_id, args.limit)
}

/// 清空日志
#[tauri::command]
fn clear_logs() {
    global_log_buffer().clear();
}

/// 动态设置日志级别（可选）
#[tauri::command]
fn set_log_level(level: String) -> Result<(), String> {
    let level_filter = match level.to_lowercase().as_str() {
        "trace" => LevelFilter::Trace,
        "debug" => LevelFilter::Debug,
        "info"  => LevelFilter::Info,
        "warn"  => LevelFilter::Warn,
        "error" => LevelFilter::Error,
        "off"   => LevelFilter::Off,
        _ => return Err("invalid level".into()),
    };
    log::set_max_level(level_filter);
    global_log_buffer().set_level(level_filter);
    Ok(())
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            // 初始化日志器：容量 5000，默认 INFO
            init_logger(&app.handle(), 5000, LevelFilter::Info).expect("init logger");
            Ok(())
        })
        .manage(Arc::new(AppState::new()))
        .invoke_handler(tauri::generate_handler![
            run_task,
            get_logs,
            clear_logs,
            set_log_level
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

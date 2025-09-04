use chrono::Local;
use log::{Level, LevelFilter, Metadata, Record, SetLoggerError};
use once_cell::sync::OnceCell;
use serde::Serialize;
use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};
use tauri::{AppHandle, Emitter};
use crate::timeUtil::{utc_to_sh_rfc3339, now_sh_rfc3339, now_utc_ms};
static APP_HANDLE: OnceCell<AppHandle> = OnceCell::new();

#[derive(Debug, Clone, Serialize)]
pub struct LogEntry {
    pub id: u64,
    pub ts: String, // RFC3339
    pub level: String,
    pub target: String,
    pub message: String,
}

#[derive(Debug)]
pub struct LogBuffer {
    inner: Mutex<Inner>,
    max_len: usize,
}

#[derive(Debug)]
struct Inner {
    seq: u64,
    buf: VecDeque<LogEntry>,
    level: LevelFilter,
}

impl LogBuffer {
    pub fn new(max_len: usize, level: LevelFilter) -> Self {
        Self {
            inner: Mutex::new(Inner {
                seq: 0,
                buf: VecDeque::with_capacity(max_len),
                level,
            }),
            max_len,
        }
    }

    pub fn set_level(&self, level: LevelFilter) {
        if let Ok(mut g) = self.inner.lock() {
            g.level = level;
        }
    }

    pub fn level(&self) -> LevelFilter {
        self.inner
            .lock()
            .map(|g| g.level)
            .unwrap_or(LevelFilter::Info)
    }

    fn push(&self, level: Level, target: &str, msg: String) {
        let mut g = self.inner.lock().unwrap();
        g.seq += 1;
        let entry = LogEntry {
            id: g.seq,
            ts: now_sh_rfc3339(),
            level: level.to_string(),
            target: target.to_string(),
            message: msg.clone(),
        };
        if g.buf.len() == self.max_len {
            g.buf.pop_front();
        }
        g.buf.push_back(entry.clone());

        if let Some(app) = APP_HANDLE.get() {
            let _ = app.emit("app_log", &entry);
        }
    }

    pub fn get_since(&self, since_id: Option<u64>, limit: Option<usize>) -> (Vec<LogEntry>, u64) {
        let g = self.inner.lock().unwrap();
        let latest = g.seq;
        let lim = limit.unwrap_or(500).min(self.max_len);
        let start_id = since_id.unwrap_or(0);

        let mut out = Vec::new();
        for e in g.buf.iter() {
            if e.id > start_id {
                out.push(e.clone());
                if out.len() >= lim {
                    break;
                }
            }
        }
        (out, latest)
    }

    pub fn clear(&self) {
        let mut g = self.inner.lock().unwrap();
        g.buf.clear();
    }
}

static LOGGER: OnceCell<Arc<LogBuffer>> = OnceCell::new();

pub fn global_log_buffer() -> &'static Arc<LogBuffer> {
    LOGGER.get().expect("logger not inited")
}

struct RingLogger;

impl log::Log for RingLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        let level = global_log_buffer().level();
        metadata.level().to_level_filter() <= level
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let msg = format!("{}", record.args());
            global_log_buffer().push(record.level(), record.target(), msg);
        }
    }

    fn flush(&self) {}
}

static RING_LOGGER: RingLogger = RingLogger;

pub fn init_logger(
    app: &AppHandle,
    max_len: usize,
    level: LevelFilter,
) -> Result<(), SetLoggerError> {
    let _ = APP_HANDLE.set(app.clone());
    let _ = LOGGER.set(Arc::new(LogBuffer::new(max_len, level)));
    log::set_logger(&RING_LOGGER)?;
    log::set_max_level(level);
    Ok(())
}

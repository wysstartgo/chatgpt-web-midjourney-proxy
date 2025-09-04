// src/timeUtil.rs
use chrono::{DateTime, Utc};

use chrono_tz::Asia::Shanghai;

/// 当前 UTC 毫秒时间戳
pub fn now_utc_ms() -> i64 {
    Utc::now().timestamp_millis()
}

/// 把 UTC 时间转为上海时区的 RFC3339 字符串
pub fn utc_to_sh_rfc3339(dt_utc: DateTime<Utc>) -> String {
    // 避免 from_utc_datetime 的 trait 问题，直接 with_timezone 更简洁
//     dt_utc.with_timezone(&Shanghai).to_rfc3339()
    // 假设 dt_utc: DateTime<Utc>
    dt_utc.with_timezone(&chrono_tz::Asia::Shanghai).to_rfc3339()

}

/// 当前“上海时区”时间的 RFC3339 字符串
pub fn now_sh_rfc3339() -> String {
    utc_to_sh_rfc3339(Utc::now())
}

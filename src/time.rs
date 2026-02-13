#[cfg(feature = "chrono")]
use chrono::DateTime;

#[cfg(feature = "chrono")]
pub fn format_timestamp_human(ts: i64) -> String {
    let dt = DateTime::from_timestamp(ts, 0).expect("invalid timestamp");
    dt.format("%Y-%m-%d %H:%M:%S").to_string()
}

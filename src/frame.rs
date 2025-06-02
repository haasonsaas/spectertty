use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use base64::prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FrameType {
    Stdout,
    Stdin,
    Stderr,
    Cursor,
    Resize,
    ResizeAck,
    Prompt,
    Idle,
    LineUpdate,
    Overflow,
    Signal,
    Exit,
    Stopped,
    Continued,
    CapsuleKill,
    Ping,
    Pong,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Frame {
    pub ts: f64,
    #[serde(rename = "type")]
    pub frame_type: FrameType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub binary: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cols: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rows: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signal: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub regex: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dur_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

impl Frame {
    pub fn new(frame_type: FrameType) -> Self {
        Self {
            ts: current_timestamp(),
            frame_type,
            data: None,
            binary: None,
            cols: None,
            rows: None,
            code: None,
            signal: None,
            regex: None,
            dur_ms: None,
            reason: None,
        }
    }

    pub fn with_data(mut self, data: String) -> Self {
        self.data = Some(data);
        self
    }

    pub fn with_binary_data(mut self, data: Vec<u8>) -> Self {
        self.data = Some(base64::prelude::BASE64_STANDARD.encode(data));
        self.binary = Some(true);
        self
    }

    pub fn with_size(mut self, cols: u16, rows: u16) -> Self {
        self.cols = Some(cols);
        self.rows = Some(rows);
        self
    }

    pub fn with_exit_code(mut self, code: i32) -> Self {
        self.code = Some(code);
        self
    }

    pub fn with_signal(mut self, signal: String) -> Self {
        self.signal = Some(signal);
        self
    }

    pub fn with_regex(mut self, regex: String) -> Self {
        self.regex = Some(regex);
        self
    }

    pub fn with_duration(mut self, dur_ms: u64) -> Self {
        self.dur_ms = Some(dur_ms);
        self
    }

    pub fn with_reason(mut self, reason: String) -> Self {
        self.reason = Some(reason);
        self
    }

    pub fn to_json(&self) -> anyhow::Result<String> {
        Ok(serde_json::to_string(self)?)
    }

    pub fn from_json(json: &str) -> anyhow::Result<Self> {
        Ok(serde_json::from_str(json)?)
    }
}

fn current_timestamp() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs_f64()
}
use crate::frame::{Frame, FrameType};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

#[derive(Serialize, Deserialize)]
struct AsciinemaHeader {
    version: u8,
    width: u16,
    height: u16,
    timestamp: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    command: Option<String>,
    env: AsciinemaEnv,
}

#[derive(Serialize, Deserialize)]
struct AsciinemaEnv {
    #[serde(rename = "SHELL")]
    shell: String,
    #[serde(rename = "TERM")]
    term: String,
}

#[derive(Serialize)]
struct AsciinemaEvent {
    time: f64,
    event_type: String,
    data: String,
}

pub struct AsciinemaRecorder {
    writer: BufWriter<File>,
    start_time: Instant,
    last_timestamp: f64,
}

impl AsciinemaRecorder {
    pub fn new<P: AsRef<Path>>(
        path: P,
        width: u16,
        height: u16,
        command: Option<String>,
    ) -> Result<Self> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);

        // Write asciinema v2 header
        let header = AsciinemaHeader {
            version: 2,
            width,
            height,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            title: Some("SpecterTTY Recording".to_string()),
            command,
            env: AsciinemaEnv {
                shell: std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string()),
                term: std::env::var("TERM").unwrap_or_else(|_| "xterm-256color".to_string()),
            },
        };

        let header_json = serde_json::to_string(&header)?;
        writeln!(writer, "{}", header_json)?;

        Ok(Self {
            writer,
            start_time: Instant::now(),
            last_timestamp: 0.0,
        })
    }

    pub fn record_frame(&mut self, frame: &Frame) -> Result<()> {
        let timestamp = self.start_time.elapsed().as_secs_f64();
        
        // Only record certain frame types for asciinema compatibility
        let (event_type, data) = match &frame.frame_type {
            FrameType::Stdout => ("o", frame.data.as_deref().unwrap_or("")),
            FrameType::Stdin => ("i", frame.data.as_deref().unwrap_or("")),
            FrameType::Stderr => ("o", frame.data.as_deref().unwrap_or("")), // stderr goes to stdout in asciinema
            FrameType::Resize => {
                if let (Some(cols), Some(rows)) = (frame.cols, frame.rows) {
                    // Asciinema doesn't have a standard resize event, so we'll output a comment
                    ("o", "# Terminal resized\r\n")
                } else {
                    return Ok(()); // Skip if no size info
                }
            }
            _ => return Ok(()), // Skip other frame types
        };

        let event = AsciinemaEvent {
            time: timestamp,
            event_type: event_type.to_string(),
            data: data.to_string(),
        };

        let event_json = serde_json::to_string(&event)?;
        writeln!(self.writer, "{}", event_json)?;
        
        self.last_timestamp = timestamp;
        self.writer.flush()?;
        
        Ok(())
    }

    pub fn finish(mut self) -> Result<()> {
        self.writer.flush()?;
        Ok(())
    }
}

pub struct RecordingManager {
    recorder: Option<AsciinemaRecorder>,
}

impl RecordingManager {
    pub fn new() -> Self {
        Self { recorder: None }
    }

    pub fn start_recording<P: AsRef<Path>>(
        &mut self,
        path: P,
        width: u16,
        height: u16,
        command: Option<String>,
    ) -> Result<()> {
        self.recorder = Some(AsciinemaRecorder::new(path, width, height, command)?);
        Ok(())
    }

    pub fn record_frame(&mut self, frame: &Frame) -> Result<()> {
        if let Some(ref mut recorder) = self.recorder {
            recorder.record_frame(frame)?;
        }
        Ok(())
    }

    pub fn stop_recording(&mut self) -> Result<()> {
        if let Some(recorder) = self.recorder.take() {
            recorder.finish()?;
        }
        Ok(())
    }

    pub fn is_recording(&self) -> bool {
        self.recorder.is_some()
    }
}
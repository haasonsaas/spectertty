use crate::cli::TokenMode;
use crate::frame::{Frame, FrameType};
use anyhow::Result;
use regex::Regex;
use std::collections::VecDeque;

pub struct OutputProcessor {
    mode: TokenMode,
    line_buffer: String,
    ansi_strip_regex: Regex,
    progress_regex: Regex,
    last_line_update: Option<String>,
    frame_buffer: VecDeque<Frame>,
}

impl OutputProcessor {
    pub fn new(mode: TokenMode) -> Self {
        Self {
            mode,
            line_buffer: String::new(),
            // Regex to strip ANSI escape sequences
            ansi_strip_regex: Regex::new(r"\x1b\[[0-9;]*[a-zA-Z]").unwrap(),
            // Regex to detect progress indicators (e.g., downloading, installing)
            progress_regex: Regex::new(r"[\r\n]*[\s]*[▌▍▎▏█░▒▓■□▪▫●○◐◑◒◓◔◕◖◗◘◙◚◛◜◝◞◟◠◡◢◣◤◥◦◧◨◩◪◫◬◭◮◯]+|[0-9]+%|\[[=>\-\s]*\]").unwrap(),
            last_line_update: None,
            frame_buffer: VecDeque::new(),
        }
    }

    pub async fn process_frame(&mut self, frame: Frame) -> Result<Vec<Frame>> {
        match self.mode {
            TokenMode::Raw => Ok(vec![frame]),
            TokenMode::Compact => self.process_compact(frame).await,
            TokenMode::Parsed => self.process_parsed(frame).await,
        }
    }

    async fn process_compact(&mut self, mut frame: Frame) -> Result<Vec<Frame>> {
        match frame.frame_type {
            FrameType::Stdout | FrameType::Stderr => {
                if let Some(ref data) = frame.data {
                    let cleaned = self.clean_output(data);
                    
                    // Check if this looks like a progress update
                    if self.is_progress_update(&cleaned) {
                        return self.handle_progress_update(frame, cleaned).await;
                    }

                    // Batch small outputs together
                    self.line_buffer.push_str(&cleaned);
                    
                    // If we have a complete line or buffer is getting large, emit it
                    if cleaned.contains('\n') || self.line_buffer.len() > 512 {
                        frame.data = Some(self.line_buffer.clone());
                        self.line_buffer.clear();
                        Ok(vec![frame])
                    } else {
                        // Buffer for later
                        Ok(vec![])
                    }
                } else {
                    Ok(vec![frame])
                }
            }
            _ => Ok(vec![frame]),
        }
    }

    async fn process_parsed(&mut self, frame: Frame) -> Result<Vec<Frame>> {
        // For parsed mode, we would implement more sophisticated parsing
        // For now, use compact mode as a base
        self.process_compact(frame).await
    }

    fn clean_output(&self, data: &str) -> String {
        // Strip ANSI escape sequences
        let mut cleaned = self.ansi_strip_regex.replace_all(data, "").to_string();
        
        // Normalize line endings
        cleaned = cleaned.replace("\r\n", "\n").replace('\r', "\n");
        
        // Remove excessive whitespace but preserve structure
        let lines: Vec<&str> = cleaned.lines().collect();
        let processed_lines: Vec<String> = lines
            .iter()
            .map(|line| {
                // Trim trailing whitespace but preserve leading indentation
                line.trim_end().to_string()
            })
            .collect();
        
        processed_lines.join("\n")
    }

    fn is_progress_update(&self, data: &str) -> bool {
        // Check for common progress indicators
        self.progress_regex.is_match(data) ||
        data.contains("downloading") ||
        data.contains("installing") ||
        data.contains("loading") ||
        data.contains("progress") ||
        // Check for repeated carriage returns (common in progress bars)
        data.chars().filter(|&c| c == '\r').count() > 2
    }

    async fn handle_progress_update(&mut self, mut frame: Frame, cleaned: String) -> Result<Vec<Frame>> {
        // Convert progress output to line_update frames
        frame.frame_type = FrameType::LineUpdate;
        
        // Only emit if this is different from the last update
        if self.last_line_update.as_ref() != Some(&cleaned) {
            frame.data = Some(cleaned.clone());
            self.last_line_update = Some(cleaned);
            Ok(vec![frame])
        } else {
            // Skip duplicate progress updates
            Ok(vec![])
        }
    }

    pub fn flush_buffer(&mut self) -> Vec<Frame> {
        let mut frames = Vec::new();
        
        // Flush any remaining line buffer
        if !self.line_buffer.is_empty() {
            let frame = Frame::new(FrameType::Stdout)
                .with_data(self.line_buffer.clone());
            frames.push(frame);
            self.line_buffer.clear();
        }
        
        // Flush frame buffer
        while let Some(frame) = self.frame_buffer.pop_front() {
            frames.push(frame);
        }
        
        frames
    }
}
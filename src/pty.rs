use crate::frame::{Frame, FrameType};
use anyhow::{anyhow, Result};
use futures::stream::Stream;
use portable_pty::{Child, CommandBuilder, PtyPair, PtySize};
use regex::Regex;
use std::io::{Read, Write};
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tokio::time::sleep;
use tracing::{debug, error, info, warn};

pub struct PtySession {
    pty_pair: PtyPair,
    child: Box<dyn Child + Send + Sync>,
    frame_tx: mpsc::UnboundedSender<Frame>,
    pub frame_rx: mpsc::UnboundedReceiver<Frame>,
    prompt_regexes: Vec<Regex>,
    idle_timeout: Duration,
    last_activity: Instant,
    buffer: Vec<u8>,
    current_line: String,
}

impl PtySession {
    pub async fn new(
        command: &str,
        args: &[String],
        cols: u16,
        rows: u16,
        prompt_regexes: Vec<String>,
        idle_timeout: Duration,
    ) -> Result<Self> {
        let pty_system = portable_pty::native_pty_system();
        let size = PtySize {
            rows: rows,
            cols: cols,
            pixel_width: 0,
            pixel_height: 0,
        };

        let pty_pair = pty_system.openpty(size)?;
        
        let mut cmd = CommandBuilder::new(command);
        for arg in args {
            cmd.arg(arg);
        }

        let child = pty_pair.slave.spawn_command(cmd)?;
        
        let (frame_tx, frame_rx) = mpsc::unbounded_channel();

        let compiled_regexes = prompt_regexes
            .into_iter()
            .map(|pattern| Regex::new(&pattern))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| anyhow!("Invalid regex pattern: {}", e))?;

        let session = Self {
            pty_pair,
            child,
            frame_tx,
            frame_rx,
            prompt_regexes: compiled_regexes,
            idle_timeout,
            last_activity: Instant::now(),
            buffer: Vec::new(),
            current_line: String::new(),
        };

        info!("PTY session started with PID: {:?}", session.child.process_id());
        Ok(session)
    }

    pub async fn run(&mut self) -> Result<()> {
        let mut reader = self.pty_pair.master.try_clone_reader()?;
        let frame_tx = self.frame_tx.clone();
        
        // Spawn output reader task
        let output_task = tokio::spawn(async move {
            let mut buffer = [0u8; 8192];
            loop {
                match reader.read(&mut buffer) {
                    Ok(0) => {
                        debug!("PTY output stream closed");
                        break;
                    }
                    Ok(n) => {
                        let data = String::from_utf8_lossy(&buffer[..n]).to_string();
                        let frame = Frame::new(FrameType::Stdout).with_data(data);
                        
                        if let Err(e) = frame_tx.send(frame) {
                            error!("Failed to send stdout frame: {}", e);
                            break;
                        }
                    }
                    Err(e) => {
                        error!("Error reading from PTY: {}", e);
                        break;
                    }
                }
            }
        });

        // Check child process status periodically
        let mut interval = tokio::time::interval(Duration::from_millis(100));
        
        loop {
            tokio::select! {
                // Check for idle timeout
                _ = sleep(self.idle_timeout) => {
                    if self.last_activity.elapsed() >= self.idle_timeout {
                        let frame = Frame::new(FrameType::Idle)
                            .with_duration(self.last_activity.elapsed().as_millis() as u64);
                        if let Err(e) = self.frame_tx.send(frame) {
                            error!("Failed to send idle frame: {}", e);
                            break;
                        }
                        self.last_activity = Instant::now();
                    }
                }
                
                // Check child process status
                _ = interval.tick() => {
                    match self.child.try_wait() {
                        Ok(Some(exit_status)) => {
                            let code = if exit_status.success() { 0 } else { 1 };
                            let frame = Frame::new(FrameType::Exit).with_exit_code(code);
                            let _ = self.frame_tx.send(frame);
                            info!("Child process exited with code: {}", code);
                            break;
                        }
                        Ok(None) => {
                            // Still running
                        }
                        Err(e) => {
                            error!("Error checking child status: {}", e);
                            break;
                        }
                    }
                }
            }
        }

        output_task.abort();
        Ok(())
    }


    pub async fn write_input(&mut self, data: &[u8]) -> Result<()> {
        let mut writer = self.pty_pair.master.take_writer()?;
        writer.write_all(data)?;
        writer.flush()?;
        
        let frame = Frame::new(FrameType::Stdin)
            .with_data(String::from_utf8_lossy(data).to_string());
        
        if let Err(e) = self.frame_tx.send(frame) {
            warn!("Failed to send stdin frame: {}", e);
        }

        self.last_activity = Instant::now();
        Ok(())
    }

    pub async fn resize(&mut self, cols: u16, rows: u16) -> Result<()> {
        let size = PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        };

        self.pty_pair.master.resize(size)?;
        
        let frame = Frame::new(FrameType::Resize).with_size(cols, rows);
        if let Err(e) = self.frame_tx.send(frame) {
            warn!("Failed to send resize frame: {}", e);
        }

        Ok(())
    }

    pub fn next_frame(&mut self) -> Option<Frame> {
        self.frame_rx.try_recv().ok()
    }

    pub async fn wait_for_frame(&mut self) -> Option<Frame> {
        self.frame_rx.recv().await
    }


    pub fn is_alive(&mut self) -> bool {
        self.child.try_wait().unwrap_or(None).is_none()
    }
}

impl Stream for PtySession {
    type Item = Frame;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.frame_rx.poll_recv(cx) {
            Poll::Ready(Some(frame)) => Poll::Ready(Some(frame)),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}
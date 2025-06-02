use clap::{Parser, ValueEnum};
use std::path::PathBuf;
use std::time::Duration;

#[derive(Parser)]
#[command(name = "spectertty")]
#[command(about = "AI-Native Terminal Automation Platform")]
#[command(version)]
pub struct Cli {
    #[arg(long, help = "Output frames to stdout")]
    pub json: bool,

    #[arg(long, help = "Unix socket transport")]
    pub socket: Option<PathBuf>,

    #[arg(long, help = "TCP transport (HOST:PORT)")]
    pub bind: Option<String>,

    #[arg(long, default_value = "120", help = "Initial window columns")]
    pub cols: u16,

    #[arg(long, default_value = "40", help = "Initial window rows")]
    pub rows: u16,

    #[arg(long, default_value = "200", help = "Idle duration before idle frame (ms)")]
    pub idle: u64,

    #[arg(long, value_enum, default_value = "raw", help = "Token processing mode")]
    pub token_mode: TokenMode,

    #[arg(long, help = "Register prompt matcher (repeatable)")]
    pub prompt_regex: Vec<String>,

    #[arg(long, default_value = "8388608", help = "Max in-mem queue before back-pressure (bytes)")]
    pub buffer: usize,

    #[arg(long, default_value = "5000", help = "Grace before SIGKILL on overflow (ms)")]
    pub overflow_timeout: u64,

    #[arg(long, help = "asciinema v2 output file")]
    pub record: Option<PathBuf>,

    #[arg(long, help = "Run target via capsule-run")]
    pub capsule: bool,

    #[arg(long, help = "Sandbox profile")]
    pub sandbox_profile: Option<String>,

    #[arg(long, help = "Enable session resurrection")]
    pub state_dir: Option<PathBuf>,

    #[arg(long, value_enum, default_value = "none", help = "Compress frame payloads")]
    pub compress: CompressionMode,

    #[arg(long, short, help = "Verbose logging")]
    pub verbose: bool,

    #[arg(help = "Command to execute")]
    pub command: String,

    #[arg(help = "Arguments for the command")]
    pub args: Vec<String>,
}

#[derive(Clone, Copy, ValueEnum)]
pub enum TokenMode {
    Raw,
    Compact,
    Parsed,
}

#[derive(Clone, Copy, ValueEnum)]
pub enum CompressionMode {
    None,
    #[cfg(feature = "compression")]
    Zstd,
}

impl Cli {
    pub fn idle_duration(&self) -> Duration {
        Duration::from_millis(self.idle)
    }

    pub fn overflow_timeout(&self) -> Duration {
        Duration::from_millis(self.overflow_timeout)
    }

    pub fn validate(&self) -> anyhow::Result<()> {
        if self.cols == 0 || self.rows == 0 {
            return Err(anyhow::anyhow!("Window size must be greater than 0"));
        }

        if self.idle == 0 {
            return Err(anyhow::anyhow!("Idle timeout must be greater than 0"));
        }

        if self.buffer == 0 {
            return Err(anyhow::anyhow!("Buffer size must be greater than 0"));
        }

        // Validate prompt regexes
        for pattern in &self.prompt_regex {
            regex::Regex::new(pattern)
                .map_err(|e| anyhow::anyhow!("Invalid prompt regex '{}': {}", pattern, e))?;
        }

        Ok(())
    }
}
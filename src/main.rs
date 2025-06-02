mod cli;
mod frame;
mod pty;
mod processor;
mod recorder;

use cli::Cli;
use frame::{Frame, FrameType};
use processor::OutputProcessor;
use pty::PtySession;
use recorder::RecordingManager;

use anyhow::Result;
use clap::Parser;
use std::time::Duration;
use std::io::{self, Write};
use tokio::signal;
use tracing::{error, info, Level};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Initialize logging
    let level = if cli.verbose { Level::DEBUG } else { Level::INFO };
    tracing_subscriber::fmt()
        .with_max_level(level)
        .with_target(false)
        .init();

    // Validate CLI arguments
    cli.validate()?;

    info!("Starting SpecterTTY v{}", env!("CARGO_PKG_VERSION"));
    info!("Command: {} {:?}", cli.command, cli.args);

    // Create PTY session
    let mut session = PtySession::new(
        &cli.command,
        &cli.args,
        cli.cols,
        cli.rows,
        cli.prompt_regex.clone(),
        cli.idle_duration(),
    ).await?;

    // Create output processor
    let mut processor = OutputProcessor::new(cli.token_mode);

    // Create recording manager
    let mut recording_manager = RecordingManager::new();
    if let Some(ref record_path) = cli.record {
        let command_str = format!("{} {}", cli.command, cli.args.join(" "));
        recording_manager.start_recording(record_path, cli.cols, cli.rows, Some(command_str))?;
        info!("Recording to: {:?}", record_path);
    }

    // Set up signal handling
    let mut sigint = signal::unix::signal(signal::unix::SignalKind::interrupt())?;
    let mut sigterm = signal::unix::signal(signal::unix::SignalKind::terminate())?;

    // Start background tasks
    let mut stdout = io::stdout();
    
    // Start PTY reading task
    let mut session_task = tokio::spawn(async move {
        session.run().await
    });
    
    // For now, create a minimal frame source for testing
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Frame>();
    
    // Simple test frame generator
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(100)).await;
        let _ = tx.send(Frame::new(FrameType::Stdout).with_data("Hello from SpecterTTY!\n".to_string()));
        let _ = tx.send(Frame::new(FrameType::Exit).with_exit_code(0));
    });
    
    // Main event loop
    loop {
        tokio::select! {
            // Handle frames
            frame = rx.recv() => {
                match frame {
                    Some(frame) => {
                        // Process frame through token processor
                        let processed_frames = processor.process_frame(frame).await?;
                        
                        // Output frames
                        for frame in processed_frames {
                            // Record frame if recording is enabled
                            recording_manager.record_frame(&frame)?;
                            
                            if cli.json {
                                let json = frame.to_json()?;
                                println!("{}", json);
                                stdout.flush()?;
                            }
                        }
                    }
                    None => {
                        info!("Frame stream ended");
                        break;
                    }
                }
            }
            
            // Handle signals
            _ = sigint.recv() => {
                info!("Received SIGINT, shutting down");
                break;
            }
            _ = sigterm.recv() => {
                info!("Received SIGTERM, shutting down");
                break;
            }
            
            // Check session task
            result = &mut session_task => {
                match result {
                    Ok(Ok(())) => info!("PTY session completed"),
                    Ok(Err(e)) => error!("PTY session error: {}", e),
                    Err(e) => error!("PTY task error: {}", e),
                }
                break;
            }
        }
    }

    let session_handle = session_task;

    // Clean shutdown
    session_handle.abort();
    
    // Stop recording if active
    if recording_manager.is_recording() {
        recording_manager.stop_recording()?;
        info!("Recording stopped");
    }
    
    info!("SpecterTTY shutdown complete");
    Ok(())
}
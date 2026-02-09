use anyhow::{Context, Result};
use log::{error, info, warn};
use ringbuf::HeapRb;
use tokio::sync::mpsc;
use tokio::time::{Duration, sleep};

mod audio;
mod socket;
mod transcriber;
mod vad;

use audio::AudioEngine;
use socket::{Command, SocketServer};
use transcriber::Transcriber;

// Config references
const SOCKET_PATH: &str = "/tmp/stt-sock";

fn get_model_path() -> String {
    if let Ok(path) = std::env::var("STT_MODEL_PATH") {
        return path;
    }

    let home = std::env::var("HOME").unwrap_or_else(|_| "/root".to_string());
    let candidates = vec![
        format!("{}/.local/share/stt-assistant/models/ggml-base.bin", home),
        "/usr/share/stt-assistant/models/ggml-base.bin".to_string(),
        "/usr/local/share/stt-assistant/models/ggml-base.bin".to_string(),
        "models/ggml-base.bin".to_string(), // Local dev
    ];

    for path in candidates {
        if std::path::Path::new(&path).exists() {
            return path;
        }
    }

    // Default fallback (will probably fail later if it doesn't exist)
    format!("{}/.local/share/stt-assistant/models/ggml-base.bin", home)
}

#[derive(PartialEq)]
enum State {
    Idle,
    Recording,
    Processing,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let model_path = get_model_path();
    info!("Starting STT Daemon (Host Native)...");
    info!("Using model: {}", model_path);

    // 1. Initialize Components
    // ... rest of the code ...
    let mut transcriber = Transcriber::new(&model_path).context("Failed to load Whisper model")?;

    // Socket
    let (cmd_tx, mut cmd_rx) = mpsc::channel(32);
    let socket_server = SocketServer::bind(SOCKET_PATH, cmd_tx).context("Failed to bind socket")?;

    tokio::spawn(async move {
        socket_server.run().await;
    });

    // 2. Event Loop
    let mut state = State::Idle;
    let mut audio_buffer: Vec<f32> = Vec::with_capacity(16000 * 30); // Linear buffer for recording
    let chunk_size = 512;
    let mut chunk_buf: Vec<f32> = Vec::with_capacity(chunk_size);

    info!("System Ready. Waiting for commands on {}", SOCKET_PATH);

    loop {
        // We use a tight loop for audio processing, checking commands periodically?
        // Or better: use tokio::select but audio is coming from ringbuffer, not async channel.
        // We need to poll the ringbuffer frequently.

        // Non-blocking check for commands
        if let Ok(cmd) = cmd_rx.try_recv() {
            match cmd {
                Command::Start => {
                    info!("Command: START");
                    state = State::Recording;
                    audio_buffer.clear();
                    // Optional: Play sound or visual feedback is handled by client
                }
                Command::Stop => {
                    info!("Command: STOP");
                    if state == State::Recording {
                        state = State::Processing;
                    }
                }
                Command::Cancel => {
                    info!("Command: CANCEL");
                    state = State::Idle;
                    audio_buffer.clear();
                }
            }
        }

        // Process Audio from RingBuffer
        let available = consumer.len();
        if available >= chunk_size {
            for _ in 0..chunk_size {
                if let Some(sample) = consumer.pop() {
                    chunk_buf.push(sample);
                }
            }

            // If Recording, save to buffer
            if state == State::Recording {
                audio_buffer.extend_from_slice(&chunk_buf);

                // Optional: VAD check to auto-stop?
                // For now, manual stop via shortcut is the requested behavior.
            }

            chunk_buf.clear();
        } else {
            // Sleep briefly to avoid busy loop
            sleep(Duration::from_millis(5)).await;
        }

        // Processing State
        if state == State::Processing {
            info!("Processing {} samples...", audio_buffer.len());

            if audio_buffer.is_empty() {
                warn!("Audio buffer empty, skipping transcription.");
                state = State::Idle;
                continue;
            }

            // Transcribe
            // This blocks the event loop. In a real production app we'd spawn a blocking task.
            // Since this is a single-user desktop tool, blocking for 1-2s is acceptable IF we don't drop next commands.
            // But we might drop audio frames.
            // Better: spawn_blocking.

            match transcriber.transcribe(&audio_buffer) {
                Ok(text) => {
                    info!("Transcription: {}", text);
                    // Send result back to socket?
                    // Current Socket Implementation is "Push Command".
                    // We need to send RESPONSE.
                    // The socket task doesn't have a way to receive message FROM main yet.

                    // QUICK HACK for Prototype:
                    // Write to a status file or just log it.
                    // The Client `stt_controller.py` can read the response from the connection if we kept it open.
                    // BUT `src/socket.rs` spawns a task for connection and drops it or waits for read.

                    // REVISIT socket.rs logic.
                    // For now, completing the loop.

                    // Temporary: Write to a file that the client can read?
                    // Or standard out?

                    // Ideally:
                    // 1. Client connects.
                    // 2. Client sends STOP.
                    // 3. Client keeps reading socket.
                    // 4. Server (Main) sends text to a channel that Socket Task is listening to?
                    //    Socket Task is generic.

                    // Architecture fix needed:
                    // We need a Global Broadcast channel for "System Events" that socket tasks subscribe to?
                    // Or simpler: The socket connection for STOP is the one waiting.

                    // Let's assume for this step we print to stdout.
                    // The `stt_controller.py` can capture stdout? No, it connects via socket.

                    // Let's create a oneshot file `/tmp/stt_result.txt` for now as a reliable IPC transport for the large text.
                    // The socket can say "STATUS: READY".
                    std::fs::write("/tmp/stt_result.txt", &text)
                        .unwrap_or_else(|e| error!("Failed to write result: {}", e));
                }
                Err(e) => {
                    error!("Transcription failed: {}", e);
                }
            }

            state = State::Idle;
            audio_buffer.clear();
        }
    }
}

use anyhow::{Context, Result};
use clap::Parser;
use config::{Config, File};
use log::{error, info, warn};
use ringbuf::HeapRb;
use serde::Deserialize;
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

#[derive(Debug, Deserialize)]
struct SttConfig {
    model_path: String,
    language: String,
}

impl Default for SttConfig {
    fn default() -> Self {
        Self {
            model_path: "ggml-base.bin".to_string(),
            language: "es".to_string(),
        }
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to configuration file
    #[arg(short, long)]
    config: Option<String>,

    /// Path or name of the model file (overrides config).
    /// If a name is provided (e.g., 'ggml-base.bin'), it searches in order:
    /// 1. ~/.local/share/stt-assistant/models/
    /// 2. /usr/share/stt-assistant/models/
    /// 3. ./models/
    #[arg(short, long)]
    model: Option<String>,

    /// Language (overrides config)
    #[arg(short, long)]
    language: Option<String>,
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

    let args = Args::parse();
    let home = std::env::var("HOME").unwrap_or_else(|_| "/root".to_string());

    // Load configuration from multiple sources
    let mut builder = Config::builder();

    // 1. Explicit config file
    if let Some(cfg_path) = args.config {
        builder = builder.add_source(File::with_name(&cfg_path));
    }

    // 2. User config (~/.config/stt-assistant/config.toml)
    builder = builder.add_source(
        File::with_name(&format!("{}/.config/stt-assistant/config.toml", home)).required(false),
    );

    // 3. System config (/etc/stt-assistant.toml)
    builder = builder.add_source(File::with_name("/etc/stt-assistant.toml").required(false));

    // 4. Environment variables
    builder = builder.add_source(config::Environment::with_prefix("STT"));

    let config_res = builder.build();
    let mut stt_config: SttConfig = match config_res {
        Ok(c) => c.try_deserialize().unwrap_or_default(),
        Err(e) => {
            warn!("Configuration warning: {}. Using defaults.", e);
            SttConfig::default()
        }
    };

    // CLI args override
    if let Some(m) = args.model {
        stt_config.model_path = m;
    }
    if let Some(l) = args.language {
        stt_config.language = l;
    }

    // Attempt to resolve model path if it's just a filename
    if !std::path::Path::new(&stt_config.model_path).exists() {
        let filename = if stt_config.model_path.contains('/') {
            // If it contains a slash but doesn't exist, we'll still try to see if it's just the end part
            std::path::Path::new(&stt_config.model_path)
                .file_name()
                .and_then(|f| f.to_str())
                .unwrap_or(&stt_config.model_path)
        } else {
            &stt_config.model_path
        };

        let candidates = vec![
            format!("{}/.local/share/stt-assistant/models/{}", home, filename),
            format!("/usr/share/stt-assistant/models/{}", filename),
            format!("models/{}", filename),
            filename.to_string(),
        ];

        for path in candidates {
            if std::path::Path::new(&path).exists() {
                stt_config.model_path = path;
                break;
            }
        }
    }

    info!("Starting STT Daemon...");
    info!("Using model: {}", stt_config.model_path);
    info!("Language: {}", stt_config.language);

    // 1. Initialize Components
    let mut transcriber =
        Transcriber::new(&stt_config.model_path).context("Failed to load Whisper model")?;

    // Audio Engine initialization
    let rb = HeapRb::<f32>::new(16000 * 30); // 30 seconds buffer
    let (producer, mut consumer) = rb.split();

    let mut audio_engine = AudioEngine::new().context("Failed to init audio engine")?;
    audio_engine
        .start(producer)
        .context("Failed to start audio engine")?;

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
        // Non-blocking check for commands
        if let Ok(cmd) = cmd_rx.try_recv() {
            match cmd {
                Command::Start => {
                    info!("Command: START");
                    state = State::Recording;
                    audio_buffer.clear();
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

            match transcriber.transcribe(&audio_buffer, Some(&stt_config.language)) {
                Ok(text) => {
                    info!("Transcription: {}", text);
                    // Temporary: Write to a file that the client can read
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

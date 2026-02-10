use anyhow::{Context, Result};
use log::{error, info};
use std::os::unix::fs::PermissionsExt;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixListener;
use tokio::sync::{mpsc, oneshot};

#[derive(Debug)]
pub enum Command {
    Start,
    Stop {
        response_tx: oneshot::Sender<String>,
    },
    Cancel,
}

pub struct SocketServer {
    listener: UnixListener,
    cmd_tx: mpsc::Sender<Command>,
}

impl SocketServer {
    pub fn bind(path: &str, cmd_tx: mpsc::Sender<Command>) -> Result<Self> {
        if std::fs::metadata(path).is_ok() {
            info!("Removing existing socket file: {}", path);
            std::fs::remove_file(path).context("Failed to remove existing socket")?;
        }

        let listener = UnixListener::bind(path).context("Failed to bind unix socket")?;

        // Set permissions to 0600 (owner read/write only)
        let mut perms = std::fs::metadata(path)?.permissions();
        perms.set_mode(0o600);
        std::fs::set_permissions(path, perms).context("Failed to set socket permissions")?;

        info!("Listening on unix socket: {} (restricted to 0600)", path);

        Ok(Self { listener, cmd_tx })
    }

    pub async fn run(&self) {
        loop {
            match self.listener.accept().await {
                Ok((mut stream, _addr)) => {
                    let cmd_tx = self.cmd_tx.clone();
                    tokio::spawn(async move {
                        let mut buf = [0; 1024];
                        match stream.read(&mut buf).await {
                            Ok(n) if n > 0 => {
                                let command_str =
                                    String::from_utf8_lossy(&buf[..n]).trim().to_string();
                                info!("Received command: {}", command_str);

                                match command_str.as_str() {
                                    "START" => {
                                        if let Err(e) = cmd_tx.send(Command::Start).await {
                                            error!("Failed to send start command: {}", e);
                                            let _ = stream
                                                .write_all(b"ERROR: Internal channel error")
                                                .await;
                                        } else {
                                            let _ = stream.write_all(b"STATUS: RECORDING").await;
                                        }
                                    }
                                    "STOP" => {
                                        let (tx, rx) = oneshot::channel();
                                        if let Err(e) =
                                            cmd_tx.send(Command::Stop { response_tx: tx }).await
                                        {
                                            error!("Failed to send stop command: {}", e);
                                            let _ = stream
                                                .write_all(b"ERROR: Internal channel error")
                                                .await;
                                        } else {
                                            // Wait for the transcription result from the main loop
                                            match rx.await {
                                                Ok(text) => {
                                                    let _ = stream.write_all(text.as_bytes()).await;
                                                }
                                                Err(_) => {
                                                    let _ = stream.write_all(b"ERROR: Transcription cancelled or failed").await;
                                                }
                                            }
                                        }
                                    }
                                    "CANCEL" => {
                                        let _ = cmd_tx.send(Command::Cancel).await;
                                        let _ = stream.write_all(b"STATUS: CANCELLED").await;
                                    }
                                    _ => {
                                        let _ = stream.write_all(b"ERROR: Unknown command").await;
                                    }
                                };

                                // TODO: Implementing full bidirectional wait for transcription is tricky here without a shared state or response channel.
                                // Quick fix: The main loop will handle the logic, but how does it send back to THIS stream?
                                // Architecture choice:
                                // 1. Client connects, sends STOP, waits.
                                // 2. Socket task sends StopRecording to Main.
                                // 3. Socket task waits for Result from Main (via oneshot channel?).
                                // 4. Socket task writes Result to Stream.
                                //
                                // Let's implement that pattern in the next step (Main).
                                // For now, this is a good skeleton.
                            }
                            Ok(_) => {} // EOF
                            Err(e) => error!("Failed to read from socket: {}", e),
                        }
                    });
                }
                Err(e) => error!("Failed to accept connection: {}", e),
            }
        }
    }
}

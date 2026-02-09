use anyhow::{Context, Result};
use log::{error, info};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixListener;
use tokio::sync::mpsc;

#[derive(Debug)]
pub enum Command {
    Start,
    Stop,
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
        info!("Listening on unix socket: {}", path);

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

                                let response = match command_str.as_str() {
                                    "START" => {
                                        if let Err(e) = cmd_tx.send(Command::Start).await {
                                            error!("Failed to send start command: {}", e);
                                            Some("ERROR: Internal channel error")
                                        } else {
                                            Some("STATUS: RECORDING")
                                        }
                                    }
                                    "STOP" => {
                                        if let Err(e) = cmd_tx.send(Command::Stop).await {
                                            error!("Failed to send stop command: {}", e);
                                            Some("ERROR: Internal channel error")
                                        } else {
                                            // The actual transcription will be sent by the main loop via a separate connection or mechanism?
                                            // Ideally we want to keep THIS connection open to send the result back.
                                            // But for now let's say we acknowledge the STOP.
                                            // WAIT! The architecture implies the CLIENT waits for the response on the SAME connection.
                                            // So we should probably hold the connection?
                                            // Complex: The MAIN LOOP does the processing.
                                            // Simplified for now: We just signal the main loop.
                                            // The main loop needs a way to send back the transcription.
                                            // For this iteration, let's assume the main loop will write to a 'response' channel we pass?
                                            // No, that's too complex for this step.
                                            // Let's make it simpler: Client sends STOP. Server says "PROCESSING".
                                            // Client waits. Server finishes processing and needs to send "RESULT: ...".
                                            // Hmmm. The `stt_controller.py` will wait on the socket.
                                            // So we need to keep `stream` alive? But we are spawning a task.

                                            // Alternative: `stt_controller.py` sends STOP and waits.
                                            // We need to bridge the Main Thread (Whisper) result back to this socket task.
                                            // This requires a response channel.

                                            Some("STATUS: PROCESSING")
                                        }
                                    }
                                    "CANCEL" => {
                                        let _ = cmd_tx.send(Command::Cancel).await;
                                        Some("STATUS: CANCELLED")
                                    }
                                    _ => Some("ERROR: Unknown command"),
                                };

                                if let Some(resp) = response
                                    && let Err(e) = stream.write_all(resp.as_bytes()).await
                                {
                                    error!("Failed to write response: {}", e);
                                }

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

use anyhow::{Context, Result};
use std::path::Path;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{UnixListener, UnixStream};
use tokio::time::{Duration, sleep};

pub const DAEMON_SOCKET: &str = "/tmp/stt-sock";
pub const CONTROL_SOCKET: &str = "/tmp/stt-control.sock";
const RESULT_FILE: &str = "/tmp/stt_result.txt";

pub struct SocketClient;

impl SocketClient {
    pub async fn send_command(cmd: &str) -> Result<()> {
        let mut stream = UnixStream::connect(DAEMON_SOCKET)
            .await
            .context("Failed to connect to daemon")?;
        stream
            .write_all(cmd.as_bytes())
            .await
            .context("Failed to send command")?;
        Ok(())
    }

    pub async fn send_control_command(cmd: &str) -> Result<()> {
        let mut stream = UnixStream::connect(CONTROL_SOCKET)
            .await
            .context("Failed to connect to control socket (is the GUI running?)")?;
        stream
            .write_all(cmd.as_bytes())
            .await
            .context("Failed to send control command")?;
        Ok(())
    }

    pub async fn wait_for_result(timeout_s: u64) -> Option<String> {
        let start = std::time::Instant::now();
        while start.elapsed().as_secs_f32() < timeout_s as f32 {
            if Path::new(RESULT_FILE).exists() {
                match std::fs::read_to_string(RESULT_FILE) {
                    Ok(text) if !text.trim().is_empty() => {
                        let _ = std::fs::remove_file(RESULT_FILE);
                        return Some(text);
                    }
                    _ => {}
                }
            }
            sleep(Duration::from_millis(100)).await;
        }
        None
    }
}

pub struct ControlServer {
    listener: UnixListener,
}

impl ControlServer {
    pub fn bind() -> Result<Self> {
        if Path::new(CONTROL_SOCKET).exists() {
            let _ = std::fs::remove_file(CONTROL_SOCKET);
        }
        let listener =
            UnixListener::bind(CONTROL_SOCKET).context("Failed to bind control socket")?;
        Ok(Self { listener })
    }

    pub async fn next_command(&self) -> Result<String> {
        let (mut stream, _) = self.listener.accept().await?;
        let mut buf = [0; 1024];
        let n = stream.read(&mut buf).await?;
        Ok(String::from_utf8_lossy(&buf[..n]).trim().to_string())
    }
}

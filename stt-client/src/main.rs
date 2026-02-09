use async_channel::Sender;
use gtk4::prelude::*;
use gtk4::{Application, glib};
use std::thread;
use tokio::runtime::Runtime;
use tokio::sync::mpsc;

use log::info;

mod connection;
mod input;
mod ui;

use connection::{ControlServer, SocketClient};
use ui::Osd;

#[derive(Debug, Clone)]
enum AppAction {
    ToggleRecording(String), // "TYPE" or "COPY"
    CancelRecording,
    OsdUpdate(String, String), // Text, Color
    OsdHide,
}

#[derive(Debug)]
enum DaemonCommand {
    Start,
    Stop {
        mode: String,
        response_tx: Sender<AppAction>,
    },
    Cancel,
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    // Check CLI args
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        let cmd = &args[1];
        let rt = Runtime::new().expect("Failed to create Tokio runtime");
        rt.block_on(async {
            match SocketClient::send_control_command(cmd).await {
                Ok(_) => info!("Command '{}' sent successfully.", cmd),
                Err(e) => log::error!("Failed to send command: {}", e),
            }
        });
        return;
    }

    // Initialize GTK Application
    let app = Application::builder()
        .application_id("com.stt.client")
        .build();

    app.connect_activate(move |app| {
        // Keep the app running even without visible windows
        let _hold_guard = app.hold();

        // Create async channel for communication between Tokio and GTK
        let (tx, rx) = async_channel::unbounded::<AppAction>();

        // Create mpsc channel for sending commands TO the Tokio runtime
        let (daemon_tx, daemon_rx) = mpsc::unbounded_channel::<DaemonCommand>();

        // Start Tokio Runtime in a separate thread
        // This happens AFTER GTK confirms we're the primary instance
        let tx_clone = tx.clone();
        thread::spawn(move || {
            let rt = Runtime::new().expect("Failed to create Tokio runtime");
            rt.block_on(async {
                tokio::select! {
                    result = run_control_server(tx_clone.clone()) => {
                        if let Err(e) = result {
                            log::error!("Control server failed: {}", e);
                        }
                    }
                    _ = handle_daemon_commands(daemon_rx, tx_clone) => {}
                }
            });
        });

        let osd = Osd::new(app);
        let osd_clone = osd.clone();
        let tx_back = tx.clone();

        // GTK Main Loop Context
        glib::MainContext::default().spawn_local(async move {
            let mut recording = false;
            let mut last_type = String::new(); // "TYPE" or "COPY"

            while let Ok(action) = rx.recv().await {
                match action {
                    AppAction::ToggleRecording(mode) => {
                        if !recording {
                            // START
                            recording = true;
                            last_type = mode;
                            osd_clone.show("● GRABANDO", "red");
                            let _ = daemon_tx.send(DaemonCommand::Start);
                        } else {
                            // STOP
                            recording = false;
                            osd_clone.show("Procesando...", "orange");
                            let _ = daemon_tx.send(DaemonCommand::Stop {
                                mode: last_type.clone(),
                                response_tx: tx_back.clone(),
                            });
                        }
                    }
                    AppAction::CancelRecording => {
                        if recording {
                            recording = false;
                            osd_clone.show("Cancelado", "gray");
                            let _ = daemon_tx.send(DaemonCommand::Cancel);
                            // Delay hide
                            let tx_inner = tx_back.clone();
                            glib::timeout_add_seconds_local(1, move || {
                                let _ = tx_inner.send_blocking(AppAction::OsdHide);
                                glib::ControlFlow::Break
                            });
                        }
                    }
                    AppAction::OsdUpdate(text, color) => {
                        osd_clone.show(&text, &color);
                    }
                    AppAction::OsdHide => {
                        osd_clone.hide();
                    }
                }
            }
        });
    });

    app.run();
}

async fn handle_daemon_commands(
    mut rx: mpsc::UnboundedReceiver<DaemonCommand>,
    _tx: Sender<AppAction>,
) {
    while let Some(cmd) = rx.recv().await {
        match cmd {
            DaemonCommand::Start => {
                let _ = SocketClient::send_command("START").await;
            }
            DaemonCommand::Stop { mode, response_tx } => {
                let _ = SocketClient::send_command("STOP").await;
                // Wait for result
                if let Some(text) = SocketClient::wait_for_result(5).await {
                    if mode == "TYPE" {
                        input::type_text(&text);
                        let _ = response_tx.send(AppAction::OsdHide).await;
                    } else {
                        input::copy_text(&text);
                        let _ = response_tx
                            .send(AppAction::OsdUpdate(
                                "Copiado".to_string(),
                                "green".to_string(),
                            ))
                            .await;
                        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                        let _ = response_tx.send(AppAction::OsdHide).await;
                    }
                } else {
                    let _ = response_tx.send(AppAction::OsdHide).await;
                }
            }
            DaemonCommand::Cancel => {
                let _ = SocketClient::send_command("CANCEL").await;
            }
        }
    }
}

async fn run_control_server(tx: Sender<AppAction>) -> anyhow::Result<()> {
    let server = ControlServer::bind()?;
    info!("Control server listening...");

    loop {
        match server.next_command().await {
            Ok(cmd) => {
                info!("Control command: {}", cmd);
                match cmd.as_str() {
                    "TOGGLE_TYPE" => {
                        let _ = tx
                            .send(AppAction::ToggleRecording("TYPE".to_string()))
                            .await;
                    }
                    "TOGGLE_COPY" => {
                        let _ = tx
                            .send(AppAction::ToggleRecording("COPY".to_string()))
                            .await;
                    }
                    "CANCEL" => {
                        let _ = tx.send(AppAction::CancelRecording).await;
                    }
                    _ => {}
                }
            }
            Err(e) => {
                log::error!("Control server error: {}", e);
            }
        }
    }
}

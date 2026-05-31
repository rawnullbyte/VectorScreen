mod app;
mod config;
mod logging;
mod moonraker;
mod ui;
mod util;
mod klipper;
mod system;

slint::include_modules!();

use std::collections::HashMap;
use std::io::Write;
use std::time::Duration;

use moonraker::message::Notification;
use moonraker::MoonrakerConfig;
use tokio::sync::mpsc;
use ui::console::ConsoleState;
use ui::motion::{MotionControl, MovementSpeed};
use ui::led::LedControl;
use ui::tuning::TuningControl;
use app::update_ui;

/// Commands from UI to backend.
pub(crate) enum UiCommand {
EmergencyStop,
SendGcode(String),
HomeAll,
HomeAxis(char),
MoveAxis { axis: char, distance: f64 },
SetSpeed(MovementSpeed),
ToggleLed,
SetLedBrightness(u8),
    ConsoleSendGcode(String),
    RefreshFiles,
    StartPrint(String),
    DeleteFile(String),
    CalibrateMesh,
    RunShaperCalibration,
    RefreshWifiNetworks,
    DisconnectWifi,
    ConnectWifi { ssid: String, password: String },
    SetPressureAdvance(f64),
    SetMaxVelocity { x: f64, y: f64, z: f64 },
    SetMaxAcceleration { x: f64, y: f64, z: f64 },
    SetSquareCornerVelocity(f64),
    StartTouchCal,
}



fn main() -> Result<(), slint::PlatformError> {
    let app_config = config::Config::load();
    logging::init_logging(&app_config.logging.level, None);

    tracing::info!("VectorScreen starting up");

    // Install panic handler that logs to file for embedded debugging.
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let thread = std::thread::current();
        let thread_name = thread.name().unwrap_or("unnamed");
        let payload = if let Some(s) = info.payload().downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "Box<dyn Any>".to_string()
        };
        let location = info
            .location()
            .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
            .unwrap_or_else(|| "unknown".to_string());
        let msg = format!("panic on thread '{thread_name}': {payload}\n  at {location}\n");
        if let Ok(mut f) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("/tmp/vector-screen-panic.log")
        {
            let _ = f.write_all(msg.as_bytes());
        }
        // Still print to stderr for non-embedded debugging.
        default_hook(info);
    }));

    let main_window = AppWindow::new()?;

    let printer_ip = app_config.printer.ip.clone();
    let printer_port = app_config.printer.port;

    let (cmd_tx, cmd_rx) = mpsc::unbounded_channel::<UiCommand>();
    let weak_window = main_window.as_weak();

    // Shared state for the console screen (UI thread only)
    let console_state = std::sync::Arc::new(std::sync::Mutex::new(ConsoleState::new()));

    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(run_backend(printer_ip, printer_port, cmd_rx, weak_window));
    });

    app::wire_callbacks(&main_window, &cmd_tx, &console_state);

    main_window.run()
}

async fn run_backend(
    printer_ip: String,
    printer_port: u16,
    mut cmd_rx: mpsc::UnboundedReceiver<UiCommand>,
    weak_window: slint::Weak<AppWindow>,
) {
    let http_url = format!("http://{}:{}", printer_ip, printer_port);
    let ws_url = format!("ws://{}:{}/websocket", printer_ip, printer_port);

    let config = MoonrakerConfig {
        http_url,
        ws_url,
        ..Default::default()
    };

    let mut client = moonraker::MoonrakerClient::new(config);
    let (_, dummy_rx) = mpsc::unbounded_channel();
    let mut notification_rx = std::mem::replace(&mut client.notification_rx, dummy_rx);
    let mut motion = MotionControl::new();
    let mut led = LedControl::new();
    let mut tuning = TuningControl::new();

    let mut objects = HashMap::new();
    objects.insert("extruder".to_string(), None);
    objects.insert("heater_bed".to_string(), None);
    objects.insert("toolhead".to_string(), None);
    objects.insert("print_stats".to_string(), None);

    let mut subscribed = false;

    let ctrl_c = tokio::signal::ctrl_c();
    tokio::pin!(ctrl_c);

    loop {
        tokio::select! {
            notif = notification_rx.recv() => {
                match notif {
                    Some(Notification::KlippyReady) if !subscribed => {
                        if let Ok(Ok(())) = tokio::time::timeout(
                            Duration::from_secs(5),
                            client.subscribe_objects(objects.clone())
                        ).await {
                            subscribed = true;
                            update_ui(&weak_window, |w| { w.set_ws_connected(true); });
                        }
                    }
                    Some(n) => app::handle_notification(n, &weak_window, &mut motion),
                    None => {
                        update_ui(&weak_window, |w| { w.set_ws_connected(false); });
                        break;
                    }
                }
            }
            cmd = cmd_rx.recv() => {
                match cmd {
                    Some(c) => app::handle_command(c, &client, &weak_window, &mut motion, &mut led, &mut tuning).await,
                    None => break,
                }
            }
            _ = &mut ctrl_c => {
                tracing::info!("Ctrl+C received, shutting down gracefully");
                client.shutdown().await;
                update_ui(&weak_window, |w| {
                    w.set_ws_connected(false);
                    w.set_error_visible(true);
                    w.set_error_message("Shutting down...".into());
                });
                break;
            }
        }
    }
}


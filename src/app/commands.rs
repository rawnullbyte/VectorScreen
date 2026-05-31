use crate::UiCommand;
use crate::moonraker::MoonrakerClient;
use crate::AppWindow;
use crate::ui::motion::MotionControl;
use crate::ui::led::LedControl;
use crate::ui::tuning::TuningControl;

use super::{update_positions, update_ui};

pub(crate) async fn handle_command(
    cmd: UiCommand,
    client: &MoonrakerClient,
    weak_window: &slint::Weak<AppWindow>,
    motion: &mut MotionControl,
    led: &mut LedControl,
    tuning: &mut TuningControl,
) {
    match cmd {
        UiCommand::EmergencyStop => {
            handle_emergency_stop(client, weak_window).await;
        }
        UiCommand::SendGcode(gcode) => {
            handle_send_gcode(client, weak_window, &gcode).await;
        }
        UiCommand::HomeAll => {
            handle_home_all(client, weak_window, motion).await;
        }
        UiCommand::HomeAxis(axis) => {
            handle_home_axis(client, weak_window, motion, axis).await;
        }
        UiCommand::MoveAxis { axis, distance } => {
            handle_move_axis(client, weak_window, motion, axis, distance).await;
        }
        UiCommand::SetSpeed(speed) => {
            motion.set_speed(speed);
        }
        UiCommand::ToggleLed => {
            led.toggle();
            let gcode = if led.is_on() {
                led.led_on_gcode()
            } else {
                led.led_off_gcode()
            };
            if let Err(e) = client.send_gcode(&gcode).await {
                update_ui(weak_window, move |w| {
                    w.set_error_visible(true);
                    w.set_error_message(format!("LED command failed: {e}").into());
                });
            }
        }
        UiCommand::SetLedBrightness(b) => {
            led.set_brightness(b);
            let gcode = led.set_brightness_gcode();
            if let Err(e) = client.send_gcode(&gcode).await {
                update_ui(weak_window, move |w| {
                    w.set_error_visible(true);
                    w.set_error_message(format!("LED brightness failed: {e}").into());
                });
            }
        }
        UiCommand::ConsoleSendGcode(gcode) => {
            handle_console_gcode(client, weak_window, &gcode).await;
        }
        UiCommand::RefreshFiles => {
            tracing::info!("RefreshFiles command received");
        }
        UiCommand::StartPrint(path) => {
            if let Err(e) = client.send_gcode(&format!("PRINT_FILE FILENAME={}", path)).await {
                update_ui(weak_window, move |w| {
                    w.set_error_visible(true);
                    w.set_error_message(format!("Print failed: {e}").into());
                });
            }
        }
        UiCommand::DeleteFile(path) => {
            if let Err(e) = client.send_gcode(&format!("DELETE_FILE FILENAME={}", path)).await {
                update_ui(weak_window, move |w| {
                    w.set_error_visible(true);
                    w.set_error_message(format!("Delete failed: {e}").into());
                });
            }
        }
        UiCommand::CalibrateMesh => {
            if let Err(e) = client.send_gcode("BED_MESH_CALIBRATE").await {
                update_ui(weak_window, move |w| {
                    w.set_error_visible(true);
                    w.set_error_message(format!("Bed mesh calibration failed: {e}").into());
                });
            }
        }
        UiCommand::RunShaperCalibration => {
            if let Err(e) = client.send_gcode("SHAPER_CALIBRATE").await {
                update_ui(weak_window, move |w| {
                    w.set_error_visible(true);
                    w.set_error_message(format!("Input shaper calibration failed: {e}").into());
                });
            }
        }
        UiCommand::RefreshWifiNetworks => {
            if let Err(e) = client.send_gcode("WIFI_SCAN").await {
                update_ui(weak_window, move |w| {
                    w.set_error_visible(true);
                    w.set_error_message(format!("WiFi scan failed: {e}").into());
                });
            }
        }
        UiCommand::DisconnectWifi => {
            if let Err(e) = client.send_gcode("WIFI_DISCONNECT").await {
                update_ui(weak_window, move |w| {
                    w.set_error_visible(true);
                    w.set_error_message(format!("WiFi disconnect failed: {e}").into());
                });
            }
        }
        UiCommand::ConnectWifi { ssid, password } => {
            let gcode = format!("WIFI_CONNECT SSID=\"{}\" PASSWORD=\"{}\"", ssid, password);
            if let Err(e) = client.send_gcode(&gcode).await {
                update_ui(weak_window, move |w| {
                    w.set_error_visible(true);
                    w.set_error_message(format!("WiFi connect failed: {e}").into());
                });
            }
        }
        UiCommand::SetPressureAdvance(value) => {
            tuning.pressure_advance = value;
            let gcode = tuning.pressure_advance_gcode();
            if let Err(e) = client.send_gcode(&gcode).await {
                update_ui(weak_window, move |w| {
                    w.set_error_visible(true);
                    w.set_error_message(format!("Pressure advance failed: {e}").into());
                });
            }
        }
        UiCommand::SetMaxVelocity { x, y, z } => {
            let gcode = format!("SET_VELOCITY_LIMIT VELOCITY={:.1}", x.max(y).max(z));
            if let Err(e) = client.send_gcode(&gcode).await {
                update_ui(weak_window, move |w| {
                    w.set_error_visible(true);
                    w.set_error_message(format!("Velocity limit failed: {e}").into());
                });
            }
        }
        UiCommand::SetMaxAcceleration { x, y, z } => {
            let gcode = format!("SET_VELOCITY_LIMIT ACCEL={:.0}", x.max(y).max(z));
            if let Err(e) = client.send_gcode(&gcode).await {
                update_ui(weak_window, move |w| {
                    w.set_error_visible(true);
                    w.set_error_message(format!("Acceleration limit failed: {e}").into());
                });
            }
        }
        UiCommand::SetSquareCornerVelocity(value) => {
            let gcode = format!("SET_VELOCITY_LIMIT SQUARE_CORNER_VELOCITY={:.1}", value);
            if let Err(e) = client.send_gcode(&gcode).await {
                update_ui(weak_window, move |w| {
                    w.set_error_visible(true);
                    w.set_error_message(format!("Square corner velocity failed: {e}").into());
                });
            }
        }
        UiCommand::StartTouchCal => {
            if let Err(e) = client.send_gcode("TOUCH_CALIBRATE").await {
                update_ui(weak_window, move |w| {
                    w.set_error_visible(true);
                    w.set_error_message(format!("Touch calibration failed: {e}").into());
                });
            }
        }
    }
}

async fn handle_emergency_stop(
    client: &MoonrakerClient,
    weak_window: &slint::Weak<AppWindow>,
) {
    if let Err(e) = client.emergency_stop().await {
        update_ui(weak_window, move |w| {
            w.set_error_visible(true);
            w.set_error_message(format!("Emergency stop failed: {e}").into());
        });
    }
}

async fn handle_send_gcode(
    client: &MoonrakerClient,
    weak_window: &slint::Weak<AppWindow>,
    gcode: &str,
) {
    if let Err(e) = client.send_gcode(gcode).await {
        update_ui(weak_window, move |w| {
            w.set_error_visible(true);
            w.set_error_message(format!("Command failed: {e}").into());
        });
    }
}

async fn handle_home_all(
    client: &MoonrakerClient,
    weak_window: &slint::Weak<AppWindow>,
    motion: &mut MotionControl,
) {
    let gcode = motion.home_all_gcode();
    if let Err(e) = client.send_gcode(&gcode).await {
        update_ui(weak_window, move |w| {
            w.set_error_visible(true);
            w.set_error_message(format!("Home failed: {e}").into());
        });
    } else {
        motion.reset_positions();
        update_positions(weak_window, motion);
    }
}

async fn handle_home_axis(
    client: &MoonrakerClient,
    weak_window: &slint::Weak<AppWindow>,
    motion: &mut MotionControl,
    axis: char,
) {
    let gcode = motion.home_axis_gcode(axis);
    if let Err(e) = client.send_gcode(&gcode).await {
        update_ui(weak_window, move |w| {
            w.set_error_visible(true);
            w.set_error_message(format!("Home failed: {e}").into());
        });
    }
}

async fn handle_move_axis(
    client: &MoonrakerClient,
    weak_window: &slint::Weak<AppWindow>,
    motion: &mut MotionControl,
    axis: char,
    distance: f64,
) {
    let gcode = motion.move_axis_gcode(axis, distance);
    if let Err(e) = client.send_gcode(&gcode).await {
        update_ui(weak_window, move |w| {
            w.set_error_visible(true);
            w.set_error_message(format!("Move failed: {e}").into());
        });
    } else {
        motion.update_position(axis, distance);
        update_positions(weak_window, motion);
    }
}

async fn handle_console_gcode(
    client: &MoonrakerClient,
    weak_window: &slint::Weak<AppWindow>,
    gcode: &str,
) {
    let response = match client.send_gcode(gcode).await {
        Ok(resp) => format!("> {}\n{}", gcode, resp),
        Err(e) => format!("> {}\nError: {}", gcode, e),
    };
    update_ui(weak_window, move |w| {
        let current = w.get_console_response().to_string();
        let new_text = if current.is_empty() {
            response
        } else {
            format!("{}\n{}", current, response)
        };
        // Trim to last 50 lines to prevent unbounded growth
        let line_count = new_text.lines().count();
        let trimmed = if line_count > 50 {
            new_text
                .lines()
                .skip(line_count - 50)
                .collect::<Vec<_>>()
                .join("\n")
        } else {
            new_text
        };
        w.set_console_response(trimmed.into());
    });
}

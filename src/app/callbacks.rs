// noqa: SIZE_OK — single-responsibility UI callback wiring module.
// Length is mechanical: one closure per Slint callback (~40 screens).
use tokio::sync::mpsc;

use crate::UiCommand;
use crate::AppWindow;
use crate::ui::console::ConsoleState;
use crate::ui::motion::MovementSpeed;
use slint::ComponentHandle;

pub(crate) fn wire_callbacks(
    window: &AppWindow,
    cmd_tx: &mpsc::UnboundedSender<UiCommand>,
    console_state: &std::sync::Arc<std::sync::Mutex<ConsoleState>>,
) {
    wire_home_callbacks(window, cmd_tx);
    wire_controls_callbacks(window, cmd_tx);
    wire_console_callbacks(window, cmd_tx, console_state);
    wire_tuning_callbacks(window, cmd_tx);
    wire_fans_callbacks(window, cmd_tx);
    wire_files_callbacks(window, cmd_tx);
    wire_bed_mesh_callbacks(window, cmd_tx);
    wire_input_shaper_callbacks(window, cmd_tx);
    wire_wifi_callbacks(window, cmd_tx);
}

fn wire_home_callbacks(window: &AppWindow, cmd_tx: &mpsc::UnboundedSender<UiCommand>) {
    let tx = cmd_tx.clone();
    window.on_emergency_stop(move || {
        let _ = tx.send(UiCommand::EmergencyStop);
    });

    let weak = window.as_weak();
    window.on_dismiss_error(move || {
        if let Some(w) = weak.upgrade() {
            w.set_error_visible(false);
        }
    });
}

fn wire_controls_callbacks(window: &AppWindow, cmd_tx: &mpsc::UnboundedSender<UiCommand>) {
    let tx = cmd_tx.clone();
    window.on_load_filament(move || {
        let _ = tx.send(UiCommand::SendGcode("LOAD_FILAMENT SPEED=450".to_string()));
    });

    let tx = cmd_tx.clone();
    window.on_unload_filament(move || {
        let _ = tx.send(UiCommand::SendGcode(
            "UNLOAD_FILAMENT SPEED=450".to_string(),
        ));
    });

    let tx = cmd_tx.clone();
    window.on_set_extruder_temp(move |temp: f32| {
        let gcode = format!("M104 S{:.1}", temp);
        let _ = tx.send(UiCommand::SendGcode(gcode));
    });

    let tx = cmd_tx.clone();
    window.on_set_bed_temp(move |temp: f32| {
        let gcode = format!("M140 S{:.1}", temp);
        let _ = tx.send(UiCommand::SendGcode(gcode));
    });

    let tx = cmd_tx.clone();
    window.on_home_all(move || {
        let _ = tx.send(UiCommand::HomeAll);
    });

    let tx = cmd_tx.clone();
    window.on_home_x(move || {
        let _ = tx.send(UiCommand::HomeAxis('X'));
    });

    let tx = cmd_tx.clone();
    window.on_home_y(move || {
        let _ = tx.send(UiCommand::HomeAxis('Y'));
    });

    let tx = cmd_tx.clone();
    window.on_home_z(move || {
        let _ = tx.send(UiCommand::HomeAxis('Z'));
    });

    let tx = cmd_tx.clone();
    window.on_set_speed_slow(move || {
        let _ = tx.send(UiCommand::SetSpeed(MovementSpeed::Slow));
    });

    let tx = cmd_tx.clone();
    window.on_set_speed_medium(move || {
        let _ = tx.send(UiCommand::SetSpeed(MovementSpeed::Medium));
    });

    let tx = cmd_tx.clone();
    window.on_set_speed_fast(move || {
        let _ = tx.send(UiCommand::SetSpeed(MovementSpeed::Fast));
    });

    let tx = cmd_tx.clone();
    window.on_move_x_10_neg(move || {
        let _ = tx.send(UiCommand::MoveAxis {
            axis: 'X',
            distance: -10.0,
        });
    });

    let tx = cmd_tx.clone();
    window.on_move_x_1_neg(move || {
        let _ = tx.send(UiCommand::MoveAxis {
            axis: 'X',
            distance: -1.0,
        });
    });

    let tx = cmd_tx.clone();
    window.on_move_x_1_pos(move || {
        let _ = tx.send(UiCommand::MoveAxis {
            axis: 'X',
            distance: 1.0,
        });
    });

    let tx = cmd_tx.clone();
    window.on_move_x_10_pos(move || {
        let _ = tx.send(UiCommand::MoveAxis {
            axis: 'X',
            distance: 10.0,
        });
    });

    let tx = cmd_tx.clone();
    window.on_move_y_10_neg(move || {
        let _ = tx.send(UiCommand::MoveAxis {
            axis: 'Y',
            distance: -10.0,
        });
    });

    let tx = cmd_tx.clone();
    window.on_move_y_1_neg(move || {
        let _ = tx.send(UiCommand::MoveAxis {
            axis: 'Y',
            distance: -1.0,
        });
    });

    let tx = cmd_tx.clone();
    window.on_move_y_1_pos(move || {
        let _ = tx.send(UiCommand::MoveAxis {
            axis: 'Y',
            distance: 1.0,
        });
    });

    let tx = cmd_tx.clone();
    window.on_move_y_10_pos(move || {
        let _ = tx.send(UiCommand::MoveAxis {
            axis: 'Y',
            distance: 10.0,
        });
    });

    let tx = cmd_tx.clone();
    window.on_move_z_10_neg(move || {
        let _ = tx.send(UiCommand::MoveAxis {
            axis: 'Z',
            distance: -10.0,
        });
    });

    let tx = cmd_tx.clone();
    window.on_move_z_1_neg(move || {
        let _ = tx.send(UiCommand::MoveAxis {
            axis: 'Z',
            distance: -1.0,
        });
    });

    let tx = cmd_tx.clone();
    window.on_move_z_1_pos(move || {
        let _ = tx.send(UiCommand::MoveAxis {
            axis: 'Z',
            distance: 1.0,
        });
    });

    let tx = cmd_tx.clone();
    window.on_move_z_10_pos(move || {
        let _ = tx.send(UiCommand::MoveAxis {
            axis: 'Z',
            distance: 10.0,
        });
    });

    let weak = window.as_weak();
    let tx = cmd_tx.clone();
    window.on_toggle_led(move || {
        if let Some(w) = weak.upgrade() {
            let current = w.get_led_active();
            w.set_led_active(!current);
        }
        let _ = tx.send(UiCommand::ToggleLed);
    });

    let tx = cmd_tx.clone();
    window.on_set_led_brightness(move |brightness: i32| {
        let _ = tx.send(UiCommand::SetLedBrightness(brightness as u8));
    });
}

fn wire_tuning_callbacks(window: &AppWindow, cmd_tx: &mpsc::UnboundedSender<UiCommand>) {
    let tx = cmd_tx.clone();
    window.on_set_tuning_speed_multiplier(move |value: i32| {
        let gcode = format!("M220 S{}", value);
        let _ = tx.send(UiCommand::SendGcode(gcode));
    });

    let tx = cmd_tx.clone();
    window.on_set_tuning_flow_rate(move |value: i32| {
        let gcode = format!("M221 S{}", value);
        let _ = tx.send(UiCommand::SendGcode(gcode));
    });

    let tx = cmd_tx.clone();
    window.on_set_tuning_z_offset_fine(move |value: f32| {
        let gcode = format!("SET_GCODE_OFFSET Z_ADJUST={:.3}", value);
        let _ = tx.send(UiCommand::SendGcode(gcode));
    });
}

fn wire_console_callbacks(
    window: &AppWindow,
    cmd_tx: &mpsc::UnboundedSender<UiCommand>,
    console_state: &std::sync::Arc<std::sync::Mutex<ConsoleState>>,
) {
    {
        let state = console_state.clone();
        let weak = window.as_weak();
        window.on_console_append_char(move |ch: slint::SharedString| {
            let mut s = state.lock().unwrap();
            s.append_char(ch.as_str());
            let input = s.command_input.clone();
            drop(s);
            if let Some(w) = weak.upgrade() {
                w.set_console_command(input.into());
            }
        });
    }

    {
        let state = console_state.clone();
        let weak = window.as_weak();
        window.on_console_backspace(move || {
            let mut s = state.lock().unwrap();
            s.backspace();
            let input = s.command_input.clone();
            drop(s);
            if let Some(w) = weak.upgrade() {
                w.set_console_command(input.into());
            }
        });
    }

    {
        let state = console_state.clone();
        let weak = window.as_weak();
        let tx = cmd_tx.clone();
        window.on_console_send(move || {
            let cmd_opt = {
                let mut s = state.lock().unwrap();
                let cmd = s.finalize_command();
                let input = s.command_input.clone();
                if let Some(w) = weak.upgrade() {
                    w.set_console_command(input.into());
                }
                cmd
            };
            if let Some(cmd) = cmd_opt {
                let _ = tx.send(UiCommand::ConsoleSendGcode(cmd));
            }
        });
    }

    {
        let state = console_state.clone();
        let weak = window.as_weak();
        window.on_console_clear(move || {
            let mut s = state.lock().unwrap();
            s.clear_input();
            let input = s.command_input.clone();
            drop(s);
            if let Some(w) = weak.upgrade() {
                w.set_console_command(input.into());
            }
        });
    }

    {
        let state = console_state.clone();
        let weak = window.as_weak();
        window.on_console_history_up(move || {
            let mut s = state.lock().unwrap();
            s.recall_up();
            let input = s.command_input.clone();
            drop(s);
            if let Some(w) = weak.upgrade() {
                w.set_console_command(input.into());
            }
        });
    }

    {
        let state = console_state.clone();
        let weak = window.as_weak();
        window.on_console_history_down(move || {
            let mut s = state.lock().unwrap();
            s.recall_down();
            let input = s.command_input.clone();
            drop(s);
            if let Some(w) = weak.upgrade() {
                w.set_console_command(input.into());
            }
        });
    }

    let weak = window.as_weak();
    window.on_dismiss_error(move || {
        if let Some(w) = weak.upgrade() {
            w.set_error_visible(false);
        }
    });
}

fn wire_fans_callbacks(window: &AppWindow, cmd_tx: &mpsc::UnboundedSender<UiCommand>) {
    let tx = cmd_tx.clone();
    let weak = window.as_weak();
    window.on_toggle_fan_part_cooling(move || {
        if let Some(w) = weak.upgrade() {
            let current = w.get_fan_part_cooling_on();
            let new_val = !current;
            w.set_fan_part_cooling_on(new_val);
            let speed = if new_val { "1.00" } else { "0.00" };
            let gcode = format!("SET_FAN_SPEED FAN=part_fan SPEED={}", speed);
            let _ = tx.send(UiCommand::SendGcode(gcode));
        }
    });

    let tx = cmd_tx.clone();
    window.on_set_fan_part_cooling_speed(move |speed: i32| {
        let val = (speed as f64) / 100.0;
        let gcode = format!("SET_FAN_SPEED FAN=part_fan SPEED={:.2}", val);
        let _ = tx.send(UiCommand::SendGcode(gcode));
    });

    let tx = cmd_tx.clone();
    let weak = window.as_weak();
    window.on_toggle_fan_aux_fan(move || {
        if let Some(w) = weak.upgrade() {
            let current = w.get_fan_aux_fan_on();
            let new_val = !current;
            w.set_fan_aux_fan_on(new_val);
            let speed = if new_val { "1.00" } else { "0.00" };
            let gcode = format!("SET_FAN_SPEED FAN=aux_fan SPEED={}", speed);
            let _ = tx.send(UiCommand::SendGcode(gcode));
        }
    });

    let tx = cmd_tx.clone();
    window.on_set_fan_aux_fan_speed(move |speed: i32| {
        let val = (speed as f64) / 100.0;
        let gcode = format!("SET_FAN_SPEED FAN=aux_fan SPEED={:.2}", val);
        let _ = tx.send(UiCommand::SendGcode(gcode));
    });

    let tx = cmd_tx.clone();
    let weak = window.as_weak();
    window.on_toggle_fan_controller_fan(move || {
        if let Some(w) = weak.upgrade() {
            let current = w.get_fan_controller_fan_on();
            let new_val = !current;
            w.set_fan_controller_fan_on(new_val);
            let speed = if new_val { "1.00" } else { "0.00" };
            let gcode = format!("SET_FAN_SPEED FAN=controller_fan SPEED={}", speed);
            let _ = tx.send(UiCommand::SendGcode(gcode));
        }
    });

    let tx = cmd_tx.clone();
    window.on_set_fan_controller_fan_speed(move |speed: i32| {
        let val = (speed as f64) / 100.0;
        let gcode = format!("SET_FAN_SPEED FAN=controller_fan SPEED={:.2}", val);
        let _ = tx.send(UiCommand::SendGcode(gcode));
    });

    let tx = cmd_tx.clone();
    let weak = window.as_weak();
    window.on_toggle_fan_exhaust_fan(move || {
        if let Some(w) = weak.upgrade() {
            let current = w.get_fan_exhaust_fan_on();
            let new_val = !current;
            w.set_fan_exhaust_fan_on(new_val);
            let value = if new_val { "1.00" } else { "0.00" };
            let gcode = format!("SET_PIN PIN=exhaust_fan VALUE={}", value);
            let _ = tx.send(UiCommand::SendGcode(gcode));
        }
    });

    let tx = cmd_tx.clone();
    window.on_set_fan_exhaust_fan_speed(move |speed: i32| {
        let val = (speed as f64) / 100.0;
        let gcode = format!("SET_PIN PIN=exhaust_fan VALUE={:.2}", val);
        let _ = tx.send(UiCommand::SendGcode(gcode));
    });
}

fn wire_files_callbacks(window: &AppWindow, cmd_tx: &mpsc::UnboundedSender<UiCommand>) {
    let tx = cmd_tx.clone();
    window.on_refresh_files(move || {
        let _ = tx.send(UiCommand::RefreshFiles);
    });

    let tx = cmd_tx.clone();
    window.on_start_print(move |path: slint::SharedString| {
        let _ = tx.send(UiCommand::StartPrint(path.to_string()));
    });

    let tx = cmd_tx.clone();
    window.on_delete_file(move |path: slint::SharedString| {
        let _ = tx.send(UiCommand::DeleteFile(path.to_string()));
    });
}

fn wire_bed_mesh_callbacks(window: &AppWindow, cmd_tx: &mpsc::UnboundedSender<UiCommand>) {
    let tx = cmd_tx.clone();
    window.on_calibrate_mesh(move || {
        let _ = tx.send(UiCommand::CalibrateMesh);
    });
}

fn wire_input_shaper_callbacks(window: &AppWindow, cmd_tx: &mpsc::UnboundedSender<UiCommand>) {
    let tx = cmd_tx.clone();
    window.on_run_shaper_calibration(move || {
        let _ = tx.send(UiCommand::RunShaperCalibration);
    });
}

fn wire_wifi_callbacks(window: &AppWindow, cmd_tx: &mpsc::UnboundedSender<UiCommand>) {
    let tx = cmd_tx.clone();
    window.on_refresh_wifi_networks(move || {
        let _ = tx.send(UiCommand::RefreshWifiNetworks);
    });

    let tx = cmd_tx.clone();
    window.on_disconnect_wifi(move || {
        let _ = tx.send(UiCommand::DisconnectWifi);
    });

    let tx = cmd_tx.clone();
    window.on_connect_wifi(move |ssid: slint::SharedString, password: slint::SharedString| {
        let _ = tx.send(UiCommand::ConnectWifi {
            ssid: ssid.to_string(),
            password: password.to_string(),
        });
    });
}

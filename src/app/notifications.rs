use crate::moonraker::message::Notification;
use crate::AppWindow;
use crate::ui::motion::MotionControl;

use super::update_ui;

use std::sync::{Mutex, OnceLock};
use crate::util::ring_buffer::RingBuffer;

/// Maximum number of temperature data points retained per channel.
/// Prevents unbounded memory growth on long-running ARM sessions.
const TEMP_HISTORY_CAP: usize = 120;

static EXTRUDER_HISTORY: OnceLock<Mutex<RingBuffer<f64>>> = OnceLock::new();
static BED_HISTORY: OnceLock<Mutex<RingBuffer<f64>>> = OnceLock::new();

fn extruder_history() -> &'static Mutex<RingBuffer<f64>> {
    EXTRUDER_HISTORY.get_or_init(|| Mutex::new(RingBuffer::new(TEMP_HISTORY_CAP)))
}

fn bed_history() -> &'static Mutex<RingBuffer<f64>> {
    BED_HISTORY.get_or_init(|| Mutex::new(RingBuffer::new(TEMP_HISTORY_CAP)))
}

pub(crate) fn handle_notification(
    notif: Notification,
    weak_window: &slint::Weak<AppWindow>,
    motion: &mut MotionControl,
) {
    match notif {
        Notification::StatusUpdate { objects, .. } => {
            handle_status_update(objects, weak_window, motion);
        }
        Notification::KlippyDisconnected => {
            update_ui(weak_window, |w| {
                w.set_ws_connected(false);
                w.set_error_visible(true);
                w.set_error_message("Klipper disconnected".into());
            });
        }
        Notification::KlippyError(msg) => {
            update_ui(weak_window, move |w| {
                w.set_error_visible(true);
                w.set_error_message(msg.into());
            });
        }
        Notification::KlippyReady => {
            tracing::info!("Klipper ready");
        }
        Notification::ConnectionState { connected } => {
            update_ui(weak_window, move |w| {
                w.set_ws_connected(connected);
                if !connected {
                    w.set_error_visible(true);
                    w.set_error_message("Connection lost — reconnecting...".into());
                } else {
                    w.set_error_visible(false);
                }
            });
        }
        Notification::KlippyShutdown => {
            tracing::info!("Klipper ready");
        }
        Notification::KlippyShutdown => {
            update_ui(weak_window, |w| {
                w.set_ws_connected(false);
                w.set_error_visible(true);
                w.set_error_message("Klipper shutdown".into());
            });
        }
        Notification::GcodeResponse(resp) => {
            tracing::info!("G-code response: {}", resp);
        }
        Notification::Unknown { method, .. } => {
            tracing::debug!("Unhandled notification: {}", method);
        }
    }
}

fn handle_status_update(
    objects: std::collections::HashMap<String, serde_json::Value>,
    weak_window: &slint::Weak<AppWindow>,
    motion: &mut MotionControl,
) {
    // Extract positions before moving objects into closure
    let mut new_pos_x = None;
    let mut new_pos_y = None;
    let mut new_pos_z = None;

    if let Some(toolhead) = objects.get("toolhead") {
        if let Some(pos) = toolhead.get("position").and_then(|v| v.as_array()) {
            new_pos_x = pos.first().and_then(|v| v.as_f64());
            new_pos_y = pos.get(1).and_then(|v| v.as_f64());
            new_pos_z = pos.get(2).and_then(|v| v.as_f64());
        }
    }

    if let Some(x) = new_pos_x {
        motion.pos_x = x;
    }
    if let Some(y) = new_pos_y {
        motion.pos_y = y;
    }
    if let Some(z) = new_pos_z {
        motion.pos_z = z;
    }

    let weak = weak_window.clone();
    let _ = slint::invoke_from_event_loop(move || {
        if let Some(w) = weak.upgrade() {
            if let Some(extruder) = objects.get("extruder") {
                if let Some(temp) = extruder.get("temperature").and_then(|v| v.as_f64()) {
                    w.set_extruder_temp(temp as f32);
                    w.set_controls_extruder_temp(format!("{:.0}", temp).into());
                    let mut buf = extruder_history().lock().unwrap();
                    buf.push(temp);
                    let history: Vec<f32> = buf.iter().map(|&t| t as f32).collect();
                    w.set_extruder_temp_history(slint::ModelRc::from(history.as_slice()));
                }
                if let Some(target) = extruder.get("target").and_then(|v| v.as_f64()) {
                    w.set_extruder_target(target as f32);
                }
            }
            if let Some(bed) = objects.get("heater_bed") {
                if let Some(temp) = bed.get("temperature").and_then(|v| v.as_f64()) {
                    w.set_bed_temp(temp as f32);
                    w.set_controls_bed_temp(format!("{:.0}", temp).into());
                    let mut buf = bed_history().lock().unwrap();
                    buf.push(temp);
                    let history: Vec<f32> = buf.iter().map(|&t| t as f32).collect();
                    w.set_bed_temp_history(slint::ModelRc::from(history.as_slice()));
                }
                if let Some(target) = bed.get("target").and_then(|v| v.as_f64()) {
                    w.set_bed_target(target as f32);
                }
            }
            if let Some(x) = new_pos_x {
                w.set_pos_x(format!("{:.1}", x).into());
            }
            if let Some(y) = new_pos_y {
                w.set_pos_y(format!("{:.1}", y).into());
            }
            if let Some(z) = new_pos_z {
                w.set_pos_z(format!("{:.1}", z).into());
            }

            if let Some(print_stats) = objects.get("print_stats") {
                if let Some(progress) = print_stats.get("progress").and_then(|v| v.as_f64()) {
                    w.set_print_progress(progress as f32);
                }
                if let Some(filename) = print_stats.get("filename").and_then(|v| v.as_str()) {
                    w.set_print_filename(filename.into());
                }
                if let Some(state) = print_stats.get("state").and_then(|v| v.as_str()) {
                    w.set_print_state(state.into());
                }
                if let Some(total) =
                    print_stats.get("total_duration").and_then(|v| v.as_f64())
                {
                    let formatted = crate::ui::progress::format_duration(total as u64);
                    w.set_print_duration(formatted.into());
                }
            }
        }
    });
}

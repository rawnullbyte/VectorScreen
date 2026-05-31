mod callbacks;
mod commands;
mod notifications;

pub(crate) use callbacks::wire_callbacks;
pub(crate) use commands::handle_command;
pub(crate) use notifications::handle_notification;

use crate::AppWindow;
use crate::ui::motion::MotionControl;

pub(crate) fn update_ui<F: FnOnce(AppWindow) + Send + 'static>(weak: &slint::Weak<AppWindow>, f: F) {
    let weak = weak.clone();
    let _ = slint::invoke_from_event_loop(move || {
        if let Some(w) = weak.upgrade() {
            f(w);
        }
    });
}

pub(crate) fn update_positions(weak: &slint::Weak<AppWindow>, motion: &MotionControl) {
    let x = motion.pos_x;
    let y = motion.pos_y;
    let z = motion.pos_z;
    update_ui(weak, move |w| {
        w.set_pos_x(MotionControl::format_position(x).into());
        w.set_pos_y(MotionControl::format_position(y).into());
        w.set_pos_z(MotionControl::format_position(z).into());
    });
}

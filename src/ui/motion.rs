fn format_home_all() -> String {
    "G28".to_string()
}

fn format_home_axis(axis: char) -> String {
    match axis {
        'X' | 'Y' | 'Z' => format!("G28 {}", axis),
        _ => "G28".to_string(),
    }
}

fn format_move_axis(axis: char, pos: f64, speed: f64) -> String {
    format!("G1 {}{:.3} F{:.0}", axis, pos, speed)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MovementSpeed {
    /// 1800 mm/min - fine positioning
    Slow,
    /// 3000 mm/min - normal movement
    Medium,
    /// 6000 mm/min - rapid movement
    Fast,
}

impl MovementSpeed {
    /// Get the feed rate in mm/min for this speed preset.
    pub fn feed_rate(self) -> f64 {
        match self {
            MovementSpeed::Slow => 1800.0,
            MovementSpeed::Medium => 3000.0,
            MovementSpeed::Fast => 6000.0,
        }
    }
}

/// Motion control state for axis movement and homing.
#[derive(Debug, Clone)]
pub struct MotionControl {
    pub pos_x: f64,
    pub pos_y: f64,
    pub pos_z: f64,
    pub speed: MovementSpeed,
}

impl Default for MotionControl {
    fn default() -> Self {
        Self {
            pos_x: 0.0,
            pos_y: 0.0,
            pos_z: 0.0,
            speed: MovementSpeed::Fast,
        }
    }
}

impl MotionControl {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn home_all_gcode(&self) -> String {
        format_home_all()
    }

    pub fn home_axis_gcode(&self, axis: char) -> String {
        format_home_axis(axis)
    }

    /// Generate G-code to move an axis by a relative distance.
    ///
    /// Uses the current speed setting for the feed rate.
    pub fn move_axis_gcode(&self, axis: char, distance: f64) -> String {
        let feed = self.get_feed_rate();
        match axis {
            'X' => format_move_axis('X', self.pos_x + distance, feed),
            'Y' => format_move_axis('Y', self.pos_y + distance, feed),
            'Z' => format_move_axis('Z', self.pos_z + distance, feed),
            _ => format!("G1 F{:.0}", feed),
        }
    }

    pub fn set_speed(&mut self, speed: MovementSpeed) {
        self.speed = speed;
    }

    pub fn get_feed_rate(&self) -> f64 {
        self.speed.feed_rate()
    }

    pub fn update_position(&mut self, axis: char, distance: f64) {
        match axis {
            'X' => self.pos_x += distance,
            'Y' => self.pos_y += distance,
            'Z' => self.pos_z += distance,
            _ => {}
        }
    }

    pub fn reset_positions(&mut self) {
        self.pos_x = 0.0;
        self.pos_y = 0.0;
        self.pos_z = 0.0;
    }

    pub fn format_position(value: f64) -> String {
        format!("{:.1}", value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn test_speed_feed_rates() {
        assert_eq!(MovementSpeed::Slow.feed_rate(), 1800.0);
        assert_eq!(MovementSpeed::Medium.feed_rate(), 3000.0);
        assert_eq!(MovementSpeed::Fast.feed_rate(), 6000.0);
    }


    #[test]
    fn test_default_motion_control() {
        let mc = MotionControl::default();
        assert_eq!(mc.pos_x, 0.0);
        assert_eq!(mc.pos_y, 0.0);
        assert_eq!(mc.pos_z, 0.0);
        assert_eq!(mc.speed, MovementSpeed::Fast);
    }

    #[test]
    fn test_new_motion_control() {
        let mc = MotionControl::new();
        assert_eq!(mc.pos_x, 0.0);
        assert_eq!(mc.speed, MovementSpeed::Fast);
    }


    #[test]
    fn test_home_all_gcode() {
        let mc = MotionControl::new();
        assert_eq!(mc.home_all_gcode(), "G28");
    }

    #[test]
    fn test_home_axis_gcode_x() {
        let mc = MotionControl::new();
        assert_eq!(mc.home_axis_gcode('X'), "G28 X");
    }

    #[test]
    fn test_home_axis_gcode_y() {
        let mc = MotionControl::new();
        assert_eq!(mc.home_axis_gcode('Y'), "G28 Y");
    }

    #[test]
    fn test_home_axis_gcode_z() {
        let mc = MotionControl::new();
        assert_eq!(mc.home_axis_gcode('Z'), "G28 Z");
    }

    #[test]
    fn test_home_axis_invalid_falls_back() {
        let mc = MotionControl::new();
        assert_eq!(mc.home_axis_gcode('A'), "G28");
    }


    #[test]
    fn test_move_axis_gcode_x() {
        let mc = MotionControl::new();
        assert_eq!(mc.move_axis_gcode('X', 10.0), "G1 X10.000 F6000");
    }

    #[test]
    fn test_move_axis_gcode_y() {
        let mc = MotionControl::new();
        assert_eq!(mc.move_axis_gcode('Y', -5.5), "G1 Y-5.500 F6000");
    }

    #[test]
    fn test_move_axis_gcode_z() {
        let mc = MotionControl::new();
        assert_eq!(mc.move_axis_gcode('Z', 0.5), "G1 Z0.500 F6000");
    }

    #[test]
    fn test_move_axis_gcode_slow_speed() {
        let mut mc = MotionControl::new();
        mc.set_speed(MovementSpeed::Slow);
        assert_eq!(mc.move_axis_gcode('X', 1.0), "G1 X1.000 F1800");
    }

    #[test]
    fn test_move_axis_gcode_medium_speed() {
        let mut mc = MotionControl::new();
        mc.set_speed(MovementSpeed::Medium);
        assert_eq!(mc.move_axis_gcode('X', 1.0), "G1 X1.000 F3000");
    }

    #[test]
    fn test_move_axis_gcode_invalid_axis() {
        let mc = MotionControl::new();
        assert_eq!(mc.move_axis_gcode('A', 10.0), "G1 F6000");
    }


    #[test]
    fn test_set_speed() {
        let mut mc = MotionControl::new();
        assert_eq!(mc.speed, MovementSpeed::Fast);
        mc.set_speed(MovementSpeed::Slow);
        assert_eq!(mc.speed, MovementSpeed::Slow);
        mc.set_speed(MovementSpeed::Medium);
        assert_eq!(mc.speed, MovementSpeed::Medium);
    }

    #[test]
    fn test_get_feed_rate() {
        let mut mc = MotionControl::new();
        assert_eq!(mc.get_feed_rate(), 6000.0);
        mc.set_speed(MovementSpeed::Slow);
        assert_eq!(mc.get_feed_rate(), 1800.0);
    }


    #[test]
    fn test_update_position_x() {
        let mut mc = MotionControl::new();
        mc.update_position('X', 10.0);
        assert_eq!(mc.pos_x, 10.0);
        mc.update_position('X', -3.5);
        assert_eq!(mc.pos_x, 6.5);
    }

    #[test]
    fn test_update_position_y() {
        let mut mc = MotionControl::new();
        mc.update_position('Y', 20.0);
        assert_eq!(mc.pos_y, 20.0);
    }

    #[test]
    fn test_update_position_z() {
        let mut mc = MotionControl::new();
        mc.update_position('Z', 5.0);
        assert_eq!(mc.pos_z, 5.0);
        mc.update_position('Z', -10.0);
        assert_eq!(mc.pos_z, -5.0);
    }

    #[test]
    fn test_update_position_invalid_axis() {
        let mut mc = MotionControl::new();
        mc.update_position('A', 10.0);
        // No change to any position
        assert_eq!(mc.pos_x, 0.0);
        assert_eq!(mc.pos_y, 0.0);
        assert_eq!(mc.pos_z, 0.0);
    }

    #[test]
    fn test_reset_positions() {
        let mut mc = MotionControl::new();
        mc.update_position('X', 10.0);
        mc.update_position('Y', 20.0);
        mc.update_position('Z', 5.0);
        mc.reset_positions();
        assert_eq!(mc.pos_x, 0.0);
        assert_eq!(mc.pos_y, 0.0);
        assert_eq!(mc.pos_z, 0.0);
    }


    #[test]
    fn test_format_position() {
        assert_eq!(MotionControl::format_position(0.0), "0.0");
        assert_eq!(MotionControl::format_position(10.5), "10.5");
        assert_eq!(MotionControl::format_position(-3.0), "-3.0");
    }


    #[test]
    fn test_move_and_track_position() {
        let mut mc = MotionControl::new();
        // Generate G-code for X+10, then track
        let gcode = mc.move_axis_gcode('X', 10.0);
        assert_eq!(gcode, "G1 X10.000 F6000");
        mc.update_position('X', 10.0);
        assert_eq!(mc.pos_x, 10.0);

        // Now move another 5mm — position in G-code should be 15
        let gcode = mc.move_axis_gcode('X', 5.0);
        assert_eq!(gcode, "G1 X15.000 F6000");
        mc.update_position('X', 5.0);
        assert_eq!(mc.pos_x, 15.0);
    }
}

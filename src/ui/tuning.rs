
//! Tuning screen state and logic.
//!
//! This module handles on-the-fly tuning controls:
//! - Speed multiplier (M220)
//! - Flow rate (M221)
//! - Z-offset adjustment (SET_GCODE_OFFSET)
//! - Pressure advance display
//! - Motion limits configuration

/// Tuning control state.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TuningControl {
    pub speed_multiplier: i32,
    pub flow_rate: i32,
    pub z_offset: f64,
    pub pressure_advance: f64,
}

impl Default for TuningControl {
    fn default() -> Self {
        Self {
            speed_multiplier: 100,
            flow_rate: 100,
            z_offset: 0.0,
            pressure_advance: 0.0,
        }
    }
}

#[allow(dead_code)]
impl TuningControl {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn adjust_speed(&mut self, delta: i32) {
        self.speed_multiplier = (self.speed_multiplier + delta).clamp(50, 200);
    }

    pub fn adjust_flow(&mut self, delta: i32) {
        self.flow_rate = (self.flow_rate + delta).clamp(50, 200);
    }

    pub fn adjust_z_offset(&mut self, delta: f64) {
        self.z_offset = (self.z_offset + delta).clamp(-2.0, 2.0);
    }

    pub fn speed_gcode(&self) -> String {
        format!("M220 S{}", self.speed_multiplier)
    }

    pub fn flow_gcode(&self) -> String {
        format!("M221 S{}", self.flow_rate)
    }

    pub fn z_offset_gcode(&self) -> String {
        format!("SET_GCODE_OFFSET Z_ADJUST={:.3}", self.z_offset)
    }

    pub fn pressure_advance_gcode(&self) -> String {
        format!("SET_PRESSURE_ADVANCE ADVANCE={:.4}", self.pressure_advance)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_defaults() {
        let control = TuningControl::new();
        assert_eq!(control.speed_multiplier, 100);
        assert_eq!(control.flow_rate, 100);
        assert_eq!(control.z_offset, 0.0);
        assert_eq!(control.pressure_advance, 0.0);
    }

    #[test]
    fn test_adjust_speed() {
        let mut control = TuningControl::new();
        control.adjust_speed(10);
        assert_eq!(control.speed_multiplier, 110);
        control.adjust_speed(-20);
        assert_eq!(control.speed_multiplier, 90);
    }

    #[test]
    fn test_speed_clamping() {
        let mut control = TuningControl::new();
        control.adjust_speed(-100);
        assert_eq!(control.speed_multiplier, 50);
        control.adjust_speed(200);
        assert_eq!(control.speed_multiplier, 200);
    }

    #[test]
    fn test_adjust_flow() {
        let mut control = TuningControl::new();
        control.adjust_flow(5);
        assert_eq!(control.flow_rate, 105);
        control.adjust_flow(-10);
        assert_eq!(control.flow_rate, 95);
    }

    #[test]
    fn test_flow_clamping() {
        let mut control = TuningControl::new();
        control.adjust_flow(-100);
        assert_eq!(control.flow_rate, 50);
        control.adjust_flow(200);
        assert_eq!(control.flow_rate, 200);
    }

    #[test]
    fn test_adjust_z_offset() {
        let mut control = TuningControl::new();
        control.adjust_z_offset(0.05);
        assert!((control.z_offset - 0.05).abs() < 0.001);
        control.adjust_z_offset(-0.1);
        assert!((control.z_offset - (-0.05)).abs() < 0.001);
    }

    #[test]
    fn test_z_offset_clamping() {
        let mut control = TuningControl::new();
        control.adjust_z_offset(-5.0);
        assert_eq!(control.z_offset, -2.0);
        control.adjust_z_offset(5.0);
        assert_eq!(control.z_offset, 2.0);
    }

    #[test]
    fn test_speed_gcode() {
        let mut control = TuningControl::new();
        control.adjust_speed(20);
        assert_eq!(control.speed_gcode(), "M220 S120");
    }

    #[test]
    fn test_flow_gcode() {
        let mut control = TuningControl::new();
        control.adjust_flow(-5);
        assert_eq!(control.flow_gcode(), "M221 S95");
    }

    #[test]
    fn test_z_offset_gcode() {
        let mut control = TuningControl::new();
        control.adjust_z_offset(0.1);
        assert_eq!(control.z_offset_gcode(), "SET_GCODE_OFFSET Z_ADJUST=0.100");
    }

    #[test]
    fn test_pressure_advance_gcode() {
        let mut control = TuningControl::new();
        control.pressure_advance = 0.045;
        assert_eq!(control.pressure_advance_gcode(), "SET_PRESSURE_ADVANCE ADVANCE=0.0450");
    }
}

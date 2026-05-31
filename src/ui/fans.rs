
/// Fan control state for the controls screen.
///
/// Each fan speed is stored as 0-100% and converted to 0.0-1.0 for G-code.
pub struct FanControl {
    pub part_cooling: u8,
    pub aux_fan: u8,
    pub controller_fan: u8,
    /// Exhaust fan speed (0-100%). Uses SET_PIN instead of SET_FAN_SPEED.
    pub exhaust_fan: u8,
}

impl FanControl {
    /// Create a new FanControl with all fans off.
    pub fn new() -> Self {
        Self {
            part_cooling: 0,
            aux_fan: 0,
            controller_fan: 0,
            exhaust_fan: 0,
        }
    }

    pub fn set_part_cooling(&mut self, speed: u8) {
        self.part_cooling = speed.min(100);
    }

    pub fn set_aux_fan(&mut self, speed: u8) {
        self.aux_fan = speed.min(100);
    }

    pub fn set_controller_fan(&mut self, speed: u8) {
        self.controller_fan = speed.min(100);
    }

    pub fn set_exhaust_fan(&mut self, speed: u8) {
        self.exhaust_fan = speed.min(100);
    }

    pub fn part_cooling_gcode(&self) -> String {
        format_fan_speed("part_fan", self.part_cooling)
    }

    pub fn aux_fan_gcode(&self) -> String {
        format_fan_speed("aux_fan", self.aux_fan)
    }

    pub fn controller_fan_gcode(&self) -> String {
        format_fan_speed("controller_fan", self.controller_fan)
    }

    /// Format G-code to set the exhaust fan speed.
    ///
    /// Uses SET_PIN instead of SET_FAN_SPEED since exhaust is a digital output.
    pub fn exhaust_fan_gcode(&self) -> String {
        format_output_pin("exhaust_fan", self.exhaust_fan)
    }
}

impl Default for FanControl {
    fn default() -> Self {
        Self::new()
    }
}

fn format_fan_speed(fan_name: &str, speed: u8) -> String {
    let value = speed as f64 / 100.0;
    format!("SET_FAN_SPEED FAN={fan_name} SPEED={:.2}", value)
}

fn format_output_pin(pin_name: &str, value: u8) -> String {
    let val = value as f64 / 100.0;
    format!("SET_PIN PIN={pin_name} VALUE={:.2}", val)
}

#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn test_new_defaults() {
        let fan = FanControl::new();
        assert_eq!(fan.part_cooling, 0);
        assert_eq!(fan.aux_fan, 0);
        assert_eq!(fan.controller_fan, 0);
        assert_eq!(fan.exhaust_fan, 0);
    }

    #[test]
    fn test_default_trait() {
        let fan = FanControl::default();
        assert_eq!(fan.part_cooling, 0);
        assert_eq!(fan.aux_fan, 0);
        assert_eq!(fan.controller_fan, 0);
        assert_eq!(fan.exhaust_fan, 0);
    }


    #[test]
    fn test_set_part_cooling() {
        let mut fan = FanControl::new();
        fan.set_part_cooling(75);
        assert_eq!(fan.part_cooling, 75);
    }

    #[test]
    fn test_set_part_cooling_clamps_max() {
        let mut fan = FanControl::new();
        fan.set_part_cooling(150);
        assert_eq!(fan.part_cooling, 100);
    }

    #[test]
    fn test_set_aux_fan() {
        let mut fan = FanControl::new();
        fan.set_aux_fan(50);
        assert_eq!(fan.aux_fan, 50);
    }

    #[test]
    fn test_set_aux_fan_clamps_max() {
        let mut fan = FanControl::new();
        fan.set_aux_fan(200);
        assert_eq!(fan.aux_fan, 100);
    }

    #[test]
    fn test_set_controller_fan() {
        let mut fan = FanControl::new();
        fan.set_controller_fan(100);
        assert_eq!(fan.controller_fan, 100);
    }

    #[test]
    fn test_set_controller_fan_clamps_max() {
        let mut fan = FanControl::new();
        fan.set_controller_fan(128);
        assert_eq!(fan.controller_fan, 100);
    }

    #[test]
    fn test_set_exhaust_fan() {
        let mut fan = FanControl::new();
        fan.set_exhaust_fan(30);
        assert_eq!(fan.exhaust_fan, 30);
    }

    #[test]
    fn test_set_exhaust_fan_clamps_max() {
        let mut fan = FanControl::new();
        fan.set_exhaust_fan(255);
        assert_eq!(fan.exhaust_fan, 100);
    }


    #[test]
    fn test_part_cooling_gcode_zero() {
        let mut fan = FanControl::new();
        fan.set_part_cooling(0);
        assert_eq!(
            fan.part_cooling_gcode(),
            "SET_FAN_SPEED FAN=part_fan SPEED=0.00"
        );
    }

    #[test]
    fn test_part_cooling_gcode_fifty() {
        let mut fan = FanControl::new();
        fan.set_part_cooling(50);
        assert_eq!(
            fan.part_cooling_gcode(),
            "SET_FAN_SPEED FAN=part_fan SPEED=0.50"
        );
    }

    #[test]
    fn test_part_cooling_gcode_hundred() {
        let mut fan = FanControl::new();
        fan.set_part_cooling(100);
        assert_eq!(
            fan.part_cooling_gcode(),
            "SET_FAN_SPEED FAN=part_fan SPEED=1.00"
        );
    }

    #[test]
    fn test_aux_fan_gcode() {
        let mut fan = FanControl::new();
        fan.set_aux_fan(75);
        assert_eq!(
            fan.aux_fan_gcode(),
            "SET_FAN_SPEED FAN=aux_fan SPEED=0.75"
        );
    }

    #[test]
    fn test_controller_fan_gcode() {
        let mut fan = FanControl::new();
        fan.set_controller_fan(100);
        assert_eq!(
            fan.controller_fan_gcode(),
            "SET_FAN_SPEED FAN=controller_fan SPEED=1.00"
        );
    }

    #[test]
    fn test_exhaust_fan_gcode_zero() {
        let mut fan = FanControl::new();
        fan.set_exhaust_fan(0);
        assert_eq!(
            fan.exhaust_fan_gcode(),
            "SET_PIN PIN=exhaust_fan VALUE=0.00"
        );
    }

    #[test]
    fn test_exhaust_fan_gcode_fifty() {
        let mut fan = FanControl::new();
        fan.set_exhaust_fan(50);
        assert_eq!(
            fan.exhaust_fan_gcode(),
            "SET_PIN PIN=exhaust_fan VALUE=0.50"
        );
    }

    #[test]
    fn test_exhaust_fan_gcode_hundred() {
        let mut fan = FanControl::new();
        fan.set_exhaust_fan(100);
        assert_eq!(
            fan.exhaust_fan_gcode(),
            "SET_PIN PIN=exhaust_fan VALUE=1.00"
        );
    }


    #[test]
    fn test_format_fan_speed_zero() {
        assert_eq!(
            format_fan_speed("part_fan", 0),
            "SET_FAN_SPEED FAN=part_fan SPEED=0.00"
        );
    }

    #[test]
    fn test_format_fan_speed_boundary_values() {
        assert_eq!(
            format_fan_speed("test_fan", 1),
            "SET_FAN_SPEED FAN=test_fan SPEED=0.01"
        );
        assert_eq!(
            format_fan_speed("test_fan", 99),
            "SET_FAN_SPEED FAN=test_fan SPEED=0.99"
        );
    }

    #[test]
    fn test_format_output_pin() {
        assert_eq!(
            format_output_pin("exhaust_fan", 75),
            "SET_PIN PIN=exhaust_fan VALUE=0.75"
        );
    }
}

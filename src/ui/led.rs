
/// LED control state for the controls screen.
///
/// Manages chamber light on/off state and brightness (0-100%),
/// formats G-code commands matching ff5m macros: `SET_LED LED=chamber_light WHITE={value}`.
pub struct LedControl {
    is_on: bool,
    brightness: u8,
}

impl LedControl {
    /// Create a new LedControl (off, 50% brightness).
    pub fn new() -> Self {
        Self {
            is_on: false,
            brightness: 50,
        }
    }

    pub fn toggle(&mut self) {
        self.is_on = !self.is_on;
    }

    pub fn set_on(&mut self, on: bool) {
        self.is_on = on;
    }

    pub fn is_on(&self) -> bool {
        self.is_on
    }

    pub fn set_brightness(&mut self, brightness: u8) {
        self.brightness = brightness.min(100);
    }

    pub fn brightness(&self) -> u8 {
        self.brightness
    }

    /// Format G-code to turn the LED on at current brightness.
    pub fn led_on_gcode(&self) -> String {
        self.set_brightness_gcode()
    }

    pub fn led_off_gcode(&self) -> String {
        "SET_LED LED=chamber_light WHITE=0.0".to_string()
    }

    /// Format G-code to set the LED brightness.
    ///
    /// Uses the current brightness value, converting from 0-100 integer
    /// to 0.0-1.0 float for the Klipper SET_LED command.
    pub fn set_brightness_gcode(&self) -> String {
        let value = self.brightness as f64 / 100.0;
        format!("SET_LED LED=chamber_light WHITE={:.1}", value)
    }
}

impl Default for LedControl {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_defaults() {
        let led = LedControl::new();
        assert!(!led.is_on());
        assert_eq!(led.brightness(), 50);
    }

    #[test]
    fn test_default_trait() {
        let led = LedControl::default();
        assert!(!led.is_on());
        assert_eq!(led.brightness(), 50);
    }

    #[test]
    fn test_toggle() {
        let mut led = LedControl::new();
        assert!(!led.is_on());
        led.toggle();
        assert!(led.is_on());
        led.toggle();
        assert!(!led.is_on());
    }

    #[test]
    fn test_set_on() {
        let mut led = LedControl::new();
        led.set_on(true);
        assert!(led.is_on());
        led.set_on(false);
        assert!(!led.is_on());
    }

    #[test]
    fn test_set_brightness() {
        let mut led = LedControl::new();
        led.set_brightness(75);
        assert_eq!(led.brightness(), 75);
    }

    #[test]
    fn test_set_brightness_clamps_max() {
        let mut led = LedControl::new();
        led.set_brightness(150);
        assert_eq!(led.brightness(), 100);
    }

    #[test]
    fn test_set_brightness_zero() {
        let mut led = LedControl::new();
        led.set_brightness(0);
        assert_eq!(led.brightness(), 0);
    }

    #[test]
    fn test_led_on_gcode_at_50() {
        let mut led = LedControl::new();
        led.set_brightness(50);
        assert_eq!(led.led_on_gcode(), "SET_LED LED=chamber_light WHITE=0.5");
    }

    #[test]
    fn test_led_on_gcode_at_100() {
        let mut led = LedControl::new();
        led.set_brightness(100);
        assert_eq!(led.led_on_gcode(), "SET_LED LED=chamber_light WHITE=1.0");
    }

    #[test]
    fn test_led_on_gcode_at_0() {
        let mut led = LedControl::new();
        led.set_brightness(0);
        assert_eq!(led.led_on_gcode(), "SET_LED LED=chamber_light WHITE=0.0");
    }

    #[test]
    fn test_led_off_gcode() {
        let led = LedControl::new();
        assert_eq!(led.led_off_gcode(), "SET_LED LED=chamber_light WHITE=0.0");
    }

    #[test]
    fn test_set_brightness_gcode() {
        let mut led = LedControl::new();
        led.set_brightness(30);
        assert_eq!(
            led.set_brightness_gcode(),
            "SET_LED LED=chamber_light WHITE=0.3"
        );
    }

}

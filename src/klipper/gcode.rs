

use std::fmt;

pub const EXTRUDER_TEMP_MIN: f64 = 0.0;
pub const EXTRUDER_TEMP_MAX: f64 = 300.0;
pub const BED_TEMP_MIN: f64 = 0.0;
pub const BED_TEMP_MAX: f64 = 120.0;

pub const DEFAULT_FILAMENT_SPEED: f64 = 450.0;
#[derive(Debug, Clone, PartialEq)]
pub enum GcodeError {
    ExtruderTempOutOfRange(f64),
    BedTempOutOfRange(f64),
}

impl fmt::Display for GcodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GcodeError::ExtruderTempOutOfRange(t) => {
                write!(f, "extruder temp {t} out of range [{EXTRUDER_TEMP_MIN}, {EXTRUDER_TEMP_MAX}]")
            }
            GcodeError::BedTempOutOfRange(t) => {
                write!(f, "bed temp {t} out of range [{BED_TEMP_MIN}, {BED_TEMP_MAX}]")
            }
        }
    }
}

impl std::error::Error for GcodeError {}

pub fn validate_extruder_temp(temp: f64) -> Result<(), GcodeError> {
    if temp < EXTRUDER_TEMP_MIN || temp > EXTRUDER_TEMP_MAX {
        return Err(GcodeError::ExtruderTempOutOfRange(temp));
    }
    Ok(())
}

pub fn validate_bed_temp(temp: f64) -> Result<(), GcodeError> {
    if temp < BED_TEMP_MIN || temp > BED_TEMP_MAX {
        return Err(GcodeError::BedTempOutOfRange(temp));
    }
    Ok(())
}

pub fn format_emergency_stop() -> String {
    "M112".to_string()
}

pub fn format_home_all() -> String {
    "G28".to_string()
}

pub fn format_home_axis(axis: char) -> String {
    match axis {
        'X' | 'Y' | 'Z' => format!("G28 {}", axis),
        _ => "G28".to_string(),
    }
}

pub fn format_load_filament(speed: Option<f64>) -> String {
    let s = speed.unwrap_or(DEFAULT_FILAMENT_SPEED);
    format!("LOAD_FILAMENT SPEED={:.0}", s)
}

pub fn format_unload_filament(speed: Option<f64>) -> String {
    let s = speed.unwrap_or(DEFAULT_FILAMENT_SPEED);
    format!("UNLOAD_FILAMENT SPEED={:.0}", s)
}

pub fn format_set_bed_temp(temp: f64) -> Result<String, GcodeError> {
    validate_bed_temp(temp)?;
    Ok(format!("M140 S{:.1}", temp))
}

pub fn format_set_extruder_temp(temp: f64) -> Result<String, GcodeError> {
    validate_extruder_temp(temp)?;
    Ok(format!("M104 S{:.1}", temp))
}

pub fn format_move_x(pos: f64, speed: f64) -> String {
    format!("G1 X{:.3} F{:.0}", pos, speed)
}

pub fn format_move_y(pos: f64, speed: f64) -> String {
    format!("G1 Y{:.3} F{:.0}", pos, speed)
}

pub fn format_move_z(pos: f64, speed: f64) -> String {
    format!("G1 Z{:.3} F{:.0}", pos, speed)
}

pub fn format_led_on() -> String {
    "SET_LED LED=chamber_light WHITE=1".to_string()
}

pub fn format_led_off() -> String {
    "SET_LED LED=chamber_light WHITE=0".to_string()
}

pub fn format_led_brightness(brightness: u8) -> String {
    let value = brightness as f64 / 100.0;
    format!("SET_LED LED=chamber_light WHITE={:.2}", value)
}

pub fn format_set_fan_speed(fan_name: &str, speed: u8) -> String {
    let value = speed as f64 / 100.0;
    format!("SET_FAN_SPEED FAN={fan_name} SPEED={:.2}", value)
}

pub fn format_set_output_pin(pin_name: &str, value: u8) -> String {
    let val = value as f64 / 100.0;
    format!("SET_PIN PIN={pin_name} VALUE={:.2}", val)
}

pub fn format_set_pressure_advance(value: f64) -> String {
    format!("SET_PRESSURE_ADVANCE ADVANCE={:.4}", value)
}

pub fn format_max_velocity(x: f64, y: f64, z: f64) -> String {
    format!("M203 X{:.0} Y{:.0} Z{:.0}", x, y, z)
}

pub fn format_max_acceleration(x: f64, y: f64, z: f64) -> String {
    format!("M201 X{:.0} Y{:.0} Z{:.0}", x, y, z)
}

pub fn format_square_corner_velocity(value: f64) -> String {
    format!("SET_VELOCITY_LIMIT SQUARE_CORNER_VELOCITY={:.2}", value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_extruder_temp_in_range() {
        assert!(validate_extruder_temp(0.0).is_ok());
        assert!(validate_extruder_temp(200.0).is_ok());
        assert!(validate_extruder_temp(300.0).is_ok());
    }

    #[test]
    fn test_validate_extruder_temp_out_of_range() {
        assert!(validate_extruder_temp(-1.0).is_err());
        assert!(validate_extruder_temp(301.0).is_err());
        assert!(validate_extruder_temp(500.0).is_err());
    }

    #[test]
    fn test_validate_bed_temp_in_range() {
        assert!(validate_bed_temp(0.0).is_ok());
        assert!(validate_bed_temp(60.0).is_ok());
        assert!(validate_bed_temp(120.0).is_ok());
    }

    #[test]
    fn test_validate_bed_temp_out_of_range() {
        assert!(validate_bed_temp(-1.0).is_err());
        assert!(validate_bed_temp(121.0).is_err());
        assert!(validate_bed_temp(200.0).is_err());
    }

    #[test]
    fn test_format_emergency_stop() {
        assert_eq!(format_emergency_stop(), "M112");
    }

    #[test]
    fn test_format_home_all() {
        assert_eq!(format_home_all(), "G28");
    }

    #[test]
    fn test_format_home_axis() {
        assert_eq!(format_home_axis('X'), "G28 X");
        assert_eq!(format_home_axis('Y'), "G28 Y");
        assert_eq!(format_home_axis('Z'), "G28 Z");
        assert_eq!(format_home_axis('A'), "G28");
    }

    #[test]
    fn test_format_load_filament() {
        assert_eq!(format_load_filament(None), "LOAD_FILAMENT SPEED=450");
        assert_eq!(format_load_filament(Some(300.0)), "LOAD_FILAMENT SPEED=300");
    }

    #[test]
    fn test_format_unload_filament() {
        assert_eq!(format_unload_filament(None), "UNLOAD_FILAMENT SPEED=450");
    }

    #[test]
    fn test_format_set_bed_temp_valid() {
        assert_eq!(format_set_bed_temp(60.0).unwrap(), "M140 S60.0");
    }

    #[test]
    fn test_format_set_bed_temp_invalid() {
        assert!(format_set_bed_temp(150.0).is_err());
    }

    #[test]
    fn test_format_set_extruder_temp_valid() {
        assert_eq!(format_set_extruder_temp(220.0).unwrap(), "M104 S220.0");
    }

    #[test]
    fn test_format_set_extruder_temp_invalid() {
        assert!(format_set_extruder_temp(350.0).is_err());
    }

    #[test]
    fn test_format_move_x() {
        assert_eq!(format_move_x(55.0, 6000.0), "G1 X55.000 F6000");
    }

    #[test]
    fn test_format_move_y() {
        assert_eq!(format_move_y(100.5, 3000.0), "G1 Y100.500 F3000");
    }

    #[test]
    fn test_format_move_z() {
        assert_eq!(format_move_z(10.0, 1800.0), "G1 Z10.000 F1800");
    }

    #[test]
    fn test_format_led_on() {
        assert_eq!(format_led_on(), "SET_LED LED=chamber_light WHITE=1");
    }

    #[test]
    fn test_format_led_off() {
        assert_eq!(format_led_off(), "SET_LED LED=chamber_light WHITE=0");
    }

    #[test]
    fn test_format_led_brightness() {
        assert_eq!(format_led_brightness(0), "SET_LED LED=chamber_light WHITE=0.00");
        assert_eq!(format_led_brightness(50), "SET_LED LED=chamber_light WHITE=0.50");
        assert_eq!(format_led_brightness(100), "SET_LED LED=chamber_light WHITE=1.00");
    }

    #[test]
    fn test_format_set_fan_speed() {
        assert_eq!(format_set_fan_speed("part_fan", 0), "SET_FAN_SPEED FAN=part_fan SPEED=0.00");
        assert_eq!(format_set_fan_speed("part_fan", 50), "SET_FAN_SPEED FAN=part_fan SPEED=0.50");
        assert_eq!(format_set_fan_speed("part_fan", 100), "SET_FAN_SPEED FAN=part_fan SPEED=1.00");
    }

    #[test]
    fn test_format_set_output_pin() {
        assert_eq!(format_set_output_pin("exhaust_fan", 75), "SET_PIN PIN=exhaust_fan VALUE=0.75");
    }

    #[test]
    fn test_format_set_pressure_advance() {
        assert_eq!(format_set_pressure_advance(0.0), "SET_PRESSURE_ADVANCE ADVANCE=0.0000");
        assert_eq!(format_set_pressure_advance(0.045), "SET_PRESSURE_ADVANCE ADVANCE=0.0450");
    }

    #[test]
    fn test_format_max_velocity() {
        assert_eq!(format_max_velocity(300.0, 300.0, 5.0), "M203 X300 Y300 Z5");
    }

    #[test]
    fn test_format_max_acceleration() {
        assert_eq!(format_max_acceleration(3000.0, 3000.0, 100.0), "M201 X3000 Y3000 Z100");
    }

    #[test]
    fn test_format_square_corner_velocity() {
        assert_eq!(format_square_corner_velocity(5.0), "SET_VELOCITY_LIMIT SQUARE_CORNER_VELOCITY=5.00");
    }

    #[test]
    fn test_gcode_error_display() {
        let err = GcodeError::ExtruderTempOutOfRange(350.0);
        assert!(err.to_string().contains("extruder temp 350"));

        let err = GcodeError::BedTempOutOfRange(150.0);
        assert!(err.to_string().contains("bed temp 150"));
    }
}

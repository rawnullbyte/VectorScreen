
pub mod gcode;
pub mod input_shaper;
pub mod bed_mesh;
pub mod tmc;
pub mod macros;

pub use gcode::GcodeError;

/// All Klipper commands for the FlashForge AD5M printer.
#[derive(Debug, Clone, PartialEq)]
pub enum KlipperCommand {
    /// Emergency stop - M112
    EmergencyStop,
    /// Home all axes - G28
    HomeAll,
    /// Home specific axis - G28 X/Y/Z
    HomeX,
    HomeY,
    HomeZ,
    /// Load filament via macro - LOAD_FILAMENT
    LoadFilament {
        speed: Option<f64>,
    },
    /// Unload filament via macro - UNLOAD_FILAMENT
    UnloadFilament {
        speed: Option<f64>,
    },
    /// Set bed temperature - M140 S{temp}
    SetBedTemp(f64),
    /// Set extruder temperature - M104 S{temp}
    SetExtruderTemp(f64),
    /// Move X axis - G1 X{pos} F{speed}
    MoveX {
        pos: f64,
        speed: f64,
    },
    /// Move Y axis - G1 Y{pos} F{speed}
    MoveY {
        pos: f64,
        speed: f64,
    },
    /// Move Z axis - G1 Z{pos} F{speed}
    MoveZ {
        pos: f64,
        speed: f64,
    },
    /// Turn on chamber LED - SET_LED LED=chamber_light WHITE=1
    LedOn,
    /// Turn off chamber LED - SET_LED LED=chamber_light WHITE=0
    LedOff,
    /// Set chamber LED brightness (0-100) - SET_LED LED=chamber_light WHITE={brightness/100.0}
    LedBrightness(u8),
    /// Set fan speed - SET_FAN_SPEED FAN={name} SPEED={speed/100}
    SetFanSpeed {
        name: String,
        speed: u8,
    },
    /// Set pressure advance - SET_PRESSURE_ADVANCE ADVANCE={value}
    SetPressureAdvance { value: f64 },
    /// Set max velocity - M203 X{x} Y{y} Z{z}
    SetMaxVelocity { x: f64, y: f64, z: f64 },
    /// Set max acceleration - M201 X{x} Y{y} Z{z}
    SetMaxAcceleration { x: f64, y: f64, z: f64 },
    /// Set square corner velocity - SET_VELOCITY_LIMIT SQUARE_CORNER_VELOCITY={value}
    SetSquareCornerVelocity { value: f64 },
}

impl KlipperCommand {
    /// Format this command as a G-code string.
    ///
    /// Returns `Err` if temperature validation fails.
    pub fn format_gcode(&self) -> Result<String, GcodeError> {
        match self {
            KlipperCommand::EmergencyStop => Ok(gcode::format_emergency_stop()),
            KlipperCommand::HomeAll => Ok(gcode::format_home_all()),
            KlipperCommand::HomeX => Ok(gcode::format_home_axis('X')),
            KlipperCommand::HomeY => Ok(gcode::format_home_axis('Y')),
            KlipperCommand::HomeZ => Ok(gcode::format_home_axis('Z')),
            KlipperCommand::LoadFilament { speed } => Ok(gcode::format_load_filament(*speed)),
            KlipperCommand::UnloadFilament { speed } => Ok(gcode::format_unload_filament(*speed)),
            KlipperCommand::SetBedTemp(temp) => gcode::format_set_bed_temp(*temp),
            KlipperCommand::SetExtruderTemp(temp) => gcode::format_set_extruder_temp(*temp),
            KlipperCommand::MoveX { pos, speed } => Ok(gcode::format_move_x(*pos, *speed)),
            KlipperCommand::MoveY { pos, speed } => Ok(gcode::format_move_y(*pos, *speed)),
            KlipperCommand::MoveZ { pos, speed } => Ok(gcode::format_move_z(*pos, *speed)),
            KlipperCommand::LedOn => Ok(gcode::format_led_on()),
            KlipperCommand::LedOff => Ok(gcode::format_led_off()),
            KlipperCommand::LedBrightness(brightness) => {
                Ok(gcode::format_led_brightness(*brightness))
            }
            KlipperCommand::SetFanSpeed { name, speed } => {
                Ok(gcode::format_set_fan_speed(name, *speed))
            }
            KlipperCommand::SetPressureAdvance { value } => {
                Ok(gcode::format_set_pressure_advance(*value))
            }
            KlipperCommand::SetMaxVelocity { x, y, z } => {
                Ok(gcode::format_max_velocity(*x, *y, *z))
            }
            KlipperCommand::SetMaxAcceleration { x, y, z } => {
                Ok(gcode::format_max_acceleration(*x, *y, *z))
            }
            KlipperCommand::SetSquareCornerVelocity { value } => {
                Ok(gcode::format_square_corner_velocity(*value))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_emergency_stop() {
        assert_eq!(
            KlipperCommand::EmergencyStop.format_gcode().unwrap(),
            "M112"
        );
    }

    #[test]
    fn test_home_all() {
        assert_eq!(KlipperCommand::HomeAll.format_gcode().unwrap(), "G28");
    }

    #[test]
    fn test_home_axes() {
        assert_eq!(KlipperCommand::HomeX.format_gcode().unwrap(), "G28 X");
        assert_eq!(KlipperCommand::HomeY.format_gcode().unwrap(), "G28 Y");
        assert_eq!(KlipperCommand::HomeZ.format_gcode().unwrap(), "G28 Z");
    }

    #[test]
    fn test_load_filament() {
        assert_eq!(
            KlipperCommand::LoadFilament { speed: None }
                .format_gcode()
                .unwrap(),
            "LOAD_FILAMENT SPEED=450"
        );
        assert_eq!(
            KlipperCommand::LoadFilament { speed: Some(300.0) }
                .format_gcode()
                .unwrap(),
            "LOAD_FILAMENT SPEED=300"
        );
    }

    #[test]
    fn test_unload_filament() {
        assert_eq!(
            KlipperCommand::UnloadFilament { speed: None }
                .format_gcode()
                .unwrap(),
            "UNLOAD_FILAMENT SPEED=450"
        );
    }

    #[test]
    fn test_set_bed_temp_valid() {
        assert_eq!(
            KlipperCommand::SetBedTemp(60.0).format_gcode().unwrap(),
            "M140 S60.0"
        );
    }

    #[test]
    fn test_set_bed_temp_invalid() {
        assert!(KlipperCommand::SetBedTemp(150.0).format_gcode().is_err());
    }

    #[test]
    fn test_set_extruder_temp_valid() {
        assert_eq!(
            KlipperCommand::SetExtruderTemp(220.0)
                .format_gcode()
                .unwrap(),
            "M104 S220.0"
        );
    }

    #[test]
    fn test_set_extruder_temp_invalid() {
        assert!(KlipperCommand::SetExtruderTemp(350.0)
            .format_gcode()
            .is_err());
    }

    #[test]
    fn test_move_commands() {
        assert_eq!(
            KlipperCommand::MoveX {
                pos: 55.0,
                speed: 6000.0
            }
            .format_gcode()
            .unwrap(),
            "G1 X55.000 F6000"
        );
        assert_eq!(
            KlipperCommand::MoveY {
                pos: 100.5,
                speed: 3000.0
            }
            .format_gcode()
            .unwrap(),
            "G1 Y100.500 F3000"
        );
        assert_eq!(
            KlipperCommand::MoveZ {
                pos: 10.0,
                speed: 1800.0
            }
            .format_gcode()
            .unwrap(),
            "G1 Z10.000 F1800"
        );
    }

    #[test]
    fn test_led_commands() {
        assert_eq!(
            KlipperCommand::LedOn.format_gcode().unwrap(),
            "SET_LED LED=chamber_light WHITE=1"
        );
        assert_eq!(
            KlipperCommand::LedOff.format_gcode().unwrap(),
            "SET_LED LED=chamber_light WHITE=0"
        );
        assert_eq!(
            KlipperCommand::LedBrightness(50).format_gcode().unwrap(),
            "SET_LED LED=chamber_light WHITE=0.50"
        );
    }

    #[test]
    fn test_set_fan_speed() {
        assert_eq!(
            KlipperCommand::SetFanSpeed {
                name: "part_fan".to_string(),
                speed: 50,
            }
            .format_gcode()
            .unwrap(),
            "SET_FAN_SPEED FAN=part_fan SPEED=0.50"
        );
        assert_eq!(
            KlipperCommand::SetFanSpeed {
                name: "aux_fan".to_string(),
                speed: 0,
            }
            .format_gcode()
            .unwrap(),
            "SET_FAN_SPEED FAN=aux_fan SPEED=0.00"
        );
        assert_eq!(
            KlipperCommand::SetFanSpeed {
                name: "controller_fan".to_string(),
                speed: 100,
            }
            .format_gcode()
            .unwrap(),
            "SET_FAN_SPEED FAN=controller_fan SPEED=1.00"
        );
    }

    #[test]
    fn test_all_commands_compile() {
        // Verify every variant can be formatted without panicking
        let commands = vec![
            KlipperCommand::EmergencyStop,
            KlipperCommand::HomeAll,
            KlipperCommand::HomeX,
            KlipperCommand::HomeY,
            KlipperCommand::HomeZ,
            KlipperCommand::LoadFilament { speed: None },
            KlipperCommand::UnloadFilament { speed: None },
            KlipperCommand::SetBedTemp(60.0),
            KlipperCommand::SetExtruderTemp(200.0),
            KlipperCommand::MoveX {
                pos: 0.0,
                speed: 6000.0,
            },
            KlipperCommand::MoveY {
                pos: 0.0,
                speed: 6000.0,
            },
            KlipperCommand::MoveZ {
                pos: 0.0,
                speed: 1800.0,
            },
            KlipperCommand::LedOn,
            KlipperCommand::LedOff,
            KlipperCommand::LedBrightness(75),
            KlipperCommand::SetFanSpeed {
                name: "part_fan".to_string(),
                speed: 50,
            },
            KlipperCommand::SetPressureAdvance { value: 0.045 },
            KlipperCommand::SetMaxVelocity { x: 300.0, y: 300.0, z: 5.0 },
            KlipperCommand::SetMaxAcceleration { x: 3000.0, y: 3000.0, z: 100.0 },
            KlipperCommand::SetSquareCornerVelocity { value: 5.0 },
        ];

        for cmd in &commands {
            let _ = cmd.format_gcode();
        }
    }
}

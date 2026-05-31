use crate::klipper::KlipperCommand;

/// The app shell wires Slint callbacks to these methods.
pub struct ControlsScreen {
    extruder_temp: f64,
}

impl ControlsScreen {
    pub fn new() -> Self {
        Self { extruder_temp: 0.0 }
    }

    pub fn emergency_stop_gcode(&self) -> Result<String, crate::klipper::GcodeError> {
        KlipperCommand::EmergencyStop.format_gcode()
    }

    pub fn load_filament_gcode(&self) -> Result<String, crate::klipper::GcodeError> {
        KlipperCommand::LoadFilament { speed: None }.format_gcode()
    }

    pub fn unload_filament_gcode(&self) -> Result<String, crate::klipper::GcodeError> {
        KlipperCommand::UnloadFilament { speed: None }.format_gcode()
    }

    pub fn set_extruder_temp(&mut self, temp: f64) {
        self.extruder_temp = temp;
    }

    pub fn extruder_temp(&self) -> f64 {
        self.extruder_temp
    }
}

impl Default for ControlsScreen {
    fn default() -> Self {
        Self::new()
    }
}

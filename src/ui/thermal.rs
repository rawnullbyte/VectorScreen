
use crate::klipper::gcode::{validate_bed_temp, validate_extruder_temp, GcodeError};
use crate::klipper::KlipperCommand;

/// Material temperature presets (extruder / bed) from ff5m macros.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaterialPreset {
    /// PLA: Extruder 220°C, Bed 60°C
    Pla,
    /// PETG: Extruder 250°C, Bed 80°C
    Petg,
    /// ABS: Extruder 260°C, Bed 100°C
    Abs,
}

#[allow(dead_code)]
impl MaterialPreset {
    /// Extruder temperature for this material preset.
    pub fn extruder_temp(self) -> f64 {
        match self {
            MaterialPreset::Pla => 220.0,
            MaterialPreset::Petg => 250.0,
            MaterialPreset::Abs => 260.0,
        }
    }

    /// Bed temperature for this material preset.
    pub fn bed_temp(self) -> f64 {
        match self {
            MaterialPreset::Pla => 60.0,
            MaterialPreset::Petg => 80.0,
            MaterialPreset::Abs => 100.0,
        }
    }
}

/// Thermal control state for the controls screen.
///
/// Manages bed and extruder temperature targets, provides material presets,
/// and formats G-code commands via `KlipperCommand`.
#[allow(dead_code)]
pub struct ThermalControl {
    extruder_target: f64,
    bed_target: f64,
    active_preset: Option<MaterialPreset>,
}

#[allow(dead_code)]
impl ThermalControl {
    pub fn new() -> Self {
        Self {
            extruder_target: 0.0,
            bed_target: 0.0,
            active_preset: None,
        }
    }

    pub fn set_extruder_temp(&mut self, temp: f64) -> Result<(), GcodeError> {
        validate_extruder_temp(temp)?;
        self.extruder_target = temp;
        self.active_preset = None;
        Ok(())
    }

    pub fn set_bed_temp(&mut self, temp: f64) -> Result<(), GcodeError> {
        validate_bed_temp(temp)?;
        self.bed_target = temp;
        self.active_preset = None;
        Ok(())
    }

    pub fn apply_preset(&mut self, preset: MaterialPreset) -> Result<(), GcodeError> {
        validate_extruder_temp(preset.extruder_temp())?;
        validate_bed_temp(preset.bed_temp())?;
        self.extruder_target = preset.extruder_temp();
        self.bed_target = preset.bed_temp();
        self.active_preset = Some(preset);
        Ok(())
    }

    pub fn adjust_extruder_temp(&mut self, delta: f64) -> Result<(), GcodeError> {
        let new_temp = (self.extruder_target + delta).clamp(
            crate::klipper::gcode::EXTRUDER_TEMP_MIN,
            crate::klipper::gcode::EXTRUDER_TEMP_MAX,
        );
        self.set_extruder_temp(new_temp)
    }

    pub fn adjust_bed_temp(&mut self, delta: f64) -> Result<(), GcodeError> {
        let new_temp = (self.bed_target + delta).clamp(
            crate::klipper::gcode::BED_TEMP_MIN,
            crate::klipper::gcode::BED_TEMP_MAX,
        );
        self.set_bed_temp(new_temp)
    }

    pub fn extruder_gcode(&self) -> Result<String, GcodeError> {
        KlipperCommand::SetExtruderTemp(self.extruder_target).format_gcode()
    }

    pub fn bed_gcode(&self) -> Result<String, GcodeError> {
        KlipperCommand::SetBedTemp(self.bed_target).format_gcode()
    }

    pub fn extruder_target(&self) -> f64 {
        self.extruder_target
    }

    pub fn bed_target(&self) -> f64 {
        self.bed_target
    }

    pub fn active_preset(&self) -> Option<MaterialPreset> {
        self.active_preset
    }
}

impl Default for ThermalControl {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn test_pla_temps() {
        assert_eq!(MaterialPreset::Pla.extruder_temp(), 220.0);
        assert_eq!(MaterialPreset::Pla.bed_temp(), 60.0);
    }

    #[test]
    fn test_petg_temps() {
        assert_eq!(MaterialPreset::Petg.extruder_temp(), 250.0);
        assert_eq!(MaterialPreset::Petg.bed_temp(), 80.0);
    }

    #[test]
    fn test_abs_temps() {
        assert_eq!(MaterialPreset::Abs.extruder_temp(), 260.0);
        assert_eq!(MaterialPreset::Abs.bed_temp(), 100.0);
    }


    #[test]
    fn test_new_defaults() {
        let tc = ThermalControl::new();
        assert_eq!(tc.extruder_target(), 0.0);
        assert_eq!(tc.bed_target(), 0.0);
        assert_eq!(tc.active_preset(), None);
    }

    #[test]
    fn test_set_extruder_temp() {
        let mut tc = ThermalControl::new();
        assert!(tc.set_extruder_temp(200.0).is_ok());
        assert_eq!(tc.extruder_target(), 200.0);
        assert_eq!(tc.active_preset(), None);
    }

    #[test]
    fn test_set_extruder_temp_out_of_range() {
        let mut tc = ThermalControl::new();
        assert!(tc.set_extruder_temp(350.0).is_err());
        assert_eq!(tc.extruder_target(), 0.0);
    }

    #[test]
    fn test_set_bed_temp() {
        let mut tc = ThermalControl::new();
        assert!(tc.set_bed_temp(60.0).is_ok());
        assert_eq!(tc.bed_target(), 60.0);
        assert_eq!(tc.active_preset(), None);
    }

    #[test]
    fn test_set_bed_temp_out_of_range() {
        let mut tc = ThermalControl::new();
        assert!(tc.set_bed_temp(150.0).is_err());
        assert_eq!(tc.bed_target(), 0.0);
    }

    #[test]
    fn test_apply_preset_pla() {
        let mut tc = ThermalControl::new();
        assert!(tc.apply_preset(MaterialPreset::Pla).is_ok());
        assert_eq!(tc.extruder_target(), 220.0);
        assert_eq!(tc.bed_target(), 60.0);
        assert_eq!(tc.active_preset(), Some(MaterialPreset::Pla));
    }

    #[test]
    fn test_apply_preset_petg() {
        let mut tc = ThermalControl::new();
        assert!(tc.apply_preset(MaterialPreset::Petg).is_ok());
        assert_eq!(tc.extruder_target(), 250.0);
        assert_eq!(tc.bed_target(), 80.0);
        assert_eq!(tc.active_preset(), Some(MaterialPreset::Petg));
    }

    #[test]
    fn test_apply_preset_abs() {
        let mut tc = ThermalControl::new();
        assert!(tc.apply_preset(MaterialPreset::Abs).is_ok());
        assert_eq!(tc.extruder_target(), 260.0);
        assert_eq!(tc.bed_target(), 100.0);
        assert_eq!(tc.active_preset(), Some(MaterialPreset::Abs));
    }

    #[test]
    fn test_preset_clears_on_manual_adjust() {
        let mut tc = ThermalControl::new();
        tc.apply_preset(MaterialPreset::Pla).unwrap();
        assert_eq!(tc.active_preset(), Some(MaterialPreset::Pla));

        tc.set_extruder_temp(210.0).unwrap();
        assert_eq!(tc.active_preset(), None);
    }

    #[test]
    fn test_adjust_extruder_temp() {
        let mut tc = ThermalControl::new();
        tc.set_extruder_temp(200.0).unwrap();
        tc.adjust_extruder_temp(10.0).unwrap();
        assert_eq!(tc.extruder_target(), 210.0);
        tc.adjust_extruder_temp(-10.0).unwrap();
        assert_eq!(tc.extruder_target(), 200.0);
    }

    #[test]
    fn test_adjust_extruder_clamps_to_max() {
        let mut tc = ThermalControl::new();
        tc.set_extruder_temp(295.0).unwrap();
        tc.adjust_extruder_temp(10.0).unwrap();
        assert_eq!(tc.extruder_target(), 300.0);
    }

    #[test]
    fn test_adjust_extruder_clamps_to_min() {
        let mut tc = ThermalControl::new();
        tc.set_extruder_temp(5.0).unwrap();
        tc.adjust_extruder_temp(-10.0).unwrap();
        assert_eq!(tc.extruder_target(), 0.0);
    }

    #[test]
    fn test_adjust_bed_temp() {
        let mut tc = ThermalControl::new();
        tc.set_bed_temp(50.0).unwrap();
        tc.adjust_bed_temp(10.0).unwrap();
        assert_eq!(tc.bed_target(), 60.0);
        tc.adjust_bed_temp(-10.0).unwrap();
        assert_eq!(tc.bed_target(), 50.0);
    }

    #[test]
    fn test_adjust_bed_clamps_to_max() {
        let mut tc = ThermalControl::new();
        tc.set_bed_temp(115.0).unwrap();
        tc.adjust_bed_temp(10.0).unwrap();
        assert_eq!(tc.bed_target(), 120.0);
    }

    #[test]
    fn test_adjust_bed_clamps_to_min() {
        let mut tc = ThermalControl::new();
        tc.set_bed_temp(5.0).unwrap();
        tc.adjust_bed_temp(-10.0).unwrap();
        assert_eq!(tc.bed_target(), 0.0);
    }

    #[test]
    fn test_extruder_gcode() {
        let mut tc = ThermalControl::new();
        tc.set_extruder_temp(220.0).unwrap();
        assert_eq!(tc.extruder_gcode().unwrap(), "M104 S220.0");
    }

    #[test]
    fn test_bed_gcode() {
        let mut tc = ThermalControl::new();
        tc.set_bed_temp(60.0).unwrap();
        assert_eq!(tc.bed_gcode().unwrap(), "M140 S60.0");
    }

    #[test]
    fn test_boundary_temps() {
        let mut tc = ThermalControl::new();
        assert!(tc.set_extruder_temp(0.0).is_ok());
        assert!(tc.set_extruder_temp(300.0).is_ok());
        assert!(tc.set_bed_temp(0.0).is_ok());
        assert!(tc.set_bed_temp(120.0).is_ok());
    }
}

#![expect(dead_code)]


/// Input shaper analysis module for VectorScreen.
///
/// Provides data structures for Klipper input shaper calibration results.

/// Input shaper calibration result for a single axis.
#[derive(Debug, Clone, Default)]
pub struct InputShaperResult {
    /// Recommended shaper type (e.g., "zv", "mzv", "ei").
    pub shaper_type: String,
    pub frequency: f64,
    pub vibration_reduction: f64,
    pub freq_data: Vec<f64>,
    pub amplitude_data: Vec<f64>,
}

impl InputShaperResult {
    pub fn new() -> Self {
        Self::default()
    }

    /// Parse a Moonraker `/printer/input_shaper/profile` response.
    ///
    /// Expected format:
    /// ```json
    /// {
    ///   "shaper_type": "mzv",
    ///   "frequency": 45.2,
    ///   "vibration_reduction": 0.85
    /// }
    /// ```
    pub fn parse(response: &serde_json::Value) -> Option<Self> {
        let shaper_type = response.get("shaper_type")?.as_str()?.to_string();
        let frequency = response.get("frequency")?.as_f64()?;
        let vibration_reduction = response.get("vibration_reduction")?.as_f64().unwrap_or(0.0);

        Some(Self {
            shaper_type,
            frequency,
            vibration_reduction,
            freq_data: Vec::new(),
            amplitude_data: Vec::new(),
        })
    }
}

/// Combined input shaper results for X and Y axes.
#[derive(Debug, Clone, Default)]
pub struct InputShaperData {
    pub x: InputShaperResult,
    pub y: InputShaperResult,
}

impl InputShaperData {
    pub fn new() -> Self {
        Self::default()
    }
}

#![expect(dead_code)]


#[derive(Debug, Clone, Default)]
pub struct TmcStatus {
    /// Run current in mA.
    pub current: f64,
    /// Driver temperature in °C.
    pub temp: f64,
    pub errors: u32,
    pub stallguard: u32,
}

impl TmcStatus {
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Debug, Clone, Default)]
pub struct TmcData {
    pub x: TmcStatus,
    pub y: TmcStatus,
    pub z: TmcStatus,
}

impl TmcData {
    pub fn new() -> Self {
        Self::default()
    }
}

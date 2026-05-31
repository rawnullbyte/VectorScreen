
/// Bed mesh analysis module for VectorScreen.
///
/// Provides data structures for Klipper bed mesh calibration results.

/// A single point in the bed mesh grid.
#[derive(Debug, Clone, Copy)]
pub struct BedMeshPoint {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

/// Bed mesh calibration result.
#[derive(Debug, Clone, Default)]
pub struct BedMeshResult {
    /// Grid size (e.g., 5 for 5×5).
    pub grid_size: usize,
    /// Z height values in row-major order.
    pub values: Vec<f64>,
    /// Minimum Z value.
    pub min: f64,
    /// Maximum Z value.
    pub max: f64,
    /// Average Z value.
    pub average: f64,
}

impl BedMeshResult {
    pub fn new() -> Self {
        Self::default()
    }

    /// Parse a Moonraker `/printer/bed_mesh/profile` response.
    ///
    /// Expected format:
    /// ```json
    /// {
    ///   "mesh_matrix": [[0.1, 0.2], [0.3, 0.4]],
    ///   "min_x": 10.0,
    ///   "max_x": 200.0,
    ///   "min_y": 10.0,
    ///   "max_y": 200.0
    /// }
    /// ```
    pub fn parse(response: &serde_json::Value) -> Option<Self> {
        let mesh_matrix = response.get("mesh_matrix")?.as_array()?;
        let mut values = Vec::new();

        for row in mesh_matrix {
            let row_arr = row.as_array()?;
            for val in row_arr {
                values.push(val.as_f64()?);
            }
        }

        let grid_size = (values.len() as f64).sqrt() as usize;
        let min = values.iter().copied().fold(f64::INFINITY, f64::min);
        let max = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let average = if !values.is_empty() {
            values.iter().sum::<f64>() / values.len() as f64
        } else {
            0.0
        };

        Some(Self {
            grid_size,
            values,
            min,
            max,
            average,
        })
    }
}

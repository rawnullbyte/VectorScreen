
/// Touch calibration module for VectorScreen.
///
/// Provides single-touch calibration using an affine transform:
/// ```text
/// screen_x = a * raw_x + b * raw_y + c
/// screen_y = d * raw_x + e * raw_y + f
/// ```
///
/// Requires >= 3 calibration points to solve for the 6 coefficients.
// The calibration matrix is persisted as a TOML file.
use serde::{Deserialize, Serialize};

/// A single calibration point mapping raw touch coordinates to screen coordinates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalibrationPoint {
    /// Raw touch X coordinate (from input device).
    pub raw_x: f64,
    /// Raw touch Y coordinate (from input device).
    pub raw_y: f64,
    /// Desired screen X coordinate.
    pub screen_x: f64,
    /// Desired screen Y coordinate.
    pub screen_y: f64,
}

/// Affine transform coefficients for coordinate mapping.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CalibrationMatrix {
    /// Coefficient a: screen_x = a * raw_x + ...
    pub a: f64,
    /// Coefficient b: screen_x = ... + b * raw_y + ...
    pub b: f64,
    /// Coefficient c: screen_x = ... + c
    pub c: f64,
    /// Coefficient d: screen_y = d * raw_x + ...
    pub d: f64,
    /// Coefficient e: screen_y = ... + e * raw_y + ...
    pub e: f64,
    /// Coefficient f: screen_y = ... + f
    pub f: f64,
}

impl CalibrationMatrix {
    /// Apply the affine transform to the given raw coordinates.
    pub fn transform(&self, x: f64, y: f64) -> (f64, f64) {
        let screen_x = self.a * x + self.b * y + self.c;
        let screen_y = self.d * x + self.e * y + self.f;
        (screen_x, screen_y)
    }
}

/// Touch calibration state machine.
///
/// Collects calibration points, computes the affine transform matrix,
/// and provides save/load for persistence.
#[derive(Debug, Clone)]
pub struct TouchCalibrator {
    /// Collected calibration points.
    points: Vec<CalibrationPoint>,
    /// Computed calibration matrix (None until `calculate_matrix` succeeds).
    matrix: Option<CalibrationMatrix>,
}

impl TouchCalibrator {
    /// Create a new calibrator with no points.
    pub fn new() -> Self {
        Self {
            points: Vec::new(),
            matrix: None,
        }
    }

    /// Add a calibration point mapping raw to screen coordinates.
    pub fn add_point(&mut self, point: CalibrationPoint) {
        self.points.push(point);
    }

    /// Calculate the affine transform matrix from collected points.
    ///
    /// Returns `None` if fewer than 3 points have been added, or if the
    /// raw points are degenerate (collinear / singular matrix).
    pub fn calculate_matrix(&mut self) -> Option<CalibrationMatrix> {
        if self.points.len() < 3 {
            return None;
        }

        // Use first 3 points for the 3×3 linear system.
        //
        // For screen_x:  a·raw_x + b·raw_y + c = screen_x
        // For screen_y:  d·raw_x + e·raw_y + f = screen_y
        //
        // Matrix form: M · [a,b,c]ᵀ = [sx1,sx2,sx3]ᵀ
        //              M · [d,e,f]ᵀ = [sy1,sy2,sy3]ᵀ
        let p = &self.points;
        let m00 = p[0].raw_x;
        let m01 = p[0].raw_y;
        let m02 = 1.0;
        let m10 = p[1].raw_x;
        let m11 = p[1].raw_y;
        let m12 = 1.0;
        let m20 = p[2].raw_x;
        let m21 = p[2].raw_y;
        let m22 = 1.0;

        // Determinant of M using cofactor expansion.
        let det = m00 * (m11 * m22 - m12 * m21)
            - m01 * (m10 * m22 - m12 * m20)
            + m02 * (m10 * m21 - m11 * m20);

        if det.abs() < f64::EPSILON {
            // Singular matrix — points are collinear or duplicate.
            return None;
        }

        let inv_det = 1.0 / det;

        // Inverse of M (adjugate / det).
        let i00 = (m11 * m22 - m12 * m21) * inv_det;
        let i01 = (m02 * m21 - m01 * m22) * inv_det;
        let i02 = (m01 * m12 - m02 * m11) * inv_det;
        let i10 = (m12 * m20 - m10 * m22) * inv_det;
        let i11 = (m00 * m22 - m02 * m20) * inv_det;
        let i12 = (m02 * m10 - m00 * m12) * inv_det;
        let i20 = (m10 * m21 - m11 * m20) * inv_det;
        let i21 = (m01 * m20 - m00 * m21) * inv_det;
        let i22 = (m00 * m11 - m01 * m10) * inv_det;

        // Solve for screen_x coefficients [a, b, c].
        let sx = [p[0].screen_x, p[1].screen_x, p[2].screen_x];
        let a = i00 * sx[0] + i01 * sx[1] + i02 * sx[2];
        let b = i10 * sx[0] + i11 * sx[1] + i12 * sx[2];
        let c = i20 * sx[0] + i21 * sx[1] + i22 * sx[2];

        // Solve for screen_y coefficients [d, e, f].
        let sy = [p[0].screen_y, p[1].screen_y, p[2].screen_y];
        let d = i00 * sy[0] + i01 * sy[1] + i02 * sy[2];
        let e = i10 * sy[0] + i11 * sy[1] + i12 * sy[2];
        let f = i20 * sy[0] + i21 * sy[1] + i22 * sy[2];

        let matrix = CalibrationMatrix { a, b, c, d, e, f };
        self.matrix = Some(matrix.clone());
        Some(matrix)
    }

    /// Transform raw coordinates using the previously computed matrix.
    ///
    /// If no matrix has been computed yet, returns the raw coordinates unchanged.
    pub fn transform(&self, raw_x: f64, raw_y: f64) -> (f64, f64) {
        match &self.matrix {
            Some(m) => m.transform(raw_x, raw_y),
            None => (raw_x, raw_y),
        }
    }

    /// Save the calibration matrix to a TOML file.
    ///
    /// Creates parent directories if needed. Returns an error if no matrix
    /// has been computed or if file I/O fails.
    pub fn save(&self, path: &str) -> Result<(), String> {
        let matrix = self
            .matrix
            .as_ref()
            .ok_or("No calibration matrix computed")?;

        let toml_str = toml::to_string(matrix)
            .map_err(|e| format!("Failed to serialize matrix: {e}"))?;

        if let Some(parent) = std::path::Path::new(path).parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create directory: {e}"))?;
        }

        std::fs::write(path, toml_str)
            .map_err(|e| format!("Failed to write calibration file: {e}"))
    }

    /// Load a calibration matrix from a TOML file.
    ///
    /// Returns a new `TouchCalibrator` with the matrix pre-loaded (no points).
    pub fn load(path: &str) -> Result<Self, String> {
        let contents = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read calibration file: {e}"))?;

        let matrix: CalibrationMatrix = toml::from_str(&contents)
            .map_err(|e| format!("Failed to parse calibration file: {e}"))?;

        Ok(Self {
            points: Vec::new(),
            matrix: Some(matrix),
        })
    }
}

impl Default for TouchCalibrator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn identity_points() -> Vec<CalibrationPoint> {
        vec![
            CalibrationPoint { raw_x: 0.0, raw_y: 0.0, screen_x: 0.0, screen_y: 0.0 },
            CalibrationPoint { raw_x: 100.0, raw_y: 0.0, screen_x: 100.0, screen_y: 0.0 },
            CalibrationPoint { raw_x: 0.0, raw_y: 100.0, screen_x: 0.0, screen_y: 100.0 },
        ]
    }

    #[test]
    fn test_new_calibrator_has_no_matrix() {
        let cal = TouchCalibrator::new();
        assert!(cal.matrix.is_none());
        assert!(cal.points.is_empty());
    }

    #[test]
    fn test_default_matches_new() {
        let a = TouchCalibrator::new();
        let b = TouchCalibrator::default();
        assert!(a.matrix.is_none() && b.matrix.is_none());
    }

    #[test]
    fn test_add_point() {
        let mut cal = TouchCalibrator::new();
        cal.add_point(CalibrationPoint {
            raw_x: 10.0,
            raw_y: 20.0,
            screen_x: 100.0,
            screen_y: 200.0,
        });
        assert_eq!(cal.points.len(), 1);
    }

    #[test]
    fn test_calculate_matrix_insufficient_points() {
        let mut cal = TouchCalibrator::new();
        assert!(cal.calculate_matrix().is_none());

        cal.add_point(CalibrationPoint {
            raw_x: 0.0,
            raw_y: 0.0,
            screen_x: 0.0,
            screen_y: 0.0,
        });
        assert!(cal.calculate_matrix().is_none());

        cal.add_point(CalibrationPoint {
            raw_x: 100.0,
            raw_y: 0.0,
            screen_x: 100.0,
            screen_y: 0.0,
        });
        assert!(cal.calculate_matrix().is_none());
    }

    #[test]
    fn test_calculate_matrix_identity() {
        let mut cal = TouchCalibrator::new();
        for p in identity_points() {
            cal.add_point(p);
        }

        let m = cal.calculate_matrix().expect("should compute matrix");
        // Identity transform: a=1, b=0, c=0, d=0, e=1, f=0
        assert!((m.a - 1.0).abs() < 1e-10);
        assert!(m.b.abs() < 1e-10);
        assert!(m.c.abs() < 1e-10);
        assert!(m.d.abs() < 1e-10);
        assert!((m.e - 1.0).abs() < 1e-10);
        assert!(m.f.abs() < 1e-10);
    }

    #[test]
    fn test_calculate_matrix_with_scaling_and_offset() {
        // Map raw [0,100]×[0,100] → screen [0,800]×[0,480]
        let mut cal = TouchCalibrator::new();
        cal.add_point(CalibrationPoint { raw_x: 0.0, raw_y: 0.0, screen_x: 0.0, screen_y: 0.0 });
        cal.add_point(CalibrationPoint { raw_x: 100.0, raw_y: 0.0, screen_x: 800.0, screen_y: 0.0 });
        cal.add_point(CalibrationPoint { raw_x: 0.0, raw_y: 100.0, screen_x: 0.0, screen_y: 480.0 });

        let m = cal.calculate_matrix().expect("should compute matrix");
        // a = 8 (800/100), b = 0, c = 0
        assert!((m.a - 8.0).abs() < 1e-10);
        assert!(m.b.abs() < 1e-10);
        assert!(m.c.abs() < 1e-10);
        // d = 0, e = 4.8 (480/100), f = 0
        assert!(m.d.abs() < 1e-10);
        assert!((m.e - 4.8).abs() < 1e-10);
        assert!(m.f.abs() < 1e-10);
    }

    #[test]
    fn test_calculate_matrix_with_offset() {
        // Translation only: raw (10,20) → screen (110,220)
        let mut cal = TouchCalibrator::new();
        cal.add_point(CalibrationPoint { raw_x: 0.0, raw_y: 0.0, screen_x: 100.0, screen_y: 200.0 });
        cal.add_point(CalibrationPoint { raw_x: 100.0, raw_y: 0.0, screen_x: 200.0, screen_y: 200.0 });
        cal.add_point(CalibrationPoint { raw_x: 0.0, raw_y: 100.0, screen_x: 100.0, screen_y: 300.0 });

        let m = cal.calculate_matrix().expect("should compute matrix");
        // a=1, b=0, c=100
        assert!((m.a - 1.0).abs() < 1e-10);
        assert!(m.b.abs() < 1e-10);
        assert!((m.c - 100.0).abs() < 1e-10);
        // d=0, e=1, f=200
        assert!(m.d.abs() < 1e-10);
        assert!((m.e - 1.0).abs() < 1e-10);
        assert!((m.f - 200.0).abs() < 1e-10);
    }

    #[test]
    fn test_calculate_matrix_singular_returns_none() {
        // All three points collinear (same raw_x = 0)
        let mut cal = TouchCalibrator::new();
        cal.add_point(CalibrationPoint { raw_x: 0.0, raw_y: 0.0, screen_x: 0.0, screen_y: 0.0 });
        cal.add_point(CalibrationPoint { raw_x: 0.0, raw_y: 50.0, screen_x: 0.0, screen_y: 50.0 });
        cal.add_point(CalibrationPoint { raw_x: 0.0, raw_y: 100.0, screen_x: 0.0, screen_y: 100.0 });

        assert!(cal.calculate_matrix().is_none());
    }

    #[test]
    fn test_transform_with_matrix() {
        let mut cal = TouchCalibrator::new();
        for p in identity_points() {
            cal.add_point(p);
        }
        cal.calculate_matrix().unwrap();

        let (sx, sy) = cal.transform(42.0, 73.0);
        assert!((sx - 42.0).abs() < 1e-10);
        assert!((sy - 73.0).abs() < 1e-10);
    }

    #[test]
    fn test_transform_without_matrix_returns_raw() {
        let cal = TouchCalibrator::new();
        let (sx, sy) = cal.transform(50.0, 60.0);
        assert!((sx - 50.0).abs() < 1e-10);
        assert!((sy - 60.0).abs() < 1e-10);
    }

    #[test]
    fn test_matrix_transform_method() {
        let m = CalibrationMatrix {
            a: 8.0, b: 0.0, c: 0.0,
            d: 0.0, e: 4.8, f: 0.0,
        };
        let (sx, sy) = m.transform(50.0, 50.0);
        assert!((sx - 400.0).abs() < 1e-10);
        assert!((sy - 240.0).abs() < 1e-10);
    }

    #[test]
    fn test_transform_full_pipeline() {
        let mut cal = TouchCalibrator::new();
        cal.add_point(CalibrationPoint { raw_x: 0.0, raw_y: 0.0, screen_x: 0.0, screen_y: 0.0 });
        cal.add_point(CalibrationPoint { raw_x: 100.0, raw_y: 0.0, screen_x: 800.0, screen_y: 0.0 });
        cal.add_point(CalibrationPoint { raw_x: 0.0, raw_y: 100.0, screen_x: 0.0, screen_y: 480.0 });
        cal.calculate_matrix().unwrap();

        let (sx, sy) = cal.transform(50.0, 50.0);
        assert!((sx - 400.0).abs() < 1e-10);
        assert!((sy - 240.0).abs() < 1e-10);
    }

    #[test]
    fn test_save_load_roundtrip() {
        let mut cal = TouchCalibrator::new();
        for p in identity_points() {
            cal.add_point(p);
        }
        let m = cal.calculate_matrix().unwrap();

        let dir = std::env::temp_dir().join("vector_screen_test_touch_cal");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("calibration.toml");

        cal.save(path.to_str().unwrap()).expect("save should succeed");

        let loaded = TouchCalibrator::load(path.to_str().unwrap()).expect("load should succeed");
        let loaded_m = loaded.matrix.as_ref().expect("loaded matrix should exist");
        assert_eq!(&m, loaded_m);

        // Clean up
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_save_fails_without_matrix() {
        let cal = TouchCalibrator::new();
        let result = cal.save("/tmp/test_no_matrix.toml");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("No calibration matrix"));
    }

    #[test]
    fn test_load_fails_on_missing_file() {
        let result = TouchCalibrator::load("/tmp/nonexistent_vector_screen_cal_test_xyz.toml");
        assert!(result.is_err());
    }

    #[test]
    fn test_load_fails_on_invalid_toml() {
        let dir = std::env::temp_dir().join("vector_screen_test_touch_cal_invalid");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("bad.toml");
        std::fs::write(&path, "this is not valid toml {{{{").unwrap();

        let result = TouchCalibrator::load(path.to_str().unwrap());
        assert!(result.is_err());

        let _ = std::fs::remove_dir_all(&dir);
    }
}

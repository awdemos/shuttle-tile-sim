use crate::{Dimensions, Result, TileError, TileGeometry, TileShape};

const MIN_LENGTH_MM: f64 = 50.0;
const MAX_LENGTH_MM: f64 = 203.2;
const MIN_WIDTH_MM: f64 = 50.0;
const MAX_WIDTH_MM: f64 = 203.2;
const MIN_THICKNESS_MM: f64 = 12.7;
const MAX_THICKNESS_MM: f64 = 76.2;
const MIN_TOLERANCE_MM: f64 = 0.05;
const MAX_TOLERANCE_MM: f64 = 1.0;
const MIN_EDGE_RADIUS_MM: f64 = 0.5;
const MAX_EDGE_RADIUS_MM: f64 = 6.35;
const MAX_CUSTOM_COMPLEXITY: u8 = 10;
const SURFACE_AREA_TOLERANCE: f64 = 0.001;
const VOLUME_TOLERANCE: f64 = 1e-9;

impl TileGeometry {
    pub fn from_dimensions(dimensions: Dimensions, shape_type: TileShape) -> Result<Self> {
        let (surface_area_m2, volume_m3, machining_tolerance_mm, edge_radius_mm) =
            match shape_type {
                TileShape::Flat => (
                    calculate_surface_area(&dimensions, 1.0),
                    calculate_volume(&dimensions),
                    0.25,
                    None,
                ),
                TileShape::Contoured => (
                    calculate_surface_area(&dimensions, 1.15),
                    calculate_volume(&dimensions),
                    0.38,
                    Some(1.0),
                ),
                TileShape::Custom { complexity } => {
                    if complexity > MAX_CUSTOM_COMPLEXITY {
                        return Err(TileError::GeometryError(format!(
                            "Custom complexity {} exceeds maximum {}",
                            complexity, MAX_CUSTOM_COMPLEXITY
                        )));
                    }
                    let multiplier = 1.0 + (complexity as f64 * 0.05);
                    (
                        calculate_surface_area(&dimensions, multiplier),
                        calculate_volume(&dimensions),
                        0.5,
                        None,
                    )
                }
            };

        let geometry = TileGeometry {
            dimensions,
            surface_area_m2,
            volume_m3,
            shape_type,
            machining_tolerance_mm,
            edge_radius_mm,
        };

        geometry.validate_geometry()?;
        Ok(geometry)
    }

    pub fn validate_geometry(&self) -> Result<()> {
        if self.dimensions.length_mm < MIN_LENGTH_MM || self.dimensions.length_mm > MAX_LENGTH_MM {
            return Err(TileError::GeometryError(format!(
                "Length {} mm out of bounds [{}, {}]",
                self.dimensions.length_mm, MIN_LENGTH_MM, MAX_LENGTH_MM
            )));
        }

        if self.dimensions.width_mm < MIN_WIDTH_MM || self.dimensions.width_mm > MAX_WIDTH_MM {
            return Err(TileError::GeometryError(format!(
                "Width {} mm out of bounds [{}, {}]",
                self.dimensions.width_mm, MIN_WIDTH_MM, MAX_WIDTH_MM
            )));
        }

        if self.dimensions.thickness_mm < MIN_THICKNESS_MM
            || self.dimensions.thickness_mm > MAX_THICKNESS_MM
        {
            return Err(TileError::GeometryError(format!(
                "Thickness {} mm out of bounds [{}, {}]",
                self.dimensions.thickness_mm, MIN_THICKNESS_MM, MAX_THICKNESS_MM
            )));
        }

        if self.machining_tolerance_mm < MIN_TOLERANCE_MM
            || self.machining_tolerance_mm > MAX_TOLERANCE_MM
        {
            return Err(TileError::GeometryError(format!(
                "Machining tolerance {} mm out of bounds [{}, {}]",
                self.machining_tolerance_mm, MIN_TOLERANCE_MM, MAX_TOLERANCE_MM
            )));
        }

        if let Some(radius) = self.edge_radius_mm {
            if radius < MIN_EDGE_RADIUS_MM || radius > MAX_EDGE_RADIUS_MM {
                return Err(TileError::GeometryError(format!(
                    "Edge radius {} mm out of bounds [{}, {}]",
                    radius, MIN_EDGE_RADIUS_MM, MAX_EDGE_RADIUS_MM
                )));
            }
        }

        if self.surface_area_m2 <= 0.0 {
            return Err(TileError::GeometryError(
                "Surface area must be positive".to_string(),
            ));
        }

        if self.volume_m3 <= 0.0 {
            return Err(TileError::GeometryError("Volume must be positive".to_string()));
        }

        let expected_multiplier = match self.shape_type {
            TileShape::Flat => 1.0,
            TileShape::Contoured => 1.15,
            TileShape::Custom { complexity } => 1.0 + (complexity as f64 * 0.05),
        };
        let expected_surface_area = calculate_surface_area(&self.dimensions, expected_multiplier);
        if (self.surface_area_m2 - expected_surface_area).abs() > SURFACE_AREA_TOLERANCE {
            return Err(TileError::GeometryError(format!(
                "Surface area {} m² does not match expected {} m²",
                self.surface_area_m2, expected_surface_area
            )));
        }

        let expected_volume = calculate_volume(&self.dimensions);
        if (self.volume_m3 - expected_volume).abs() > VOLUME_TOLERANCE {
            return Err(TileError::GeometryError(format!(
                "Volume {} m³ does not match expected {} m³",
                self.volume_m3, expected_volume
            )));
        }

        if let TileShape::Custom { complexity } = self.shape_type {
            if complexity > MAX_CUSTOM_COMPLEXITY {
                return Err(TileError::GeometryError(format!(
                    "Custom complexity {} exceeds maximum {}",
                    complexity, MAX_CUSTOM_COMPLEXITY
                )));
            }
        }

        Ok(())
    }
}

fn calculate_surface_area(dimensions: &Dimensions, multiplier: f64) -> f64 {
    let length = dimensions.length_mm / 1000.0;
    let width = dimensions.width_mm / 1000.0;
    let thickness = dimensions.thickness_mm / 1000.0;

    let base_area = 2.0 * (length * width + length * thickness + width * thickness);
    base_area * multiplier
}

fn calculate_volume(dimensions: &Dimensions) -> f64 {
    let length = dimensions.length_mm / 1000.0;
    let width = dimensions.width_mm / 1000.0;
    let thickness = dimensions.thickness_mm / 1000.0;

    length * width * thickness
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flat_tile_geometry_creation() {
        let dim = Dimensions {
            length_mm: 150.0,
            width_mm: 150.0,
            thickness_mm: 25.4,
        };
        let geometry = TileGeometry::from_dimensions(dim, TileShape::Flat).unwrap();

        assert_eq!(geometry.dimensions.length_mm, 150.0);
        assert_eq!(geometry.dimensions.width_mm, 150.0);
        assert_eq!(geometry.dimensions.thickness_mm, 25.4);
        assert_eq!(geometry.shape_type, TileShape::Flat);
        assert_eq!(geometry.machining_tolerance_mm, 0.25);
        assert_eq!(geometry.edge_radius_mm, None);

        let expected_sa = calculate_surface_area(&dim, 1.0);
        let expected_vol = calculate_volume(&dim);
        assert!((geometry.surface_area_m2 - expected_sa).abs() < SURFACE_AREA_TOLERANCE);
        assert!((geometry.volume_m3 - expected_vol).abs() < VOLUME_TOLERANCE);
    }

    #[test]
    fn test_contoured_tile_geometry_creation() {
        let dim = Dimensions {
            length_mm: 100.0,
            width_mm: 100.0,
            thickness_mm: 19.05,
        };
        let geometry = TileGeometry::from_dimensions(dim, TileShape::Contoured).unwrap();

        assert_eq!(geometry.shape_type, TileShape::Contoured);
        assert_eq!(geometry.machining_tolerance_mm, 0.38);
        assert_eq!(geometry.edge_radius_mm, Some(1.0));

        let expected_sa = calculate_surface_area(&dim, 1.15);
        assert!((geometry.surface_area_m2 - expected_sa).abs() < SURFACE_AREA_TOLERANCE);
    }

    #[test]
    fn test_custom_tile_geometry_creation() {
        let dim = Dimensions {
            length_mm: 75.0,
            width_mm: 75.0,
            thickness_mm: 15.0,
        };
        let geometry =
            TileGeometry::from_dimensions(dim, TileShape::Custom { complexity: 5 }).unwrap();

        assert_eq!(geometry.shape_type, TileShape::Custom { complexity: 5 });
        assert_eq!(geometry.machining_tolerance_mm, 0.5);
        assert_eq!(geometry.edge_radius_mm, None);

        let expected_sa = calculate_surface_area(&dim, 1.25);
        assert!((geometry.surface_area_m2 - expected_sa).abs() < SURFACE_AREA_TOLERANCE);
    }

    #[test]
    fn test_length_too_small() {
        let dim = Dimensions {
            length_mm: 10.0,
            width_mm: 100.0,
            thickness_mm: 25.4,
        };
        let result = TileGeometry::from_dimensions(dim, TileShape::Flat);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Length"));
    }

    #[test]
    fn test_length_too_large() {
        let dim = Dimensions {
            length_mm: 300.0,
            width_mm: 100.0,
            thickness_mm: 25.4,
        };
        let result = TileGeometry::from_dimensions(dim, TileShape::Flat);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Length"));
    }

    #[test]
    fn test_width_too_small() {
        let dim = Dimensions {
            length_mm: 100.0,
            width_mm: 10.0,
            thickness_mm: 25.4,
        };
        let result = TileGeometry::from_dimensions(dim, TileShape::Flat);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Width"));
    }

    #[test]
    fn test_width_too_large() {
        let dim = Dimensions {
            length_mm: 100.0,
            width_mm: 300.0,
            thickness_mm: 25.4,
        };
        let result = TileGeometry::from_dimensions(dim, TileShape::Flat);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Width"));
    }

    #[test]
    fn test_thickness_too_small() {
        let dim = Dimensions {
            length_mm: 100.0,
            width_mm: 100.0,
            thickness_mm: 5.0,
        };
        let result = TileGeometry::from_dimensions(dim, TileShape::Flat);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Thickness"));
    }

    #[test]
    fn test_thickness_too_large() {
        let dim = Dimensions {
            length_mm: 100.0,
            width_mm: 100.0,
            thickness_mm: 100.0,
        };
        let result = TileGeometry::from_dimensions(dim, TileShape::Flat);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Thickness"));
    }

    #[test]
    fn test_custom_complexity_too_high() {
        let dim = Dimensions {
            length_mm: 100.0,
            width_mm: 100.0,
            thickness_mm: 25.4,
        };
        let result = TileGeometry::from_dimensions(dim, TileShape::Custom { complexity: 15 });
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("complexity"));
    }

    #[test]
    fn test_validate_machining_tolerance() {
        let dim = Dimensions {
            length_mm: 100.0,
            width_mm: 100.0,
            thickness_mm: 25.4,
        };
        let mut geometry = TileGeometry::from_dimensions(dim, TileShape::Flat).unwrap();

        geometry.machining_tolerance_mm = 0.01;
        let result = geometry.validate_geometry();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Machining tolerance"));

        geometry.machining_tolerance_mm = 2.0;
        let result = geometry.validate_geometry();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Machining tolerance"));
    }

    #[test]
    fn test_validate_edge_radius() {
        let dim = Dimensions {
            length_mm: 100.0,
            width_mm: 100.0,
            thickness_mm: 25.4,
        };
        let mut geometry = TileGeometry::from_dimensions(dim, TileShape::Contoured).unwrap();

        geometry.edge_radius_mm = Some(0.1);
        let result = geometry.validate_geometry();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Edge radius"));

        geometry.edge_radius_mm = Some(10.0);
        let result = geometry.validate_geometry();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Edge radius"));
    }

    #[test]
    fn test_validate_surface_area_mismatch() {
        let dim = Dimensions {
            length_mm: 100.0,
            width_mm: 100.0,
            thickness_mm: 25.4,
        };
        let mut geometry = TileGeometry::from_dimensions(dim, TileShape::Flat).unwrap();

        geometry.surface_area_m2 = 999.0;
        let result = geometry.validate_geometry();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Surface area"));
    }

    #[test]
    fn test_validate_volume_mismatch() {
        let dim = Dimensions {
            length_mm: 100.0,
            width_mm: 100.0,
            thickness_mm: 25.4,
        };
        let mut geometry = TileGeometry::from_dimensions(dim, TileShape::Flat).unwrap();

        geometry.volume_m3 = 999.0;
        let result = geometry.validate_geometry();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Volume"));
    }

    #[test]
    fn test_validate_positive_surface_area() {
        let dim = Dimensions {
            length_mm: 100.0,
            width_mm: 100.0,
            thickness_mm: 25.4,
        };
        let mut geometry = TileGeometry::from_dimensions(dim, TileShape::Flat).unwrap();

        geometry.surface_area_m2 = -1.0;
        let result = geometry.validate_geometry();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("positive"));
    }

    #[test]
    fn test_validate_positive_volume() {
        let dim = Dimensions {
            length_mm: 100.0,
            width_mm: 100.0,
            thickness_mm: 25.4,
        };
        let mut geometry = TileGeometry::from_dimensions(dim, TileShape::Flat).unwrap();

        geometry.volume_m3 = -1.0;
        let result = geometry.validate_geometry();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("positive"));
    }

    #[test]
    fn test_boundary_values() {
        let dim_min = Dimensions {
            length_mm: MIN_LENGTH_MM,
            width_mm: MIN_WIDTH_MM,
            thickness_mm: MIN_THICKNESS_MM,
        };
        let result = TileGeometry::from_dimensions(dim_min, TileShape::Flat);
        assert!(result.is_ok());

        let dim_max = Dimensions {
            length_mm: MAX_LENGTH_MM,
            width_mm: MAX_WIDTH_MM,
            thickness_mm: MAX_THICKNESS_MM,
        };
        let result = TileGeometry::from_dimensions(dim_max, TileShape::Flat);
        assert!(result.is_ok());
    }

    #[test]
    fn test_complexity_zero() {
        let dim = Dimensions {
            length_mm: 100.0,
            width_mm: 100.0,
            thickness_mm: 25.4,
        };
        let geometry =
            TileGeometry::from_dimensions(dim, TileShape::Custom { complexity: 0 }).unwrap();
        assert_eq!(geometry.shape_type, TileShape::Custom { complexity: 0 });

        let expected_sa = calculate_surface_area(&dim, 1.0);
        assert!((geometry.surface_area_m2 - expected_sa).abs() < SURFACE_AREA_TOLERANCE);
    }
}

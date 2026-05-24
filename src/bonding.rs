use crate::{Adhesive, Result, StrainIsolationPad, TileError, TileGeometry, TileMaterial};

const MIN_ADHESIVE_THICKNESS_MM: f64 = 0.15;
const MAX_ADHESIVE_THICKNESS_MM: f64 = 0.35;
const MIN_SIP_THICKNESS_MM: f64 = 3.0;
const MAX_SIP_THICKNESS_MM: f64 = 6.0;
const MIN_SHEAR_STRENGTH_MPA: f64 = 1.0;
const MIN_PEEL_STRENGTH_N_M: f64 = 2000.0;

#[derive(Debug, Clone, PartialEq)]
pub struct BondAssembly {
    pub sip: StrainIsolationPad,
    pub adhesive: Adhesive,
    pub bond_strength_mpa: f64,
    pub bond_stress_mpa: f64,
}

pub fn calculate_bond_strength(
    material: &TileMaterial,
    sip: &StrainIsolationPad,
    adhesive: &Adhesive,
    geometry: &TileGeometry,
) -> Result<f64> {
    if geometry.surface_area_m2 <= 0.0 {
        return Err(TileError::BondingError(
            "Surface area must be positive".to_string(),
        ));
    }
    if material.density_kg_m3 <= 0.0 {
        return Err(TileError::BondingError(
            "Material density must be positive".to_string(),
        ));
    }
    if sip.shear_modulus_mpa <= 0.0 {
        return Err(TileError::BondingError(
            "SIP shear modulus must be positive".to_string(),
        ));
    }
    if adhesive.shear_strength_mpa <= 0.0 {
        return Err(TileError::BondingError(
            "Adhesive shear strength must be positive".to_string(),
        ));
    }

    let area_factor = geometry.surface_area_m2.min(1.0).max(0.0001);
    let density_factor = material.density_kg_m3 / 144.0;
    let sip_factor = sip.shear_modulus_mpa / 0.5;
    let adhesive_contribution = adhesive.shear_strength_mpa * 0.6;
    let geometric_factor = area_factor.sqrt();

    let strength = adhesive_contribution * sip_factor * density_factor * geometric_factor;
    Ok(strength)
}

pub fn calculate_bond_stress(thermal_expansion: f64, geometry: &TileGeometry) -> Result<f64> {
    if geometry.dimensions.thickness_mm <= 0.0 {
        return Err(TileError::BondingError(
            "Tile thickness must be positive".to_string(),
        ));
    }
    if geometry.surface_area_m2 <= 0.0 {
        return Err(TileError::BondingError(
            "Surface area must be positive".to_string(),
        ));
    }

    let thickness_m = geometry.dimensions.thickness_mm / 1000.0;
    let stress = thermal_expansion * geometry.surface_area_m2 / thickness_m;
    Ok(stress)
}

pub fn validate_bond_assembly(assembly: &BondAssembly) -> Result<()> {
    if assembly.adhesive.thickness_mm < MIN_ADHESIVE_THICKNESS_MM
        || assembly.adhesive.thickness_mm > MAX_ADHESIVE_THICKNESS_MM
    {
        return Err(TileError::BondingError(format!(
            "Adhesive thickness {} mm out of range {}-{} mm",
            assembly.adhesive.thickness_mm,
            MIN_ADHESIVE_THICKNESS_MM,
            MAX_ADHESIVE_THICKNESS_MM
        )));
    }

    if assembly.sip.thickness_mm < MIN_SIP_THICKNESS_MM
        || assembly.sip.thickness_mm > MAX_SIP_THICKNESS_MM
    {
        return Err(TileError::BondingError(format!(
            "SIP thickness {} mm out of range {}-{} mm",
            assembly.sip.thickness_mm,
            MIN_SIP_THICKNESS_MM,
            MAX_SIP_THICKNESS_MM
        )));
    }

    if assembly.bond_strength_mpa <= MIN_SHEAR_STRENGTH_MPA {
        return Err(TileError::BondingError(format!(
            "Bond strength {} MPa must exceed {} MPa",
            assembly.bond_strength_mpa, MIN_SHEAR_STRENGTH_MPA
        )));
    }

    if assembly.adhesive.peel_strength_n_m <= MIN_PEEL_STRENGTH_N_M {
        return Err(TileError::BondingError(format!(
            "Peel strength {} N/m must exceed {} N/m",
            assembly.adhesive.peel_strength_n_m, MIN_PEEL_STRENGTH_N_M
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Dimensions, TileGeometry, TileMaterial, TileShape};

    fn test_material() -> TileMaterial {
        TileMaterial::li_900()
    }

    fn test_sip() -> StrainIsolationPad {
        StrainIsolationPad::standard()
    }

    fn test_adhesive() -> Adhesive {
        Adhesive::rtv_560()
    }

    fn test_geometry() -> TileGeometry {
        TileGeometry {
            dimensions: Dimensions {
                length_mm: 150.0,
                width_mm: 150.0,
                thickness_mm: 25.0,
            },
            surface_area_m2: 0.0225,
            volume_m3: 0.0005625,
            shape_type: TileShape::Flat,
            machining_tolerance_mm: 0.1,
            edge_radius_mm: None,
        }
    }

    #[test]
    fn test_calculate_bond_strength() {
        let strength = calculate_bond_strength(
            &test_material(),
            &test_sip(),
            &test_adhesive(),
            &test_geometry(),
        )
        .unwrap();
        assert!(strength > 0.0);
    }

    #[test]
    fn test_calculate_bond_strength_invalid_area() {
        let mut geom = test_geometry();
        geom.surface_area_m2 = 0.0;
        let result = calculate_bond_strength(&test_material(), &test_sip(), &test_adhesive(), &geom);
        assert!(result.is_err());
    }

    #[test]
    fn test_calculate_bond_strength_invalid_density() {
        let mut mat = test_material();
        mat.density_kg_m3 = 0.0;
        let result = calculate_bond_strength(&mat, &test_sip(), &test_adhesive(), &test_geometry());
        assert!(result.is_err());
    }

    #[test]
    fn test_calculate_bond_strength_invalid_shear_modulus() {
        let mut sip = test_sip();
        sip.shear_modulus_mpa = 0.0;
        let result =
            calculate_bond_strength(&test_material(), &sip, &test_adhesive(), &test_geometry());
        assert!(result.is_err());
    }

    #[test]
    fn test_calculate_bond_strength_invalid_adhesive_strength() {
        let mut adhesive = test_adhesive();
        adhesive.shear_strength_mpa = 0.0;
        let result =
            calculate_bond_strength(&test_material(), &test_sip(), &adhesive, &test_geometry());
        assert!(result.is_err());
    }

    #[test]
    fn test_calculate_bond_stress() {
        let stress = calculate_bond_stress(0.001, &test_geometry()).unwrap();
        assert!(stress > 0.0);
    }

    #[test]
    fn test_calculate_bond_stress_invalid_thickness() {
        let mut geom = test_geometry();
        geom.dimensions.thickness_mm = 0.0;
        let result = calculate_bond_stress(0.001, &geom);
        assert!(result.is_err());
    }

    #[test]
    fn test_calculate_bond_stress_invalid_area() {
        let mut geom = test_geometry();
        geom.surface_area_m2 = 0.0;
        let result = calculate_bond_stress(0.001, &geom);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_bond_assembly_valid() {
        let assembly = BondAssembly {
            sip: test_sip(),
            adhesive: test_adhesive(),
            bond_strength_mpa: 2.0,
            bond_stress_mpa: 0.5,
        };
        assert!(validate_bond_assembly(&assembly).is_ok());
    }

    #[test]
    fn test_validate_bond_assembly_adhesive_too_thin() {
        let mut adhesive = test_adhesive();
        adhesive.thickness_mm = 0.1;
        let assembly = BondAssembly {
            sip: test_sip(),
            adhesive,
            bond_strength_mpa: 2.0,
            bond_stress_mpa: 0.5,
        };
        assert!(validate_bond_assembly(&assembly).is_err());
    }

    #[test]
    fn test_validate_bond_assembly_adhesive_too_thick() {
        let mut adhesive = test_adhesive();
        adhesive.thickness_mm = 0.4;
        let assembly = BondAssembly {
            sip: test_sip(),
            adhesive,
            bond_strength_mpa: 2.0,
            bond_stress_mpa: 0.5,
        };
        assert!(validate_bond_assembly(&assembly).is_err());
    }

    #[test]
    fn test_validate_bond_assembly_sip_too_thin() {
        let mut sip = test_sip();
        sip.thickness_mm = 2.0;
        let assembly = BondAssembly {
            sip,
            adhesive: test_adhesive(),
            bond_strength_mpa: 2.0,
            bond_stress_mpa: 0.5,
        };
        assert!(validate_bond_assembly(&assembly).is_err());
    }

    #[test]
    fn test_validate_bond_assembly_sip_too_thick() {
        let mut sip = test_sip();
        sip.thickness_mm = 7.0;
        let assembly = BondAssembly {
            sip,
            adhesive: test_adhesive(),
            bond_strength_mpa: 2.0,
            bond_stress_mpa: 0.5,
        };
        assert!(validate_bond_assembly(&assembly).is_err());
    }

    #[test]
    fn test_validate_bond_assembly_shear_too_low() {
        let assembly = BondAssembly {
            sip: test_sip(),
            adhesive: test_adhesive(),
            bond_strength_mpa: 0.5,
            bond_stress_mpa: 0.5,
        };
        assert!(validate_bond_assembly(&assembly).is_err());
    }

    #[test]
    fn test_validate_bond_assembly_peel_too_low() {
        let mut adhesive = test_adhesive();
        adhesive.peel_strength_n_m = 1000.0;
        let assembly = BondAssembly {
            sip: test_sip(),
            adhesive,
            bond_strength_mpa: 2.0,
            bond_stress_mpa: 0.5,
        };
        assert!(validate_bond_assembly(&assembly).is_err());
    }

    #[test]
    fn test_validate_bond_assembly_bond_strength_at_threshold() {
        let assembly = BondAssembly {
            sip: test_sip(),
            adhesive: test_adhesive(),
            bond_strength_mpa: 1.0,
            bond_stress_mpa: 0.5,
        };
        assert!(validate_bond_assembly(&assembly).is_err());
    }

    #[test]
    fn test_validate_bond_assembly_peel_at_threshold() {
        let mut adhesive = test_adhesive();
        adhesive.peel_strength_n_m = 2000.0;
        let assembly = BondAssembly {
            sip: test_sip(),
            adhesive,
            bond_strength_mpa: 2.0,
            bond_stress_mpa: 0.5,
        };
        assert!(validate_bond_assembly(&assembly).is_err());
    }
}

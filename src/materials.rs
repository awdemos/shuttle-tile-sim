use crate::{Adhesive, Coating, Result, StrainIsolationPad, TileError, TileMaterial};

const LI900_DENSITY_MIN: f64 = 130.0;
const LI900_DENSITY_MAX: f64 = 160.0;
const POROSITY_MIN: f64 = 0.90;
const POROSITY_MAX: f64 = 0.96;
const THERMAL_CONDUCTIVITY_MIN: f64 = 0.0;
const FIBER_DIAMETER_MIN: f64 = 0.0;
const COATING_THICKNESS_MIN: f64 = 0.25;
const COATING_THICKNESS_MAX: f64 = 0.50;
const EMISSIVITY_MIN: f64 = 0.0;
const EMISSIVITY_MAX: f64 = 1.0;
const ABSORPTIVITY_MIN: f64 = 0.0;
const ABSORPTIVITY_MAX: f64 = 1.0;
const SIP_THICKNESS_MIN: f64 = 0.0;
const ADHESIVE_SHEAR_STRENGTH_MIN: f64 = 0.0;
const ADHESIVE_THICKNESS_MIN: f64 = 0.0;
const ADHESIVE_CURE_TEMP_MIN: f64 = 0.0;
const ADHESIVE_CURE_TIME_MIN: f64 = 0.0;
const ADHESIVE_PEEL_STRENGTH_MIN: f64 = 0.0;
const ADHESIVE_MAX_SERVICE_TEMP_MIN: f64 = 0.0;
const SIP_DENSITY_MIN: f64 = 0.0;
const SIP_SHEAR_MODULUS_MIN: f64 = 0.0;
const SIP_MAX_TEMP_MIN: f64 = 0.0;
const COATING_MAX_TEMP_MIN: f64 = 0.0;
const SPECIFIC_HEAT_MIN: f64 = 0.0;
const MAX_SERVICE_TEMP_MIN: f64 = 0.0;

pub fn validate_material(material: &TileMaterial) -> Result<()> {
    if material.density_kg_m3 < LI900_DENSITY_MIN || material.density_kg_m3 > LI900_DENSITY_MAX {
        return Err(TileError::MaterialError(format!(
            "density {} out of range [{}, {}]",
            material.density_kg_m3, LI900_DENSITY_MIN, LI900_DENSITY_MAX
        )));
    }
    if material.porosity_fraction < POROSITY_MIN || material.porosity_fraction > POROSITY_MAX {
        return Err(TileError::MaterialError(format!(
            "porosity {} out of range [{}, {}]",
            material.porosity_fraction, POROSITY_MIN, POROSITY_MAX
        )));
    }
    if material.thermal_conductivity_w_m_k <= THERMAL_CONDUCTIVITY_MIN {
        return Err(TileError::MaterialError(format!(
            "thermal_conductivity {} must be > {}",
            material.thermal_conductivity_w_m_k, THERMAL_CONDUCTIVITY_MIN
        )));
    }
    if material.fiber_diameter_um <= FIBER_DIAMETER_MIN {
        return Err(TileError::MaterialError(format!(
            "fiber_diameter {} must be > {}",
            material.fiber_diameter_um, FIBER_DIAMETER_MIN
        )));
    }
    if material.specific_heat_j_kg_k <= SPECIFIC_HEAT_MIN {
        return Err(TileError::MaterialError(format!(
            "specific_heat {} must be > {}",
            material.specific_heat_j_kg_k, SPECIFIC_HEAT_MIN
        )));
    }
    if material.max_service_temp_c <= MAX_SERVICE_TEMP_MIN {
        return Err(TileError::MaterialError(format!(
            "max_service_temp {} must be > {}",
            material.max_service_temp_c, MAX_SERVICE_TEMP_MIN
        )));
    }
    Ok(())
}

pub fn validate_coating(coating: &Coating) -> Result<()> {
    if coating.thickness_mm < COATING_THICKNESS_MIN || coating.thickness_mm > COATING_THICKNESS_MAX {
        return Err(TileError::MaterialError(format!(
            "coating thickness {} out of range [{}, {}]",
            coating.thickness_mm, COATING_THICKNESS_MIN, COATING_THICKNESS_MAX
        )));
    }
    if coating.emissivity < EMISSIVITY_MIN || coating.emissivity > EMISSIVITY_MAX {
        return Err(TileError::MaterialError(format!(
            "emissivity {} out of range [{}, {}]",
            coating.emissivity, EMISSIVITY_MIN, EMISSIVITY_MAX
        )));
    }
    if coating.absorptivity < ABSORPTIVITY_MIN || coating.absorptivity > ABSORPTIVITY_MAX {
        return Err(TileError::MaterialError(format!(
            "absorptivity {} out of range [{}, {}]",
            coating.absorptivity, ABSORPTIVITY_MIN, ABSORPTIVITY_MAX
        )));
    }
    if coating.max_temp_c <= COATING_MAX_TEMP_MIN {
        return Err(TileError::MaterialError(format!(
            "coating max_temp {} must be > {}",
            coating.max_temp_c, COATING_MAX_TEMP_MIN
        )));
    }
    if coating.material.is_empty() {
        return Err(TileError::MaterialError(
            "coating material must not be empty".to_string(),
        ));
    }
    if coating.application_method.is_empty() {
        return Err(TileError::MaterialError(
            "coating application_method must not be empty".to_string(),
        ));
    }
    Ok(())
}

pub fn validate_sip(sip: &StrainIsolationPad) -> Result<()> {
    if sip.thickness_mm <= SIP_THICKNESS_MIN {
        return Err(TileError::MaterialError(format!(
            "sip thickness {} must be > {}",
            sip.thickness_mm, SIP_THICKNESS_MIN
        )));
    }
    if sip.density_kg_m3 <= SIP_DENSITY_MIN {
        return Err(TileError::MaterialError(format!(
            "sip density {} must be > {}",
            sip.density_kg_m3, SIP_DENSITY_MIN
        )));
    }
    if sip.shear_modulus_mpa <= SIP_SHEAR_MODULUS_MIN {
        return Err(TileError::MaterialError(format!(
            "sip shear_modulus {} must be > {}",
            sip.shear_modulus_mpa, SIP_SHEAR_MODULUS_MIN
        )));
    }
    if sip.max_temp_c <= SIP_MAX_TEMP_MIN {
        return Err(TileError::MaterialError(format!(
            "sip max_temp {} must be > {}",
            sip.max_temp_c, SIP_MAX_TEMP_MIN
        )));
    }
    if sip.material.is_empty() {
        return Err(TileError::MaterialError(
            "sip material must not be empty".to_string(),
        ));
    }
    Ok(())
}

pub fn validate_adhesive(adhesive: &Adhesive) -> Result<()> {
    if adhesive.shear_strength_mpa <= ADHESIVE_SHEAR_STRENGTH_MIN {
        return Err(TileError::MaterialError(format!(
            "adhesive shear_strength {} must be > {}",
            adhesive.shear_strength_mpa, ADHESIVE_SHEAR_STRENGTH_MIN
        )));
    }
    if adhesive.thickness_mm <= ADHESIVE_THICKNESS_MIN {
        return Err(TileError::MaterialError(format!(
            "adhesive thickness {} must be > {}",
            adhesive.thickness_mm, ADHESIVE_THICKNESS_MIN
        )));
    }
    if adhesive.cure_temp_c <= ADHESIVE_CURE_TEMP_MIN {
        return Err(TileError::MaterialError(format!(
            "adhesive cure_temp {} must be > {}",
            adhesive.cure_temp_c, ADHESIVE_CURE_TEMP_MIN
        )));
    }
    if adhesive.cure_time_hours <= ADHESIVE_CURE_TIME_MIN {
        return Err(TileError::MaterialError(format!(
            "adhesive cure_time {} must be > {}",
            adhesive.cure_time_hours, ADHESIVE_CURE_TIME_MIN
        )));
    }
    if adhesive.peel_strength_n_m <= ADHESIVE_PEEL_STRENGTH_MIN {
        return Err(TileError::MaterialError(format!(
            "adhesive peel_strength {} must be > {}",
            adhesive.peel_strength_n_m, ADHESIVE_PEEL_STRENGTH_MIN
        )));
    }
    if adhesive.max_service_temp_c <= ADHESIVE_MAX_SERVICE_TEMP_MIN {
        return Err(TileError::MaterialError(format!(
            "adhesive max_service_temp {} must be > {}",
            adhesive.max_service_temp_c, ADHESIVE_MAX_SERVICE_TEMP_MIN
        )));
    }
    if adhesive.material.is_empty() {
        return Err(TileError::MaterialError(
            "adhesive material must not be empty".to_string(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_material() -> TileMaterial {
        TileMaterial {
            density_kg_m3: 144.0,
            porosity_fraction: 0.93,
            thermal_conductivity_w_m_k: 0.017,
            specific_heat_j_kg_k: 628.0,
            max_service_temp_c: 1260.0,
            fiber_diameter_um: 1.0,
        }
    }

    fn valid_coating() -> Coating {
        Coating {
            material: "Borosilicate glass".to_string(),
            thickness_mm: 0.38,
            emissivity: 0.85,
            absorptivity: 0.85,
            max_temp_c: 1260.0,
            application_method: "Spray + sinter".to_string(),
        }
    }

    fn valid_sip() -> StrainIsolationPad {
        StrainIsolationPad {
            material: "Nomex felt".to_string(),
            thickness_mm: 4.76,
            density_kg_m3: 64.0,
            shear_modulus_mpa: 0.5,
            max_temp_c: 260.0,
        }
    }

    fn valid_adhesive() -> Adhesive {
        Adhesive {
            material: "RTV-560 silicone".to_string(),
            thickness_mm: 0.25,
            cure_temp_c: 25.0,
            cure_time_hours: 72.0,
            shear_strength_mpa: 1.5,
            peel_strength_n_m: 3500.0,
            max_service_temp_c: 260.0,
        }
    }

    #[test]
    fn test_validate_material_valid() {
        assert!(validate_material(&valid_material()).is_ok());
    }

    #[test]
    fn test_validate_material_density_too_low() {
        let mut m = valid_material();
        m.density_kg_m3 = 129.0;
        assert!(validate_material(&m).is_err());
    }

    #[test]
    fn test_validate_material_density_too_high() {
        let mut m = valid_material();
        m.density_kg_m3 = 161.0;
        assert!(validate_material(&m).is_err());
    }

    #[test]
    fn test_validate_material_density_at_bounds() {
        let mut m = valid_material();
        m.density_kg_m3 = 130.0;
        assert!(validate_material(&m).is_ok());
        m.density_kg_m3 = 160.0;
        assert!(validate_material(&m).is_ok());
    }

    #[test]
    fn test_validate_material_porosity_too_low() {
        let mut m = valid_material();
        m.porosity_fraction = 0.89;
        assert!(validate_material(&m).is_err());
    }

    #[test]
    fn test_validate_material_porosity_too_high() {
        let mut m = valid_material();
        m.porosity_fraction = 0.97;
        assert!(validate_material(&m).is_err());
    }

    #[test]
    fn test_validate_material_porosity_at_bounds() {
        let mut m = valid_material();
        m.porosity_fraction = 0.90;
        assert!(validate_material(&m).is_ok());
        m.porosity_fraction = 0.96;
        assert!(validate_material(&m).is_ok());
    }

    #[test]
    fn test_validate_material_thermal_conductivity_zero() {
        let mut m = valid_material();
        m.thermal_conductivity_w_m_k = 0.0;
        assert!(validate_material(&m).is_err());
    }

    #[test]
    fn test_validate_material_thermal_conductivity_negative() {
        let mut m = valid_material();
        m.thermal_conductivity_w_m_k = -0.01;
        assert!(validate_material(&m).is_err());
    }

    #[test]
    fn test_validate_material_fiber_diameter_zero() {
        let mut m = valid_material();
        m.fiber_diameter_um = 0.0;
        assert!(validate_material(&m).is_err());
    }

    #[test]
    fn test_validate_material_fiber_diameter_negative() {
        let mut m = valid_material();
        m.fiber_diameter_um = -1.0;
        assert!(validate_material(&m).is_err());
    }

    #[test]
    fn test_validate_material_specific_heat_zero() {
        let mut m = valid_material();
        m.specific_heat_j_kg_k = 0.0;
        assert!(validate_material(&m).is_err());
    }

    #[test]
    fn test_validate_material_max_service_temp_zero() {
        let mut m = valid_material();
        m.max_service_temp_c = 0.0;
        assert!(validate_material(&m).is_err());
    }

    #[test]
    fn test_validate_coating_valid() {
        assert!(validate_coating(&valid_coating()).is_ok());
    }

    #[test]
    fn test_validate_coating_thickness_too_low() {
        let mut c = valid_coating();
        c.thickness_mm = 0.24;
        assert!(validate_coating(&c).is_err());
    }

    #[test]
    fn test_validate_coating_thickness_too_high() {
        let mut c = valid_coating();
        c.thickness_mm = 0.51;
        assert!(validate_coating(&c).is_err());
    }

    #[test]
    fn test_validate_coating_thickness_at_bounds() {
        let mut c = valid_coating();
        c.thickness_mm = 0.25;
        assert!(validate_coating(&c).is_ok());
        c.thickness_mm = 0.50;
        assert!(validate_coating(&c).is_ok());
    }

    #[test]
    fn test_validate_coating_emissivity_too_low() {
        let mut c = valid_coating();
        c.emissivity = -0.01;
        assert!(validate_coating(&c).is_err());
    }

    #[test]
    fn test_validate_coating_emissivity_too_high() {
        let mut c = valid_coating();
        c.emissivity = 1.01;
        assert!(validate_coating(&c).is_err());
    }

    #[test]
    fn test_validate_coating_emissivity_at_bounds() {
        let mut c = valid_coating();
        c.emissivity = 0.0;
        assert!(validate_coating(&c).is_ok());
        c.emissivity = 1.0;
        assert!(validate_coating(&c).is_ok());
    }

    #[test]
    fn test_validate_coating_absorptivity_too_low() {
        let mut c = valid_coating();
        c.absorptivity = -0.01;
        assert!(validate_coating(&c).is_err());
    }

    #[test]
    fn test_validate_coating_absorptivity_too_high() {
        let mut c = valid_coating();
        c.absorptivity = 1.01;
        assert!(validate_coating(&c).is_err());
    }

    #[test]
    fn test_validate_coating_absorptivity_at_bounds() {
        let mut c = valid_coating();
        c.absorptivity = 0.0;
        assert!(validate_coating(&c).is_ok());
        c.absorptivity = 1.0;
        assert!(validate_coating(&c).is_ok());
    }

    #[test]
    fn test_validate_coating_max_temp_zero() {
        let mut c = valid_coating();
        c.max_temp_c = 0.0;
        assert!(validate_coating(&c).is_err());
    }

    #[test]
    fn test_validate_coating_material_empty() {
        let mut c = valid_coating();
        c.material = "".to_string();
        assert!(validate_coating(&c).is_err());
    }

    #[test]
    fn test_validate_coating_application_method_empty() {
        let mut c = valid_coating();
        c.application_method = "".to_string();
        assert!(validate_coating(&c).is_err());
    }

    #[test]
    fn test_validate_sip_valid() {
        assert!(validate_sip(&valid_sip()).is_ok());
    }

    #[test]
    fn test_validate_sip_thickness_zero() {
        let mut s = valid_sip();
        s.thickness_mm = 0.0;
        assert!(validate_sip(&s).is_err());
    }

    #[test]
    fn test_validate_sip_thickness_negative() {
        let mut s = valid_sip();
        s.thickness_mm = -1.0;
        assert!(validate_sip(&s).is_err());
    }

    #[test]
    fn test_validate_sip_density_zero() {
        let mut s = valid_sip();
        s.density_kg_m3 = 0.0;
        assert!(validate_sip(&s).is_err());
    }

    #[test]
    fn test_validate_sip_shear_modulus_zero() {
        let mut s = valid_sip();
        s.shear_modulus_mpa = 0.0;
        assert!(validate_sip(&s).is_err());
    }

    #[test]
    fn test_validate_sip_max_temp_zero() {
        let mut s = valid_sip();
        s.max_temp_c = 0.0;
        assert!(validate_sip(&s).is_err());
    }

    #[test]
    fn test_validate_sip_material_empty() {
        let mut s = valid_sip();
        s.material = "".to_string();
        assert!(validate_sip(&s).is_err());
    }

    #[test]
    fn test_validate_adhesive_valid() {
        assert!(validate_adhesive(&valid_adhesive()).is_ok());
    }

    #[test]
    fn test_validate_adhesive_shear_strength_zero() {
        let mut a = valid_adhesive();
        a.shear_strength_mpa = 0.0;
        assert!(validate_adhesive(&a).is_err());
    }

    #[test]
    fn test_validate_adhesive_shear_strength_negative() {
        let mut a = valid_adhesive();
        a.shear_strength_mpa = -1.0;
        assert!(validate_adhesive(&a).is_err());
    }

    #[test]
    fn test_validate_adhesive_thickness_zero() {
        let mut a = valid_adhesive();
        a.thickness_mm = 0.0;
        assert!(validate_adhesive(&a).is_err());
    }

    #[test]
    fn test_validate_adhesive_cure_temp_zero() {
        let mut a = valid_adhesive();
        a.cure_temp_c = 0.0;
        assert!(validate_adhesive(&a).is_err());
    }

    #[test]
    fn test_validate_adhesive_cure_time_zero() {
        let mut a = valid_adhesive();
        a.cure_time_hours = 0.0;
        assert!(validate_adhesive(&a).is_err());
    }

    #[test]
    fn test_validate_adhesive_peel_strength_zero() {
        let mut a = valid_adhesive();
        a.peel_strength_n_m = 0.0;
        assert!(validate_adhesive(&a).is_err());
    }

    #[test]
    fn test_validate_adhesive_max_service_temp_zero() {
        let mut a = valid_adhesive();
        a.max_service_temp_c = 0.0;
        assert!(validate_adhesive(&a).is_err());
    }

    #[test]
    fn test_validate_adhesive_material_empty() {
        let mut a = valid_adhesive();
        a.material = "".to_string();
        assert!(validate_adhesive(&a).is_err());
    }

    #[test]
    fn test_li900_factory_method_passes_validation() {
        let m = TileMaterial::li_900();
        assert!(validate_material(&m).is_ok());
    }

    #[test]
    fn test_black_hrsi_factory_method_passes_validation() {
        let c = Coating::black_hrsi();
        assert!(validate_coating(&c).is_ok());
    }

    #[test]
    fn test_standard_sip_factory_method_passes_validation() {
        let s = StrainIsolationPad::standard();
        assert!(validate_sip(&s).is_ok());
    }

    #[test]
    fn test_rtv560_factory_method_passes_validation() {
        let a = Adhesive::rtv_560();
        assert!(validate_adhesive(&a).is_ok());
    }
}

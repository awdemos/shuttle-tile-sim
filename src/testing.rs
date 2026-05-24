use crate::{
    Adhesive, AdhesionTestResult, Coating, FailureMode, MechanicalTestResult, Result,
    StrainIsolationPad, TestOutcome, TestRecord, TestType, ThermalTestResult, TileError,
    TileGeometry, TileMaterial,
};
use chrono::Utc;
use rand::Rng;
use std::collections::HashMap;

pub const THERMAL_MAX_TEMP_NOMINAL_C: f64 = 1260.0;
pub const THERMAL_BACKFACE_MAX_C: f64 = 150.0;
pub const THERMAL_CYCLES_NOMINAL: u32 = 100;
pub const MECH_COMPRESSIVE_MIN_MPA: f64 = 0.1;
pub const MECH_TENSILE_MIN_MPA: f64 = 0.05;
pub const MECH_FLEXURAL_MIN_MPA: f64 = 0.08;
pub const MECH_YOUNGS_MIN_GPA: f64 = 0.5;
pub const ADHESION_BOND_MIN_MPA: f64 = 1.0;
pub const ADHESION_PEEL_MIN_N_M: f64 = 2000.0;
pub const COATING_THICKNESS_MIN_MM: f64 = 0.25;
pub const COATING_THICKNESS_MAX_MM: f64 = 0.50;
pub const DENSITY_MIN_KG_M3: f64 = 130.0;
pub const DENSITY_MAX_KG_M3: f64 = 160.0;

pub fn run_thermal_test(
    material: &TileMaterial,
    geometry: &TileGeometry,
    coating: &Coating,
    cycles: u32,
    heat_flux_w_cm2: f64,
    duration_seconds: f64,
) -> Result<ThermalTestResult> {
    if geometry.volume_m3 <= 0.0 {
        return Err(TileError::TestError("Invalid geometry volume".to_string()));
    }

    let max_temp_reached_c = heat_flux_w_cm2 * 18.0 + 150.0;
    let backface_temp_c = max_temp_reached_c * (1.0 - material.porosity_fraction) * 0.2;
    let degradation_percent = (cycles as f64 / THERMAL_CYCLES_NOMINAL as f64) * 10.0;

    let passed = max_temp_reached_c < THERMAL_MAX_TEMP_NOMINAL_C
        && backface_temp_c < THERMAL_BACKFACE_MAX_C
        && coating.thickness_mm >= COATING_THICKNESS_MIN_MM
        && coating.thickness_mm <= COATING_THICKNESS_MAX_MM
        && material.density_kg_m3 >= DENSITY_MIN_KG_M3
        && material.density_kg_m3 <= DENSITY_MAX_KG_M3;

    Ok(ThermalTestResult {
        max_temp_reached_c,
        backface_temp_c,
        heat_flux_w_cm2,
        duration_seconds,
        cycles_completed: cycles,
        degradation_percent,
        passed,
    })
}

pub fn run_mechanical_test(
    material: &TileMaterial,
    geometry: &TileGeometry,
) -> Result<MechanicalTestResult> {
    if geometry.volume_m3 <= 0.0 {
        return Err(TileError::TestError("Invalid geometry volume".to_string()));
    }

    let compressive_strength_mpa =
        material.density_kg_m3 * 0.001 * (1.0 - material.porosity_fraction).powi(2);
    let tensile_strength_mpa = compressive_strength_mpa * 0.5;
    let flexural_strength_mpa = compressive_strength_mpa * 0.8;
    let youngs_modulus_gpa = material.density_kg_m3 * 0.0035;

    let passed = compressive_strength_mpa >= MECH_COMPRESSIVE_MIN_MPA
        && tensile_strength_mpa >= MECH_TENSILE_MIN_MPA
        && flexural_strength_mpa >= MECH_FLEXURAL_MIN_MPA
        && youngs_modulus_gpa >= MECH_YOUNGS_MIN_GPA
        && material.density_kg_m3 >= DENSITY_MIN_KG_M3
        && material.density_kg_m3 <= DENSITY_MAX_KG_M3;

    Ok(MechanicalTestResult {
        compressive_strength_mpa,
        tensile_strength_mpa,
        flexural_strength_mpa,
        youngs_modulus_gpa,
        passed,
    })
}

pub fn run_adhesion_test(
    _sip: &StrainIsolationPad,
    adhesive: &Adhesive,
    geometry: &TileGeometry,
) -> Result<AdhesionTestResult> {
    if geometry.volume_m3 <= 0.0 {
        return Err(TileError::TestError("Invalid geometry volume".to_string()));
    }

    let random_factor: f64 = rand::thread_rng().gen();
    let bond_strength_mpa = adhesive.shear_strength_mpa * (1.0 - 0.1 * random_factor);
    let peel_strength_n_m = adhesive.peel_strength_n_m * (1.0 - 0.05 * random_factor);

    let failure_mode = determine_failure_mode(bond_strength_mpa, peel_strength_n_m, adhesive);

    let passed = bond_strength_mpa >= ADHESION_BOND_MIN_MPA
        && peel_strength_n_m >= ADHESION_PEEL_MIN_N_M;

    Ok(AdhesionTestResult {
        bond_strength_mpa,
        peel_strength_n_m,
        failure_mode,
        passed,
    })
}

pub fn determine_failure_mode(
    bond_strength: f64,
    peel_strength: f64,
    adhesive: &Adhesive,
) -> FailureMode {
    let shear_threshold = adhesive.shear_strength_mpa * 0.5;
    let peel_threshold = adhesive.peel_strength_n_m * 0.5;
    let shear_low = adhesive.shear_strength_mpa * 0.25;
    let peel_low = adhesive.peel_strength_n_m * 0.25;

    if bond_strength >= shear_threshold && peel_strength >= peel_threshold {
        FailureMode::None
    } else if bond_strength < shear_low && peel_strength < peel_low {
        FailureMode::Substrate
    } else if bond_strength < shear_threshold && peel_strength < peel_threshold {
        FailureMode::Mixed
    } else if bond_strength < shear_threshold {
        FailureMode::Cohesive
    } else if peel_strength < peel_threshold {
        FailureMode::Adhesive
    } else {
        FailureMode::None
    }
}

pub fn run_full_test_suite(
    material: &TileMaterial,
    geometry: &TileGeometry,
    coating: &Coating,
    sip: &StrainIsolationPad,
    adhesive: &Adhesive,
) -> Result<Vec<TestRecord>> {
    let thermal_result = run_thermal_test(material, geometry, coating, 100, 40.0, 900.0)?;
    let mechanical_result = run_mechanical_test(material, geometry)?;
    let adhesion_result = run_adhesion_test(sip, adhesive, geometry)?;

    let mut records = Vec::new();

    let mut thermal_data = HashMap::new();
    thermal_data.insert(
        "max_temp_reached_c".to_string(),
        thermal_result.max_temp_reached_c,
    );
    thermal_data.insert("backface_temp_c".to_string(), thermal_result.backface_temp_c);
    thermal_data.insert("heat_flux_w_cm2".to_string(), thermal_result.heat_flux_w_cm2);
    thermal_data.insert(
        "duration_seconds".to_string(),
        thermal_result.duration_seconds,
    );
    thermal_data.insert(
        "cycles_completed".to_string(),
        thermal_result.cycles_completed as f64,
    );
    thermal_data.insert(
        "degradation_percent".to_string(),
        thermal_result.degradation_percent,
    );
    records.push(create_test_record(TestType::Thermal, thermal_result.passed, thermal_data));

    let mut mechanical_data = HashMap::new();
    mechanical_data.insert(
        "compressive_strength_mpa".to_string(),
        mechanical_result.compressive_strength_mpa,
    );
    mechanical_data.insert(
        "tensile_strength_mpa".to_string(),
        mechanical_result.tensile_strength_mpa,
    );
    mechanical_data.insert(
        "flexural_strength_mpa".to_string(),
        mechanical_result.flexural_strength_mpa,
    );
    mechanical_data.insert(
        "youngs_modulus_gpa".to_string(),
        mechanical_result.youngs_modulus_gpa,
    );
    records.push(create_test_record(
        TestType::Mechanical,
        mechanical_result.passed,
        mechanical_data,
    ));

    let mut adhesion_data = HashMap::new();
    adhesion_data.insert("bond_strength_mpa".to_string(), adhesion_result.bond_strength_mpa);
    adhesion_data.insert("peel_strength_n_m".to_string(), adhesion_result.peel_strength_n_m);
    records.push(create_test_record(
        TestType::Adhesion,
        adhesion_result.passed,
        adhesion_data,
    ));

    Ok(records)
}

pub fn create_test_record(
    test_type: TestType,
    passed: bool,
    data: HashMap<String, f64>,
) -> TestRecord {
    TestRecord {
        test_type,
        result: if passed {
            TestOutcome::Pass
        } else {
            TestOutcome::Fail
        },
        timestamp: Utc::now(),
        data,
        notes: String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Dimensions, FailureMode, TestOutcome, TestType, TileShape};

    fn test_material() -> TileMaterial {
        TileMaterial {
            density_kg_m3: 160.0,
            porosity_fraction: 0.15,
            thermal_conductivity_w_m_k: 0.017,
            specific_heat_j_kg_k: 628.0,
            max_service_temp_c: 1260.0,
            fiber_diameter_um: 1.0,
        }
    }

    fn test_geometry() -> TileGeometry {
        TileGeometry {
            dimensions: Dimensions {
                length_mm: 150.0,
                width_mm: 150.0,
                thickness_mm: 15.0,
            },
            surface_area_m2: 0.0225,
            volume_m3: 0.0003375,
            shape_type: TileShape::Flat,
            machining_tolerance_mm: 0.1,
            edge_radius_mm: Some(2.0),
        }
    }

    fn test_coating() -> Coating {
        Coating {
            material: "Test".to_string(),
            thickness_mm: 0.38,
            emissivity: 0.85,
            absorptivity: 0.85,
            max_temp_c: 1260.0,
            application_method: "Spray".to_string(),
        }
    }

    fn test_sip() -> StrainIsolationPad {
        StrainIsolationPad {
            material: "Nomex".to_string(),
            thickness_mm: 4.76,
            density_kg_m3: 64.0,
            shear_modulus_mpa: 0.5,
            max_temp_c: 260.0,
        }
    }

    fn test_adhesive() -> Adhesive {
        Adhesive {
            material: "RTV-560".to_string(),
            thickness_mm: 0.25,
            cure_temp_c: 25.0,
            cure_time_hours: 72.0,
            shear_strength_mpa: 2.0,
            peel_strength_n_m: 4000.0,
            max_service_temp_c: 260.0,
        }
    }

    fn weak_material() -> TileMaterial {
        TileMaterial {
            density_kg_m3: 100.0,
            porosity_fraction: 0.95,
            thermal_conductivity_w_m_k: 0.01,
            specific_heat_j_kg_k: 500.0,
            max_service_temp_c: 800.0,
            fiber_diameter_um: 1.0,
        }
    }

    fn weak_adhesive() -> Adhesive {
        Adhesive {
            material: "Weak".to_string(),
            thickness_mm: 0.25,
            cure_temp_c: 25.0,
            cure_time_hours: 24.0,
            shear_strength_mpa: 0.5,
            peel_strength_n_m: 1000.0,
            max_service_temp_c: 100.0,
        }
    }

    #[test]
    fn thermal_test_nominal_pass() {
        let material = test_material();
        let geometry = test_geometry();
        let coating = test_coating();
        let result =
            run_thermal_test(&material, &geometry, &coating, 100, 40.0, 900.0).unwrap();
        assert!(result.passed);
        assert!(result.max_temp_reached_c < THERMAL_MAX_TEMP_NOMINAL_C);
        assert!(result.backface_temp_c < THERMAL_BACKFACE_MAX_C);
    }

    #[test]
    fn thermal_test_extreme_heat_fail() {
        let material = test_material();
        let geometry = test_geometry();
        let coating = test_coating();
        let result =
            run_thermal_test(&material, &geometry, &coating, 100, 100.0, 900.0).unwrap();
        assert!(!result.passed);
        assert!(result.max_temp_reached_c >= THERMAL_MAX_TEMP_NOMINAL_C);
    }

    #[test]
    fn thermal_test_coating_too_thin_fail() {
        let material = test_material();
        let geometry = test_geometry();
        let mut coating = test_coating();
        coating.thickness_mm = 0.1;
        let result =
            run_thermal_test(&material, &geometry, &coating, 100, 40.0, 900.0).unwrap();
        assert!(!result.passed);
    }

    #[test]
    fn thermal_test_density_too_low_fail() {
        let mut material = test_material();
        material.density_kg_m3 = 100.0;
        let geometry = test_geometry();
        let coating = test_coating();
        let result =
            run_thermal_test(&material, &geometry, &coating, 100, 40.0, 900.0).unwrap();
        assert!(!result.passed);
    }

    #[test]
    fn thermal_test_backface_overload_fail() {
        let mut material = test_material();
        material.porosity_fraction = 0.05;
        let geometry = test_geometry();
        let coating = test_coating();
        let result =
            run_thermal_test(&material, &geometry, &coating, 100, 40.0, 900.0).unwrap();
        assert!(!result.passed);
        assert!(result.backface_temp_c >= THERMAL_BACKFACE_MAX_C);
    }

    #[test]
    fn mechanical_test_strong_pass() {
        let material = test_material();
        let geometry = test_geometry();
        let result = run_mechanical_test(&material, &geometry).unwrap();
        assert!(result.passed);
        assert!(result.compressive_strength_mpa >= MECH_COMPRESSIVE_MIN_MPA);
        assert!(result.tensile_strength_mpa >= MECH_TENSILE_MIN_MPA);
        assert!(result.flexural_strength_mpa >= MECH_FLEXURAL_MIN_MPA);
        assert!(result.youngs_modulus_gpa >= MECH_YOUNGS_MIN_GPA);
    }

    #[test]
    fn mechanical_test_weak_fail() {
        let material = weak_material();
        let geometry = test_geometry();
        let result = run_mechanical_test(&material, &geometry).unwrap();
        assert!(!result.passed);
    }

    #[test]
    fn mechanical_test_density_out_of_range_fail() {
        let mut material = test_material();
        material.density_kg_m3 = 200.0;
        let geometry = test_geometry();
        let result = run_mechanical_test(&material, &geometry).unwrap();
        assert!(!result.passed);
    }

    #[test]
    fn mechanical_test_compressive_fail() {
        let mut material = test_material();
        material.porosity_fraction = 0.95;
        let geometry = test_geometry();
        let result = run_mechanical_test(&material, &geometry).unwrap();
        assert!(!result.passed);
    }

    #[test]
    fn adhesion_test_strong_pass() {
        let sip = test_sip();
        let adhesive = test_adhesive();
        let geometry = test_geometry();
        let result = run_adhesion_test(&sip, &adhesive, &geometry).unwrap();
        assert!(result.bond_strength_mpa >= ADHESION_BOND_MIN_MPA);
        assert!(result.peel_strength_n_m >= ADHESION_PEEL_MIN_N_M);
        assert!(result.passed);
    }

    #[test]
    fn adhesion_test_weak_fail() {
        let sip = test_sip();
        let adhesive = weak_adhesive();
        let geometry = test_geometry();
        let result = run_adhesion_test(&sip, &adhesive, &geometry).unwrap();
        assert!(!result.passed);
    }

    #[test]
    fn failure_mode_none() {
        let adhesive = test_adhesive();
        let mode = determine_failure_mode(1.5, 3000.0, &adhesive);
        assert_eq!(mode, FailureMode::None);
    }

    #[test]
    fn failure_mode_cohesive() {
        let adhesive = test_adhesive();
        let mode = determine_failure_mode(0.5, 3000.0, &adhesive);
        assert_eq!(mode, FailureMode::Cohesive);
    }

    #[test]
    fn failure_mode_adhesive() {
        let adhesive = test_adhesive();
        let mode = determine_failure_mode(1.5, 1000.0, &adhesive);
        assert_eq!(mode, FailureMode::Adhesive);
    }

    #[test]
    fn failure_mode_substrate() {
        let adhesive = test_adhesive();
        let mode = determine_failure_mode(0.1, 200.0, &adhesive);
        assert_eq!(mode, FailureMode::Substrate);
    }

    #[test]
    fn failure_mode_mixed() {
        let adhesive = test_adhesive();
        let mode = determine_failure_mode(0.6, 1500.0, &adhesive);
        assert_eq!(mode, FailureMode::Mixed);
    }

    #[test]
    fn full_test_suite_pass() {
        let material = test_material();
        let geometry = test_geometry();
        let coating = test_coating();
        let sip = test_sip();
        let adhesive = test_adhesive();
        let records =
            run_full_test_suite(&material, &geometry, &coating, &sip, &adhesive).unwrap();
        assert_eq!(records.len(), 3);
        assert_eq!(records[0].test_type, TestType::Thermal);
        assert_eq!(records[1].test_type, TestType::Mechanical);
        assert_eq!(records[2].test_type, TestType::Adhesion);
        assert_eq!(records[0].result, TestOutcome::Pass);
        assert_eq!(records[1].result, TestOutcome::Pass);
        assert_eq!(records[2].result, TestOutcome::Pass);
    }

    #[test]
    fn full_test_suite_some_fail() {
        let material = weak_material();
        let geometry = test_geometry();
        let coating = test_coating();
        let sip = test_sip();
        let adhesive = weak_adhesive();
        let records =
            run_full_test_suite(&material, &geometry, &coating, &sip, &adhesive).unwrap();
        assert_eq!(records.len(), 3);
        assert!(
            records[0].result == TestOutcome::Fail
                || records[1].result == TestOutcome::Fail
                || records[2].result == TestOutcome::Fail
        );
    }

    #[test]
    fn create_test_record_pass() {
        let mut data = HashMap::new();
        data.insert("value".to_string(), 42.0);
        let record = create_test_record(TestType::Thermal, true, data);
        assert_eq!(record.test_type, TestType::Thermal);
        assert_eq!(record.result, TestOutcome::Pass);
        assert_eq!(record.data.get("value"), Some(&42.0));
    }

    #[test]
    fn create_test_record_fail() {
        let mut data = HashMap::new();
        data.insert("value".to_string(), 0.0);
        let record = create_test_record(TestType::Mechanical, false, data);
        assert_eq!(record.test_type, TestType::Mechanical);
        assert_eq!(record.result, TestOutcome::Fail);
    }

    #[test]
    fn thermal_test_invalid_geometry() {
        let material = test_material();
        let mut geometry = test_geometry();
        geometry.volume_m3 = 0.0;
        let coating = test_coating();
        let result = run_thermal_test(&material, &geometry, &coating, 100, 40.0, 900.0);
        assert!(result.is_err());
    }

    #[test]
    fn mechanical_test_invalid_geometry() {
        let material = test_material();
        let mut geometry = test_geometry();
        geometry.volume_m3 = -1.0;
        let result = run_mechanical_test(&material, &geometry);
        assert!(result.is_err());
    }

    #[test]
    fn adhesion_test_invalid_geometry() {
        let sip = test_sip();
        let adhesive = test_adhesive();
        let mut geometry = test_geometry();
        geometry.volume_m3 = 0.0;
        let result = run_adhesion_test(&sip, &adhesive, &geometry);
        assert!(result.is_err());
    }

    #[test]
    fn thermal_test_degradation_calculation() {
        let material = test_material();
        let geometry = test_geometry();
        let coating = test_coating();
        let result =
            run_thermal_test(&material, &geometry, &coating, 50, 40.0, 900.0).unwrap();
        assert_eq!(result.degradation_percent, 5.0);
    }

    #[test]
    fn mechanical_test_formula_verification() {
        let material = TileMaterial {
            density_kg_m3: 160.0,
            porosity_fraction: 0.0,
            thermal_conductivity_w_m_k: 0.017,
            specific_heat_j_kg_k: 628.0,
            max_service_temp_c: 1260.0,
            fiber_diameter_um: 1.0,
        };
        let geometry = test_geometry();
        let result = run_mechanical_test(&material, &geometry).unwrap();
        assert!((result.compressive_strength_mpa - 0.16).abs() < 0.001);
        assert!((result.tensile_strength_mpa - 0.08).abs() < 0.001);
        assert!((result.flexural_strength_mpa - 0.128).abs() < 0.001);
        assert!((result.youngs_modulus_gpa - 0.56).abs() < 0.001);
    }
}

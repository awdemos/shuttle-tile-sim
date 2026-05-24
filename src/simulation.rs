use crate::{Coating, DegradationState, Result, SimulationState, TileError, TileGeometry, TileMaterial};
use rand::Rng;
use rand::SeedableRng;

pub const REENTRY_HEAT_FLUX_W_CM2: f64 = 30.0;
pub const EXTREME_HEAT_FLUX_W_CM2: f64 = 80.0;
pub const REENTRY_DURATION_SECONDS: f64 = 1200.0;
pub const AMBIENT_TEMP_C: f64 = 25.0;
pub const MAX_ACCEPTABLE_BACKFACE_TEMP_C: f64 = 150.0;
pub const STEFAN_BOLTZMANN_W_M2_K4: f64 = 5.67e-8;
pub const COATING_DEGRADATION_RATE_PER_CYCLE: f64 = 0.001;
pub const BOND_STRESS_ACCUMULATION_RATE: f64 = 0.01;

const COATING_THERMAL_CONDUCTIVITY_W_M_K: f64 = 1.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SimulationConfig {
    pub seed: u64,
}

pub fn calculate_effective_thermal_conductivity(
    material: &TileMaterial,
    coating: &Coating,
    geometry: &TileGeometry,
) -> f64 {
    let thickness_ratio = coating.thickness_mm / geometry.dimensions.thickness_mm;
    material.thermal_conductivity_w_m_k * (1.0 + material.porosity_fraction)
        + COATING_THERMAL_CONDUCTIVITY_W_M_K * thickness_ratio
}

pub fn calculate_heat_transfer(
    heat_flux_w_cm2: f64,
    _material: &TileMaterial,
    geometry: &TileGeometry,
    coating: &Coating,
    duration_seconds: f64,
) -> Result<(f64, f64)> {
    if heat_flux_w_cm2 < 0.0 {
        return Err(TileError::SimulationError(
            "Heat flux must be non-negative".to_string(),
        ));
    }
    if duration_seconds < 0.0 {
        return Err(TileError::SimulationError(
            "Duration must be non-negative".to_string(),
        ));
    }
    if geometry.surface_area_m2 <= 0.0 {
        return Err(TileError::SimulationError(
            "Surface area must be positive".to_string(),
        ));
    }

    let reflectivity = 1.0 - coating.absorptivity;
    let total_energy_kj = heat_flux_w_cm2 * geometry.surface_area_m2 * duration_seconds * 10.0;

    let heat_absorbed_kj = total_energy_kj * (1.0 - reflectivity);
    let heat_reflected_kj = total_energy_kj * reflectivity;

    Ok((heat_absorbed_kj, heat_reflected_kj))
}

pub fn calculate_backface_temp(
    heat_flux_w_cm2: f64,
    material: &TileMaterial,
    geometry: &TileGeometry,
) -> f64 {
    let q_w_m2 = heat_flux_w_cm2 * 10_000.0;
    let t_front_k = (q_w_m2 / STEFAN_BOLTZMANN_W_M2_K4).powf(0.25);
    let t_front_c = t_front_k - 273.15;

    let k_eff = material.thermal_conductivity_w_m_k * (1.0 + material.porosity_fraction);
    let thickness_m = geometry.dimensions.thickness_mm / 1000.0;
    let delta_t = q_w_m2 * thickness_m / k_eff;

    t_front_c - delta_t
}

pub fn simulate_thermal_cycle(
    state: &mut SimulationState,
    material: &TileMaterial,
    geometry: &TileGeometry,
    coating: &Coating,
    heat_flux_w_cm2: f64,
    duration_seconds: f64,
) -> Result<()> {
    if heat_flux_w_cm2 < 0.0 {
        return Err(TileError::SimulationError(
            "Heat flux must be non-negative".to_string(),
        ));
    }
    if duration_seconds < 0.0 {
        return Err(TileError::SimulationError(
            "Duration must be non-negative".to_string(),
        ));
    }

    let q_w_m2 = heat_flux_w_cm2 * 10_000.0;
    let t_front_k = (q_w_m2 / STEFAN_BOLTZMANN_W_M2_K4).powf(0.25);
    let t_front_c = t_front_k - 273.15;

    let k_eff = calculate_effective_thermal_conductivity(material, coating, geometry);
    let thickness_m = geometry.dimensions.thickness_mm / 1000.0;
    let delta_t = q_w_m2 * thickness_m / k_eff;
    let t_back_c = t_front_c - delta_t;

    state.current_temp_c = t_front_c;
    if t_front_c > state.max_temp_seen_c {
        state.max_temp_seen_c = t_front_c;
    }
    if t_back_c > state.max_temp_seen_c {
        state.max_temp_seen_c = t_back_c;
    }

    let (heat_absorbed_kj, heat_reflected_kj) =
        calculate_heat_transfer(heat_flux_w_cm2, material, geometry, coating, duration_seconds)?;

    state.heat_absorbed_kj += heat_absorbed_kj;
    state.heat_reflected_kj += heat_reflected_kj;
    state.thermal_cycles += 1;
    state.coating_wear_percent += COATING_DEGRADATION_RATE_PER_CYCLE;
    state.degradation_state = determine_degradation_state(
        state.thermal_cycles,
        state.coating_wear_percent,
        state.max_temp_seen_c,
        material,
    );
    state.bond_stress_mpa += BOND_STRESS_ACCUMULATION_RATE;

    Ok(())
}

pub fn simulate_reentry(
    material: &TileMaterial,
    geometry: &TileGeometry,
    coating: &Coating,
    cycles: u32,
) -> Result<SimulationState> {
    let mut state = SimulationState {
        current_temp_c: AMBIENT_TEMP_C,
        max_temp_seen_c: AMBIENT_TEMP_C,
        heat_absorbed_kj: 0.0,
        heat_reflected_kj: 0.0,
        thermal_cycles: 0,
        degradation_state: DegradationState::Nominal,
        coating_wear_percent: 0.0,
        bond_stress_mpa: 0.0,
    };

    for _ in 0..cycles {
        simulate_thermal_cycle(
            &mut state,
            material,
            geometry,
            coating,
            REENTRY_HEAT_FLUX_W_CM2,
            REENTRY_DURATION_SECONDS,
        )?;
    }

    Ok(state)
}

pub fn determine_degradation_state(
    cycles: u32,
    coating_wear: f64,
    max_temp: f64,
    material: &TileMaterial,
) -> DegradationState {
    if cycles < 100 && coating_wear < 0.05 && max_temp < 800.0 {
        DegradationState::Nominal
    } else if cycles < 200 && coating_wear < 0.10 && max_temp < 1000.0 {
        DegradationState::Slight
    } else if cycles < 500 && coating_wear < 0.25 && max_temp < 1200.0 {
        DegradationState::Moderate
    } else if cycles < 1000 && coating_wear < 0.50 && max_temp < material.max_service_temp_c {
        DegradationState::Severe
    } else {
        DegradationState::Failed
    }
}

pub fn calculate_coating_wear(cycles: u32, base_rate: f64) -> f64 {
    cycles as f64 * base_rate
}

pub fn simulate_reentry_seeded(
    material: &TileMaterial,
    geometry: &TileGeometry,
    coating: &Coating,
    cycles: u32,
    config: &SimulationConfig,
) -> Result<SimulationState> {
    let mut rng = rand::rngs::SmallRng::seed_from_u64(config.seed);
    let mut state = SimulationState {
        current_temp_c: AMBIENT_TEMP_C,
        max_temp_seen_c: AMBIENT_TEMP_C,
        heat_absorbed_kj: 0.0,
        heat_reflected_kj: 0.0,
        thermal_cycles: 0,
        degradation_state: DegradationState::Nominal,
        coating_wear_percent: 0.0,
        bond_stress_mpa: 0.0,
    };

    for _ in 0..cycles {
        let variation: f64 = rng.gen_range(-0.05..=0.05);
        let perturbed_heat_flux = REENTRY_HEAT_FLUX_W_CM2 * (1.0 + variation);

        simulate_thermal_cycle(
            &mut state,
            material,
            geometry,
            coating,
            perturbed_heat_flux,
            REENTRY_DURATION_SECONDS,
        )?;
    }

    Ok(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Dimensions, TileShape};

    fn test_material() -> TileMaterial {
        TileMaterial::li_900()
    }

    fn test_coating() -> Coating {
        Coating::black_hrsi()
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
    fn test_calculate_effective_thermal_conductivity() {
        let material = test_material();
        let coating = test_coating();
        let geometry = test_geometry();

        let k_eff = calculate_effective_thermal_conductivity(&material, &coating, &geometry);
        let expected = material.thermal_conductivity_w_m_k * (1.0 + material.porosity_fraction)
            + COATING_THERMAL_CONDUCTIVITY_W_M_K * (coating.thickness_mm / geometry.dimensions.thickness_mm);
        assert!((k_eff - expected).abs() < 1e-10);
    }

    #[test]
    fn test_calculate_heat_transfer() {
        let material = test_material();
        let coating = test_coating();
        let geometry = test_geometry();

        let (absorbed, reflected) = calculate_heat_transfer(
            REENTRY_HEAT_FLUX_W_CM2,
            &material,
            &geometry,
            &coating,
            REENTRY_DURATION_SECONDS,
        ).unwrap();

        let total_energy_kj = REENTRY_HEAT_FLUX_W_CM2 * geometry.surface_area_m2 * REENTRY_DURATION_SECONDS * 10.0;
        assert!((absorbed + reflected - total_energy_kj).abs() < 1e-6);
        assert!(absorbed > 0.0);
        assert!(reflected >= 0.0);
    }

    #[test]
    fn test_calculate_backface_temp() {
        let material = test_material();
        let geometry = test_geometry();

        let t_back = calculate_backface_temp(REENTRY_HEAT_FLUX_W_CM2, &material, &geometry);
        let t_front = {
            let q_w_m2 = REENTRY_HEAT_FLUX_W_CM2 * 10_000.0;
            let t_front_k = (q_w_m2 / STEFAN_BOLTZMANN_W_M2_K4).powf(0.25);
            t_front_k - 273.15
        };

        assert!(t_back < t_front);
    }

    #[test]
    fn test_simulate_thermal_cycle() {
        let material = test_material();
        let geometry = test_geometry();
        let coating = test_coating();

        let mut state = SimulationState {
            current_temp_c: AMBIENT_TEMP_C,
            max_temp_seen_c: AMBIENT_TEMP_C,
            heat_absorbed_kj: 0.0,
            heat_reflected_kj: 0.0,
            thermal_cycles: 0,
            degradation_state: DegradationState::Nominal,
            coating_wear_percent: 0.0,
            bond_stress_mpa: 0.0,
        };

        simulate_thermal_cycle(
            &mut state,
            &material,
            &geometry,
            &coating,
            REENTRY_HEAT_FLUX_W_CM2,
            REENTRY_DURATION_SECONDS,
        ).unwrap();

        assert_eq!(state.thermal_cycles, 1);
        assert!(state.heat_absorbed_kj > 0.0);
        assert!(state.heat_reflected_kj >= 0.0);
        assert!(state.current_temp_c > AMBIENT_TEMP_C);
        assert!(state.max_temp_seen_c > AMBIENT_TEMP_C);
        assert!(state.coating_wear_percent > 0.0);
        assert!(state.bond_stress_mpa > 0.0);
    }

    #[test]
    fn test_simulate_reentry() {
        let material = test_material();
        let geometry = test_geometry();
        let coating = test_coating();

        let state = simulate_reentry(&material, &geometry, &coating, 10).unwrap();

        assert_eq!(state.thermal_cycles, 10);
        assert!(state.heat_absorbed_kj > 0.0);
        assert!(state.max_temp_seen_c > AMBIENT_TEMP_C);
        assert!(state.coating_wear_percent > 0.0);
        assert_eq!(state.degradation_state, DegradationState::Severe);
    }

    #[test]
    fn test_determine_degradation_state_nominal() {
        let material = test_material();
        assert_eq!(
            determine_degradation_state(50, 0.04, 750.0, &material),
            DegradationState::Nominal
        );
    }

    #[test]
    fn test_determine_degradation_state_slight() {
        let material = test_material();
        assert_eq!(
            determine_degradation_state(150, 0.08, 900.0, &material),
            DegradationState::Slight
        );
    }

    #[test]
    fn test_determine_degradation_state_moderate() {
        let material = test_material();
        assert_eq!(
            determine_degradation_state(300, 0.20, 1100.0, &material),
            DegradationState::Moderate
        );
    }

    #[test]
    fn test_determine_degradation_state_severe() {
        let material = test_material();
        assert_eq!(
            determine_degradation_state(750, 0.40, 1250.0, &material),
            DegradationState::Severe
        );
    }

    #[test]
    fn test_determine_degradation_state_failed() {
        let material = test_material();
        assert_eq!(
            determine_degradation_state(1500, 0.75, 1500.0, &material),
            DegradationState::Failed
        );
    }

    #[test]
    fn test_calculate_coating_wear() {
        assert_eq!(calculate_coating_wear(100, 0.001), 0.1);
        assert_eq!(calculate_coating_wear(0, 0.001), 0.0);
    }

    #[test]
    fn test_seeded_reproducibility() {
        let material = test_material();
        let geometry = test_geometry();
        let coating = test_coating();
        let config = SimulationConfig { seed: 42 };

        let state1 = simulate_reentry_seeded(&material, &geometry, &coating, 10, &config).unwrap();
        let state2 = simulate_reentry_seeded(&material, &geometry, &coating, 10, &config).unwrap();

        assert_eq!(state1.current_temp_c, state2.current_temp_c);
        assert_eq!(state1.max_temp_seen_c, state2.max_temp_seen_c);
        assert_eq!(state1.heat_absorbed_kj, state2.heat_absorbed_kj);
        assert_eq!(state1.heat_reflected_kj, state2.heat_reflected_kj);
        assert_eq!(state1.thermal_cycles, state2.thermal_cycles);
        assert_eq!(state1.coating_wear_percent, state2.coating_wear_percent);
        assert_eq!(state1.bond_stress_mpa, state2.bond_stress_mpa);
        assert_eq!(state1.degradation_state, state2.degradation_state);
    }

    #[test]
    fn test_different_seeds_differ() {
        let material = test_material();
        let geometry = test_geometry();
        let coating = test_coating();
        let config1 = SimulationConfig { seed: 42 };
        let config2 = SimulationConfig { seed: 12345 };

        let state1 = simulate_reentry_seeded(&material, &geometry, &coating, 10, &config1).unwrap();
        let state2 = simulate_reentry_seeded(&material, &geometry, &coating, 10, &config2).unwrap();

        assert!(state1.current_temp_c != state2.current_temp_c || state1.heat_absorbed_kj != state2.heat_absorbed_kj);
    }
}

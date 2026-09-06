use crate::{
    Adhesive, AppliedId, Coating, ManufacturingStage, ManufacturingStep, Result, StepResult,
    StrainIsolationPad, TileBatch, TileError, TileGeometry, TileLocation, TileMaterial, TileState,
};
use chrono::Utc;
use std::collections::HashMap;

pub struct ManufacturingPipeline {
    pub stages: Vec<ManufacturingStage>,
    pub current_stage_index: usize,
}

impl ManufacturingPipeline {
    pub fn new() -> Self {
        Self {
            stages: vec![
                ManufacturingStage::RawMaterial,
                ManufacturingStage::SlurryFormation,
                ManufacturingStage::MoldCasting,
                ManufacturingStage::Drying,
                ManufacturingStage::Sintering,
                ManufacturingStage::CncMachining,
                ManufacturingStage::Coating,
                ManufacturingStage::Bonding,
                ManufacturingStage::Identification,
                ManufacturingStage::Testing,
                ManufacturingStage::Complete,
            ],
            current_stage_index: 0,
        }
    }

    pub fn current_stage(&self) -> ManufacturingStage {
        self.stages[self.current_stage_index]
    }

    pub fn advance_stage(&mut self) -> Result<ManufacturingStage> {
        if self.current_stage_index >= self.stages.len() - 1 {
            return Err(TileError::ManufacturingError(
                "Pipeline is already complete".to_string(),
            ));
        }
        self.current_stage_index += 1;
        Ok(self.stages[self.current_stage_index])
    }

    pub fn is_complete(&self) -> bool {
        self.current_stage_index == self.stages.len() - 1
    }

    pub fn stages_completed(&self) -> Vec<ManufacturingStage> {
        self.stages[0..self.current_stage_index].to_vec()
    }

    fn get_next_stage(&self, stage: ManufacturingStage) -> Option<ManufacturingStage> {
        self.stages
            .iter()
            .position(|&s| s == stage)
            .and_then(|idx| self.stages.get(idx + 1).copied())
    }
}

impl Default for ManufacturingPipeline {
    fn default() -> Self {
        Self::new()
    }
}

pub fn create_tile_state() -> TileState {
    TileState {
        id: None,
        material: None,
        geometry: None,
        coating: None,
        strain_isolation_pad: None,
        adhesive: None,
        applied_id: None,
        manufacturing_steps: Vec::new(),
        test_results: Vec::new(),
        audit_results: Vec::new(),
        simulation_state: None,
        defects: Vec::new(),
        created_at: Utc::now(),
        current_stage: ManufacturingStage::RawMaterial,
    }
}

fn stage_name(stage: ManufacturingStage) -> &'static str {
    match stage {
        ManufacturingStage::RawMaterial => "Raw Material Preparation",
        ManufacturingStage::SlurryFormation => "Slurry Formation",
        ManufacturingStage::MoldCasting => "Mold Casting",
        ManufacturingStage::Drying => "Drying",
        ManufacturingStage::Sintering => "Sintering",
        ManufacturingStage::CncMachining => "CNC Machining",
        ManufacturingStage::Coating => "Coating Application",
        ManufacturingStage::Bonding => "Bonding Assembly",
        ManufacturingStage::Identification => "Identification Application",
        ManufacturingStage::Testing => "Testing and QA",
        ManufacturingStage::Complete => "Final Inspection",
        ManufacturingStage::Rejected => "Rejected",
    }
}

pub fn execute_stage(
    tile: &mut TileState,
    stage: ManufacturingStage,
    operator_id: &str,
) -> Result<ManufacturingStep> {
    if tile.current_stage != stage {
        return Err(TileError::ManufacturingError(format!(
            "Tile is at stage {:?}, expected {:?}",
            tile.current_stage, stage
        )));
    }

    let step = ManufacturingStep {
        name: stage_name(stage).to_string(),
        step_number: tile.manufacturing_steps.len() as u32 + 1,
        timestamp: Utc::now(),
        operator_id: operator_id.to_string(),
        parameters: HashMap::new(),
        result: StepResult::Success,
    };

    tile.manufacturing_steps.push(step.clone());

    let pipeline = ManufacturingPipeline::new();
    if let Some(next) = pipeline.get_next_stage(stage) {
        tile.current_stage = next;
    }

    Ok(step)
}

pub fn run_full_manufacturing(tile: &mut TileState, operator_id: &str) -> Result<()> {
    let pipeline = ManufacturingPipeline::new();

    let start_idx = pipeline
        .stages
        .iter()
        .position(|&s| s == tile.current_stage)
        .ok_or_else(|| {
            TileError::ManufacturingError("Invalid current stage on tile".to_string())
        })?;

    for i in start_idx..pipeline.stages.len() {
        let stage = pipeline.stages[i];

        if stage == ManufacturingStage::Complete || stage == ManufacturingStage::Rejected {
            break;
        }

        match stage {
            ManufacturingStage::Sintering => {
                if tile.material.is_none() {
                    return Err(TileError::ManufacturingError(
                        "Material required before Sintering".to_string(),
                    ));
                }
            }
            ManufacturingStage::CncMachining => {
                if tile.geometry.is_none() {
                    return Err(TileError::ManufacturingError(
                        "Geometry required before CNC Machining".to_string(),
                    ));
                }
            }
            ManufacturingStage::Coating => {
                if tile.coating.is_none() {
                    return Err(TileError::ManufacturingError(
                        "Coating required before Coating stage".to_string(),
                    ));
                }
            }
            ManufacturingStage::Bonding => {
                if tile.strain_isolation_pad.is_none() || tile.adhesive.is_none() {
                    return Err(TileError::ManufacturingError(
                        "SIP and adhesive required before Bonding".to_string(),
                    ));
                }
            }
            ManufacturingStage::Identification => {
                if tile.id.is_none() {
                    return Err(TileError::ManufacturingError(
                        "ID required before Identification".to_string(),
                    ));
                }
            }
            ManufacturingStage::Testing => {
                validate_tile_for_testing(tile)?;
            }
            _ => {}
        }

        execute_stage(tile, stage, operator_id)?;
    }

    Ok(())
}

pub fn apply_material(tile: &mut TileState, material: TileMaterial) -> Result<()> {
    crate::materials::validate_material(&material)?;
    tile.material = Some(material);
    Ok(())
}

pub fn apply_geometry(tile: &mut TileState, geometry: TileGeometry) -> Result<()> {
    geometry.validate_geometry()?;
    tile.geometry = Some(geometry);
    Ok(())
}

pub fn apply_coating(tile: &mut TileState, coating: Coating) -> Result<()> {
    crate::materials::validate_coating(&coating)?;
    tile.coating = Some(coating);
    Ok(())
}

pub fn apply_bonding(
    tile: &mut TileState,
    sip: StrainIsolationPad,
    adhesive: Adhesive,
) -> Result<()> {
    crate::materials::validate_sip(&sip)?;
    crate::materials::validate_adhesive(&adhesive)?;

    let bond_strength_mpa =
        if let (Some(material), Some(geometry)) = (tile.material.as_ref(), tile.geometry.as_ref()) {
            crate::bonding::calculate_bond_strength(material, &sip, &adhesive, geometry)
                .unwrap_or(2.0)
                .max(2.0)
        } else {
            2.0
        };

    let assembly = crate::bonding::BondAssembly {
        sip: sip.clone(),
        adhesive: adhesive.clone(),
        bond_strength_mpa,
        bond_stress_mpa: 0.5,
    };

    crate::bonding::validate_bond_assembly(&assembly)?;

    tile.strain_isolation_pad = Some(sip);
    tile.adhesive = Some(adhesive);
    Ok(())
}

pub fn apply_identification(
    tile: &mut TileState,
    batch: TileBatch,
    location: TileLocation,
    sequence: u32,
    operator_id: &str,
) -> Result<()> {
    let id = crate::identification::generate_id(&batch, &location, sequence)?;
    let applied_id = AppliedId {
        id: id.clone(),
        application_method: "Laser etching".to_string(),
        application_timestamp: Utc::now(),
        operator_id: operator_id.to_string(),
        physical_reading: id.raw.clone(),
        digital_record: crate::identification::to_machine_readable(&id),
    };
    tile.id = Some(id);
    tile.applied_id = Some(applied_id);
    Ok(())
}

pub fn validate_tile_for_testing(tile: &TileState) -> Result<()> {
    if tile.id.is_none() {
        return Err(TileError::ManufacturingError("Missing tile ID".to_string()));
    }
    if tile.material.is_none() {
        return Err(TileError::ManufacturingError(
            "Missing tile material".to_string(),
        ));
    }
    if tile.geometry.is_none() {
        return Err(TileError::ManufacturingError(
            "Missing tile geometry".to_string(),
        ));
    }
    if tile.coating.is_none() {
        return Err(TileError::ManufacturingError(
            "Missing tile coating".to_string(),
        ));
    }
    if tile.strain_isolation_pad.is_none() {
        return Err(TileError::ManufacturingError(
            "Missing strain isolation pad".to_string(),
        ));
    }
    if tile.adhesive.is_none() {
        return Err(TileError::ManufacturingError(
            "Missing adhesive".to_string(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Dimensions, TileShape, TileSurface};

    fn test_batch() -> TileBatch {
        TileBatch {
            batch_code: "A24001".to_string(),
            production_date: Utc::now(),
            oven_id: "Oven1".to_string(),
            operator_id: "OP001".to_string(),
        }
    }

    fn test_location() -> TileLocation {
        TileLocation {
            orbiter_id: "OV104".to_string(),
            surface: TileSurface::FuselageBottom,
            panel_id: "03".to_string(),
            row: 0,
            column: 3,
        }
    }

    fn test_geometry() -> TileGeometry {
        TileGeometry::from_dimensions(
            Dimensions {
                length_mm: 150.0,
                width_mm: 150.0,
                thickness_mm: 25.4,
            },
            TileShape::Flat,
        )
        .unwrap()
    }

    #[test]
    fn test_pipeline_new() {
        let pipeline = ManufacturingPipeline::new();
        assert_eq!(pipeline.stages.len(), 11);
        assert_eq!(pipeline.stages[0], ManufacturingStage::RawMaterial);
        assert_eq!(pipeline.stages[10], ManufacturingStage::Complete);
    }

    #[test]
    fn test_pipeline_current_stage() {
        let pipeline = ManufacturingPipeline::new();
        assert_eq!(pipeline.current_stage(), ManufacturingStage::RawMaterial);
    }

    #[test]
    fn test_pipeline_advance_stage() {
        let mut pipeline = ManufacturingPipeline::new();
        let next = pipeline.advance_stage().unwrap();
        assert_eq!(next, ManufacturingStage::SlurryFormation);
        assert_eq!(pipeline.current_stage_index, 1);
    }

    #[test]
    fn test_pipeline_advance_stage_at_end() {
        let mut pipeline = ManufacturingPipeline::new();
        pipeline.current_stage_index = pipeline.stages.len() - 1;
        assert!(pipeline.advance_stage().is_err());
    }

    #[test]
    fn test_pipeline_is_complete() {
        let mut pipeline = ManufacturingPipeline::new();
        assert!(!pipeline.is_complete());
        pipeline.current_stage_index = pipeline.stages.len() - 1;
        assert!(pipeline.is_complete());
    }

    #[test]
    fn test_pipeline_stages_completed() {
        let mut pipeline = ManufacturingPipeline::new();
        assert!(pipeline.stages_completed().is_empty());
        pipeline.advance_stage().unwrap();
        pipeline.advance_stage().unwrap();
        let completed = pipeline.stages_completed();
        assert_eq!(completed.len(), 2);
        assert_eq!(completed[0], ManufacturingStage::RawMaterial);
        assert_eq!(completed[1], ManufacturingStage::SlurryFormation);
    }

    #[test]
    fn test_create_tile_state() {
        let tile = create_tile_state();
        assert!(tile.id.is_none());
        assert!(tile.material.is_none());
        assert!(tile.geometry.is_none());
        assert!(tile.coating.is_none());
        assert!(tile.strain_isolation_pad.is_none());
        assert!(tile.adhesive.is_none());
        assert!(tile.applied_id.is_none());
        assert!(tile.manufacturing_steps.is_empty());
        assert!(tile.test_results.is_empty());
        assert!(tile.audit_results.is_empty());
        assert!(tile.simulation_state.is_none());
        assert!(tile.defects.is_empty());
        assert_eq!(tile.current_stage, ManufacturingStage::RawMaterial);
    }

    #[test]
    fn test_execute_stage_success() {
        let mut tile = create_tile_state();
        let step = execute_stage(&mut tile, ManufacturingStage::RawMaterial, "OP001").unwrap();
        assert_eq!(step.name, "Raw Material Preparation");
        assert_eq!(step.step_number, 1);
        assert_eq!(step.operator_id, "OP001");
        assert_eq!(step.result, StepResult::Success);
        assert_eq!(tile.manufacturing_steps.len(), 1);
        assert_eq!(tile.current_stage, ManufacturingStage::SlurryFormation);
    }

    #[test]
    fn test_execute_stage_wrong_stage() {
        let mut tile = create_tile_state();
        let result = execute_stage(&mut tile, ManufacturingStage::Sintering, "OP001");
        assert!(result.is_err());
    }

    #[test]
    fn test_execute_stage_multiple() {
        let mut tile = create_tile_state();
        execute_stage(&mut tile, ManufacturingStage::RawMaterial, "OP001").unwrap();
        execute_stage(&mut tile, ManufacturingStage::SlurryFormation, "OP001").unwrap();
        assert_eq!(tile.manufacturing_steps.len(), 2);
        assert_eq!(tile.current_stage, ManufacturingStage::MoldCasting);
    }

    #[test]
    fn test_run_full_manufacturing_success() {
        let mut tile = create_tile_state();
        apply_material(&mut tile, TileMaterial::li_900()).unwrap();
        apply_geometry(&mut tile, test_geometry()).unwrap();
        apply_coating(&mut tile, Coating::black_hrsi()).unwrap();
        apply_bonding(&mut tile, StrainIsolationPad::standard(), Adhesive::rtv_560()).unwrap();
        apply_identification(&mut tile, test_batch(), test_location(), 1, "OP001").unwrap();

        run_full_manufacturing(&mut tile, "OP001").unwrap();
        assert_eq!(tile.current_stage, ManufacturingStage::Complete);
        assert_eq!(tile.manufacturing_steps.len(), 10);
    }

    #[test]
    fn test_run_full_manufacturing_missing_material() {
        let mut tile = create_tile_state();
        let result = run_full_manufacturing(&mut tile, "OP001");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Material"));
    }

    #[test]
    fn test_run_full_manufacturing_missing_geometry() {
        let mut tile = create_tile_state();
        apply_material(&mut tile, TileMaterial::li_900()).unwrap();
        let result = run_full_manufacturing(&mut tile, "OP001");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Geometry"));
    }

    #[test]
    fn test_run_full_manufacturing_missing_coating() {
        let mut tile = create_tile_state();
        apply_material(&mut tile, TileMaterial::li_900()).unwrap();
        apply_geometry(&mut tile, test_geometry()).unwrap();
        let result = run_full_manufacturing(&mut tile, "OP001");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Coating"));
    }

    #[test]
    fn test_run_full_manufacturing_missing_bonding() {
        let mut tile = create_tile_state();
        apply_material(&mut tile, TileMaterial::li_900()).unwrap();
        apply_geometry(&mut tile, test_geometry()).unwrap();
        apply_coating(&mut tile, Coating::black_hrsi()).unwrap();
        let result = run_full_manufacturing(&mut tile, "OP001");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("SIP"));
    }

    #[test]
    fn test_run_full_manufacturing_missing_id() {
        let mut tile = create_tile_state();
        apply_material(&mut tile, TileMaterial::li_900()).unwrap();
        apply_geometry(&mut tile, test_geometry()).unwrap();
        apply_coating(&mut tile, Coating::black_hrsi()).unwrap();
        apply_bonding(&mut tile, StrainIsolationPad::standard(), Adhesive::rtv_560()).unwrap();
        let result = run_full_manufacturing(&mut tile, "OP001");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("ID"));
    }

    #[test]
    fn test_apply_material() {
        let mut tile = create_tile_state();
        apply_material(&mut tile, TileMaterial::li_900()).unwrap();
        assert!(tile.material.is_some());
    }

    #[test]
    fn test_apply_material_invalid() {
        let mut tile = create_tile_state();
        let mut material = TileMaterial::li_900();
        material.density_kg_m3 = 0.0;
        assert!(apply_material(&mut tile, material).is_err());
        assert!(tile.material.is_none());
    }

    #[test]
    fn test_apply_geometry() {
        let mut tile = create_tile_state();
        apply_geometry(&mut tile, test_geometry()).unwrap();
        assert!(tile.geometry.is_some());
    }

    #[test]
    fn test_apply_geometry_invalid() {
        let mut tile = create_tile_state();
        let mut geometry = test_geometry();
        geometry.dimensions.length_mm = 10.0;
        assert!(apply_geometry(&mut tile, geometry).is_err());
        assert!(tile.geometry.is_none());
    }

    #[test]
    fn test_apply_coating() {
        let mut tile = create_tile_state();
        apply_coating(&mut tile, Coating::black_hrsi()).unwrap();
        assert!(tile.coating.is_some());
    }

    #[test]
    fn test_apply_coating_invalid() {
        let mut tile = create_tile_state();
        let mut coating = Coating::black_hrsi();
        coating.thickness_mm = 0.0;
        assert!(apply_coating(&mut tile, coating).is_err());
        assert!(tile.coating.is_none());
    }

    #[test]
    fn test_apply_bonding() {
        let mut tile = create_tile_state();
        apply_bonding(&mut tile, StrainIsolationPad::standard(), Adhesive::rtv_560()).unwrap();
        assert!(tile.strain_isolation_pad.is_some());
        assert!(tile.adhesive.is_some());
    }

    #[test]
    fn test_apply_bonding_invalid_sip() {
        let mut tile = create_tile_state();
        let mut sip = StrainIsolationPad::standard();
        sip.thickness_mm = 0.0;
        assert!(apply_bonding(&mut tile, sip, Adhesive::rtv_560()).is_err());
    }

    #[test]
    fn test_apply_bonding_invalid_adhesive() {
        let mut tile = create_tile_state();
        let mut adhesive = Adhesive::rtv_560();
        adhesive.shear_strength_mpa = 0.0;
        assert!(apply_bonding(&mut tile, StrainIsolationPad::standard(), adhesive).is_err());
    }

    #[test]
    fn test_apply_identification() {
        let mut tile = create_tile_state();
        apply_identification(&mut tile, test_batch(), test_location(), 1, "OP001").unwrap();
        assert!(tile.id.is_some());
        assert!(tile.applied_id.is_some());
        let applied = tile.applied_id.unwrap();
        assert_eq!(applied.application_method, "Laser etching");
        assert_eq!(applied.operator_id, "OP001");
        assert!(!applied.digital_record.contains('-'));
    }

    #[test]
    fn test_validate_tile_for_testing_complete() {
        let mut tile = create_tile_state();
        apply_material(&mut tile, TileMaterial::li_900()).unwrap();
        apply_geometry(&mut tile, test_geometry()).unwrap();
        apply_coating(&mut tile, Coating::black_hrsi()).unwrap();
        apply_bonding(&mut tile, StrainIsolationPad::standard(), Adhesive::rtv_560()).unwrap();
        apply_identification(&mut tile, test_batch(), test_location(), 1, "OP001").unwrap();
        assert!(validate_tile_for_testing(&tile).is_ok());
    }

    #[test]
    fn test_validate_tile_for_testing_missing_material() {
        let mut tile = create_tile_state();
        apply_geometry(&mut tile, test_geometry()).unwrap();
        apply_coating(&mut tile, Coating::black_hrsi()).unwrap();
        apply_bonding(&mut tile, StrainIsolationPad::standard(), Adhesive::rtv_560()).unwrap();
        apply_identification(&mut tile, test_batch(), test_location(), 1, "OP001").unwrap();
        let result = validate_tile_for_testing(&tile);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("material"));
    }

    #[test]
    fn test_validate_tile_for_testing_missing_geometry() {
        let mut tile = create_tile_state();
        apply_material(&mut tile, TileMaterial::li_900()).unwrap();
        apply_coating(&mut tile, Coating::black_hrsi()).unwrap();
        apply_bonding(&mut tile, StrainIsolationPad::standard(), Adhesive::rtv_560()).unwrap();
        apply_identification(&mut tile, test_batch(), test_location(), 1, "OP001").unwrap();
        let result = validate_tile_for_testing(&tile);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("geometry"));
    }

    #[test]
    fn test_validate_tile_for_testing_missing_coating() {
        let mut tile = create_tile_state();
        apply_material(&mut tile, TileMaterial::li_900()).unwrap();
        apply_geometry(&mut tile, test_geometry()).unwrap();
        apply_bonding(&mut tile, StrainIsolationPad::standard(), Adhesive::rtv_560()).unwrap();
        apply_identification(&mut tile, test_batch(), test_location(), 1, "OP001").unwrap();
        let result = validate_tile_for_testing(&tile);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("coating"));
    }

    #[test]
    fn test_validate_tile_for_testing_missing_sip() {
        let mut tile = create_tile_state();
        apply_material(&mut tile, TileMaterial::li_900()).unwrap();
        apply_geometry(&mut tile, test_geometry()).unwrap();
        apply_coating(&mut tile, Coating::black_hrsi()).unwrap();
        tile.adhesive = Some(Adhesive::rtv_560());
        apply_identification(&mut tile, test_batch(), test_location(), 1, "OP001").unwrap();
        let result = validate_tile_for_testing(&tile);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("isolation pad"));
    }

    #[test]
    fn test_validate_tile_for_testing_missing_adhesive() {
        let mut tile = create_tile_state();
        apply_material(&mut tile, TileMaterial::li_900()).unwrap();
        apply_geometry(&mut tile, test_geometry()).unwrap();
        apply_coating(&mut tile, Coating::black_hrsi()).unwrap();
        tile.strain_isolation_pad = Some(StrainIsolationPad::standard());
        apply_identification(&mut tile, test_batch(), test_location(), 1, "OP001").unwrap();
        let result = validate_tile_for_testing(&tile);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("adhesive"));
    }

    #[test]
    fn test_validate_tile_for_testing_missing_id() {
        let mut tile = create_tile_state();
        apply_material(&mut tile, TileMaterial::li_900()).unwrap();
        apply_geometry(&mut tile, test_geometry()).unwrap();
        apply_coating(&mut tile, Coating::black_hrsi()).unwrap();
        apply_bonding(&mut tile, StrainIsolationPad::standard(), Adhesive::rtv_560()).unwrap();
        let result = validate_tile_for_testing(&tile);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("ID"));
    }
}

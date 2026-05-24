#[allow(unused_imports)]
use crate::{
    AdhesionTestResult, AuditFinding, AuditStatus, DegradationState, Defect, DefectCategory,
    DefectSeverity, FailureMode, IdAuditResult, ManufacturingStage, ManufacturingStep,
    MechanicalTestResult, OverallStatus, SimulationState, SimulationSummary, StepResult,
    TestOutcome, TestRecord, TestSummary, TestType, ThermalTestResult, TileError, TileId,
    TileLocation, TileReport, TileState, AuditSummary, ManufacturingSummary, TileBatch,
    TileSurface,
};
use chrono::Utc;
#[allow(unused_imports)]
use std::collections::HashMap;

pub fn generate_tile_report(tile: &TileState) -> crate::Result<TileReport> {
    let id = tile
        .id
        .as_ref()
        .ok_or_else(|| TileError::TestError("Cannot generate report: tile has no ID".to_string()))?;

    let manufacturing_summary = summarize_manufacturing(&tile.manufacturing_steps);
    let test_summary = summarize_tests(&tile.test_results);
    let audit_summary = summarize_audits(&tile.audit_results);

    let mut report = TileReport {
        tile_id: Some(id.raw.clone()),
        batch_info: Some(id.batch.clone()),
        location_info: Some(id.location.clone()),
        manufacturing_summary,
        test_summary,
        audit_summary,
        simulation_summary: tile.simulation_state.as_ref().map(summarize_simulation).unwrap_or_else(|| SimulationSummary {
            cycles_simulated: 0,
            final_degradation: DegradationState::Nominal,
            coating_wear_percent: 0.0,
            max_temp_seen_c: 25.0,
            estimated_service_life_cycles: 1000,
        }),
        defects: tile.defects.clone(),
        overall_status: OverallStatus::Pending,
        generated_at: Utc::now(),
    };

    report.overall_status = determine_overall_status(&report);
    Ok(report)
}

pub fn format_report_text(report: &TileReport) -> String {
    let mut lines = Vec::new();

    lines.push("=== TILE REPORT ===".to_string());
    lines.push(format!(
        "Tile ID: {}",
        report.tile_id.as_deref().unwrap_or("N/A")
    ));
    lines.push(format!(
        "Batch: {}",
        report
            .batch_info
            .as_ref()
            .map(|b| b.batch_code.as_str())
            .unwrap_or("N/A")
    ));
    lines.push(format!(
        "Location: {}",
        report
            .location_info
            .as_ref()
            .map(|l| format!("{} {}", l.orbiter_id, l.surface))
            .unwrap_or_else(|| "N/A".to_string())
    ));

    lines.push(String::new());
    lines.push("--- Manufacturing Summary ---".to_string());
    lines.push(format!(
        "Total Steps: {}",
        report.manufacturing_summary.total_steps
    ));
    lines.push(format!(
        "Completed: {}",
        report.manufacturing_summary.steps_completed
    ));
    lines.push(format!(
        "Warnings: {}",
        report.manufacturing_summary.steps_with_warnings
    ));
    lines.push(format!(
        "Failures: {}",
        report.manufacturing_summary.steps_with_failures
    ));
    lines.push(format!(
        "Final Stage: {:?}",
        report.manufacturing_summary.final_stage
    ));

    lines.push(String::new());
    lines.push("--- Test Summary ---".to_string());
    lines.push(format!(
        "Total Tests: {}",
        report.test_summary.total_tests
    ));
    lines.push(format!("Passed: {}", report.test_summary.passed));
    lines.push(format!("Failed: {}", report.test_summary.failed));
    lines.push(format!(
        "Inconclusive: {}",
        report.test_summary.inconclusive
    ));

    lines.push(String::new());
    lines.push("--- Audit Summary ---".to_string());
    lines.push(format!(
        "Total Audits: {}",
        report.audit_summary.total_audits
    ));
    lines.push(format!("Passed: {}", report.audit_summary.passed));
    lines.push(format!("Warned: {}", report.audit_summary.warned));
    lines.push(format!("Failed: {}", report.audit_summary.failed));

    lines.push(String::new());
    lines.push("--- Simulation Summary ---".to_string());
    let sim = &report.simulation_summary;
    if sim.cycles_simulated > 0 {
        lines.push(format!("Cycles Simulated: {}", sim.cycles_simulated));
        lines.push(format!("Final Degradation: {:?}", sim.final_degradation));
        lines.push(format!("Coating Wear: {:.1}%", sim.coating_wear_percent));
        lines.push(format!("Max Temp: {:.1} C", sim.max_temp_seen_c));
        lines.push(format!(
            "Est. Service Life: {} cycles",
            sim.estimated_service_life_cycles
        ));
    } else {
        lines.push("No simulation data".to_string());
    }

    lines.push(String::new());
    lines.push("--- Defects ---".to_string());
    if report.defects.is_empty() {
        lines.push("No defects reported".to_string());
    } else {
        for defect in &report.defects {
            lines.push(format!(
                "- {:?} ({:?}): {}",
                defect.severity, defect.category, defect.description
            ));
        }
    }

    lines.push(String::new());
    lines.push(format!("Overall Status: {:?}", report.overall_status));

    lines.join("\n")
}

pub fn format_report_json(report: &TileReport) -> crate::Result<String> {
    serde_json::to_string_pretty(report)
        .map_err(|e| TileError::TestError(format!("JSON serialization failed: {}", e)))
}

pub fn determine_overall_status(report: &TileReport) -> OverallStatus {
    let has_test_failures = report.test_summary.failed > 0;
    let has_audit_failures = report.audit_summary.failed > 0;
    let has_critical_defects = report
        .defects
        .iter()
        .any(|d| d.severity == DefectSeverity::Critical);

    if has_test_failures || has_audit_failures || has_critical_defects {
        return OverallStatus::Rejected;
    }

    let is_incomplete = report.test_summary.total_tests == 0
        && report.audit_summary.total_audits == 0
        && report.manufacturing_summary.total_steps == 0;

    if is_incomplete {
        return OverallStatus::Pending;
    }

    let has_warnings = report.test_summary.inconclusive > 0
        || report.audit_summary.warned > 0
        || report
            .defects
            .iter()
            .any(|d| d.severity == DefectSeverity::Minor || d.severity == DefectSeverity::Major);

    if has_warnings {
        OverallStatus::AcceptedWithNotes
    } else {
        OverallStatus::Accepted
    }
}

pub fn summarize_manufacturing(steps: &[ManufacturingStep]) -> ManufacturingSummary {
    let total_steps = steps.len();
    let steps_completed = steps
        .iter()
        .filter(|s| matches!(s.result, StepResult::Success))
        .count();
    let steps_with_warnings = steps
        .iter()
        .filter(|s| matches!(s.result, StepResult::Warning(_)))
        .count();
    let steps_with_failures = steps
        .iter()
        .filter(|s| matches!(s.result, StepResult::Failure(_)))
        .count();

    let final_stage = steps
        .last()
        .and_then(|step| match step.name.as_str() {
            "RawMaterial" => Some(ManufacturingStage::RawMaterial),
            "SlurryFormation" => Some(ManufacturingStage::SlurryFormation),
            "MoldCasting" => Some(ManufacturingStage::MoldCasting),
            "Drying" => Some(ManufacturingStage::Drying),
            "Sintering" => Some(ManufacturingStage::Sintering),
            "CncMachining" => Some(ManufacturingStage::CncMachining),
            "Coating" => Some(ManufacturingStage::Coating),
            "Bonding" => Some(ManufacturingStage::Bonding),
            "Identification" => Some(ManufacturingStage::Identification),
            "Testing" => Some(ManufacturingStage::Testing),
            "Complete" => Some(ManufacturingStage::Complete),
            "Rejected" => Some(ManufacturingStage::Rejected),
            _ => None,
        })
        .unwrap_or(ManufacturingStage::RawMaterial);

    ManufacturingSummary {
        total_steps,
        steps_completed,
        steps_with_warnings,
        steps_with_failures,
        final_stage,
    }
}

pub fn summarize_tests(records: &[TestRecord]) -> TestSummary {
    let total_tests = records.len();
    let passed = records
        .iter()
        .filter(|r| r.result == TestOutcome::Pass)
        .count();
    let failed = records
        .iter()
        .filter(|r| r.result == TestOutcome::Fail)
        .count();
    let inconclusive = records
        .iter()
        .filter(|r| r.result == TestOutcome::Inconclusive)
        .count();

    let thermal_result = records
        .iter()
        .find(|r| r.test_type == TestType::Thermal)
        .map(|r| ThermalTestResult {
            max_temp_reached_c: r.data.get("max_temp_reached_c").copied().unwrap_or(0.0),
            backface_temp_c: r.data.get("backface_temp_c").copied().unwrap_or(0.0),
            heat_flux_w_cm2: r.data.get("heat_flux_w_cm2").copied().unwrap_or(0.0),
            duration_seconds: r.data.get("duration_seconds").copied().unwrap_or(0.0),
            cycles_completed: r.data.get("cycles_completed").copied().unwrap_or(0.0) as u32,
            degradation_percent: r.data.get("degradation_percent").copied().unwrap_or(0.0),
            passed: r.result == TestOutcome::Pass,
        });

    let mechanical_result = records
        .iter()
        .find(|r| r.test_type == TestType::Mechanical)
        .map(|r| MechanicalTestResult {
            compressive_strength_mpa: r
                .data
                .get("compressive_strength_mpa")
                .copied()
                .unwrap_or(0.0),
            tensile_strength_mpa: r
                .data
                .get("tensile_strength_mpa")
                .copied()
                .unwrap_or(0.0),
            flexural_strength_mpa: r
                .data
                .get("flexural_strength_mpa")
                .copied()
                .unwrap_or(0.0),
            youngs_modulus_gpa: r.data.get("youngs_modulus_gpa").copied().unwrap_or(0.0),
            passed: r.result == TestOutcome::Pass,
        });

    let adhesion_result = records
        .iter()
        .find(|r| r.test_type == TestType::Adhesion)
        .map(|r| AdhesionTestResult {
            bond_strength_mpa: r.data.get("bond_strength_mpa").copied().unwrap_or(0.0),
            peel_strength_n_m: r.data.get("peel_strength_n_m").copied().unwrap_or(0.0),
            failure_mode: r
                .data
                .get("failure_mode")
                .copied()
                .map(|v| match v as u8 {
                    0 => FailureMode::Cohesive,
                    1 => FailureMode::Adhesive,
                    2 => FailureMode::Substrate,
                    3 => FailureMode::Mixed,
                    _ => FailureMode::None,
                })
                .unwrap_or(FailureMode::None),
            passed: r.result == TestOutcome::Pass,
        });

    TestSummary {
        total_tests,
        passed,
        failed,
        inconclusive,
        thermal_result,
        mechanical_result,
        adhesion_result,
    }
}

pub fn summarize_audits(audits: &[IdAuditResult]) -> AuditSummary {
    let total_audits = audits.len();
    let passed = audits
        .iter()
        .filter(|a| a.status == AuditStatus::Pass)
        .count();
    let warned = audits
        .iter()
        .filter(|a| a.status == AuditStatus::Warn)
        .count();
    let failed = audits
        .iter()
        .filter(|a| a.status == AuditStatus::Fail)
        .count();

    let findings: Vec<AuditFinding> = audits.iter().flat_map(|a| a.findings.clone()).collect();

    AuditSummary {
        total_audits,
        passed,
        warned,
        failed,
        findings,
    }
}

pub fn summarize_simulation(state: &SimulationState) -> SimulationSummary {
    let estimated_service_life_cycles = match state.degradation_state {
        DegradationState::Nominal | DegradationState::Slight => 1000,
        DegradationState::Moderate => 500,
        DegradationState::Severe => 200,
        DegradationState::Failed => 0,
    };

    SimulationSummary {
        cycles_simulated: state.thermal_cycles,
        final_degradation: state.degradation_state,
        coating_wear_percent: state.coating_wear_percent,
        max_temp_seen_c: state.max_temp_seen_c,
        estimated_service_life_cycles,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_tile_id() -> TileId {
        TileId {
            raw: "ABC123-NC0001-01-A1".to_string(),
            batch: TileBatch {
                batch_code: "ABC123".to_string(),
                production_date: Utc::now(),
                oven_id: "Oven1".to_string(),
                operator_id: "OP1".to_string(),
            },
            location: TileLocation {
                orbiter_id: "OV-101".to_string(),
                surface: TileSurface::NoseCap,
                panel_id: "P01".to_string(),
                row: 1,
                column: 2,
            },
            sequence: 1,
            checksum: 42,
        }
    }

    fn make_tile_state() -> TileState {
        TileState {
            id: Some(make_tile_id()),
            material: None,
            geometry: None,
            coating: None,
            strain_isolation_pad: None,
            adhesive: None,
            applied_id: None,
            manufacturing_steps: vec![],
            test_results: vec![],
            audit_results: vec![],
            simulation_state: None,
            defects: vec![],
            created_at: Utc::now(),
            current_stage: ManufacturingStage::RawMaterial,
        }
    }

    #[test]
    fn test_generate_tile_report_ok() {
        let tile = make_tile_state();
        let report = generate_tile_report(&tile).unwrap();
        assert_eq!(report.tile_id, Some("ABC123-NC0001-01-A1".to_string()));
        assert!(report.batch_info.is_some());
        assert!(report.location_info.is_some());
    }

    #[test]
    fn test_generate_tile_report_missing_id() {
        let mut tile = make_tile_state();
        tile.id = None;
        let err = generate_tile_report(&tile).unwrap_err();
        assert!(matches!(err, TileError::TestError(_)));
    }

    #[test]
    fn test_format_report_text_contains_header() {
        let tile = make_tile_state();
        let report = generate_tile_report(&tile).unwrap();
        let text = format_report_text(&report);
        assert!(text.contains("=== TILE REPORT ==="));
        assert!(text.contains("Tile ID:"));
        assert!(text.contains("ABC123"));
    }

    #[test]
    fn test_format_report_json_ok() {
        let tile = make_tile_state();
        let report = generate_tile_report(&tile).unwrap();
        let json = format_report_json(&report).unwrap();
        assert!(json.contains("tile_id"));
        assert!(json.contains("ABC123-NC0001-01-A1"));
    }

    #[test]
    fn test_determine_overall_status_accepted() {
        let report = TileReport {
            tile_id: None,
            batch_info: None,
            location_info: None,
            manufacturing_summary: ManufacturingSummary {
                total_steps: 1,
                steps_completed: 1,
                steps_with_warnings: 0,
                steps_with_failures: 0,
                final_stage: ManufacturingStage::Complete,
            },
            test_summary: TestSummary {
                total_tests: 1,
                passed: 1,
                failed: 0,
                inconclusive: 0,
                thermal_result: None,
                mechanical_result: None,
                adhesion_result: None,
            },
            audit_summary: AuditSummary {
                total_audits: 1,
                passed: 1,
                warned: 0,
                failed: 0,
                findings: vec![],
            },
            simulation_summary: SimulationSummary {
                cycles_simulated: 0,
                final_degradation: DegradationState::Nominal,
                coating_wear_percent: 0.0,
                max_temp_seen_c: 0.0,
                estimated_service_life_cycles: 1000,
            },
            defects: vec![],
            overall_status: OverallStatus::Pending,
            generated_at: Utc::now(),
        };
        assert_eq!(determine_overall_status(&report), OverallStatus::Accepted);
    }

    #[test]
    fn test_determine_overall_status_rejected_test_failure() {
        let report = TileReport {
            tile_id: None,
            batch_info: None,
            location_info: None,
            manufacturing_summary: ManufacturingSummary {
                total_steps: 1,
                steps_completed: 1,
                steps_with_warnings: 0,
                steps_with_failures: 0,
                final_stage: ManufacturingStage::Complete,
            },
            test_summary: TestSummary {
                total_tests: 1,
                passed: 0,
                failed: 1,
                inconclusive: 0,
                thermal_result: None,
                mechanical_result: None,
                adhesion_result: None,
            },
            audit_summary: AuditSummary {
                total_audits: 0,
                passed: 0,
                warned: 0,
                failed: 0,
                findings: vec![],
            },
            simulation_summary: SimulationSummary {
                cycles_simulated: 0,
                final_degradation: DegradationState::Nominal,
                coating_wear_percent: 0.0,
                max_temp_seen_c: 0.0,
                estimated_service_life_cycles: 1000,
            },
            defects: vec![],
            overall_status: OverallStatus::Pending,
            generated_at: Utc::now(),
        };
        assert_eq!(determine_overall_status(&report), OverallStatus::Rejected);
    }

    #[test]
    fn test_determine_overall_status_rejected_critical_defect() {
        let report = TileReport {
            tile_id: None,
            batch_info: None,
            location_info: None,
            manufacturing_summary: ManufacturingSummary {
                total_steps: 1,
                steps_completed: 1,
                steps_with_warnings: 0,
                steps_with_failures: 0,
                final_stage: ManufacturingStage::Complete,
            },
            test_summary: TestSummary {
                total_tests: 1,
                passed: 1,
                failed: 0,
                inconclusive: 0,
                thermal_result: None,
                mechanical_result: None,
                adhesion_result: None,
            },
            audit_summary: AuditSummary {
                total_audits: 1,
                passed: 1,
                warned: 0,
                failed: 0,
                findings: vec![],
            },
            simulation_summary: SimulationSummary {
                cycles_simulated: 0,
                final_degradation: DegradationState::Nominal,
                coating_wear_percent: 0.0,
                max_temp_seen_c: 0.0,
                estimated_service_life_cycles: 1000,
            },
            defects: vec![Defect {
                category: DefectCategory::Material,
                description: "Crack".to_string(),
                severity: DefectSeverity::Critical,
                detected_at: ManufacturingStage::RawMaterial,
                timestamp: Utc::now(),
            }],
            overall_status: OverallStatus::Pending,
            generated_at: Utc::now(),
        };
        assert_eq!(determine_overall_status(&report), OverallStatus::Rejected);
    }

    #[test]
    fn test_determine_overall_status_accepted_with_notes() {
        let report = TileReport {
            tile_id: None,
            batch_info: None,
            location_info: None,
            manufacturing_summary: ManufacturingSummary {
                total_steps: 1,
                steps_completed: 1,
                steps_with_warnings: 0,
                steps_with_failures: 0,
                final_stage: ManufacturingStage::Complete,
            },
            test_summary: TestSummary {
                total_tests: 1,
                passed: 0,
                failed: 0,
                inconclusive: 1,
                thermal_result: None,
                mechanical_result: None,
                adhesion_result: None,
            },
            audit_summary: AuditSummary {
                total_audits: 0,
                passed: 0,
                warned: 0,
                failed: 0,
                findings: vec![],
            },
            simulation_summary: SimulationSummary {
                cycles_simulated: 0,
                final_degradation: DegradationState::Nominal,
                coating_wear_percent: 0.0,
                max_temp_seen_c: 0.0,
                estimated_service_life_cycles: 1000,
            },
            defects: vec![],
            overall_status: OverallStatus::Pending,
            generated_at: Utc::now(),
        };
        assert_eq!(
            determine_overall_status(&report),
            OverallStatus::AcceptedWithNotes
        );
    }

    #[test]
    fn test_determine_overall_status_pending() {
        let report = TileReport {
            tile_id: None,
            batch_info: None,
            location_info: None,
            manufacturing_summary: ManufacturingSummary {
                total_steps: 0,
                steps_completed: 0,
                steps_with_warnings: 0,
                steps_with_failures: 0,
                final_stage: ManufacturingStage::RawMaterial,
            },
            test_summary: TestSummary {
                total_tests: 0,
                passed: 0,
                failed: 0,
                inconclusive: 0,
                thermal_result: None,
                mechanical_result: None,
                adhesion_result: None,
            },
            audit_summary: AuditSummary {
                total_audits: 0,
                passed: 0,
                warned: 0,
                failed: 0,
                findings: vec![],
            },
            simulation_summary: SimulationSummary {
                cycles_simulated: 0,
                final_degradation: DegradationState::Nominal,
                coating_wear_percent: 0.0,
                max_temp_seen_c: 0.0,
                estimated_service_life_cycles: 1000,
            },
            defects: vec![],
            overall_status: OverallStatus::Pending,
            generated_at: Utc::now(),
        };
        assert_eq!(determine_overall_status(&report), OverallStatus::Pending);
    }

    #[test]
    fn test_summarize_manufacturing_counts() {
        let steps = vec![
            ManufacturingStep {
                name: "RawMaterial".to_string(),
                step_number: 1,
                timestamp: Utc::now(),
                operator_id: "OP1".to_string(),
                parameters: HashMap::new(),
                result: StepResult::Success,
            },
            ManufacturingStep {
                name: "Coating".to_string(),
                step_number: 2,
                timestamp: Utc::now(),
                operator_id: "OP2".to_string(),
                parameters: HashMap::new(),
                result: StepResult::Warning("thin coat".to_string()),
            },
            ManufacturingStep {
                name: "Testing".to_string(),
                step_number: 3,
                timestamp: Utc::now(),
                operator_id: "OP3".to_string(),
                parameters: HashMap::new(),
                result: StepResult::Failure("delam".to_string()),
            },
        ];
        let summary = summarize_manufacturing(&steps);
        assert_eq!(summary.total_steps, 3);
        assert_eq!(summary.steps_completed, 1);
        assert_eq!(summary.steps_with_warnings, 1);
        assert_eq!(summary.steps_with_failures, 1);
        assert_eq!(summary.final_stage, ManufacturingStage::Testing);
    }

    #[test]
    fn test_summarize_manufacturing_empty() {
        let summary = summarize_manufacturing(&[]);
        assert_eq!(summary.total_steps, 0);
        assert_eq!(summary.final_stage, ManufacturingStage::RawMaterial);
    }

    #[test]
    fn test_summarize_tests_counts_and_extraction() {
        let mut thermal_data = HashMap::new();
        thermal_data.insert("max_temp_reached_c".to_string(), 1200.0);
        thermal_data.insert("backface_temp_c".to_string(), 150.0);
        thermal_data.insert("heat_flux_w_cm2".to_string(), 25.0);
        thermal_data.insert("duration_seconds".to_string(), 300.0);
        thermal_data.insert("cycles_completed".to_string(), 50.0);
        thermal_data.insert("degradation_percent".to_string(), 2.5);

        let mut mechanical_data = HashMap::new();
        mechanical_data.insert("compressive_strength_mpa".to_string(), 1.5);
        mechanical_data.insert("tensile_strength_mpa".to_string(), 0.8);
        mechanical_data.insert("flexural_strength_mpa".to_string(), 1.2);
        mechanical_data.insert("youngs_modulus_gpa".to_string(), 0.3);

        let mut adhesion_data = HashMap::new();
        adhesion_data.insert("bond_strength_mpa".to_string(), 2.0);
        adhesion_data.insert("peel_strength_n_m".to_string(), 4000.0);
        adhesion_data.insert("failure_mode".to_string(), 4.0);

        let records = vec![
            TestRecord {
                test_type: TestType::Thermal,
                result: TestOutcome::Pass,
                timestamp: Utc::now(),
                data: thermal_data,
                notes: "ok".to_string(),
            },
            TestRecord {
                test_type: TestType::Mechanical,
                result: TestOutcome::Pass,
                timestamp: Utc::now(),
                data: mechanical_data,
                notes: "ok".to_string(),
            },
            TestRecord {
                test_type: TestType::Adhesion,
                result: TestOutcome::Fail,
                timestamp: Utc::now(),
                data: adhesion_data,
                notes: "delam".to_string(),
            },
        ];

        let summary = summarize_tests(&records);
        assert_eq!(summary.total_tests, 3);
        assert_eq!(summary.passed, 2);
        assert_eq!(summary.failed, 1);
        assert_eq!(summary.inconclusive, 0);

        assert!(summary.thermal_result.is_some());
        let tr = summary.thermal_result.unwrap();
        assert_eq!(tr.max_temp_reached_c, 1200.0);
        assert_eq!(tr.cycles_completed, 50);
        assert!(tr.passed);

        assert!(summary.mechanical_result.is_some());
        let mr = summary.mechanical_result.unwrap();
        assert_eq!(mr.compressive_strength_mpa, 1.5);
        assert!(mr.passed);

        assert!(summary.adhesion_result.is_some());
        let ar = summary.adhesion_result.unwrap();
        assert_eq!(ar.bond_strength_mpa, 2.0);
        assert!(!ar.passed);
        assert_eq!(ar.failure_mode, FailureMode::None);
    }

    #[test]
    fn test_summarize_tests_defaults() {
        let records = vec![TestRecord {
            test_type: TestType::Thermal,
            result: TestOutcome::Inconclusive,
            timestamp: Utc::now(),
            data: HashMap::new(),
            notes: "inconclusive".to_string(),
        }];
        let summary = summarize_tests(&records);
        assert_eq!(summary.inconclusive, 1);
        let tr = summary.thermal_result.unwrap();
        assert_eq!(tr.max_temp_reached_c, 0.0);
        assert_eq!(tr.cycles_completed, 0);
    }

    #[test]
    fn test_summarize_audits() {
        let audits = vec![
            IdAuditResult {
                status: AuditStatus::Pass,
                findings: vec![],
                summary: "ok".to_string(),
                audited_id: "id1".to_string(),
                timestamp: Utc::now(),
            },
            IdAuditResult {
                status: AuditStatus::Warn,
                findings: vec![AuditFinding {
                    category: crate::FindingCategory::SchemaCompliance,
                    severity: crate::FindingSeverity::Warning,
                    message: "format off".to_string(),
                    expected: None,
                    actual: None,
                }],
                summary: "warn".to_string(),
                audited_id: "id2".to_string(),
                timestamp: Utc::now(),
            },
            IdAuditResult {
                status: AuditStatus::Fail,
                findings: vec![AuditFinding {
                    category: crate::FindingCategory::ChecksumValidity,
                    severity: crate::FindingSeverity::Critical,
                    message: "bad checksum".to_string(),
                    expected: None,
                    actual: None,
                }],
                summary: "fail".to_string(),
                audited_id: "id3".to_string(),
                timestamp: Utc::now(),
            },
        ];

        let summary = summarize_audits(&audits);
        assert_eq!(summary.total_audits, 3);
        assert_eq!(summary.passed, 1);
        assert_eq!(summary.warned, 1);
        assert_eq!(summary.failed, 1);
        assert_eq!(summary.findings.len(), 2);
    }

    #[test]
    fn test_summarize_simulation_nominal() {
        let state = SimulationState {
            current_temp_c: 25.0,
            max_temp_seen_c: 1200.0,
            heat_absorbed_kj: 500.0,
            heat_reflected_kj: 200.0,
            thermal_cycles: 100,
            degradation_state: DegradationState::Nominal,
            coating_wear_percent: 5.0,
            bond_stress_mpa: 0.5,
        };
        let summary = summarize_simulation(&state);
        assert_eq!(summary.cycles_simulated, 100);
        assert_eq!(summary.max_temp_seen_c, 1200.0);
        assert_eq!(summary.final_degradation, DegradationState::Nominal);
        assert_eq!(summary.estimated_service_life_cycles, 1000);
    }

    #[test]
    fn test_summarize_simulation_severe() {
        let state = SimulationState {
            current_temp_c: 25.0,
            max_temp_seen_c: 1200.0,
            heat_absorbed_kj: 500.0,
            heat_reflected_kj: 200.0,
            thermal_cycles: 300,
            degradation_state: DegradationState::Severe,
            coating_wear_percent: 50.0,
            bond_stress_mpa: 1.5,
        };
        let summary = summarize_simulation(&state);
        assert_eq!(summary.estimated_service_life_cycles, 200);
    }

    #[test]
    fn test_summarize_simulation_failed() {
        let state = SimulationState {
            current_temp_c: 25.0,
            max_temp_seen_c: 1200.0,
            heat_absorbed_kj: 500.0,
            heat_reflected_kj: 200.0,
            thermal_cycles: 500,
            degradation_state: DegradationState::Failed,
            coating_wear_percent: 95.0,
            bond_stress_mpa: 3.0,
        };
        let summary = summarize_simulation(&state);
        assert_eq!(summary.estimated_service_life_cycles, 0);
    }
}

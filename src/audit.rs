use crate::{
    AppliedId, AuditFinding, AuditStatus, FindingCategory, FindingSeverity, IdAuditResult,
    IdSchema, TileBatch, TileId, TileLocation,
};
use chrono::Utc;
use std::collections::HashSet;

pub struct AuditEngine {
    seen_ids: HashSet<String>,
}

impl AuditEngine {
    pub fn new() -> Self {
        Self {
            seen_ids: HashSet::new(),
        }
    }

    pub fn audit_id(
        &mut self,
        id: &TileId,
        expected_batch: &TileBatch,
        expected_location: &TileLocation,
    ) -> IdAuditResult {
        let mut findings = Vec::new();

        findings.extend(self.audit_schema_compliance(&id.raw));
        findings.extend(self.audit_checksum(id));
        findings.extend(self.audit_batch_consistency(id, expected_batch));
        findings.extend(self.audit_location_consistency(id, expected_location));
        findings.extend(self.audit_duplicate(id));
        findings.extend(self.audit_serialization(id));
        findings.extend(self.audit_human_readable(id));

        let status = Self::compute_status(&findings);
        let summary = Self::build_summary(&findings, &status);

        IdAuditResult {
            status,
            findings,
            summary,
            audited_id: id.raw.clone(),
            timestamp: Utc::now(),
        }
    }

    pub fn audit_schema_compliance(&self, raw: &str) -> Vec<AuditFinding> {
        let mut findings = Vec::new();
        let schema = IdSchema::default();

        match crate::identification::validate_schema(raw, &schema) {
            Ok(true) => {
                findings.push(AuditFinding {
                    category: FindingCategory::SchemaCompliance,
                    severity: FindingSeverity::Info,
                    message: "Schema validation passed".to_string(),
                    expected: Some(schema.format.clone()),
                    actual: Some(raw.to_string()),
                });
            }
            Ok(false) => {
                findings.push(AuditFinding {
                    category: FindingCategory::SchemaCompliance,
                    severity: FindingSeverity::Critical,
                    message: "Schema validation failed: format does not match BATCH-LOC-SEQ-CHECK"
                        .to_string(),
                    expected: Some(schema.format.clone()),
                    actual: Some(raw.to_string()),
                });
            }
            Err(e) => {
                findings.push(AuditFinding {
                    category: FindingCategory::SchemaCompliance,
                    severity: FindingSeverity::Critical,
                    message: format!("Schema validation error: {}", e),
                    expected: Some(schema.format.clone()),
                    actual: Some(raw.to_string()),
                });
            }
        }

        findings
    }

    pub fn audit_checksum(&self, id: &TileId) -> Vec<AuditFinding> {
        let mut findings = Vec::new();

        match crate::identification::validate_checksum(id) {
            Ok(true) => {
                findings.push(AuditFinding {
                    category: FindingCategory::ChecksumValidity,
                    severity: FindingSeverity::Info,
                    message: "Checksum valid".to_string(),
                    expected: Some(format!("{:02}", id.checksum)),
                    actual: Some(format!("{:02}", id.checksum)),
                });
            }
            Ok(false) => {
                findings.push(AuditFinding {
                    category: FindingCategory::ChecksumValidity,
                    severity: FindingSeverity::Critical,
                    message: "Checksum mismatch".to_string(),
                    expected: Some(format!("valid checksum for {}", id.raw)),
                    actual: Some(format!("{:02}", id.checksum)),
                });
            }
            Err(e) => {
                findings.push(AuditFinding {
                    category: FindingCategory::ChecksumValidity,
                    severity: FindingSeverity::Critical,
                    message: format!("Checksum validation error: {}", e),
                    expected: Some("valid checksum".to_string()),
                    actual: Some(format!("{:02}", id.checksum)),
                });
            }
        }

        findings
    }

    pub fn audit_batch_consistency(&self, id: &TileId, expected: &TileBatch) -> Vec<AuditFinding> {
        let mut findings = Vec::new();
        let expected_code = if expected.batch_code.len() >= 6 {
            expected.batch_code[..6].to_string()
        } else {
            format!("{:0<6}", expected.batch_code)
        };
        let actual_code = if id.batch.batch_code.len() >= 6 {
            id.batch.batch_code[..6].to_string()
        } else {
            format!("{:0<6}", id.batch.batch_code)
        };

        if actual_code == expected_code {
            findings.push(AuditFinding {
                category: FindingCategory::BatchConsistency,
                severity: FindingSeverity::Info,
                message: "Batch code matches expected".to_string(),
                expected: Some(expected_code),
                actual: Some(actual_code),
            });
        } else {
            findings.push(AuditFinding {
                category: FindingCategory::BatchConsistency,
                severity: FindingSeverity::Critical,
                message: "Batch code mismatch".to_string(),
                expected: Some(expected_code),
                actual: Some(actual_code),
            });
        }

        findings
    }

    pub fn audit_location_consistency(
        &self,
        id: &TileId,
        expected: &TileLocation,
    ) -> Vec<AuditFinding> {
        let mut findings = Vec::new();

        let orbiter_match = id.location.orbiter_id[..2.min(id.location.orbiter_id.len())]
            == expected.orbiter_id[..2.min(expected.orbiter_id.len())];
        let panel_match = id.location.panel_id[..2.min(id.location.panel_id.len())]
            == expected.panel_id[..2.min(expected.panel_id.len())];
        let row_match = id.location.row == expected.row;
        let col_match = id.location.column == expected.column;
        let surface_match = std::mem::discriminant(&id.location.surface)
            == std::mem::discriminant(&expected.surface);

        if orbiter_match && panel_match && row_match && col_match && surface_match {
            findings.push(AuditFinding {
                category: FindingCategory::LocationConsistency,
                severity: FindingSeverity::Info,
                message: "Location matches expected".to_string(),
                expected: Some(format!("{:?}", expected)),
                actual: Some(format!("{:?}", id.location)),
            });
        } else {
            let mut mismatches = Vec::new();
            if !orbiter_match {
                mismatches.push("orbiter");
            }
            if !surface_match {
                mismatches.push("surface");
            }
            if !panel_match {
                mismatches.push("panel");
            }
            if !row_match {
                mismatches.push("row");
            }
            if !col_match {
                mismatches.push("column");
            }
            findings.push(AuditFinding {
                category: FindingCategory::LocationConsistency,
                severity: FindingSeverity::Critical,
                message: format!("Location mismatch: {}", mismatches.join(", ")),
                expected: Some(format!("{:?}", expected)),
                actual: Some(format!("{:?}", id.location)),
            });
        }

        findings
    }

    pub fn audit_duplicate(&mut self, id: &TileId) -> Vec<AuditFinding> {
        let mut findings = Vec::new();

        if self.seen_ids.contains(&id.raw) {
            findings.push(AuditFinding {
                category: FindingCategory::DuplicateDetection,
                severity: FindingSeverity::Critical,
                message: "Duplicate ID detected".to_string(),
                expected: Some("unique ID".to_string()),
                actual: Some(id.raw.clone()),
            });
        } else {
            findings.push(AuditFinding {
                category: FindingCategory::DuplicateDetection,
                severity: FindingSeverity::Info,
                message: "ID is unique".to_string(),
                expected: Some("unique ID".to_string()),
                actual: Some(id.raw.clone()),
            });
            self.seen_ids.insert(id.raw.clone());
        }

        findings
    }

    pub fn audit_application(&self, applied: &AppliedId) -> Vec<AuditFinding> {
        let mut findings = Vec::new();

        if applied.physical_reading == applied.digital_record {
            findings.push(AuditFinding {
                category: FindingCategory::ApplicationCorrectness,
                severity: FindingSeverity::Info,
                message: "Physical reading matches digital record".to_string(),
                expected: Some(applied.digital_record.clone()),
                actual: Some(applied.physical_reading.clone()),
            });
        } else {
            findings.push(AuditFinding {
                category: FindingCategory::ApplicationCorrectness,
                severity: FindingSeverity::Critical,
                message: "Physical reading does not match digital record".to_string(),
                expected: Some(applied.digital_record.clone()),
                actual: Some(applied.physical_reading.clone()),
            });
        }

        let id_raw = crate::identification::encode_id(&applied.id, &IdSchema::default());
        if applied.digital_record == id_raw {
            findings.push(AuditFinding {
                category: FindingCategory::ApplicationCorrectness,
                severity: FindingSeverity::Info,
                message: "Digital record matches encoded ID".to_string(),
                expected: Some(id_raw),
                actual: Some(applied.digital_record.clone()),
            });
        } else {
            findings.push(AuditFinding {
                category: FindingCategory::ApplicationCorrectness,
                severity: FindingSeverity::Warning,
                message: "Digital record does not match encoded ID".to_string(),
                expected: Some(id_raw),
                actual: Some(applied.digital_record.clone()),
            });
        }

        findings
    }

    pub fn audit_serialization(&self, id: &TileId) -> Vec<AuditFinding> {
        let mut findings = Vec::new();
        let schema = IdSchema::default();

        let encoded = crate::identification::encode_id(id, &schema);
        match crate::identification::parse_id(&encoded, &schema) {
            Ok(parsed) => {
                if parsed.sequence == id.sequence
                    && parsed.checksum == id.checksum
                    && parsed.raw == id.raw
                {
                    findings.push(AuditFinding {
                        category: FindingCategory::SerializationIntegrity,
                        severity: FindingSeverity::Info,
                        message: "Encode/decode roundtrip successful".to_string(),
                        expected: Some(id.raw.clone()),
                        actual: Some(parsed.raw),
                    });
                } else {
                    findings.push(AuditFinding {
                        category: FindingCategory::SerializationIntegrity,
                        severity: FindingSeverity::Critical,
                        message: "Encode/decode roundtrip data mismatch".to_string(),
                        expected: Some(id.raw.clone()),
                        actual: Some(parsed.raw),
                    });
                }
            }
            Err(e) => {
                findings.push(AuditFinding {
                    category: FindingCategory::SerializationIntegrity,
                    severity: FindingSeverity::Critical,
                    message: format!("Encode/decode roundtrip failed: {}", e),
                    expected: Some(id.raw.clone()),
                    actual: Some(encoded),
                });
            }
        }

        findings
    }

    pub fn audit_human_readable(&self, id: &TileId) -> Vec<AuditFinding> {
        let mut findings = Vec::new();
        let hr = crate::identification::to_human_readable(id);

        let contains_batch = hr.contains(&id.batch.batch_code);
        let contains_seq = hr.contains(&format!("{:04}", id.sequence));
        let contains_checksum = hr.contains(&format!("{:02}", id.checksum));

        if contains_batch && contains_seq && contains_checksum {
            findings.push(AuditFinding {
                category: FindingCategory::HumanReadability,
                severity: FindingSeverity::Info,
                message: "Human-readable format contains all required fields".to_string(),
                expected: Some("batch, sequence, checksum present".to_string()),
                actual: Some(hr),
            });
        } else {
            let mut missing = Vec::new();
            if !contains_batch {
                missing.push("batch");
            }
            if !contains_seq {
                missing.push("sequence");
            }
            if !contains_checksum {
                missing.push("checksum");
            }
            findings.push(AuditFinding {
                category: FindingCategory::HumanReadability,
                severity: FindingSeverity::Warning,
                message: format!("Human-readable format missing fields: {}", missing.join(", ")),
                expected: Some("batch, sequence, checksum present".to_string()),
                actual: Some(hr),
            });
        }

        findings
    }

    fn compute_status(findings: &[AuditFinding]) -> AuditStatus {
        let has_critical = findings.iter().any(|f| f.severity == FindingSeverity::Critical);
        let has_warning = findings.iter().any(|f| f.severity == FindingSeverity::Warning);

        if has_critical {
            AuditStatus::Fail
        } else if has_warning {
            AuditStatus::Warn
        } else {
            AuditStatus::Pass
        }
    }

    fn build_summary(findings: &[AuditFinding], status: &AuditStatus) -> String {
        let info_count = findings.iter().filter(|f| f.severity == FindingSeverity::Info).count();
        let warning_count = findings
            .iter()
            .filter(|f| f.severity == FindingSeverity::Warning)
            .count();
        let critical_count = findings
            .iter()
            .filter(|f| f.severity == FindingSeverity::Critical)
            .count();

        format!(
            "Audit {:?}: {} info, {} warnings, {} critical findings",
            status, info_count, warning_count, critical_count
        )
    }
}

impl Default for AuditEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{TileBatch, TileId, TileLocation, TileSurface};
    use chrono::Utc;

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

    #[test]
    fn test_audit_id_full_pass() {
        let mut engine = AuditEngine::new();
        let batch = test_batch();
        let location = test_location();
        let id = crate::identification::generate_id(&batch, &location, 1).unwrap();

        let result = engine.audit_id(&id, &batch, &location);
        assert_eq!(result.status, AuditStatus::Pass);
        assert!(result
            .findings
            .iter()
            .any(|f| f.category == FindingCategory::SchemaCompliance
                && f.severity == FindingSeverity::Info));
    }

    #[test]
    fn test_audit_schema_compliance_broken() {
        let engine = AuditEngine::new();
        let broken = "INVALID-ID-FORMAT";
        let findings = engine.audit_schema_compliance(broken);

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].severity, FindingSeverity::Critical);
        assert_eq!(findings[0].category, FindingCategory::SchemaCompliance);
    }

    #[test]
    fn test_audit_duplicate_detection() {
        let mut engine = AuditEngine::new();
        let batch = test_batch();
        let location = test_location();
        let id = crate::identification::generate_id(&batch, &location, 1).unwrap();

        let first = engine.audit_duplicate(&id);
        assert_eq!(first[0].severity, FindingSeverity::Info);

        let second = engine.audit_duplicate(&id);
        assert_eq!(second[0].severity, FindingSeverity::Critical);
        assert_eq!(second[0].category, FindingCategory::DuplicateDetection);
    }

    #[test]
    fn test_audit_batch_mismatch() {
        let engine = AuditEngine::new();
        let batch = test_batch();
        let location = test_location();
        let id = crate::identification::generate_id(&batch, &location, 1).unwrap();

        let wrong_batch = TileBatch {
            batch_code: "B99999".to_string(),
            production_date: Utc::now(),
            oven_id: "O2".to_string(),
            operator_id: "OP002".to_string(),
        };

        let findings = engine.audit_batch_consistency(&id, &wrong_batch);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].severity, FindingSeverity::Critical);
        assert_eq!(findings[0].category, FindingCategory::BatchConsistency);
    }

    #[test]
    fn test_audit_location_mismatch() {
        let engine = AuditEngine::new();
        let batch = test_batch();
        let location = test_location();
        let id = crate::identification::generate_id(&batch, &location, 1).unwrap();

        let wrong_location = TileLocation {
            orbiter_id: "OV099".to_string(),
            surface: TileSurface::NoseCap,
            panel_id: "99".to_string(),
            row: 9,
            column: 9,
        };

        let findings = engine.audit_location_consistency(&id, &wrong_location);
        assert!(findings.iter().any(|f| f.severity == FindingSeverity::Critical));
        assert_eq!(findings[0].category, FindingCategory::LocationConsistency);
    }

    #[test]
    fn test_audit_application_mismatch() {
        let engine = AuditEngine::new();
        let batch = test_batch();
        let location = test_location();
        let id = crate::identification::generate_id(&batch, &location, 1).unwrap();

        let applied = AppliedId {
            id: id.clone(),
            application_method: "Laser".to_string(),
            application_timestamp: Utc::now(),
            operator_id: "OP001".to_string(),
            physical_reading: "WRONG-READING".to_string(),
            digital_record: id.raw.clone(),
        };

        let findings = engine.audit_application(&applied);
        assert!(findings
            .iter()
            .any(|f| f.severity == FindingSeverity::Critical
                && f.category == FindingCategory::ApplicationCorrectness));
    }

    #[test]
    fn test_audit_checksum_invalid() {
        let engine = AuditEngine::new();
        let batch = test_batch();
        let location = test_location();
        let mut id = crate::identification::generate_id(&batch, &location, 1).unwrap();
        id.checksum = 99;

        let findings = engine.audit_checksum(&id);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].severity, FindingSeverity::Critical);
        assert_eq!(findings[0].category, FindingCategory::ChecksumValidity);
    }

    #[test]
    fn test_audit_serialization_roundtrip() {
        let engine = AuditEngine::new();
        let batch = test_batch();
        let location = test_location();
        let id = crate::identification::generate_id(&batch, &location, 42).unwrap();

        let findings = engine.audit_serialization(&id);
        assert_eq!(findings[0].severity, FindingSeverity::Info);
        assert_eq!(findings[0].category, FindingCategory::SerializationIntegrity);
    }

    #[test]
    fn test_audit_human_readable() {
        let engine = AuditEngine::new();
        let batch = test_batch();
        let location = test_location();
        let id = crate::identification::generate_id(&batch, &location, 1).unwrap();

        let findings = engine.audit_human_readable(&id);
        assert_eq!(findings[0].severity, FindingSeverity::Info);
        assert_eq!(findings[0].category, FindingCategory::HumanReadability);
    }

    #[test]
    fn test_audit_id_fails_with_broken_schema() {
        let mut engine = AuditEngine::new();
        let batch = test_batch();
        let location = test_location();

        let broken_id = TileId {
            raw: "BAD-ID".to_string(),
            batch: batch.clone(),
            location: location.clone(),
            sequence: 1,
            checksum: 0,
        };

        let result = engine.audit_id(&broken_id, &batch, &location);
        assert_eq!(result.status, AuditStatus::Fail);
        assert!(result
            .findings
            .iter()
            .any(|f| f.category == FindingCategory::SchemaCompliance
                && f.severity == FindingSeverity::Critical));
    }

    #[test]
    fn test_batch_records_tracking() {
        let mut engine = AuditEngine::new();
        let batch = test_batch();
        let location = test_location();
        let id = crate::identification::generate_id(&batch, &location, 1).unwrap();

        engine.audit_duplicate(&id);
        assert!(engine.seen_ids.contains(&id.raw));
    }
}

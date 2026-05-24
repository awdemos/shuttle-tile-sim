use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod audit;
pub mod bonding;
pub mod geometry;
pub mod identification;
pub mod manufacturing;
pub mod materials;
pub mod report;
pub mod simulation;
pub mod testing;

#[derive(thiserror::Error, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TileError {
    #[error("Material error: {0}")]
    MaterialError(String),
    #[error("Geometry error: {0}")]
    GeometryError(String),
    #[error("Bonding error: {0}")]
    BondingError(String),
    #[error("Identification error: {0}")]
    IdentificationError(String),
    #[error("Audit failure: {0}")]
    AuditError(String),
    #[error("Test failure: {0}")]
    TestError(String),
    #[error("Simulation error: {0}")]
    SimulationError(String),
    #[error("Manufacturing error: {0}")]
    ManufacturingError(String),
}

pub type Result<T> = std::result::Result<T, TileError>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TileMaterial {
    pub density_kg_m3: f64,
    pub porosity_fraction: f64,
    pub thermal_conductivity_w_m_k: f64,
    pub specific_heat_j_kg_k: f64,
    pub max_service_temp_c: f64,
    pub fiber_diameter_um: f64,
}

impl TileMaterial {
    pub fn li_900() -> Self {
        Self {
            density_kg_m3: 144.0,
            porosity_fraction: 0.93,
            thermal_conductivity_w_m_k: 0.017,
            specific_heat_j_kg_k: 628.0,
            max_service_temp_c: 1260.0,
            fiber_diameter_um: 1.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Coating {
    pub material: String,
    pub thickness_mm: f64,
    pub emissivity: f64,
    pub absorptivity: f64,
    pub max_temp_c: f64,
    pub application_method: String,
}

impl Coating {
    pub fn black_hrsi() -> Self {
        Self {
            material: "Borosilicate glass with silicon tetraboride".to_string(),
            thickness_mm: 0.38,
            emissivity: 0.85,
            absorptivity: 0.85,
            max_temp_c: 1260.0,
            application_method: "Spray + sinter".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StrainIsolationPad {
    pub material: String,
    pub thickness_mm: f64,
    pub density_kg_m3: f64,
    pub shear_modulus_mpa: f64,
    pub max_temp_c: f64,
}

impl StrainIsolationPad {
    pub fn standard() -> Self {
        Self {
            material: "Nomex felt".to_string(),
            thickness_mm: 4.76,
            density_kg_m3: 64.0,
            shear_modulus_mpa: 0.5,
            max_temp_c: 260.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Adhesive {
    pub material: String,
    pub thickness_mm: f64,
    pub cure_temp_c: f64,
    pub cure_time_hours: f64,
    pub shear_strength_mpa: f64,
    pub peel_strength_n_m: f64,
    pub max_service_temp_c: f64,
}

impl Adhesive {
    pub fn rtv_560() -> Self {
        Self {
            material: "RTV-560 silicone".to_string(),
            thickness_mm: 0.25,
            cure_temp_c: 25.0,
            cure_time_hours: 72.0,
            shear_strength_mpa: 1.5,
            peel_strength_n_m: 3500.0,
            max_service_temp_c: 260.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Dimensions {
    pub length_mm: f64,
    pub width_mm: f64,
    pub thickness_mm: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TileGeometry {
    pub dimensions: Dimensions,
    pub surface_area_m2: f64,
    pub volume_m3: f64,
    pub shape_type: TileShape,
    pub machining_tolerance_mm: f64,
    pub edge_radius_mm: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TileShape {
    Flat,
    Contoured,
    Custom { complexity: u8 },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TileId {
    pub raw: String,
    pub batch: TileBatch,
    pub location: TileLocation,
    pub sequence: u32,
    pub checksum: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TileBatch {
    pub batch_code: String,
    pub production_date: DateTime<Utc>,
    pub oven_id: String,
    pub operator_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TileLocation {
    pub orbiter_id: String,
    pub surface: TileSurface,
    pub panel_id: String,
    pub row: u8,
    pub column: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TileSurface {
    NoseCap,
    WingLeadingEdge,
    FuselageTop,
    FuselageBottom,
    VerticalStabilizer,
    OmsPod,
    EjectionSeat,
    Custom(String),
}

impl std::fmt::Display for TileSurface {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TileSurface::NoseCap => write!(f, "NC"),
            TileSurface::WingLeadingEdge => write!(f, "WLE"),
            TileSurface::FuselageTop => write!(f, "FT"),
            TileSurface::FuselageBottom => write!(f, "FB"),
            TileSurface::VerticalStabilizer => write!(f, "VS"),
            TileSurface::OmsPod => write!(f, "OMS"),
            TileSurface::EjectionSeat => write!(f, "ES"),
            TileSurface::Custom(s) => write!(f, "{}", s),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IdSchema {
    pub format: String,
    pub batch_code_length: usize,
    pub location_code_length: usize,
    pub sequence_length: usize,
    pub checksum_length: usize,
    pub delimiter: char,
}

impl Default for IdSchema {
    fn default() -> Self {
        Self {
            format: "BATCH-LOC-SEQ-CHECK".to_string(),
            batch_code_length: 6,
            location_code_length: 8,
            sequence_length: 4,
            checksum_length: 2,
            delimiter: '-',
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppliedId {
    pub id: TileId,
    pub application_method: String,
    pub application_timestamp: DateTime<Utc>,
    pub operator_id: String,
    pub physical_reading: String,
    pub digital_record: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IdAuditResult {
    pub status: AuditStatus,
    pub findings: Vec<AuditFinding>,
    pub summary: String,
    pub audited_id: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum AuditStatus {
    Pass,
    Warn,
    Fail,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuditFinding {
    pub category: FindingCategory,
    pub severity: FindingSeverity,
    pub message: String,
    pub expected: Option<String>,
    pub actual: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum FindingCategory {
    SchemaCompliance,
    ChecksumValidity,
    BatchConsistency,
    LocationConsistency,
    DuplicateDetection,
    ApplicationCorrectness,
    SerializationIntegrity,
    HumanReadability,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum FindingSeverity {
    Info,
    Warning,
    Critical,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ManufacturingStep {
    pub name: String,
    pub step_number: u32,
    pub timestamp: DateTime<Utc>,
    pub operator_id: String,
    pub parameters: HashMap<String, String>,
    pub result: StepResult,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StepResult {
    Success,
    Warning(String),
    Failure(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TileState {
    pub id: Option<TileId>,
    pub material: Option<TileMaterial>,
    pub geometry: Option<TileGeometry>,
    pub coating: Option<Coating>,
    pub strain_isolation_pad: Option<StrainIsolationPad>,
    pub adhesive: Option<Adhesive>,
    pub applied_id: Option<AppliedId>,
    pub manufacturing_steps: Vec<ManufacturingStep>,
    pub test_results: Vec<TestRecord>,
    pub audit_results: Vec<IdAuditResult>,
    pub simulation_state: Option<SimulationState>,
    pub defects: Vec<Defect>,
    pub created_at: DateTime<Utc>,
    pub current_stage: ManufacturingStage,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ManufacturingStage {
    RawMaterial,
    SlurryFormation,
    MoldCasting,
    Drying,
    Sintering,
    CncMachining,
    Coating,
    Bonding,
    Identification,
    Testing,
    Complete,
    Rejected,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TestRecord {
    pub test_type: TestType,
    pub result: TestOutcome,
    pub timestamp: DateTime<Utc>,
    pub data: HashMap<String, f64>,
    pub notes: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TestType {
    Thermal,
    Mechanical,
    Adhesion,
    Visual,
    Dimensional,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TestOutcome {
    Pass,
    Fail,
    Inconclusive,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Defect {
    pub category: DefectCategory,
    pub description: String,
    pub severity: DefectSeverity,
    pub detected_at: ManufacturingStage,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum DefectCategory {
    Material,
    Geometry,
    Coating,
    Bonding,
    Identification,
    Thermal,
    Mechanical,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum DefectSeverity {
    Minor,
    Major,
    Critical,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ThermalTestResult {
    pub max_temp_reached_c: f64,
    pub backface_temp_c: f64,
    pub heat_flux_w_cm2: f64,
    pub duration_seconds: f64,
    pub cycles_completed: u32,
    pub degradation_percent: f64,
    pub passed: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MechanicalTestResult {
    pub compressive_strength_mpa: f64,
    pub tensile_strength_mpa: f64,
    pub flexural_strength_mpa: f64,
    pub youngs_modulus_gpa: f64,
    pub passed: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AdhesionTestResult {
    pub bond_strength_mpa: f64,
    pub peel_strength_n_m: f64,
    pub failure_mode: FailureMode,
    pub passed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum FailureMode {
    Cohesive,
    Adhesive,
    Substrate,
    Mixed,
    None,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SimulationState {
    pub current_temp_c: f64,
    pub max_temp_seen_c: f64,
    pub heat_absorbed_kj: f64,
    pub heat_reflected_kj: f64,
    pub thermal_cycles: u32,
    pub degradation_state: DegradationState,
    pub coating_wear_percent: f64,
    pub bond_stress_mpa: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum DegradationState {
    Nominal,
    Slight,
    Moderate,
    Severe,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TileReport {
    pub tile_id: Option<String>,
    pub batch_info: Option<TileBatch>,
    pub location_info: Option<TileLocation>,
    pub manufacturing_summary: ManufacturingSummary,
    pub test_summary: TestSummary,
    pub audit_summary: AuditSummary,
    pub simulation_summary: SimulationSummary,
    pub defects: Vec<Defect>,
    pub overall_status: OverallStatus,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ManufacturingSummary {
    pub total_steps: usize,
    pub steps_completed: usize,
    pub steps_with_warnings: usize,
    pub steps_with_failures: usize,
    pub final_stage: ManufacturingStage,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TestSummary {
    pub total_tests: usize,
    pub passed: usize,
    pub failed: usize,
    pub inconclusive: usize,
    pub thermal_result: Option<ThermalTestResult>,
    pub mechanical_result: Option<MechanicalTestResult>,
    pub adhesion_result: Option<AdhesionTestResult>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuditSummary {
    pub total_audits: usize,
    pub passed: usize,
    pub warned: usize,
    pub failed: usize,
    pub findings: Vec<AuditFinding>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SimulationSummary {
    pub cycles_simulated: u32,
    pub final_degradation: DegradationState,
    pub coating_wear_percent: f64,
    pub max_temp_seen_c: f64,
    pub estimated_service_life_cycles: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum OverallStatus {
    Accepted,
    AcceptedWithNotes,
    Rejected,
    Pending,
}

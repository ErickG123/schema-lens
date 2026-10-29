use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ObjectId {
    pub schema: String,
    pub name: String,
}

impl ObjectId {
    pub fn new(schema: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            schema: schema.into(),
            name: name.into(),
        }
    }

    pub fn full_name(&self) -> String {
        format!("[{}].[{}]", self.schema, self.name)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SchemaObjectType {
    Schema,
    Table,
    Column,
    PrimaryKey,
    ForeignKey,
    CheckConstraint,
    DefaultConstraint,
    UniqueConstraint,
    Index,
    View,
    StoredProcedure,
    Function,
    Trigger,
    Sequence,
    Synonym,
    CustomType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DifferenceCategory {
    MissingInTarget,
    MissingInSource,
    PropertyMismatch,
    DefinitionMismatch,
    DependencyMismatch,
    UnsupportedComparison,
    Indeterminate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DifferenceSeverity {
    Info,
    Low,
    Medium,
    Critical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfidenceStatus {
    High,
    Medium,
    Indeterminate,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SchemaDifference {
    pub id: String,
    pub object_type: SchemaObjectType,
    pub schema_name: String,
    pub object_name: String,
    pub category: DifferenceCategory,
    pub property_name: Option<String>,
    pub value_source: Option<String>,
    pub value_target: Option<String>,
    pub severity: DifferenceSeverity,
    pub confidence: ConfidenceStatus,
    pub explanation: String,
    pub risk_assessment: Option<String>,
    pub can_generate_sql: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SchemaMetadata {
    pub name: String,
    pub owner: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ColumnMetadata {
    pub name: String,
    pub ordinal_position: u32,
    pub data_type: String,
    pub max_length: i16,
    pub precision: u8,
    pub scale: u8,
    pub is_nullable: bool,
    pub is_identity: bool,
    pub identity_seed: Option<i64>,
    pub identity_increment: Option<i64>,
    pub is_computed: bool,
    pub computed_definition: Option<String>,
    #[serde(default)]
    pub computed_persisted: bool,
    pub collation: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyConstraintMetadata {
    pub name: String,
    pub is_clustered: bool,
    pub columns: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ForeignKeyMetadata {
    pub name: String,
    pub referenced_schema: String,
    pub referenced_table: String,
    pub column_mappings: Vec<(String, String)>,
    pub delete_rule: String,
    pub update_rule: String,
    pub is_disabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckConstraintMetadata {
    pub name: String,
    pub definition: String,
    pub is_disabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DefaultConstraintMetadata {
    pub name: String,
    pub column_name: String,
    pub definition: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndexMetadata {
    pub name: String,
    pub is_unique: bool,
    pub is_clustered: bool,
    pub key_columns: Vec<String>,
    pub key_columns_descending: Vec<bool>,
    pub included_columns: Vec<String>,
    pub filter_definition: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TableMetadata {
    pub id: ObjectId,
    pub columns: Vec<ColumnMetadata>,
    pub primary_key: Option<KeyConstraintMetadata>,
    pub foreign_keys: Vec<ForeignKeyMetadata>,
    pub check_constraints: Vec<CheckConstraintMetadata>,
    pub default_constraints: Vec<DefaultConstraintMetadata>,
    pub unique_constraints: Vec<KeyConstraintMetadata>,
    pub indexes: Vec<IndexMetadata>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModuleMetadata {
    pub id: ObjectId,
    pub object_type: SchemaObjectType,
    pub definition: Option<String>,
    pub is_encrypted: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TriggerMetadata {
    pub id: ObjectId,
    pub parent_table: ObjectId,
    pub is_instead_of: bool,
    pub events: Vec<String>,
    pub definition: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SequenceMetadata {
    pub id: ObjectId,
    pub data_type: String,
    pub start_value: i64,
    pub increment: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SynonymMetadata {
    pub id: ObjectId,
    pub base_object: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CustomTypeMetadata {
    pub id: ObjectId,
    pub base_type: String,
    pub max_length: i16,
    pub is_nullable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DatabaseSnapshot {
    pub server_name: String,
    pub database_name: String,
    pub sql_server_version: String,
    pub collation: String,
    pub compatibility_level: u16,
    pub captured_at_utc: String,
    pub schemas: Vec<SchemaMetadata>,
    pub tables: Vec<TableMetadata>,
    pub views: Vec<ModuleMetadata>,
    pub procedures: Vec<ModuleMetadata>,
    pub functions: Vec<ModuleMetadata>,
    pub triggers: Vec<TriggerMetadata>,
    pub sequences: Vec<SequenceMetadata>,
    pub synonyms: Vec<SynonymMetadata>,
    pub custom_types: Vec<CustomTypeMetadata>,
    pub collection_warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComparisonSummary {
    pub total_differences: usize,
    pub missing_in_target: usize,
    pub missing_in_source: usize,
    pub property_mismatches: usize,
    pub definition_mismatches: usize,
    pub indeterminate: usize,
    pub critical_severity: usize,
    pub medium_severity: usize,
    pub low_severity: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComparisonRunResult {
    pub run_id: String,
    pub source_database: String,
    pub target_database: String,
    pub summary: ComparisonSummary,
    pub differences: Vec<SchemaDifference>,
    pub generated_at_utc: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MigrationStep {
    pub order: u32,
    pub description: String,
    pub sql: String,
    pub is_destructive: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MigrationPlan {
    pub run_id: String,
    pub target_database: String,
    pub generated_at_utc: String,
    pub total_steps: usize,
    pub destructive_steps: usize,
    pub unsupported_operations: Vec<String>,
    pub script_sql: String,
    pub steps: Vec<MigrationStep>,
}

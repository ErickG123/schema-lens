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

impl SchemaObjectType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Schema => "schema",
            Self::Table => "table",
            Self::Column => "column",
            Self::PrimaryKey => "primary_key",
            Self::ForeignKey => "foreign_key",
            Self::CheckConstraint => "check_constraint",
            Self::DefaultConstraint => "default_constraint",
            Self::UniqueConstraint => "unique_constraint",
            Self::Index => "index",
            Self::View => "view",
            Self::StoredProcedure => "stored_procedure",
            Self::Function => "function",
            Self::Trigger => "trigger",
            Self::Sequence => "sequence",
            Self::Synonym => "synonym",
            Self::CustomType => "custom_type",
        }
    }
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

impl DifferenceCategory {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::MissingInTarget => "missing_in_target",
            Self::MissingInSource => "missing_in_source",
            Self::PropertyMismatch => "property_mismatch",
            Self::DefinitionMismatch => "definition_mismatch",
            Self::DependencyMismatch => "dependency_mismatch",
            Self::UnsupportedComparison => "unsupported_comparison",
            Self::Indeterminate => "indeterminate",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DifferenceOperation {
    CreateSchema,
    DropSchema,
    CreateTable,
    DropTable,
    AlterTable,
    AddColumn,
    DropColumn,
    AlterColumnType,
    AlterColumnCapacity,
    AlterColumnNullability,
    AlterColumnCollation,
    AddPrimaryKey,
    DropPrimaryKey,
    AlterPrimaryKey,
    AddForeignKey,
    DropForeignKey,
    AlterForeignKey,
    AddUniqueConstraint,
    DropUniqueConstraint,
    AlterUniqueConstraint,
    AddCheckConstraint,
    DropCheckConstraint,
    AlterCheckConstraint,
    AddDefaultConstraint,
    DropDefaultConstraint,
    AlterDefaultConstraint,
    CreateIndex,
    DropIndex,
    AlterIndex,
    CreateView,
    DropView,
    AlterView,
    CreateStoredProcedure,
    DropStoredProcedure,
    AlterStoredProcedure,
    CreateFunction,
    DropFunction,
    AlterFunction,
    Unclassified,
}

impl DifferenceOperation {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::CreateSchema => "create_schema",
            Self::DropSchema => "drop_schema",
            Self::CreateTable => "create_table",
            Self::DropTable => "drop_table",
            Self::AlterTable => "alter_table",
            Self::AddColumn => "add_column",
            Self::DropColumn => "drop_column",
            Self::AlterColumnType => "alter_column_type",
            Self::AlterColumnCapacity => "alter_column_capacity",
            Self::AlterColumnNullability => "alter_column_nullability",
            Self::AlterColumnCollation => "alter_column_collation",
            Self::AddPrimaryKey => "add_primary_key",
            Self::DropPrimaryKey => "drop_primary_key",
            Self::AlterPrimaryKey => "alter_primary_key",
            Self::AddForeignKey => "add_foreign_key",
            Self::DropForeignKey => "drop_foreign_key",
            Self::AlterForeignKey => "alter_foreign_key",
            Self::AddUniqueConstraint => "add_unique_constraint",
            Self::DropUniqueConstraint => "drop_unique_constraint",
            Self::AlterUniqueConstraint => "alter_unique_constraint",
            Self::AddCheckConstraint => "add_check_constraint",
            Self::DropCheckConstraint => "drop_check_constraint",
            Self::AlterCheckConstraint => "alter_check_constraint",
            Self::AddDefaultConstraint => "add_default_constraint",
            Self::DropDefaultConstraint => "drop_default_constraint",
            Self::AlterDefaultConstraint => "alter_default_constraint",
            Self::CreateIndex => "create_index",
            Self::DropIndex => "drop_index",
            Self::AlterIndex => "alter_index",
            Self::CreateView => "create_view",
            Self::DropView => "drop_view",
            Self::AlterView => "alter_view",
            Self::CreateStoredProcedure => "create_stored_procedure",
            Self::DropStoredProcedure => "drop_stored_procedure",
            Self::AlterStoredProcedure => "alter_stored_procedure",
            Self::CreateFunction => "create_function",
            Self::DropFunction => "drop_function",
            Self::AlterFunction => "alter_function",
            Self::Unclassified => "unclassified",
        }
    }
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

impl SchemaDifference {
    pub fn operation_type(&self) -> DifferenceOperation {
        use DifferenceCategory::{
            DefinitionMismatch, Indeterminate, MissingInSource, MissingInTarget, PropertyMismatch,
            UnsupportedComparison,
        };
        use DifferenceOperation as Op;
        use SchemaObjectType as Obj;

        if matches!(self.category, Indeterminate | UnsupportedComparison)
            || self.confidence == ConfidenceStatus::Indeterminate
        {
            return Op::Unclassified;
        }

        match (self.object_type, self.category) {
            (Obj::Schema, MissingInTarget) => Op::CreateSchema,
            (Obj::Schema, MissingInSource) => Op::DropSchema,
            (Obj::Table, MissingInTarget) => Op::CreateTable,
            (Obj::Table, MissingInSource) => Op::DropTable,
            (Obj::Table, PropertyMismatch) => Op::AlterTable,
            (Obj::Column, MissingInTarget) => Op::AddColumn,
            (Obj::Column, MissingInSource) => Op::DropColumn,
            (Obj::Column, PropertyMismatch) => match self.property_name.as_deref() {
                Some("data_type") => Op::AlterColumnType,
                Some("max_length" | "precision_scale") => Op::AlterColumnCapacity,
                Some("is_nullable") => Op::AlterColumnNullability,
                Some("collation") => Op::AlterColumnCollation,
                _ => Op::Unclassified,
            },
            (Obj::PrimaryKey, MissingInTarget) => Op::AddPrimaryKey,
            (Obj::PrimaryKey, MissingInSource) => Op::DropPrimaryKey,
            (Obj::PrimaryKey, PropertyMismatch) => Op::AlterPrimaryKey,
            (Obj::ForeignKey, MissingInTarget) => Op::AddForeignKey,
            (Obj::ForeignKey, MissingInSource) => Op::DropForeignKey,
            (Obj::ForeignKey, PropertyMismatch) => Op::AlterForeignKey,
            (Obj::UniqueConstraint, MissingInTarget) => Op::AddUniqueConstraint,
            (Obj::UniqueConstraint, MissingInSource) => Op::DropUniqueConstraint,
            (Obj::UniqueConstraint, PropertyMismatch) => Op::AlterUniqueConstraint,
            (Obj::CheckConstraint, MissingInTarget) => Op::AddCheckConstraint,
            (Obj::CheckConstraint, MissingInSource) => Op::DropCheckConstraint,
            (Obj::CheckConstraint, PropertyMismatch) => Op::AlterCheckConstraint,
            (Obj::DefaultConstraint, MissingInTarget) => Op::AddDefaultConstraint,
            (Obj::DefaultConstraint, MissingInSource) => Op::DropDefaultConstraint,
            (Obj::DefaultConstraint, PropertyMismatch) => Op::AlterDefaultConstraint,
            (Obj::Index, MissingInTarget) => Op::CreateIndex,
            (Obj::Index, MissingInSource) => Op::DropIndex,
            (Obj::Index, PropertyMismatch) => Op::AlterIndex,
            (Obj::View, MissingInTarget) => Op::CreateView,
            (Obj::View, MissingInSource) => Op::DropView,
            (Obj::View, DefinitionMismatch) => Op::AlterView,
            (Obj::StoredProcedure, MissingInTarget) => Op::CreateStoredProcedure,
            (Obj::StoredProcedure, MissingInSource) => Op::DropStoredProcedure,
            (Obj::StoredProcedure, DefinitionMismatch) => Op::AlterStoredProcedure,
            (Obj::Function, MissingInTarget) => Op::CreateFunction,
            (Obj::Function, MissingInSource) => Op::DropFunction,
            (Obj::Function, DefinitionMismatch) => Op::AlterFunction,
            _ => Op::Unclassified,
        }
    }
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

#[cfg(test)]
mod operation_tests {
    use super::*;

    fn difference(
        object_type: SchemaObjectType,
        category: DifferenceCategory,
        property_name: Option<&str>,
    ) -> SchemaDifference {
        SchemaDifference {
            id: "diff-id".into(),
            object_type,
            schema_name: "alpha".into(),
            object_name: "Products.Price".into(),
            category,
            property_name: property_name.map(str::to_string),
            value_source: None,
            value_target: None,
            severity: DifferenceSeverity::Medium,
            confidence: ConfidenceStatus::High,
            explanation: String::new(),
            risk_assessment: None,
            can_generate_sql: true,
        }
    }

    #[test]
    fn classifies_supported_diff_engine_operations_from_structured_fields() {
        use DifferenceCategory::{
            DefinitionMismatch, MissingInSource, MissingInTarget, PropertyMismatch,
        };
        use DifferenceOperation as Op;
        use SchemaObjectType as Obj;

        let cases = [
            (Obj::Schema, MissingInTarget, None, Op::CreateSchema),
            (Obj::Schema, MissingInSource, None, Op::DropSchema),
            (Obj::Table, MissingInTarget, None, Op::CreateTable),
            (Obj::Table, MissingInSource, None, Op::DropTable),
            (Obj::Table, PropertyMismatch, None, Op::AlterTable),
            (Obj::Column, MissingInTarget, None, Op::AddColumn),
            (Obj::Column, MissingInSource, None, Op::DropColumn),
            (
                Obj::Column,
                PropertyMismatch,
                Some("data_type"),
                Op::AlterColumnType,
            ),
            (
                Obj::Column,
                PropertyMismatch,
                Some("max_length"),
                Op::AlterColumnCapacity,
            ),
            (
                Obj::Column,
                PropertyMismatch,
                Some("precision_scale"),
                Op::AlterColumnCapacity,
            ),
            (
                Obj::Column,
                PropertyMismatch,
                Some("is_nullable"),
                Op::AlterColumnNullability,
            ),
            (
                Obj::Column,
                PropertyMismatch,
                Some("collation"),
                Op::AlterColumnCollation,
            ),
            (Obj::PrimaryKey, MissingInTarget, None, Op::AddPrimaryKey),
            (Obj::PrimaryKey, MissingInSource, None, Op::DropPrimaryKey),
            (
                Obj::PrimaryKey,
                PropertyMismatch,
                Some("columns"),
                Op::AlterPrimaryKey,
            ),
            (Obj::ForeignKey, MissingInTarget, None, Op::AddForeignKey),
            (Obj::ForeignKey, MissingInSource, None, Op::DropForeignKey),
            (
                Obj::ForeignKey,
                PropertyMismatch,
                Some("definition"),
                Op::AlterForeignKey,
            ),
            (
                Obj::UniqueConstraint,
                MissingInTarget,
                None,
                Op::AddUniqueConstraint,
            ),
            (
                Obj::UniqueConstraint,
                MissingInSource,
                None,
                Op::DropUniqueConstraint,
            ),
            (
                Obj::UniqueConstraint,
                PropertyMismatch,
                Some("columns"),
                Op::AlterUniqueConstraint,
            ),
            (
                Obj::CheckConstraint,
                MissingInTarget,
                None,
                Op::AddCheckConstraint,
            ),
            (
                Obj::CheckConstraint,
                MissingInSource,
                None,
                Op::DropCheckConstraint,
            ),
            (
                Obj::CheckConstraint,
                PropertyMismatch,
                Some("definition"),
                Op::AlterCheckConstraint,
            ),
            (
                Obj::DefaultConstraint,
                MissingInTarget,
                None,
                Op::AddDefaultConstraint,
            ),
            (
                Obj::DefaultConstraint,
                MissingInSource,
                None,
                Op::DropDefaultConstraint,
            ),
            (
                Obj::DefaultConstraint,
                PropertyMismatch,
                Some("definition"),
                Op::AlterDefaultConstraint,
            ),
            (Obj::Index, MissingInTarget, None, Op::CreateIndex),
            (Obj::Index, MissingInSource, None, Op::DropIndex),
            (
                Obj::Index,
                PropertyMismatch,
                Some("definition"),
                Op::AlterIndex,
            ),
            (Obj::View, MissingInTarget, None, Op::CreateView),
            (Obj::View, MissingInSource, None, Op::DropView),
            (
                Obj::View,
                DefinitionMismatch,
                Some("definition"),
                Op::AlterView,
            ),
            (
                Obj::StoredProcedure,
                MissingInTarget,
                None,
                Op::CreateStoredProcedure,
            ),
            (
                Obj::StoredProcedure,
                MissingInSource,
                None,
                Op::DropStoredProcedure,
            ),
            (
                Obj::StoredProcedure,
                DefinitionMismatch,
                Some("definition"),
                Op::AlterStoredProcedure,
            ),
            (Obj::Function, MissingInTarget, None, Op::CreateFunction),
            (Obj::Function, MissingInSource, None, Op::DropFunction),
            (
                Obj::Function,
                DefinitionMismatch,
                Some("definition"),
                Op::AlterFunction,
            ),
        ];

        for (object_type, category, property_name, expected) in cases {
            assert_eq!(
                difference(object_type, category, property_name).operation_type(),
                expected,
                "classification for {object_type:?}/{category:?}/{property_name:?}"
            );
        }
    }

    #[test]
    fn unknown_properties_and_indeterminate_differences_remain_unclassified() {
        assert_eq!(
            difference(
                SchemaObjectType::Column,
                DifferenceCategory::PropertyMismatch,
                Some("future_property")
            )
            .operation_type(),
            DifferenceOperation::Unclassified
        );
        assert_eq!(
            difference(
                SchemaObjectType::Trigger,
                DifferenceCategory::MissingInTarget,
                None
            )
            .operation_type(),
            DifferenceOperation::Unclassified
        );
        assert_eq!(
            difference(
                SchemaObjectType::Table,
                DifferenceCategory::Indeterminate,
                None
            )
            .operation_type(),
            DifferenceOperation::Unclassified
        );
    }
}

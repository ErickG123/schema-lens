# SchemaLens — Modelo de Domínio (SDD)

## 1. Princípios de Modelagem
O domínio do SchemaLens é desacoplado de qualquer tecnologia de persistência, drivers de banco de dados ou bibliotecas de UI. Todos os objetos de catálogo são representados como estruturas canônicas e imutáveis.

## 2. Entidades Principais

### 2.1 Identificador Canônico (`ObjectId`)
Representa a identidade de um objeto no SQL Server:
```rust
pub struct ObjectId {
    pub schema: String,
    pub name: String,
}
```

### 2.2 Snapshot do Banco (`DatabaseSnapshot`)
```rust
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
```

### 2.3 Metadados de Tabela e Colunas
```rust
pub struct TableMetadata {
    pub id: ObjectId,
    pub columns: Vec<ColumnMetadata>,
    pub primary_key: Option<KeyConstraintMetadata>,
    pub foreign_keys: Vec<ForeignKeyMetadata>,
    pub check_constraints: Vec<CheckConstraintMetadata>,
    pub default_constraints: Vec<DefaultConstraintMetadata>,
    pub indexes: Vec<IndexMetadata>,
}

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
    pub collation: Option<String>,
}
```

### 2.4 Diferença de Esquema (`SchemaDifference`)
```rust
pub struct SchemaDifference {
    pub id: String, // UUID estável
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
```

use schema_lens_domain::*;
use std::collections::HashMap;
use uuid::Uuid;

pub struct DiffEngine;

impl DiffEngine {
    pub fn compare(source: &DatabaseSnapshot, target: &DatabaseSnapshot) -> ComparisonRunResult {
        let mut differences = Vec::new();

        // 1. Schemas
        Self::compare_schemas(&source.schemas, &target.schemas, &mut differences);

        // 2. Tables & Columns
        Self::compare_tables(&source.tables, &target.tables, &mut differences);

        // 3. Views
        Self::compare_modules(
            &source.views,
            &target.views,
            SchemaObjectType::View,
            &mut differences,
        );

        // 4. Stored Procedures
        Self::compare_modules(
            &source.procedures,
            &target.procedures,
            SchemaObjectType::StoredProcedure,
            &mut differences,
        );

        // 5. Functions
        Self::compare_modules(
            &source.functions,
            &target.functions,
            SchemaObjectType::Function,
            &mut differences,
        );

        // Ordenação canônica das diferenças para garantir determinismo estrito
        differences.sort_by(|a, b| {
            a.schema_name
                .cmp(&b.schema_name)
                .then(a.object_name.cmp(&b.object_name))
                .then(a.property_name.cmp(&b.property_name))
        });

        // Sumário de métricas
        let mut summary = ComparisonSummary {
            total_differences: differences.len(),
            missing_in_target: 0,
            missing_in_source: 0,
            property_mismatches: 0,
            definition_mismatches: 0,
            indeterminate: 0,
            critical_severity: 0,
            medium_severity: 0,
            low_severity: 0,
        };

        for d in &differences {
            match d.category {
                DifferenceCategory::MissingInTarget => summary.missing_in_target += 1,
                DifferenceCategory::MissingInSource => summary.missing_in_source += 1,
                DifferenceCategory::PropertyMismatch => summary.property_mismatches += 1,
                DifferenceCategory::DefinitionMismatch => summary.definition_mismatches += 1,
                DifferenceCategory::Indeterminate => summary.indeterminate += 1,
                _ => {}
            }

            match d.severity {
                DifferenceSeverity::Critical => summary.critical_severity += 1,
                DifferenceSeverity::Medium => summary.medium_severity += 1,
                DifferenceSeverity::Low | DifferenceSeverity::Info => summary.low_severity += 1,
            }
        }

        ComparisonRunResult {
            run_id: Uuid::new_v4().to_string(),
            source_database: source.database_name.clone(),
            target_database: target.database_name.clone(),
            summary,
            differences,
            generated_at_utc: chrono::Utc::now().to_rfc3339(),
        }
    }

    fn compare_schemas(
        src: &[SchemaMetadata],
        tgt: &[SchemaMetadata],
        diffs: &mut Vec<SchemaDifference>,
    ) {
        let tgt_map: HashMap<&str, &SchemaMetadata> =
            tgt.iter().map(|s| (s.name.as_str(), s)).collect();
        let src_map: HashMap<&str, &SchemaMetadata> =
            src.iter().map(|s| (s.name.as_str(), s)).collect();

        for s in src {
            if !tgt_map.contains_key(s.name.as_str()) {
                diffs.push(SchemaDifference {
                    id: Uuid::new_v4().to_string(),
                    object_type: SchemaObjectType::Schema,
                    schema_name: s.name.clone(),
                    object_name: s.name.clone(),
                    category: DifferenceCategory::MissingInTarget,
                    property_name: None,
                    value_source: Some(s.name.clone()),
                    value_target: None,
                    severity: DifferenceSeverity::Medium,
                    confidence: ConfidenceStatus::High,
                    explanation: format!(
                        "Schema '{}' exists in Source but is missing in Target",
                        s.name
                    ),
                    risk_assessment: None,
                    can_generate_sql: true,
                });
            }
        }

        for t in tgt {
            if !src_map.contains_key(t.name.as_str()) {
                diffs.push(SchemaDifference {
                    id: Uuid::new_v4().to_string(),
                    object_type: SchemaObjectType::Schema,
                    schema_name: t.name.clone(),
                    object_name: t.name.clone(),
                    category: DifferenceCategory::MissingInSource,
                    property_name: None,
                    value_source: None,
                    value_target: Some(t.name.clone()),
                    severity: DifferenceSeverity::Low,
                    confidence: ConfidenceStatus::High,
                    explanation: format!(
                        "Schema '{}' exists in Target but is absent in Source",
                        t.name
                    ),
                    risk_assessment: None,
                    can_generate_sql: false,
                });
            }
        }
    }

    fn compare_tables(
        src_tables: &[TableMetadata],
        tgt_tables: &[TableMetadata],
        diffs: &mut Vec<SchemaDifference>,
    ) {
        let mut tgt_map: HashMap<ObjectId, &TableMetadata> = HashMap::new();
        for t in tgt_tables {
            tgt_map.insert(t.id.clone(), t);
        }

        let mut src_map: HashMap<ObjectId, &TableMetadata> = HashMap::new();
        for s in src_tables {
            src_map.insert(s.id.clone(), s);
        }

        // Detectar ausentes em Target ou analisar colunas
        for s in src_tables {
            if let Some(t) = tgt_map.get(&s.id) {
                // Tabela existe em ambos: comparar colunas, PKs, FKs, Unique, Check, Default e Índices
                Self::compare_columns(s, t, diffs);
                Self::compare_primary_keys(s, t, diffs);
                Self::compare_foreign_keys(s, t, diffs);
                Self::compare_unique_constraints(s, t, diffs);
                Self::compare_check_constraints(s, t, diffs);
                Self::compare_default_constraints(s, t, diffs);
                Self::compare_indexes(s, t, diffs);
            } else {
                diffs.push(SchemaDifference {
                    id: Uuid::new_v4().to_string(),
                    object_type: SchemaObjectType::Table,
                    schema_name: s.id.schema.clone(),
                    object_name: s.id.name.clone(),
                    category: DifferenceCategory::MissingInTarget,
                    property_name: None,
                    value_source: Some(format!("Table with {} columns", s.columns.len())),
                    value_target: None,
                    severity: DifferenceSeverity::Medium,
                    confidence: ConfidenceStatus::High,
                    explanation: format!("Table '{}' is missing in Target", s.id.full_name()),
                    risk_assessment: Some("Creation required on target".to_string()),
                    can_generate_sql: true,
                });
            }
        }

        // Detectar ausentes em Source (exclusivas do Target)
        for t in tgt_tables {
            if !src_map.contains_key(&t.id) {
                diffs.push(SchemaDifference {
                    id: Uuid::new_v4().to_string(),
                    object_type: SchemaObjectType::Table,
                    schema_name: t.id.schema.clone(),
                    object_name: t.id.name.clone(),
                    category: DifferenceCategory::MissingInSource,
                    property_name: None,
                    value_source: None,
                    value_target: Some(format!("Table with {} columns", t.columns.len())),
                    severity: DifferenceSeverity::Critical,
                    confidence: ConfidenceStatus::High,
                    explanation: format!("Table '{}' exists only in Target", t.id.full_name()),
                    risk_assessment: Some(
                        "Dropping this table would destroy target data".to_string(),
                    ),
                    can_generate_sql: true,
                });
            }
        }
    }

    fn compare_columns(
        src_table: &TableMetadata,
        tgt_table: &TableMetadata,
        diffs: &mut Vec<SchemaDifference>,
    ) {
        let tgt_cols: HashMap<&str, &ColumnMetadata> = tgt_table
            .columns
            .iter()
            .map(|c| (c.name.as_str(), c))
            .collect();
        let src_cols: HashMap<&str, &ColumnMetadata> = src_table
            .columns
            .iter()
            .map(|c| (c.name.as_str(), c))
            .collect();

        for sc in &src_table.columns {
            if let Some(tc) = tgt_cols.get(sc.name.as_str()) {
                // Verificar tipo de dados
                if !sc.data_type.eq_ignore_ascii_case(&tc.data_type) {
                    diffs.push(SchemaDifference {
                        id: Uuid::new_v4().to_string(),
                        object_type: SchemaObjectType::Column,
                        schema_name: src_table.id.schema.clone(),
                        object_name: format!("{}.{}", src_table.id.name, sc.name),
                        category: DifferenceCategory::PropertyMismatch,
                        property_name: Some("data_type".to_string()),
                        value_source: Some(sc.data_type.clone()),
                        value_target: Some(tc.data_type.clone()),
                        severity: DifferenceSeverity::Critical,
                        confidence: ConfidenceStatus::High,
                        explanation: format!(
                            "Data type mismatch for {}.{}: Source is {}, Target is {}",
                            src_table.id.full_name(),
                            sc.name,
                            sc.data_type,
                            tc.data_type
                        ),
                        risk_assessment: Some(
                            "Type conversion may cause data corruption or failure".to_string(),
                        ),
                        can_generate_sql: true,
                    });
                }
                // Verificar tamanho
                else if sc.max_length != tc.max_length && sc.max_length > 0 && tc.max_length > 0 {
                    let is_truncation_risk = sc.max_length < tc.max_length;
                    diffs.push(SchemaDifference {
                        id: Uuid::new_v4().to_string(),
                        object_type: SchemaObjectType::Column,
                        schema_name: src_table.id.schema.clone(),
                        object_name: format!("{}.{}", src_table.id.name, sc.name),
                        category: DifferenceCategory::PropertyMismatch,
                        property_name: Some("max_length".to_string()),
                        value_source: Some(sc.max_length.to_string()),
                        value_target: Some(tc.max_length.to_string()),
                        severity: if is_truncation_risk {
                            DifferenceSeverity::Critical
                        } else {
                            DifferenceSeverity::Low
                        },
                        confidence: ConfidenceStatus::High,
                        explanation: format!(
                            "Length mismatch for {}.{}: Source is {}, Target is {}",
                            src_table.id.full_name(),
                            sc.name,
                            sc.max_length,
                            tc.max_length
                        ),
                        risk_assessment: if is_truncation_risk {
                            Some(
                                "Reducing column length can cause string truncation error"
                                    .to_string(),
                            )
                        } else {
                            None
                        },
                        can_generate_sql: true,
                    });
                }

                if sc.precision != tc.precision || sc.scale != tc.scale {
                    diffs.push(SchemaDifference {
                        id: Uuid::new_v4().to_string(),
                        object_type: SchemaObjectType::Column,
                        schema_name: src_table.id.schema.clone(),
                        object_name: format!("{}.{}", src_table.id.name, sc.name),
                        category: DifferenceCategory::PropertyMismatch,
                        property_name: Some("precision_scale".to_string()),
                        value_source: Some(format!("precision={}, scale={}", sc.precision, sc.scale)),
                        value_target: Some(format!("precision={}, scale={}", tc.precision, tc.scale)),
                        severity: if sc.precision < tc.precision || sc.scale < tc.scale {
                            DifferenceSeverity::Critical
                        } else {
                            DifferenceSeverity::Medium
                        },
                        confidence: ConfidenceStatus::High,
                        explanation: format!("Numeric precision/scale mismatch for {}.{}", src_table.id.full_name(), sc.name),
                        risk_assessment: Some("Values outside the destination range may fail conversion or lose precision; stored values were not inspected".to_string()),
                        can_generate_sql: true,
                    });
                }

                // Verificar nullability
                if sc.is_nullable != tc.is_nullable {
                    diffs.push(SchemaDifference {
                        id: Uuid::new_v4().to_string(),
                        object_type: SchemaObjectType::Column,
                        schema_name: src_table.id.schema.clone(),
                        object_name: format!("{}.{}", src_table.id.name, sc.name),
                        category: DifferenceCategory::PropertyMismatch,
                        property_name: Some("is_nullable".to_string()),
                        value_source: Some(sc.is_nullable.to_string()),
                        value_target: Some(tc.is_nullable.to_string()),
                        severity: if !sc.is_nullable { DifferenceSeverity::Medium } else { DifferenceSeverity::Low },
                        confidence: ConfidenceStatus::High,
                        explanation: format!(
                            "Nullability mismatch for {}.{}: Source is nullable={}, Target is nullable={}",
                            src_table.id.full_name(), sc.name, sc.is_nullable, tc.is_nullable
                        ),
                        risk_assessment: if !sc.is_nullable { Some("Making column NOT NULL will fail if existing target rows contain NULL".to_string()) } else { None },
                        can_generate_sql: true,
                    });
                }

                // Verificar collation
                if sc.collation != tc.collation && sc.collation.is_some() && tc.collation.is_some()
                {
                    diffs.push(SchemaDifference {
                        id: Uuid::new_v4().to_string(),
                        object_type: SchemaObjectType::Column,
                        schema_name: src_table.id.schema.clone(),
                        object_name: format!("{}.{}", src_table.id.name, sc.name),
                        category: DifferenceCategory::PropertyMismatch,
                        property_name: Some("collation".to_string()),
                        value_source: sc.collation.clone(),
                        value_target: tc.collation.clone(),
                        severity: DifferenceSeverity::Medium,
                        confidence: ConfidenceStatus::High,
                        explanation: format!(
                            "Collation mismatch for {}.{}: Source is {:?}, Target is {:?}",
                            src_table.id.full_name(),
                            sc.name,
                            sc.collation,
                            tc.collation
                        ),
                        risk_assessment: Some(
                            "Collation conflict can trigger error 468 during queries and joins"
                                .to_string(),
                        ),
                        can_generate_sql: true,
                    });
                }
            } else {
                let generated_value = if sc.is_identity {
                    " IDENTITY"
                } else if sc.is_computed {
                    " COMPUTED"
                } else {
                    ""
                };
                let default_value = src_table
                    .default_constraints
                    .iter()
                    .find(|constraint| constraint.column_name == sc.name)
                    .map(|constraint| format!(" DEFAULT {}", constraint.definition))
                    .unwrap_or_default();
                diffs.push(SchemaDifference {
                    id: Uuid::new_v4().to_string(),
                    object_type: SchemaObjectType::Column,
                    schema_name: src_table.id.schema.clone(),
                    object_name: format!("{}.{}", src_table.id.name, sc.name),
                    category: DifferenceCategory::MissingInTarget,
                    property_name: None,
                    value_source: Some(format!(
                        "{} {}{}{}",
                        sc.data_type,
                        if sc.is_nullable { "NULL" } else { "NOT NULL" },
                        generated_value,
                        default_value
                    )),
                    value_target: None,
                    severity: DifferenceSeverity::Medium,
                    confidence: ConfidenceStatus::High,
                    explanation: format!(
                        "Column '{}' exists in Source table '{}' but is missing in Target",
                        sc.name,
                        src_table.id.full_name()
                    ),
                    risk_assessment: None,
                    can_generate_sql: true,
                });
            }
        }

        for tc in &tgt_table.columns {
            if !src_cols.contains_key(tc.name.as_str()) {
                diffs.push(SchemaDifference {
                    id: Uuid::new_v4().to_string(),
                    object_type: SchemaObjectType::Column,
                    schema_name: src_table.id.schema.clone(),
                    object_name: format!("{}.{}", src_table.id.name, tc.name),
                    category: DifferenceCategory::MissingInSource,
                    property_name: None,
                    value_source: None,
                    value_target: Some(format!(
                        "{} {}",
                        tc.data_type,
                        if tc.is_nullable { "NULL" } else { "NOT NULL" }
                    )),
                    severity: DifferenceSeverity::Critical,
                    confidence: ConfidenceStatus::High,
                    explanation: format!(
                        "Column '{}' in Target table '{}' does not exist in Source",
                        tc.name,
                        src_table.id.full_name()
                    ),
                    risk_assessment: Some(
                        "Column removal will cause data loss in target".to_string(),
                    ),
                    can_generate_sql: true,
                });
            }
        }
    }

    fn compare_primary_keys(
        src_table: &TableMetadata,
        tgt_table: &TableMetadata,
        diffs: &mut Vec<SchemaDifference>,
    ) {
        match (&src_table.primary_key, &tgt_table.primary_key) {
            (Some(spk), Some(tpk)) => {
                if spk.columns != tpk.columns || spk.is_clustered != tpk.is_clustered {
                    diffs.push(SchemaDifference {
                        id: Uuid::new_v4().to_string(),
                        object_type: SchemaObjectType::PrimaryKey,
                        schema_name: src_table.id.schema.clone(),
                        object_name: format!("{}.PK", src_table.id.name),
                        category: DifferenceCategory::PropertyMismatch,
                        property_name: Some("columns".to_string()),
                        value_source: Some(format!(
                            "{:?} (clustered={})",
                            spk.columns, spk.is_clustered
                        )),
                        value_target: Some(format!(
                            "{:?} (clustered={})",
                            tpk.columns, tpk.is_clustered
                        )),
                        severity: DifferenceSeverity::Critical,
                        confidence: ConfidenceStatus::High,
                        explanation: format!(
                            "Primary Key definition mismatch on table '{}'",
                            src_table.id.full_name()
                        ),
                        risk_assessment: Some(
                            "Altering PK requires dropping dependent foreign keys".to_string(),
                        ),
                        can_generate_sql: true,
                    });
                }
            }
            (Some(spk), None) => {
                diffs.push(SchemaDifference {
                    id: Uuid::new_v4().to_string(),
                    object_type: SchemaObjectType::PrimaryKey,
                    schema_name: src_table.id.schema.clone(),
                    object_name: format!("{}.{}", src_table.id.name, spk.name),
                    category: DifferenceCategory::MissingInTarget,
                    property_name: None,
                    value_source: Some(format!("{:?}", spk.columns)),
                    value_target: None,
                    severity: DifferenceSeverity::Medium,
                    confidence: ConfidenceStatus::High,
                    explanation: format!(
                        "Primary Key '{}' is missing on target table '{}'",
                        spk.name,
                        src_table.id.full_name()
                    ),
                    risk_assessment: None,
                    can_generate_sql: true,
                });
            }
            (None, Some(tpk)) => {
                diffs.push(SchemaDifference {
                    id: Uuid::new_v4().to_string(),
                    object_type: SchemaObjectType::PrimaryKey,
                    schema_name: src_table.id.schema.clone(),
                    object_name: format!("{}.{}", src_table.id.name, tpk.name),
                    category: DifferenceCategory::MissingInSource,
                    property_name: None,
                    value_source: None,
                    value_target: Some(format!("{:?}", tpk.columns)),
                    severity: DifferenceSeverity::Critical,
                    confidence: ConfidenceStatus::High,
                    explanation: format!(
                        "Primary Key '{}' exists only on target table '{}'",
                        tpk.name,
                        src_table.id.full_name()
                    ),
                    risk_assessment: Some(
                        "Dropping PK will remove unique identity constraint".to_string(),
                    ),
                    can_generate_sql: true,
                });
            }
            (None, None) => {}
        }
    }

    fn compare_foreign_keys(
        src_table: &TableMetadata,
        tgt_table: &TableMetadata,
        diffs: &mut Vec<SchemaDifference>,
    ) {
        let tgt_fks: HashMap<&str, &ForeignKeyMetadata> = tgt_table
            .foreign_keys
            .iter()
            .map(|f| (f.name.as_str(), f))
            .collect();
        let src_fks: HashMap<&str, &ForeignKeyMetadata> = src_table
            .foreign_keys
            .iter()
            .map(|f| (f.name.as_str(), f))
            .collect();

        for sfk in &src_table.foreign_keys {
            if let Some(tfk) = tgt_fks.get(sfk.name.as_str()) {
                if sfk.referenced_table != tfk.referenced_table
                    || sfk.referenced_schema != tfk.referenced_schema
                    || sfk.column_mappings != tfk.column_mappings
                    || sfk.delete_rule != tfk.delete_rule
                    || sfk.update_rule != tfk.update_rule
                    || sfk.is_disabled != tfk.is_disabled
                {
                    diffs.push(SchemaDifference {
                        id: Uuid::new_v4().to_string(),
                        object_type: SchemaObjectType::ForeignKey,
                        schema_name: src_table.id.schema.clone(),
                        object_name: format!("{}.{}", src_table.id.name, sfk.name),
                        category: DifferenceCategory::PropertyMismatch,
                        property_name: Some("definition".to_string()),
                        value_source: Some(format!(
                            "Ref: {}.{} Del: {} Update: {} Disabled: {}",
                            sfk.referenced_schema,
                            sfk.referenced_table,
                            sfk.delete_rule,
                            sfk.update_rule,
                            sfk.is_disabled
                        )),
                        value_target: Some(format!(
                            "Ref: {}.{} Del: {} Update: {} Disabled: {}",
                            tfk.referenced_schema,
                            tfk.referenced_table,
                            tfk.delete_rule,
                            tfk.update_rule,
                            tfk.is_disabled
                        )),
                        severity: DifferenceSeverity::Critical,
                        confidence: ConfidenceStatus::High,
                        explanation: format!(
                            "Foreign key '{}' definition divergence on table '{}'",
                            sfk.name,
                            src_table.id.full_name()
                        ),
                        risk_assessment: Some("Modifying FK may reject existing data".to_string()),
                        can_generate_sql: true,
                    });
                }
            } else {
                diffs.push(SchemaDifference {
                    id: Uuid::new_v4().to_string(),
                    object_type: SchemaObjectType::ForeignKey,
                    schema_name: src_table.id.schema.clone(),
                    object_name: format!("{}.{}", src_table.id.name, sfk.name),
                    category: DifferenceCategory::MissingInTarget,
                    property_name: None,
                    value_source: Some(format!(
                        "Ref: {}.{}",
                        sfk.referenced_schema, sfk.referenced_table
                    )),
                    value_target: None,
                    severity: DifferenceSeverity::Medium,
                    confidence: ConfidenceStatus::High,
                    explanation: format!(
                        "Foreign key '{}' missing on target table '{}'",
                        sfk.name,
                        src_table.id.full_name()
                    ),
                    risk_assessment: None,
                    can_generate_sql: true,
                });
            }
        }

        for tfk in &tgt_table.foreign_keys {
            if !src_fks.contains_key(tfk.name.as_str()) {
                diffs.push(SchemaDifference {
                    id: Uuid::new_v4().to_string(),
                    object_type: SchemaObjectType::ForeignKey,
                    schema_name: src_table.id.schema.clone(),
                    object_name: format!("{}.{}", src_table.id.name, tfk.name),
                    category: DifferenceCategory::MissingInSource,
                    property_name: None,
                    value_source: None,
                    value_target: Some(format!(
                        "Ref: {}.{}",
                        tfk.referenced_schema, tfk.referenced_table
                    )),
                    severity: DifferenceSeverity::Low,
                    confidence: ConfidenceStatus::High,
                    explanation: format!(
                        "Foreign key '{}' exists only on target table '{}'",
                        tfk.name,
                        src_table.id.full_name()
                    ),
                    risk_assessment: None,
                    can_generate_sql: true,
                });
            }
        }
    }

    fn compare_unique_constraints(
        src_table: &TableMetadata,
        tgt_table: &TableMetadata,
        diffs: &mut Vec<SchemaDifference>,
    ) {
        let tgt_uqs: HashMap<&str, &KeyConstraintMetadata> = tgt_table
            .unique_constraints
            .iter()
            .map(|u| (u.name.as_str(), u))
            .collect();
        let src_uqs: HashMap<&str, &KeyConstraintMetadata> = src_table
            .unique_constraints
            .iter()
            .map(|u| (u.name.as_str(), u))
            .collect();

        for su in &src_table.unique_constraints {
            if let Some(tu) = tgt_uqs.get(su.name.as_str()) {
                if su.columns != tu.columns || su.is_clustered != tu.is_clustered {
                    diffs.push(SchemaDifference {
                        id: Uuid::new_v4().to_string(),
                        object_type: SchemaObjectType::UniqueConstraint,
                        schema_name: src_table.id.schema.clone(),
                        object_name: format!("{}.{}", src_table.id.name, su.name),
                        category: DifferenceCategory::PropertyMismatch,
                        property_name: Some("columns".to_string()),
                        value_source: Some(format!("{:?}", su.columns)),
                        value_target: Some(format!("{:?}", tu.columns)),
                        severity: DifferenceSeverity::Medium,
                        confidence: ConfidenceStatus::High,
                        explanation: format!("Unique constraint '{}' columns mismatch", su.name),
                        risk_assessment: None,
                        can_generate_sql: true,
                    });
                }
            } else {
                diffs.push(SchemaDifference {
                    id: Uuid::new_v4().to_string(),
                    object_type: SchemaObjectType::UniqueConstraint,
                    schema_name: src_table.id.schema.clone(),
                    object_name: format!("{}.{}", src_table.id.name, su.name),
                    category: DifferenceCategory::MissingInTarget,
                    property_name: None,
                    value_source: Some(format!("{:?}", su.columns)),
                    value_target: None,
                    severity: DifferenceSeverity::Medium,
                    confidence: ConfidenceStatus::High,
                    explanation: format!("Unique constraint '{}' missing in target", su.name),
                    risk_assessment: None,
                    can_generate_sql: true,
                });
            }
        }

        for tu in &tgt_table.unique_constraints {
            if !src_uqs.contains_key(tu.name.as_str()) {
                diffs.push(SchemaDifference {
                    id: Uuid::new_v4().to_string(),
                    object_type: SchemaObjectType::UniqueConstraint,
                    schema_name: src_table.id.schema.clone(),
                    object_name: format!("{}.{}", src_table.id.name, tu.name),
                    category: DifferenceCategory::MissingInSource,
                    property_name: None,
                    value_source: None,
                    value_target: Some(format!("{:?}", tu.columns)),
                    severity: DifferenceSeverity::Low,
                    confidence: ConfidenceStatus::High,
                    explanation: format!("Unique constraint '{}' exists only in target", tu.name),
                    risk_assessment: None,
                    can_generate_sql: true,
                });
            }
        }
    }

    fn compare_check_constraints(
        src_table: &TableMetadata,
        tgt_table: &TableMetadata,
        diffs: &mut Vec<SchemaDifference>,
    ) {
        let tgt_cks: HashMap<&str, &CheckConstraintMetadata> = tgt_table
            .check_constraints
            .iter()
            .map(|c| (c.name.as_str(), c))
            .collect();
        let src_cks: HashMap<&str, &CheckConstraintMetadata> = src_table
            .check_constraints
            .iter()
            .map(|c| (c.name.as_str(), c))
            .collect();

        for sc in &src_table.check_constraints {
            if let Some(tc) = tgt_cks.get(sc.name.as_str()) {
                if sc.definition != tc.definition || sc.is_disabled != tc.is_disabled {
                    diffs.push(SchemaDifference {
                        id: Uuid::new_v4().to_string(),
                        object_type: SchemaObjectType::CheckConstraint,
                        schema_name: src_table.id.schema.clone(),
                        object_name: format!("{}.{}", src_table.id.name, sc.name),
                        category: DifferenceCategory::PropertyMismatch,
                        property_name: Some("definition".to_string()),
                        value_source: Some(format!(
                            "{}; disabled={}",
                            sc.definition, sc.is_disabled
                        )),
                        value_target: Some(format!(
                            "{}; disabled={}",
                            tc.definition, tc.is_disabled
                        )),
                        severity: DifferenceSeverity::Medium,
                        confidence: ConfidenceStatus::High,
                        explanation: format!(
                            "Check constraint expression mismatch for '{}'",
                            sc.name
                        ),
                        risk_assessment: None,
                        can_generate_sql: true,
                    });
                }
            } else {
                diffs.push(SchemaDifference {
                    id: Uuid::new_v4().to_string(),
                    object_type: SchemaObjectType::CheckConstraint,
                    schema_name: src_table.id.schema.clone(),
                    object_name: format!("{}.{}", src_table.id.name, sc.name),
                    category: DifferenceCategory::MissingInTarget,
                    property_name: None,
                    value_source: Some(sc.definition.clone()),
                    value_target: None,
                    severity: DifferenceSeverity::Medium,
                    confidence: ConfidenceStatus::High,
                    explanation: format!("Check constraint '{}' missing in target", sc.name),
                    risk_assessment: None,
                    can_generate_sql: true,
                });
            }
        }

        for tc in &tgt_table.check_constraints {
            if !src_cks.contains_key(tc.name.as_str()) {
                diffs.push(SchemaDifference {
                    id: Uuid::new_v4().to_string(),
                    object_type: SchemaObjectType::CheckConstraint,
                    schema_name: src_table.id.schema.clone(),
                    object_name: format!("{}.{}", src_table.id.name, tc.name),
                    category: DifferenceCategory::MissingInSource,
                    property_name: None,
                    value_source: None,
                    value_target: Some(tc.definition.clone()),
                    severity: DifferenceSeverity::Low,
                    confidence: ConfidenceStatus::High,
                    explanation: format!("Check constraint '{}' exists only in target", tc.name),
                    risk_assessment: None,
                    can_generate_sql: true,
                });
            }
        }
    }

    fn compare_default_constraints(
        src_table: &TableMetadata,
        tgt_table: &TableMetadata,
        diffs: &mut Vec<SchemaDifference>,
    ) {
        let tgt_dfs: HashMap<&str, &DefaultConstraintMetadata> = tgt_table
            .default_constraints
            .iter()
            .map(|d| (d.column_name.as_str(), d))
            .collect();
        let src_dfs: HashMap<&str, &DefaultConstraintMetadata> = src_table
            .default_constraints
            .iter()
            .map(|d| (d.column_name.as_str(), d))
            .collect();

        for sd in &src_table.default_constraints {
            if let Some(td) = tgt_dfs.get(sd.column_name.as_str()) {
                if sd.definition.trim() != td.definition.trim() {
                    diffs.push(SchemaDifference {
                        id: Uuid::new_v4().to_string(),
                        object_type: SchemaObjectType::DefaultConstraint,
                        schema_name: src_table.id.schema.clone(),
                        object_name: format!("{}.{}.DEFAULT", src_table.id.name, sd.column_name),
                        category: DifferenceCategory::PropertyMismatch,
                        property_name: Some("definition".to_string()),
                        value_source: Some(sd.definition.clone()),
                        value_target: Some(td.definition.clone()),
                        severity: DifferenceSeverity::Low,
                        confidence: ConfidenceStatus::High,
                        explanation: format!(
                            "Default value mismatch for column '{}.{}'",
                            src_table.id.name, sd.column_name
                        ),
                        risk_assessment: None,
                        can_generate_sql: true,
                    });
                }
            } else {
                diffs.push(SchemaDifference {
                    id: Uuid::new_v4().to_string(),
                    object_type: SchemaObjectType::DefaultConstraint,
                    schema_name: src_table.id.schema.clone(),
                    object_name: format!("{}.{}.DEFAULT", src_table.id.name, sd.column_name),
                    category: DifferenceCategory::MissingInTarget,
                    property_name: None,
                    value_source: Some(sd.definition.clone()),
                    value_target: None,
                    severity: DifferenceSeverity::Low,
                    confidence: ConfidenceStatus::High,
                    explanation: format!(
                        "Default constraint missing on target column '{}.{}'",
                        src_table.id.name, sd.column_name
                    ),
                    risk_assessment: None,
                    can_generate_sql: true,
                });
            }
        }

        for td in &tgt_table.default_constraints {
            if !src_dfs.contains_key(td.column_name.as_str()) {
                diffs.push(SchemaDifference {
                    id: Uuid::new_v4().to_string(),
                    object_type: SchemaObjectType::DefaultConstraint,
                    schema_name: src_table.id.schema.clone(),
                    object_name: format!("{}.{}.DEFAULT", src_table.id.name, td.column_name),
                    category: DifferenceCategory::MissingInSource,
                    property_name: None,
                    value_source: None,
                    value_target: Some(td.definition.clone()),
                    severity: DifferenceSeverity::Low,
                    confidence: ConfidenceStatus::High,
                    explanation: format!(
                        "Default constraint exists only on target column '{}.{}'",
                        src_table.id.name, td.column_name
                    ),
                    risk_assessment: None,
                    can_generate_sql: true,
                });
            }
        }
    }

    fn compare_indexes(
        src_table: &TableMetadata,
        tgt_table: &TableMetadata,
        diffs: &mut Vec<SchemaDifference>,
    ) {
        let tgt_idxs: HashMap<&str, &IndexMetadata> = tgt_table
            .indexes
            .iter()
            .map(|i| (i.name.as_str(), i))
            .collect();
        let src_idxs: HashMap<&str, &IndexMetadata> = src_table
            .indexes
            .iter()
            .map(|i| (i.name.as_str(), i))
            .collect();

        for si in &src_table.indexes {
            if let Some(ti) = tgt_idxs.get(si.name.as_str()) {
                if si.key_columns != ti.key_columns
                    || si.included_columns != ti.included_columns
                    || si.is_unique != ti.is_unique
                    || si.is_clustered != ti.is_clustered
                    || si.filter_definition != ti.filter_definition
                {
                    diffs.push(SchemaDifference {
                        id: Uuid::new_v4().to_string(),
                        object_type: SchemaObjectType::Index,
                        schema_name: src_table.id.schema.clone(),
                        object_name: format!("{}.{}", src_table.id.name, si.name),
                        category: DifferenceCategory::PropertyMismatch,
                        property_name: Some("definition".to_string()),
                        value_source: Some(format!(
                            "Keys: {:?} Inc: {:?} Unique: {} Filter: {:?}",
                            si.key_columns, si.included_columns, si.is_unique, si.filter_definition
                        )),
                        value_target: Some(format!(
                            "Keys: {:?} Inc: {:?} Unique: {} Filter: {:?}",
                            ti.key_columns, ti.included_columns, ti.is_unique, ti.filter_definition
                        )),
                        severity: DifferenceSeverity::Medium,
                        confidence: ConfidenceStatus::High,
                        explanation: format!(
                            "Index definition mismatch for '{}' on table '{}'",
                            si.name,
                            src_table.id.full_name()
                        ),
                        risk_assessment: Some(
                            "Index recreation may require lock on large tables".to_string(),
                        ),
                        can_generate_sql: true,
                    });
                }
            } else {
                diffs.push(SchemaDifference {
                    id: Uuid::new_v4().to_string(),
                    object_type: SchemaObjectType::Index,
                    schema_name: src_table.id.schema.clone(),
                    object_name: format!("{}.{}", src_table.id.name, si.name),
                    category: DifferenceCategory::MissingInTarget,
                    property_name: None,
                    value_source: Some(format!(
                        "Keys: {:?} Inc: {:?}",
                        si.key_columns, si.included_columns
                    )),
                    value_target: None,
                    severity: DifferenceSeverity::Low,
                    confidence: ConfidenceStatus::High,
                    explanation: format!(
                        "Index '{}' missing on target table '{}'",
                        si.name,
                        src_table.id.full_name()
                    ),
                    risk_assessment: None,
                    can_generate_sql: true,
                });
            }
        }

        for ti in &tgt_table.indexes {
            if !src_idxs.contains_key(ti.name.as_str()) {
                diffs.push(SchemaDifference {
                    id: Uuid::new_v4().to_string(),
                    object_type: SchemaObjectType::Index,
                    schema_name: src_table.id.schema.clone(),
                    object_name: format!("{}.{}", src_table.id.name, ti.name),
                    category: DifferenceCategory::MissingInSource,
                    property_name: None,
                    value_source: None,
                    value_target: Some(format!(
                        "Keys: {:?} Inc: {:?}",
                        ti.key_columns, ti.included_columns
                    )),
                    severity: DifferenceSeverity::Low,
                    confidence: ConfidenceStatus::High,
                    explanation: format!(
                        "Index '{}' exists only on target table '{}'",
                        ti.name,
                        src_table.id.full_name()
                    ),
                    risk_assessment: None,
                    can_generate_sql: true,
                });
            }
        }
    }

    fn compare_modules(
        src_mods: &[ModuleMetadata],
        tgt_mods: &[ModuleMetadata],
        obj_type: SchemaObjectType,
        diffs: &mut Vec<SchemaDifference>,
    ) {
        let tgt_map: HashMap<ObjectId, &ModuleMetadata> =
            tgt_mods.iter().map(|m| (m.id.clone(), m)).collect();
        let src_map: HashMap<ObjectId, &ModuleMetadata> =
            src_mods.iter().map(|m| (m.id.clone(), m)).collect();

        for sm in src_mods {
            if let Some(tm) = tgt_map.get(&sm.id) {
                if sm.is_encrypted || tm.is_encrypted {
                    diffs.push(SchemaDifference {
                        id: Uuid::new_v4().to_string(),
                        object_type: obj_type,
                        schema_name: sm.id.schema.clone(),
                        object_name: sm.id.name.clone(),
                        category: DifferenceCategory::Indeterminate,
                        property_name: Some("definition".to_string()),
                        value_source: if sm.is_encrypted {
                            Some("[ENCRYPTED]".to_string())
                        } else {
                            Some("[SOURCE_PLAIN]".to_string())
                        },
                        value_target: if tm.is_encrypted {
                            Some("[ENCRYPTED]".to_string())
                        } else {
                            Some("[TARGET_PLAIN]".to_string())
                        },
                        severity: DifferenceSeverity::Medium,
                        confidence: ConfidenceStatus::Indeterminate,
                        explanation: format!(
                            "Module '{}' is encrypted (WITH ENCRYPTION); source code inaccessible",
                            sm.id.full_name()
                        ),
                        risk_assessment: Some(
                            "Cannot inspect encrypted module definition".to_string(),
                        ),
                        can_generate_sql: false,
                    });
                } else {
                    let s_norm = Self::normalize_sql_text(sm.definition.as_deref().unwrap_or(""));
                    let t_norm = Self::normalize_sql_text(tm.definition.as_deref().unwrap_or(""));
                    if s_norm != t_norm {
                        diffs.push(SchemaDifference {
                            id: Uuid::new_v4().to_string(),
                            object_type: obj_type,
                            schema_name: sm.id.schema.clone(),
                            object_name: sm.id.name.clone(),
                            category: DifferenceCategory::DefinitionMismatch,
                            property_name: Some("definition".to_string()),
                            value_source: sm.definition.clone(),
                            value_target: tm.definition.clone(),
                            severity: DifferenceSeverity::Medium,
                            confidence: ConfidenceStatus::High,
                            explanation: format!(
                                "DDL definition mismatch for {}",
                                sm.id.full_name()
                            ),
                            risk_assessment: None,
                            can_generate_sql: true,
                        });
                    }
                }
            } else {
                diffs.push(SchemaDifference {
                    id: Uuid::new_v4().to_string(),
                    object_type: obj_type,
                    schema_name: sm.id.schema.clone(),
                    object_name: sm.id.name.clone(),
                    category: DifferenceCategory::MissingInTarget,
                    property_name: None,
                    value_source: sm.definition.clone(),
                    value_target: None,
                    severity: DifferenceSeverity::Medium,
                    confidence: ConfidenceStatus::High,
                    explanation: format!(
                        "{:?} '{}' is missing in Target",
                        obj_type,
                        sm.id.full_name()
                    ),
                    risk_assessment: None,
                    can_generate_sql: true,
                });
            }
        }

        for tm in tgt_mods {
            if !src_map.contains_key(&tm.id) {
                diffs.push(SchemaDifference {
                    id: Uuid::new_v4().to_string(),
                    object_type: obj_type,
                    schema_name: tm.id.schema.clone(),
                    object_name: tm.id.name.clone(),
                    category: DifferenceCategory::MissingInSource,
                    property_name: None,
                    value_source: None,
                    value_target: tm.definition.clone(),
                    severity: DifferenceSeverity::Low,
                    confidence: ConfidenceStatus::High,
                    explanation: format!(
                        "{:?} '{}' exists only in Target",
                        obj_type,
                        tm.id.full_name()
                    ),
                    risk_assessment: None,
                    can_generate_sql: true,
                });
            }
        }
    }

    fn normalize_sql_text(sql: &str) -> String {
        // Normalização conservadora: remove quebras de linha e múltiplos espaços em branco redundantes
        let words: Vec<&str> = sql.split_whitespace().collect();
        words.join(" ").to_lowercase()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diff_engine_missing_and_divergence() {
        let src = DatabaseSnapshot {
            server_name: "A".to_string(),
            database_name: "SourceDB".to_string(),
            sql_server_version: "2022".to_string(),
            collation: "Latin1_General_CI_AS".to_string(),
            compatibility_level: 160,
            captured_at_utc: "".to_string(),
            schemas: vec![SchemaMetadata {
                name: "sales".to_string(),
                owner: "dbo".to_string(),
            }],
            tables: vec![TableMetadata {
                id: ObjectId::new("dbo", "Orders"),
                columns: vec![
                    ColumnMetadata {
                        name: "OrderId".to_string(),
                        ordinal_position: 1,
                        data_type: "int".to_string(),
                        max_length: 4,
                        precision: 10,
                        scale: 0,
                        is_nullable: false,
                        is_identity: true,
                        identity_seed: None,
                        identity_increment: None,
                        is_computed: false,
                        computed_definition: None,
                        collation: None,
                    },
                    ColumnMetadata {
                        name: "Notes".to_string(),
                        ordinal_position: 2,
                        data_type: "nvarchar".to_string(),
                        max_length: 500,
                        precision: 0,
                        scale: 0,
                        is_nullable: true,
                        is_identity: false,
                        identity_seed: None,
                        identity_increment: None,
                        is_computed: false,
                        computed_definition: None,
                        collation: None,
                    },
                ],
                primary_key: None,
                foreign_keys: vec![],
                check_constraints: vec![],
                default_constraints: vec![],
                unique_constraints: vec![],
                indexes: vec![],
            }],
            views: vec![],
            procedures: vec![],
            functions: vec![],
            triggers: vec![],
            sequences: vec![],
            synonyms: vec![],
            custom_types: vec![],
            collection_warnings: vec![],
        };

        let tgt = DatabaseSnapshot {
            server_name: "B".to_string(),
            database_name: "TargetDB".to_string(),
            sql_server_version: "2022".to_string(),
            collation: "Latin1_General_CI_AS".to_string(),
            compatibility_level: 160,
            captured_at_utc: "".to_string(),
            schemas: vec![], // sales ausente
            tables: vec![TableMetadata {
                id: ObjectId::new("dbo", "Orders"),
                columns: vec![ColumnMetadata {
                    name: "OrderId".to_string(),
                    ordinal_position: 1,
                    data_type: "int".to_string(),
                    max_length: 4,
                    precision: 10,
                    scale: 0,
                    is_nullable: false,
                    is_identity: true,
                    identity_seed: None,
                    identity_increment: None,
                    is_computed: false,
                    computed_definition: None,
                    collation: None,
                }], // Notes ausente
                primary_key: None,
                foreign_keys: vec![],
                check_constraints: vec![],
                default_constraints: vec![],
                unique_constraints: vec![],
                indexes: vec![],
            }],
            views: vec![],
            procedures: vec![],
            functions: vec![],
            triggers: vec![],
            sequences: vec![],
            synonyms: vec![],
            custom_types: vec![],
            collection_warnings: vec![],
        };

        let result = DiffEngine::compare(&src, &tgt);
        assert_eq!(result.summary.total_differences, 2);
        assert_eq!(result.summary.missing_in_target, 2); // schema sales e coluna Notes
    }
}

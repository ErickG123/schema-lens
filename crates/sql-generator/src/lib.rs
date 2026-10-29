use schema_lens_domain::*;
use uuid::Uuid;

pub struct SqlGenerator;

pub struct GeneratedOperation {
    pub sql: String,
    pub is_destructive: bool,
}

impl SqlGenerator {
    pub fn generate_for_difference(
        difference: &SchemaDifference,
        target_database: &str,
        source_snapshot: &DatabaseSnapshot,
        target_snapshot: &DatabaseSnapshot,
    ) -> Result<GeneratedOperation, String> {
        let plan = Self::plan_and_generate_with_snapshots(
            std::slice::from_ref(difference),
            target_database,
            Some(source_snapshot),
            Some(target_snapshot),
        );
        if !plan.unsupported_operations.is_empty() {
            return Err(plan.unsupported_operations.join("; "));
        }
        if plan.steps.is_empty() {
            return Err("No SQL operation is available for this difference.".to_string());
        }
        Ok(GeneratedOperation {
            sql: format!(
                "USE {};\nGO\n{}",
                Self::escape_identifier(target_database),
                plan.steps
                    .iter()
                    .map(|step| step.sql.as_str())
                    .collect::<Vec<_>>()
                    .join("\n")
            ),
            is_destructive: plan.steps.iter().any(|step| step.is_destructive),
        })
    }

    pub fn plan_and_generate(
        differences: &[SchemaDifference],
        target_database: &str,
    ) -> MigrationPlan {
        Self::plan_and_generate_with_snapshots(differences, target_database, None, None)
    }

    pub fn plan_and_generate_with_snapshots(
        differences: &[SchemaDifference],
        target_database: &str,
        source_snapshot: Option<&DatabaseSnapshot>,
        _target_snapshot: Option<&DatabaseSnapshot>,
    ) -> MigrationPlan {
        let mut steps = Vec::new();
        let mut destructive_count = 0;
        let mut unsupported_operations = Vec::new();

        for diff in differences {
            // Regra Inegociável: Se a confiança for Indeterminate ou can_generate_sql for falso, NUNCA gere DDL
            if !diff.can_generate_sql || diff.confidence == ConfidenceStatus::Indeterminate {
                unsupported_operations.push(format!(
                    "{}: SQL generation is disabled by the difference confidence/policy",
                    diff.object_name
                ));
                continue;
            }

            let schema_escaped = Self::escape_identifier(&diff.schema_name);

            match diff.object_type {
                SchemaObjectType::Schema => {
                    if diff.category == DifferenceCategory::MissingInTarget {
                        let schema_literal = schema_escaped.replace('\'', "''");
                        steps.push(MigrationStep {
                            order: 10,
                            description: format!("Criar schema defensivo {}", schema_escaped),
                            sql: format!(
                                "IF NOT EXISTS (SELECT * FROM sys.schemas WHERE name = N'{}')\nBEGIN\n    EXEC('CREATE SCHEMA {}');\nEND\nGO\n",
                                diff.schema_name.replace('\'', "''"),
                                schema_literal
                            ),
                            is_destructive: false,
                        });
                    }
                }
                SchemaObjectType::Table => {
                    let table_escaped = Self::escape_identifier(&diff.object_name);
                    if diff.category == DifferenceCategory::MissingInTarget {
                        let table = source_snapshot.and_then(|snapshot| {
                            snapshot.tables.iter().find(|table| {
                                table.id.schema == diff.schema_name
                                    && table.id.name == diff.object_name
                            })
                        });
                        match table.map(Self::create_table_sql).transpose() {
                            Ok(Some(create_sql)) => {
                                steps.push(MigrationStep {
                                    order: 20,
                                    description: format!("Criar tabela {}.{}", schema_escaped, table_escaped),
                                    sql: create_sql,
                                    is_destructive: false,
                                });
                                if let Some(table) = table {
                                    let qualified = format!("{}.{}", schema_escaped, table_escaped);
                                    for foreign_key in &table.foreign_keys {
                                        match Self::foreign_key_sql(&qualified, foreign_key) {
                                            Ok(sql) => steps.push(MigrationStep {
                                                order: 100,
                                                description: format!("Criar FOREIGN KEY {} em {}", Self::escape_identifier(&foreign_key.name), qualified),
                                                sql,
                                                is_destructive: false,
                                            }),
                                            Err(reason) => unsupported_operations.push(format!("{}.{}: {reason}", diff.schema_name, diff.object_name)),
                                        }
                                    }
                                }
                            }
                            Ok(None) => unsupported_operations.push(format!(
                                "{}.{}: source snapshot is missing or does not contain a complete supported table definition",
                                diff.schema_name, diff.object_name
                            )),
                            Err(reason) => unsupported_operations.push(format!(
                                "{}.{}: {reason}", diff.schema_name, diff.object_name
                            )),
                        }
                    } else if diff.category == DifferenceCategory::MissingInSource {
                        destructive_count += 1;
                        steps.push(MigrationStep {
                            order: 900,
                            description: format!("Remover tabela legada {}.{} do destino", schema_escaped, table_escaped),
                            sql: format!(
                                "-- WARNING: DESTRUCTIVE OPERATION - REVIEW DEPENDENCIES AND DATA LOSS\nIF OBJECT_ID(N'{}', 'U') IS NOT NULL\nBEGIN\n    DROP TABLE {}.{};\nEND\nGO\n",
                                format!("{}.{}", diff.schema_name, diff.object_name).replace('\'', "''"),
                                schema_escaped, table_escaped
                            ),
                            is_destructive: true,
                        });
                    }
                }
                SchemaObjectType::Column => {
                    let col_name_raw = diff
                        .object_name
                        .split('.')
                        .next_back()
                        .unwrap_or(&diff.object_name);
                    let tbl_name_raw = diff
                        .object_name
                        .split('.')
                        .next()
                        .unwrap_or(&diff.object_name);

                    let col_escaped = Self::escape_identifier(col_name_raw);
                    let tbl_escaped = Self::escape_identifier(tbl_name_raw);

                    if diff.category == DifferenceCategory::MissingInTarget {
                        let source_column = source_snapshot.and_then(|snapshot| {
                            Self::find_column(
                                snapshot,
                                &diff.schema_name,
                                tbl_name_raw,
                                col_name_raw,
                            )
                        });
                        match source_column {
                            Some((source_table, column)) => {
                                match Self::column_definition(source_table, column) {
                                    Ok(column_ddl) => {
                                        let precondition = if !column.is_nullable
                                            && !column.is_identity
                                            && !source_table.default_constraints.iter().any(
                                                |constraint| constraint.column_name == column.name,
                                            ) {
                                            "-- PRECONDITION: Existing target rows require an approved value before this NOT NULL column can be added.\n"
                                        } else {
                                            ""
                                        };
                                        steps.push(MigrationStep {
                                        order: 30,
                                        description: format!("Adicionar coluna {} em {}.{}", col_escaped, schema_escaped, tbl_escaped),
                                        sql: format!(
                                            "{precondition}IF COL_LENGTH(N'{}.{}', N'{}') IS NULL\nBEGIN\n    ALTER TABLE {}.{} ADD {};\nEND\nGO\n",
                                            schema_escaped, tbl_escaped, col_name_raw.replace('\'', "''"), schema_escaped, tbl_escaped, column_ddl
                                        ),
                                        is_destructive: false,
                                    });
                                    }
                                    Err(reason) => unsupported_operations
                                        .push(format!("{}: {reason}", diff.object_name)),
                                }
                            }
                            None => unsupported_operations.push(format!(
                                "{}: source snapshot column metadata is unavailable",
                                diff.object_name
                            )),
                        }
                    } else if diff.category == DifferenceCategory::MissingInSource {
                        destructive_count += 1;
                        steps.push(MigrationStep {
                            order: 850,
                            description: format!("Excluir coluna {} em {}.{}", col_escaped, schema_escaped, tbl_escaped),
                            sql: format!(
                                "-- WARNING: DESTRUCTIVE OPERATION - REVIEW DATA LOSS AND DEPENDENCIES\nALTER TABLE {}.{} DROP COLUMN {};\nGO\n",
                                schema_escaped, tbl_escaped, col_escaped
                            ),
                            is_destructive: true,
                        });
                    } else if diff.category == DifferenceCategory::PropertyMismatch {
                        let is_destructive = diff.severity == DifferenceSeverity::Critical;
                        if is_destructive {
                            destructive_count += 1;
                        }
                        let source_column = source_snapshot.and_then(|snapshot| {
                            Self::find_column(
                                snapshot,
                                &diff.schema_name,
                                tbl_name_raw,
                                col_name_raw,
                            )
                        });
                        match source_column {
                            Some((_source_table, column)) if !column.is_identity && !column.is_computed => {
                                match Self::column_type(column).and_then(|sql_type| {
                                    Self::collation_clause(column).map(|collation| format!("{sql_type}{collation}"))
                                }) {
                                    Ok(new_type) => {
                                        let sql = format!(
                                            "{}\nALTER TABLE {}.{} ALTER COLUMN {} {} {};\nGO\n",
                                            if is_destructive { "-- WARNING: POTENTIAL TRUNCATION / CONVERSION DATA LOSS" } else { "" },
                                            schema_escaped, tbl_escaped, col_escaped, new_type,
                                            if column.is_nullable { "NULL" } else { "NOT NULL" }
                                        );
                                        if !steps.iter().any(|step: &MigrationStep| step.sql == sql) {
                                            steps.push(MigrationStep {
                                                order: 40,
                                                description: format!("Alterar tipo/tamanho/nullability de {} em {}.{}", col_escaped, schema_escaped, tbl_escaped),
                                                sql,
                                                is_destructive,
                                            });
                                        }
                                    }
                                    Err(reason) => unsupported_operations.push(format!("{}: {reason}", diff.object_name)),
                                }
                            }
                            Some(_) => unsupported_operations.push(format!(
                                "{}: identity/computed columns cannot be modified by a generic ALTER COLUMN",
                                diff.object_name
                            )),
                            None => unsupported_operations.push(format!(
                                "{}: source snapshot column metadata is unavailable",
                                diff.object_name
                            )),
                        }
                    }
                }
                SchemaObjectType::PrimaryKey => {
                    let tbl_name_raw = diff
                        .object_name
                        .split('.')
                        .next()
                        .unwrap_or(&diff.object_name);
                    let tbl_escaped = Self::escape_identifier(tbl_name_raw);
                    if diff.category == DifferenceCategory::MissingInTarget {
                        let key = source_snapshot
                            .and_then(|snapshot| {
                                Self::find_table(snapshot, &diff.schema_name, tbl_name_raw)
                            })
                            .and_then(|table| table.primary_key.as_ref());
                        if let Some(key) = key {
                            steps.push(MigrationStep {
                                order: 50,
                                description: format!("Adicionar Primary Key {} em {}.{}", Self::escape_identifier(&key.name), schema_escaped, tbl_escaped),
                                sql: format!(
                                    "ALTER TABLE {}.{} ADD CONSTRAINT {} PRIMARY KEY {} ({});\nGO\n",
                                    schema_escaped, tbl_escaped, Self::escape_identifier(&key.name),
                                    if key.is_clustered { "CLUSTERED" } else { "NONCLUSTERED" },
                                    Self::column_list(&key.columns)
                                ),
                                is_destructive: false,
                            });
                        } else {
                            unsupported_operations.push(format!(
                                "{}: source primary key metadata is unavailable",
                                diff.object_name
                            ));
                        }
                    } else if diff.category == DifferenceCategory::MissingInSource {
                        let key = _target_snapshot
                            .and_then(|snapshot| {
                                Self::find_table(snapshot, &diff.schema_name, tbl_name_raw)
                            })
                            .and_then(|table| table.primary_key.as_ref());
                        if let Some(key) = key {
                            destructive_count += 1;
                            steps.push(MigrationStep {
                                order: 800,
                                description: format!("Remover Primary Key {} de {}.{}", Self::escape_identifier(&key.name), schema_escaped, tbl_escaped),
                                sql: format!("-- WARNING: DESTRUCTIVE STRUCTURAL CHANGE; REVIEW FOREIGN KEY DEPENDENCIES\nALTER TABLE {}.{} DROP CONSTRAINT {};\nGO\n", schema_escaped, tbl_escaped, Self::escape_identifier(&key.name)),
                                is_destructive: true,
                            });
                        } else {
                            unsupported_operations.push(format!(
                                "{}: target primary key metadata is unavailable",
                                diff.object_name
                            ));
                        }
                    } else {
                        unsupported_operations.push(format!(
                            "{}: primary key replacement requires dependency-aware planning",
                            diff.object_name
                        ));
                    }
                }
                SchemaObjectType::UniqueConstraint => {
                    let tbl_name_raw = diff
                        .object_name
                        .split('.')
                        .next()
                        .unwrap_or(&diff.object_name);
                    let tbl_escaped = Self::escape_identifier(tbl_name_raw);
                    if diff.category == DifferenceCategory::MissingInTarget {
                        let constraint = source_snapshot
                            .and_then(|snapshot| {
                                Self::find_table(snapshot, &diff.schema_name, tbl_name_raw)
                            })
                            .and_then(|table| {
                                table.unique_constraints.iter().find(|constraint| {
                                    constraint.name
                                        == diff
                                            .object_name
                                            .split('.')
                                            .next_back()
                                            .unwrap_or_default()
                                })
                            });
                        if let Some(constraint) = constraint {
                            steps.push(MigrationStep {
                                order: 55,
                                description: format!(
                                    "Adicionar Unique Constraint {} em {}.{}",
                                    Self::escape_identifier(&constraint.name),
                                    schema_escaped,
                                    tbl_escaped
                                ),
                                sql: format!(
                                    "ALTER TABLE {}.{} ADD CONSTRAINT {} UNIQUE {} ({});\nGO\n",
                                    schema_escaped,
                                    tbl_escaped,
                                    Self::escape_identifier(&constraint.name),
                                    if constraint.is_clustered {
                                        "CLUSTERED"
                                    } else {
                                        "NONCLUSTERED"
                                    },
                                    Self::column_list(&constraint.columns)
                                ),
                                is_destructive: false,
                            });
                        } else {
                            unsupported_operations.push(format!(
                                "{}: source unique constraint metadata is unavailable",
                                diff.object_name
                            ));
                        }
                    } else if diff.category == DifferenceCategory::MissingInSource {
                        let constraint = _target_snapshot
                            .and_then(|snapshot| {
                                Self::find_table(snapshot, &diff.schema_name, tbl_name_raw)
                            })
                            .and_then(|table| {
                                table.unique_constraints.iter().find(|constraint| {
                                    constraint.name
                                        == diff
                                            .object_name
                                            .split('.')
                                            .next_back()
                                            .unwrap_or_default()
                                })
                            });
                        if let Some(constraint) = constraint {
                            destructive_count += 1;
                            steps.push(MigrationStep {
                                order: 810,
                                description: format!("Remover Unique Constraint {}", Self::escape_identifier(&constraint.name)),
                                sql: format!("-- WARNING: REVIEW UNIQUE DEPENDENCIES\nALTER TABLE {}.{} DROP CONSTRAINT {};\nGO\n", schema_escaped, tbl_escaped, Self::escape_identifier(&constraint.name)),
                                is_destructive: true,
                            });
                        } else {
                            unsupported_operations.push(format!(
                                "{}: target unique constraint metadata is unavailable",
                                diff.object_name
                            ));
                        }
                    } else {
                        unsupported_operations.push(format!(
                            "{}: unique constraint replacement is not supported",
                            diff.object_name
                        ));
                    }
                }
                SchemaObjectType::DefaultConstraint => {
                    let mut parts = diff.object_name.split('.');
                    let tbl_name_raw = parts.next().unwrap_or_default();
                    let col_name_raw = parts.next().unwrap_or_default();
                    let col_escaped = Self::escape_identifier(col_name_raw);
                    let tbl_escaped = Self::escape_identifier(tbl_name_raw);
                    let source_default = source_snapshot
                        .and_then(|snapshot| {
                            Self::find_table(snapshot, &diff.schema_name, tbl_name_raw)
                        })
                        .and_then(|table| {
                            table
                                .default_constraints
                                .iter()
                                .find(|constraint| constraint.column_name == col_name_raw)
                        });
                    let target_default = _target_snapshot
                        .and_then(|snapshot| {
                            Self::find_table(snapshot, &diff.schema_name, tbl_name_raw)
                        })
                        .and_then(|table| {
                            table
                                .default_constraints
                                .iter()
                                .find(|constraint| constraint.column_name == col_name_raw)
                        });
                    match (diff.category, source_default, target_default) {
                        (DifferenceCategory::MissingInTarget, Some(source), _) => {
                            steps.push(MigrationStep {
                                order: 60,
                                description: format!(
                                    "Adicionar DEFAULT em {}.{}.{}",
                                    schema_escaped, tbl_escaped, col_escaped
                                ),
                                sql: Self::add_default_sql(&schema_escaped, &tbl_escaped, source),
                                is_destructive: false,
                            })
                        }
                        (DifferenceCategory::PropertyMismatch, Some(source), Some(target)) => steps
                            .push(MigrationStep {
                                order: 60,
                                description: format!(
                                    "Alterar DEFAULT em {}.{}.{}",
                                    schema_escaped, tbl_escaped, col_escaped
                                ),
                                sql: format!(
                                    "ALTER TABLE {}.{} DROP CONSTRAINT {};\nGO\n{}",
                                    schema_escaped,
                                    tbl_escaped,
                                    Self::escape_identifier(&target.name),
                                    Self::add_default_sql(&schema_escaped, &tbl_escaped, source)
                                ),
                                is_destructive: false,
                            }),
                        (DifferenceCategory::MissingInSource, _, Some(target)) => {
                            steps.push(MigrationStep {
                                order: 820,
                                description: format!(
                                    "Remover DEFAULT de {}.{}.{}",
                                    schema_escaped, tbl_escaped, col_escaped
                                ),
                                sql: format!(
                                    "ALTER TABLE {}.{} DROP CONSTRAINT {};\nGO\n",
                                    schema_escaped,
                                    tbl_escaped,
                                    Self::escape_identifier(&target.name)
                                ),
                                is_destructive: false,
                            });
                        }
                        _ => unsupported_operations.push(format!(
                            "{}: default constraint snapshots are unavailable or inconsistent",
                            diff.object_name
                        )),
                    }
                }
                SchemaObjectType::CheckConstraint => {
                    let (table_name, constraint_name) = Self::split_table_object(&diff.object_name);
                    let source = source_snapshot
                        .and_then(|snapshot| {
                            Self::find_table(snapshot, &diff.schema_name, table_name)
                        })
                        .and_then(|table| {
                            table
                                .check_constraints
                                .iter()
                                .find(|constraint| constraint.name == constraint_name)
                        });
                    let target = _target_snapshot
                        .and_then(|snapshot| {
                            Self::find_table(snapshot, &diff.schema_name, table_name)
                        })
                        .and_then(|table| {
                            table
                                .check_constraints
                                .iter()
                                .find(|constraint| constraint.name == constraint_name)
                        });
                    let table =
                        format!("{}.{}", schema_escaped, Self::escape_identifier(table_name));
                    match (diff.category, source, target) {
                        (DifferenceCategory::MissingInTarget, Some(check), _) => {
                            steps.push(MigrationStep {
                                order: 65,
                                description: format!(
                                    "Adicionar CHECK {} em {}",
                                    Self::escape_identifier(check.name.as_str()),
                                    table
                                ),
                                sql: format!(
                                    "ALTER TABLE {table} {} ADD CONSTRAINT {} CHECK {};\nGO\n{}",
                                    if check.is_disabled {
                                        "WITH NOCHECK"
                                    } else {
                                        "WITH CHECK"
                                    },
                                    Self::escape_identifier(&check.name),
                                    check.definition,
                                    if check.is_disabled {
                                        format!(
                                            "ALTER TABLE {table} NOCHECK CONSTRAINT {};\nGO\n",
                                            Self::escape_identifier(&check.name)
                                        )
                                    } else {
                                        String::new()
                                    }
                                ),
                                is_destructive: false,
                            })
                        }
                        (DifferenceCategory::MissingInSource, _, Some(check)) => {
                            destructive_count += 1;
                            steps.push(MigrationStep {
                                order: 830,
                                description: format!("Remover CHECK {} de {}", Self::escape_identifier(&check.name), table),
                                sql: format!("-- WARNING: REVIEW INTEGRITY REQUIREMENTS\nALTER TABLE {table} DROP CONSTRAINT {};\nGO\n", Self::escape_identifier(&check.name)),
                                is_destructive: true,
                            });
                        }
                        _ => unsupported_operations.push(format!(
                            "{}: CHECK replacement requires a reviewed constraint transition",
                            diff.object_name
                        )),
                    }
                }
                SchemaObjectType::ForeignKey => {
                    let (table_name, constraint_name) = Self::split_table_object(&diff.object_name);
                    let source = source_snapshot
                        .and_then(|snapshot| {
                            Self::find_table(snapshot, &diff.schema_name, table_name)
                        })
                        .and_then(|table| {
                            table
                                .foreign_keys
                                .iter()
                                .find(|constraint| constraint.name == constraint_name)
                        });
                    let target = _target_snapshot
                        .and_then(|snapshot| {
                            Self::find_table(snapshot, &diff.schema_name, table_name)
                        })
                        .and_then(|table| {
                            table
                                .foreign_keys
                                .iter()
                                .find(|constraint| constraint.name == constraint_name)
                        });
                    let table =
                        format!("{}.{}", schema_escaped, Self::escape_identifier(table_name));
                    match (diff.category, source, target) {
                        (DifferenceCategory::MissingInTarget, Some(fk), _) => {
                            match Self::foreign_key_sql(&table, fk) {
                                Ok(sql) => steps.push(MigrationStep {
                                    order: 100,
                                    description: format!(
                                        "Adicionar FOREIGN KEY {} em {}",
                                        Self::escape_identifier(&fk.name),
                                        table
                                    ),
                                    sql,
                                    is_destructive: false,
                                }),
                                Err(reason) => unsupported_operations
                                    .push(format!("{}: {reason}", diff.object_name)),
                            }
                        }
                        (DifferenceCategory::MissingInSource, _, Some(fk)) => {
                            destructive_count += 1;
                            steps.push(MigrationStep {
                                order: 840,
                                description: format!("Remover FOREIGN KEY {} de {}", Self::escape_identifier(&fk.name), table),
                                sql: format!("-- WARNING: REVIEW REFERENTIAL INTEGRITY\nALTER TABLE {table} DROP CONSTRAINT {};\nGO\n", Self::escape_identifier(&fk.name)),
                                is_destructive: true,
                            });
                        }
                        _ => unsupported_operations.push(format!(
                            "{}: foreign key replacement is not supported as one operation",
                            diff.object_name
                        )),
                    }
                }
                SchemaObjectType::Index => {
                    let (tbl_name_raw, idx_name_raw) = Self::split_table_object(&diff.object_name);
                    let tbl_escaped = Self::escape_identifier(tbl_name_raw);
                    let table = format!("{}.{}", schema_escaped, tbl_escaped);
                    let source = source_snapshot
                        .and_then(|snapshot| {
                            Self::find_table(snapshot, &diff.schema_name, tbl_name_raw)
                        })
                        .and_then(|table| {
                            table
                                .indexes
                                .iter()
                                .find(|index| index.name == idx_name_raw)
                        });
                    let target = _target_snapshot
                        .and_then(|snapshot| {
                            Self::find_table(snapshot, &diff.schema_name, tbl_name_raw)
                        })
                        .and_then(|table| {
                            table
                                .indexes
                                .iter()
                                .find(|index| index.name == idx_name_raw)
                        });
                    let target_table = _target_snapshot.and_then(|snapshot| {
                        Self::find_table(snapshot, &diff.schema_name, tbl_name_raw)
                    });
                    match (diff.category, source, target) {
                        (DifferenceCategory::MissingInTarget, Some(index), _) => {
                            match Self::create_index_sql(&table, index) {
                                Ok(sql) => steps.push(MigrationStep {
                                    order: 90,
                                    description: format!(
                                        "Criar índice {} em {}",
                                        Self::escape_identifier(&index.name),
                                        table
                                    ),
                                    sql,
                                    is_destructive: false,
                                }),
                                Err(reason) => unsupported_operations
                                    .push(format!("{}: {reason}", diff.object_name)),
                            }
                        }
                        (DifferenceCategory::MissingInSource, _, Some(index)) => {
                            destructive_count += 1;
                            steps.push(MigrationStep { order: 860, description: format!("Remover índice {} de {}", Self::escape_identifier(&index.name), table), sql: format!("-- WARNING: REVIEW PERFORMANCE AND UNIQUENESS IMPACT\nDROP INDEX {} ON {};\nGO\n", Self::escape_identifier(&index.name), table), is_destructive: true });
                        }
                        (
                            DifferenceCategory::PropertyMismatch,
                            Some(source_index),
                            Some(target_index),
                        ) => {
                            let has_other_target_clustered_key = target_index.is_clustered
                                && target_table.is_some_and(|table| {
                                    table.indexes.iter().any(|index| {
                                        index.name != target_index.name && index.is_clustered
                                    }) || table
                                        .primary_key
                                        .as_ref()
                                        .is_some_and(|key| key.is_clustered)
                                        || table
                                            .unique_constraints
                                            .iter()
                                            .any(|key| key.is_clustered)
                                });
                            if has_other_target_clustered_key {
                                unsupported_operations.push(format!(
                                    "{}: clustered index replacement conflicts with another target clustered key",
                                    diff.object_name
                                ));
                                continue;
                            }
                            match Self::create_index_sql(&table, source_index) {
                                Ok(create_sql) => {
                                    destructive_count += 1;
                                    steps.push(MigrationStep {
                                        order: 90,
                                        description: format!(
                                            "Replace index {} on {}",
                                            Self::escape_identifier(&source_index.name),
                                            table
                                        ),
                                        sql: format!(
                                            "-- WARNING: REVIEW INDEX AVAILABILITY, UNIQUENESS, AND LOCK IMPACT\nDROP INDEX {} ON {};\nGO\n{}",
                                            Self::escape_identifier(&target_index.name),
                                            table,
                                            create_sql
                                        ),
                                        is_destructive: true,
                                    });
                                }
                                Err(reason) => unsupported_operations
                                    .push(format!("{}: {reason}", diff.object_name)),
                            }
                        }
                        _ => unsupported_operations.push(format!(
                            "{}: index transition is not supported by the available snapshots",
                            diff.object_name
                        )),
                    }
                }
                SchemaObjectType::View
                | SchemaObjectType::StoredProcedure
                | SchemaObjectType::Function => {
                    if let Some(def) = &diff.value_source {
                        steps.push(MigrationStep {
                            order: 100,
                            description: format!("Sincronizar módulo [{}]", diff.object_name),
                            sql: format!("-- DDL da Origem:\n{}\nGO\n", def),
                            is_destructive: false,
                        });
                    } else {
                        unsupported_operations.push(format!(
                            "{}: source module definition is unavailable",
                            diff.object_name
                        ));
                    }
                }
                _ => unsupported_operations.push(format!(
                    "{}: object type {:?} is not supported by the SQL generator",
                    diff.object_name, diff.object_type
                )),
            }
        }

        // Ordenar operações canonicamente por precedência de dependência
        steps.sort_by_key(|s| s.order);

        let target_db_escaped = Self::escape_identifier(target_database);

        let mut full_script = String::new();
        full_script.push_str(&format!(
            "-- =====================================================================\n\
             -- SchemaLens Generated Migration Script\n\
             -- Target Database: {}\n\
             -- Total Operations: {}\n\
             -- Destructive Operations: {}\n\
             -- ATENÇÃO: Revise manualmente antes de executar em produção!\n\
             -- =====================================================================\n\n\
             USE {};\nGO\n\n",
            target_db_escaped,
            steps.len(),
            destructive_count,
            target_db_escaped
        ));

        for step in &steps {
            full_script.push_str(&format!("-- Passo {}: {}\n", step.order, step.description));
            full_script.push_str(&step.sql);
            full_script.push('\n');
        }

        MigrationPlan {
            run_id: Uuid::new_v4().to_string(),
            target_database: target_database.to_string(),
            generated_at_utc: chrono::Utc::now().to_rfc3339(),
            total_steps: steps.len(),
            destructive_steps: destructive_count,
            unsupported_operations,
            script_sql: full_script,
            steps,
        }
    }

    fn create_table_sql(table: &TableMetadata) -> Result<String, String> {
        if table.columns.is_empty() {
            return Err("source table has no captured columns".to_string());
        }

        let qualified = format!(
            "{}.{}",
            Self::escape_identifier(&table.id.schema),
            Self::escape_identifier(&table.id.name)
        );
        let mut definitions = Vec::new();

        for column in &table.columns {
            definitions.push(Self::column_definition(table, column)?);
        }

        if let Some(primary_key) = &table.primary_key {
            definitions.push(format!(
                "CONSTRAINT {} PRIMARY KEY {} ({})",
                Self::escape_identifier(&primary_key.name),
                if primary_key.is_clustered {
                    "CLUSTERED"
                } else {
                    "NONCLUSTERED"
                },
                Self::column_list(&primary_key.columns)
            ));
        }

        if table
            .check_constraints
            .iter()
            .any(|check| check.is_disabled)
        {
            return Err(
                "disabled CHECK constraints are not supported in a CREATE TABLE script".to_string(),
            );
        }

        for unique in &table.unique_constraints {
            definitions.push(format!(
                "CONSTRAINT {} UNIQUE {} ({})",
                Self::escape_identifier(&unique.name),
                if unique.is_clustered {
                    "CLUSTERED"
                } else {
                    "NONCLUSTERED"
                },
                Self::column_list(&unique.columns)
            ));
        }

        for check in &table.check_constraints {
            definitions.push(format!(
                "CONSTRAINT {} CHECK {}",
                Self::escape_identifier(&check.name),
                check.definition
            ));
        }

        let body = definitions
            .iter()
            .map(|definition| format!("    {definition}"))
            .collect::<Vec<_>>()
            .join(",\n");
        let object_literal = format!("{}.{}", table.id.schema, table.id.name).replace('\'', "''");
        let mut sql = format!(
            "IF OBJECT_ID(N'{object_literal}', 'U') IS NULL\nBEGIN\n    CREATE TABLE {qualified} (\n{body}\n    );\nEND\nGO\n"
        );

        for index in &table.indexes {
            if index.key_columns.is_empty()
                || index.key_columns_descending.len() != index.key_columns.len()
            {
                return Err(format!(
                    "index '{}' is missing ordered key metadata",
                    index.name
                ));
            }
            if index.is_clustered
                && (table
                    .primary_key
                    .as_ref()
                    .is_some_and(|key| key.is_clustered)
                    || table.unique_constraints.iter().any(|key| key.is_clustered))
            {
                return Err(format!(
                    "index '{}' conflicts with another clustered key in the captured definition",
                    index.name
                ));
            }
            let keys = index
                .key_columns
                .iter()
                .zip(&index.key_columns_descending)
                .map(|(column, descending)| {
                    format!(
                        "{} {}",
                        Self::escape_identifier(column),
                        if *descending { "DESC" } else { "ASC" }
                    )
                })
                .collect::<Vec<_>>()
                .join(", ");
            let include = if index.included_columns.is_empty() {
                String::new()
            } else {
                format!(" INCLUDE ({})", Self::column_list(&index.included_columns))
            };
            let filter = index
                .filter_definition
                .as_deref()
                .map(|definition| format!(" WHERE {definition}"))
                .unwrap_or_default();
            let index_name_literal = index.name.replace('\'', "''");
            let table_name_literal =
                format!("{}.{}", table.id.schema, table.id.name).replace('\'', "''");
            sql.push_str(&format!(
                "IF NOT EXISTS (SELECT 1 FROM sys.indexes WHERE name = N'{index_name_literal}' AND object_id = OBJECT_ID(N'{table_name_literal}'))\nBEGIN\n    CREATE {}{} INDEX {} ON {} ({keys}){include}{filter};\nEND\nGO\n",
                if index.is_unique { "UNIQUE " } else { "" },
                if index.is_clustered { "CLUSTERED" } else { "NONCLUSTERED" },
                Self::escape_identifier(&index.name),
                qualified
            ));
        }

        Ok(sql)
    }

    fn column_definition(table: &TableMetadata, column: &ColumnMetadata) -> Result<String, String> {
        if column.is_computed {
            let definition = column
                .computed_definition
                .as_deref()
                .filter(|definition| !definition.trim().is_empty())
                .ok_or_else(|| {
                    format!(
                        "computed column '{}' cannot be reproduced because its expression is not captured",
                        column.name
                    )
                })?;
            return Ok(format!(
                "{} AS {}{}",
                Self::escape_identifier(&column.name),
                definition,
                if column.computed_persisted {
                    " PERSISTED"
                } else {
                    ""
                }
            ));
        }

        let mut definition = format!(
            "{} {}",
            Self::escape_identifier(&column.name),
            Self::column_type(column)?
        );
        if column.is_identity {
            let seed = column.identity_seed.ok_or_else(|| {
                format!("identity seed is not captured for column '{}'", column.name)
            })?;
            let increment = column.identity_increment.ok_or_else(|| {
                format!(
                    "identity increment is not captured for column '{}'",
                    column.name
                )
            })?;
            definition.push_str(&format!(" IDENTITY({seed},{increment})"));
        }
        definition.push_str(&Self::collation_clause(column)?);
        definition.push_str(if column.is_nullable {
            " NULL"
        } else {
            " NOT NULL"
        });
        if let Some(default) = table
            .default_constraints
            .iter()
            .find(|constraint| constraint.column_name == column.name)
        {
            definition.push_str(&format!(
                " CONSTRAINT {} DEFAULT {}",
                Self::escape_identifier(&default.name),
                default.definition
            ));
        }
        Ok(definition)
    }

    fn find_table<'a>(
        snapshot: &'a DatabaseSnapshot,
        schema: &str,
        table_name: &str,
    ) -> Option<&'a TableMetadata> {
        snapshot
            .tables
            .iter()
            .find(|table| table.id.schema == schema && table.id.name == table_name)
    }

    fn collation_clause(column: &ColumnMetadata) -> Result<String, String> {
        match &column.collation {
            Some(collation)
                if collation
                    .chars()
                    .all(|character| character.is_ascii_alphanumeric() || character == '_') =>
            {
                Ok(format!(" COLLATE {collation}"))
            }
            Some(_) => Err(format!(
                "unsupported collation name for column '{}'",
                column.name
            )),
            None => Ok(String::new()),
        }
    }

    fn find_column<'a>(
        snapshot: &'a DatabaseSnapshot,
        schema: &str,
        table_name: &str,
        column_name: &str,
    ) -> Option<(&'a TableMetadata, &'a ColumnMetadata)> {
        let table = Self::find_table(snapshot, schema, table_name)?;
        let column = table
            .columns
            .iter()
            .find(|column| column.name == column_name)?;
        Some((table, column))
    }

    fn split_table_object(value: &str) -> (&str, &str) {
        match value.split_once('.') {
            Some((table, object)) => (table, object),
            None => (value, ""),
        }
    }

    fn add_default_sql(
        schema: &str,
        table: &str,
        constraint: &DefaultConstraintMetadata,
    ) -> String {
        format!(
            "ALTER TABLE {}.{} ADD CONSTRAINT {} DEFAULT {} FOR {};\nGO\n",
            schema,
            table,
            Self::escape_identifier(&constraint.name),
            constraint.definition,
            Self::escape_identifier(&constraint.column_name)
        )
    }

    fn foreign_key_sql(table: &str, foreign_key: &ForeignKeyMetadata) -> Result<String, String> {
        if foreign_key.is_disabled {
            return Err(format!(
                "foreign key '{}' is disabled; disabling behavior cannot be safely preserved",
                foreign_key.name
            ));
        }
        if foreign_key.column_mappings.is_empty() {
            return Err(format!(
                "foreign key '{}' has no captured column mappings",
                foreign_key.name
            ));
        }
        let (parent, referenced): (Vec<_>, Vec<_>) = foreign_key
            .column_mappings
            .iter()
            .map(|(left, right)| (left.clone(), right.clone()))
            .unzip();
        Ok(format!(
            "ALTER TABLE {table} WITH CHECK ADD CONSTRAINT {} FOREIGN KEY ({}) REFERENCES {}.{} ({}){}{};\nGO\n",
            Self::escape_identifier(&foreign_key.name),
            Self::column_list(&parent),
            Self::escape_identifier(&foreign_key.referenced_schema),
            Self::escape_identifier(&foreign_key.referenced_table),
            Self::column_list(&referenced),
            Self::referential_action("ON DELETE", &foreign_key.delete_rule)?,
            Self::referential_action("ON UPDATE", &foreign_key.update_rule)?
        ))
    }

    fn create_index_sql(table: &str, index: &IndexMetadata) -> Result<String, String> {
        if index.key_columns.is_empty()
            || index.key_columns_descending.len() != index.key_columns.len()
        {
            return Err(format!(
                "index '{}' is missing ordered key metadata",
                index.name
            ));
        }
        let keys = index
            .key_columns
            .iter()
            .zip(&index.key_columns_descending)
            .map(|(column, descending)| {
                format!(
                    "{} {}",
                    Self::escape_identifier(column),
                    if *descending { "DESC" } else { "ASC" }
                )
            })
            .collect::<Vec<_>>()
            .join(", ");
        let include = if index.included_columns.is_empty() {
            String::new()
        } else {
            format!(" INCLUDE ({})", Self::column_list(&index.included_columns))
        };
        let filter = index
            .filter_definition
            .as_deref()
            .map(|definition| format!(" WHERE {definition}"))
            .unwrap_or_default();
        Ok(format!(
            "CREATE {}{} INDEX {} ON {table} ({keys}){include}{filter};\nGO\n",
            if index.is_unique { "UNIQUE " } else { "" },
            if index.is_clustered {
                "CLUSTERED"
            } else {
                "NONCLUSTERED"
            },
            Self::escape_identifier(&index.name)
        ))
    }

    fn column_type(column: &ColumnMetadata) -> Result<String, String> {
        let type_name = column.data_type.to_ascii_lowercase();
        let length = match type_name.as_str() {
            "varchar" | "char" | "varbinary" | "binary" => Some(if column.max_length == -1 {
                "MAX".to_string()
            } else {
                column.max_length.to_string()
            }),
            "nvarchar" | "nchar" => Some(if column.max_length == -1 {
                "MAX".to_string()
            } else {
                (column.max_length / 2).to_string()
            }),
            _ => None,
        };
        if let Some(length) = length {
            if length != "MAX" && length.parse::<i16>().unwrap_or(0) <= 0 {
                return Err(format!(
                    "length is not captured for column '{}'",
                    column.name
                ));
            }
            return Ok(format!("{}({length})", column.data_type));
        }
        match type_name.as_str() {
            "decimal" | "numeric" => Ok(format!(
                "{}({},{})",
                column.data_type, column.precision, column.scale
            )),
            "time" | "datetime2" | "datetimeoffset" => {
                Ok(format!("{}({})", column.data_type, column.scale))
            }
            "bigint" | "int" | "smallint" | "tinyint" | "bit" | "money" | "smallmoney" | "real"
            | "date" | "datetime" | "smalldatetime" | "uniqueidentifier" | "xml" | "timestamp"
            | "rowversion" | "sql_variant" | "text" | "ntext" | "image" => {
                Ok(column.data_type.clone())
            }
            "float" if (1..=53).contains(&column.precision) => {
                Ok(format!("{}({})", column.data_type, column.precision))
            }
            _ => Err(format!(
                "unsupported or user-defined SQL type '{}'",
                column.data_type
            )),
        }
    }

    fn column_list(columns: &[String]) -> String {
        columns
            .iter()
            .map(|column| Self::escape_identifier(column))
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn referential_action(clause: &str, action: &str) -> Result<String, String> {
        match action.to_ascii_uppercase().as_str() {
            "NO_ACTION" => Ok(String::new()),
            "CASCADE" => Ok(format!(" {clause} CASCADE")),
            "SET_NULL" => Ok(format!(" {clause} SET NULL")),
            "SET_DEFAULT" => Ok(format!(" {clause} SET DEFAULT")),
            _ => Err(format!("unsupported foreign key action '{action}'")),
        }
    }

    pub fn escape_identifier(identifier: &str) -> String {
        let clean = identifier.trim().trim_matches(|c| c == '[' || c == ']');
        let escaped = clean.replace(']', "]]");
        format!("[{}]", escaped)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sql_generator_creates_safe_ddl() {
        let diffs = vec![
            SchemaDifference {
                id: "1".to_string(),
                object_type: SchemaObjectType::Schema,
                schema_name: "sales".to_string(),
                object_name: "sales".to_string(),
                category: DifferenceCategory::MissingInTarget,
                property_name: None,
                value_source: Some("sales".to_string()),
                value_target: None,
                severity: DifferenceSeverity::Medium,
                confidence: ConfidenceStatus::High,
                explanation: "Schema missing".to_string(),
                risk_assessment: None,
                can_generate_sql: true,
            },
            SchemaDifference {
                id: "2".to_string(),
                object_type: SchemaObjectType::Column,
                schema_name: "dbo".to_string(),
                object_name: "Orders.Notes".to_string(),
                category: DifferenceCategory::MissingInTarget,
                property_name: None,
                value_source: Some("NVARCHAR(500) NULL".to_string()),
                value_target: None,
                severity: DifferenceSeverity::Medium,
                confidence: ConfidenceStatus::High,
                explanation: "Column missing".to_string(),
                risk_assessment: None,
                can_generate_sql: true,
            },
        ];

        let plan = SqlGenerator::plan_and_generate(&diffs, "SchemaTarget");
        assert_eq!(plan.total_steps, 1);
        assert_eq!(plan.destructive_steps, 0);
        assert!(plan.script_sql.contains("CREATE SCHEMA [sales]"));
        assert!(!plan.script_sql.contains("ADD [Notes]"));
        assert!(plan
            .unsupported_operations
            .iter()
            .any(|issue| issue.contains("Orders.Notes")));
        assert!(!plan.script_sql.contains("PlaceholderId"));
    }

    #[test]
    fn test_sql_generator_refuses_indeterminate() {
        let diffs = vec![SchemaDifference {
            id: "enc-1".to_string(),
            object_type: SchemaObjectType::View,
            schema_name: "dbo".to_string(),
            object_name: "EncryptedView".to_string(),
            category: DifferenceCategory::Indeterminate,
            property_name: Some("definition".to_string()),
            value_source: Some("[ENCRYPTED]".to_string()),
            value_target: None,
            severity: DifferenceSeverity::Medium,
            confidence: ConfidenceStatus::Indeterminate,
            explanation: "Definition encrypted".to_string(),
            risk_assessment: None,
            can_generate_sql: false,
        }];

        let plan = SqlGenerator::plan_and_generate(&diffs, "TargetDB");
        assert_eq!(plan.total_steps, 0);
        assert!(!plan.script_sql.contains("EncryptedView"));
    }

    #[test]
    fn test_escape_special_identifiers_with_brackets() {
        let malformed = "Table]Name With Spaces";
        let escaped = SqlGenerator::escape_identifier(malformed);
        assert_eq!(escaped, "[Table]]Name With Spaces]");
    }

    fn empty_snapshot(database_name: &str, tables: Vec<TableMetadata>) -> DatabaseSnapshot {
        DatabaseSnapshot {
            server_name: "sql-test".to_string(),
            database_name: database_name.to_string(),
            sql_server_version: "16.0".to_string(),
            collation: "Latin1_General_100_CI_AS".to_string(),
            compatibility_level: 160,
            captured_at_utc: "2026-01-01T00:00:00Z".to_string(),
            schemas: vec![SchemaMetadata {
                name: "catalog".to_string(),
                owner: "dbo".to_string(),
            }],
            tables,
            views: vec![],
            procedures: vec![],
            functions: vec![],
            triggers: vec![],
            sequences: vec![],
            synonyms: vec![],
            custom_types: vec![],
            collection_warnings: vec![],
        }
    }

    fn product_table() -> TableMetadata {
        TableMetadata {
            id: ObjectId::new("catalog", "Product Ledger"),
            columns: vec![
                ColumnMetadata {
                    name: "ProductId".to_string(),
                    ordinal_position: 1,
                    data_type: "int".to_string(),
                    max_length: 4,
                    precision: 10,
                    scale: 0,
                    is_nullable: false,
                    is_identity: true,
                    identity_seed: Some(10),
                    identity_increment: Some(2),
                    is_computed: false,
                    computed_definition: None,
                    computed_persisted: false,
                    collation: None,
                },
                ColumnMetadata {
                    name: "TenantId".to_string(),
                    ordinal_position: 2,
                    data_type: "bigint".to_string(),
                    max_length: 8,
                    precision: 19,
                    scale: 0,
                    is_nullable: false,
                    is_identity: false,
                    identity_seed: None,
                    identity_increment: None,
                    is_computed: false,
                    computed_definition: None,
                    computed_persisted: false,
                    collation: None,
                },
                ColumnMetadata {
                    name: "ProductCode".to_string(),
                    ordinal_position: 3,
                    data_type: "nvarchar".to_string(),
                    max_length: 160,
                    precision: 0,
                    scale: 0,
                    is_nullable: true,
                    is_identity: false,
                    identity_seed: None,
                    identity_increment: None,
                    is_computed: false,
                    computed_definition: None,
                    computed_persisted: false,
                    collation: Some("Latin1_General_100_CI_AS".to_string()),
                },
                ColumnMetadata {
                    name: "CreatedAt".to_string(),
                    ordinal_position: 4,
                    data_type: "datetime2".to_string(),
                    max_length: 8,
                    precision: 23,
                    scale: 3,
                    is_nullable: false,
                    is_identity: false,
                    identity_seed: None,
                    identity_increment: None,
                    is_computed: false,
                    computed_definition: None,
                    computed_persisted: false,
                    collation: None,
                },
                ColumnMetadata {
                    name: "Price".to_string(),
                    ordinal_position: 5,
                    data_type: "decimal".to_string(),
                    max_length: 9,
                    precision: 12,
                    scale: 2,
                    is_nullable: false,
                    is_identity: false,
                    identity_seed: None,
                    identity_increment: None,
                    is_computed: false,
                    computed_definition: None,
                    computed_persisted: false,
                    collation: None,
                },
            ],
            primary_key: Some(KeyConstraintMetadata {
                name: "PK_Product Ledger".to_string(),
                is_clustered: true,
                columns: vec!["ProductId".to_string(), "TenantId".to_string()],
            }),
            foreign_keys: vec![],
            check_constraints: vec![CheckConstraintMetadata {
                name: "CK_Product_Price".to_string(),
                definition: "([Price] >= (0))".to_string(),
                is_disabled: false,
            }],
            default_constraints: vec![DefaultConstraintMetadata {
                name: "DF_Product_CreatedAt".to_string(),
                column_name: "CreatedAt".to_string(),
                definition: "(SYSUTCDATETIME())".to_string(),
            }],
            unique_constraints: vec![KeyConstraintMetadata {
                name: "UQ_ProductCode".to_string(),
                is_clustered: false,
                columns: vec!["ProductCode".to_string()],
            }],
            indexes: vec![IndexMetadata {
                name: "IX_ProductCode".to_string(),
                is_unique: false,
                is_clustered: false,
                key_columns: vec!["ProductCode".to_string()],
                key_columns_descending: vec![true],
                included_columns: vec!["CreatedAt".to_string()],
                filter_definition: Some("([ProductCode] IS NOT NULL)".to_string()),
            }],
        }
    }

    #[test]
    fn missing_table_ddl_uses_complete_source_snapshot_without_placeholders() {
        let table = product_table();
        let source = empty_snapshot("OriginDb", vec![table]);
        let target = empty_snapshot("TargetDb", vec![]);
        let difference = SchemaDifference {
            id: "table-create".to_string(),
            object_type: SchemaObjectType::Table,
            schema_name: "catalog".to_string(),
            object_name: "Product Ledger".to_string(),
            category: DifferenceCategory::MissingInTarget,
            property_name: None,
            value_source: Some("Table with 5 columns".to_string()),
            value_target: None,
            severity: DifferenceSeverity::Medium,
            confidence: ConfidenceStatus::High,
            explanation: "Table missing in target".to_string(),
            risk_assessment: None,
            can_generate_sql: true,
        };

        let plan = SqlGenerator::plan_and_generate_with_snapshots(
            &[difference],
            "TargetDb",
            Some(&source),
            Some(&target),
        );
        assert!(
            plan.unsupported_operations.is_empty(),
            "{:?}",
            plan.unsupported_operations
        );
        assert_eq!(plan.total_steps, 1);
        assert!(plan
            .script_sql
            .contains("[ProductId] int IDENTITY(10,2) NOT NULL"));
        assert!(plan
            .script_sql
            .contains("[ProductCode] nvarchar(80) COLLATE Latin1_General_100_CI_AS NULL"));
        assert!(plan.script_sql.contains("[CreatedAt] datetime2(3) NOT NULL CONSTRAINT [DF_Product_CreatedAt] DEFAULT (SYSUTCDATETIME())"));
        assert!(plan.script_sql.contains("[Price] decimal(12,2) NOT NULL"));
        assert!(plan
            .script_sql
            .contains("PRIMARY KEY CLUSTERED ([ProductId], [TenantId])"));
        assert!(plan
            .script_sql
            .contains("UNIQUE NONCLUSTERED ([ProductCode])"));
        assert!(plan.script_sql.contains("CHECK ([Price] >= (0))"));
        assert!(plan.script_sql.contains("[ProductCode] DESC"));
        assert!(plan.script_sql.contains("INCLUDE ([CreatedAt])"));
        assert!(plan
            .script_sql
            .contains("WHERE ([ProductCode] IS NOT NULL)"));
        assert!(!plan.script_sql.contains("PlaceholderId"));
        assert!(!plan.script_sql.contains("Adicionar colunas conforme"));
    }

    #[test]
    fn missing_table_ddl_preserves_computed_expression_and_persisted_state() {
        let mut table = product_table();
        table.columns.push(ColumnMetadata {
            name: "ComputedTotal".to_string(),
            ordinal_position: 6,
            data_type: "computed".to_string(),
            max_length: 0,
            precision: 0,
            scale: 0,
            is_nullable: true,
            is_identity: false,
            identity_seed: None,
            identity_increment: None,
            is_computed: true,
            computed_definition: Some("([Price] * 2)".to_string()),
            computed_persisted: true,
            collation: None,
        });
        let source = empty_snapshot("OriginDb", vec![table]);
        let difference = SchemaDifference {
            id: "table-with-computed".to_string(),
            object_type: SchemaObjectType::Table,
            schema_name: "catalog".to_string(),
            object_name: "Product Ledger".to_string(),
            category: DifferenceCategory::MissingInTarget,
            property_name: None,
            value_source: Some("table".to_string()),
            value_target: None,
            severity: DifferenceSeverity::Medium,
            confidence: ConfidenceStatus::High,
            explanation: "table missing".to_string(),
            risk_assessment: None,
            can_generate_sql: true,
        };

        let plan = SqlGenerator::plan_and_generate_with_snapshots(
            &[difference],
            "TargetDb",
            Some(&source),
            None,
        );

        assert!(
            plan.unsupported_operations.is_empty(),
            "{:?}",
            plan.unsupported_operations
        );
        assert!(plan
            .script_sql
            .contains("[ComputedTotal] AS ([Price] * 2) PERSISTED"));
    }

    #[test]
    fn index_definition_mismatch_generates_reviewable_drop_and_recreate() {
        let source_table = product_table();
        let mut target_table = product_table();
        target_table.indexes[0].key_columns_descending = vec![false];
        target_table.indexes[0].included_columns.clear();
        target_table.indexes[0].filter_definition = None;
        let source = empty_snapshot("OriginDb", vec![source_table]);
        let target = empty_snapshot("TargetDb", vec![target_table]);
        let difference = SchemaDifference {
            id: "index-mismatch".to_string(),
            object_type: SchemaObjectType::Index,
            schema_name: "catalog".to_string(),
            object_name: "Product Ledger.IX_ProductCode".to_string(),
            category: DifferenceCategory::PropertyMismatch,
            property_name: Some("definition".to_string()),
            value_source: Some("source index definition".to_string()),
            value_target: Some("target index definition".to_string()),
            severity: DifferenceSeverity::Medium,
            confidence: ConfidenceStatus::High,
            explanation: "index differs".to_string(),
            risk_assessment: None,
            can_generate_sql: true,
        };

        let operation =
            SqlGenerator::generate_for_difference(&difference, "TargetDb", &source, &target)
                .expect("index replacement should be available");

        let drop_position = operation.sql.find("DROP INDEX [IX_ProductCode]").unwrap();
        let create_position = operation
            .sql
            .find("CREATE NONCLUSTERED INDEX [IX_ProductCode]")
            .unwrap();
        assert!(drop_position < create_position);
        assert!(operation.sql.contains("[ProductCode] DESC"));
        assert!(operation.sql.contains("INCLUDE ([CreatedAt])"));
        assert!(operation.sql.contains("WHERE ([ProductCode] IS NOT NULL)"));
        assert!(operation.is_destructive);
    }

    #[test]
    fn missing_table_without_complete_snapshot_is_explicitly_unavailable() {
        let difference = SchemaDifference {
            id: "missing-snapshot".to_string(),
            object_type: SchemaObjectType::Table,
            schema_name: "catalog".to_string(),
            object_name: "Uncaptured".to_string(),
            category: DifferenceCategory::MissingInTarget,
            property_name: None,
            value_source: Some("Table with 2 columns".to_string()),
            value_target: None,
            severity: DifferenceSeverity::Medium,
            confidence: ConfidenceStatus::High,
            explanation: "Table missing in target".to_string(),
            risk_assessment: None,
            can_generate_sql: true,
        };
        let plan = SqlGenerator::plan_and_generate(&[difference], "TargetDb");
        assert_eq!(plan.total_steps, 0);
        assert!(!plan.unsupported_operations.is_empty());
        assert!(!plan.script_sql.contains("PlaceholderId"));
    }
}

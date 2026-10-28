use schema_lens_domain::*;
use uuid::Uuid;

pub struct SqlGenerator;

impl SqlGenerator {
    pub fn plan_and_generate(
        differences: &[SchemaDifference],
        target_database: &str,
    ) -> MigrationPlan {
        let mut steps = Vec::new();
        let mut destructive_count = 0;

        for diff in differences {
            // Regra Inegociável: Se a confiança for Indeterminate ou can_generate_sql for falso, NUNCA gere DDL
            if !diff.can_generate_sql || diff.confidence == ConfidenceStatus::Indeterminate {
                continue;
            }

            let schema_escaped = Self::escape_identifier(&diff.schema_name);

            match diff.object_type {
                SchemaObjectType::Schema => {
                    if diff.category == DifferenceCategory::MissingInTarget {
                        steps.push(MigrationStep {
                            order: 10,
                            description: format!("Criar schema defensivo {}", schema_escaped),
                            sql: format!(
                                "IF NOT EXISTS (SELECT * FROM sys.schemas WHERE name = N'{}')\nBEGIN\n    EXEC('CREATE SCHEMA {}');\nEND\nGO\n",
                                diff.schema_name.replace('\'', "''"),
                                schema_escaped
                            ),
                            is_destructive: false,
                        });
                    }
                }
                SchemaObjectType::Table => {
                    let table_escaped = Self::escape_identifier(&diff.object_name);
                    if diff.category == DifferenceCategory::MissingInTarget {
                        steps.push(MigrationStep {
                            order: 20,
                            description: format!("Criar tabela {}.{}", schema_escaped, table_escaped),
                            sql: format!(
                                "IF OBJECT_ID(N'{}.{}', 'U') IS NULL\nBEGIN\n    CREATE TABLE {}.{} (\n        -- Adicionar colunas conforme especificação da origem\n        PlaceholderId INT NOT NULL\n    );\nEND\nGO\n",
                                schema_escaped, table_escaped, schema_escaped, table_escaped
                            ),
                            is_destructive: false,
                        });
                    } else if diff.category == DifferenceCategory::MissingInSource {
                        destructive_count += 1;
                        steps.push(MigrationStep {
                            order: 900,
                            description: format!("Remover tabela legada {}.{} do destino", schema_escaped, table_escaped),
                            sql: format!(
                                "-- WARNING: DESTRUCTIVE OPERATION - POTENTIAL DATA LOSS\n-- IF OBJECT_ID(N'{}.{}', 'U') IS NOT NULL\n-- BEGIN\n--     DROP TABLE {}.{};\n-- END\n-- GO\n",
                                schema_escaped, table_escaped, schema_escaped, table_escaped
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
                        let col_type = diff.value_source.as_deref().unwrap_or("VARCHAR(100) NULL");
                        steps.push(MigrationStep {
                            order: 30,
                            description: format!("Adicionar coluna {} em {}.{}", col_escaped, schema_escaped, tbl_escaped),
                            sql: format!(
                                "IF COL_LENGTH(N'{}.{}', N'{}') IS NULL\nBEGIN\n    ALTER TABLE {}.{} ADD {} {};\nEND\nGO\n",
                                schema_escaped, tbl_escaped, col_name_raw.replace('\'', "''"), schema_escaped, tbl_escaped, col_escaped, col_type
                            ),
                            is_destructive: false,
                        });
                    } else if diff.category == DifferenceCategory::MissingInSource {
                        destructive_count += 1;
                        steps.push(MigrationStep {
                            order: 850,
                            description: format!("Excluir coluna {} em {}.{}", col_escaped, schema_escaped, tbl_escaped),
                            sql: format!(
                                "-- WARNING: DESTRUCTIVE OPERATION - POTENTIAL DATA LOSS\n-- ALTER TABLE {}.{} DROP COLUMN {};\n-- GO\n",
                                schema_escaped, tbl_escaped, col_escaped
                            ),
                            is_destructive: true,
                        });
                    } else if diff.category == DifferenceCategory::PropertyMismatch {
                        let is_destructive = diff.severity == DifferenceSeverity::Critical;
                        if is_destructive {
                            destructive_count += 1;
                        }
                        let new_type = diff.value_source.as_deref().unwrap_or("VARCHAR(100)");
                        steps.push(MigrationStep {
                            order: 40,
                            description: format!(
                                "Alterar tipo/tamanho da coluna {} em {}.{}",
                                col_escaped, schema_escaped, tbl_escaped
                            ),
                            sql: format!(
                                "{}\nALTER TABLE {}.{} ALTER COLUMN {} {};\nGO\n",
                                if is_destructive {
                                    "-- WARNING: POTENTIAL TRUNCATION / CONVERSION DATA LOSS"
                                } else {
                                    ""
                                },
                                schema_escaped,
                                tbl_escaped,
                                col_escaped,
                                new_type
                            ),
                            is_destructive,
                        });
                    }
                }
                SchemaObjectType::PrimaryKey => {
                    let tbl_name_raw = diff
                        .object_name
                        .split('.')
                        .next()
                        .unwrap_or(&diff.object_name);
                    let tbl_escaped = Self::escape_identifier(tbl_name_raw);
                    let pk_name_raw = diff
                        .object_name
                        .split('.')
                        .next_back()
                        .unwrap_or("PK_Constraint");
                    let pk_escaped = Self::escape_identifier(pk_name_raw);

                    if diff.category == DifferenceCategory::MissingInTarget {
                        let cols_raw = diff.value_source.as_deref().unwrap_or("([Id])");
                        steps.push(MigrationStep {
                            order: 50,
                            description: format!("Adicionar Primary Key {} em {}.{}", pk_escaped, schema_escaped, tbl_escaped),
                            sql: format!(
                                "ALTER TABLE {}.{} ADD CONSTRAINT {} PRIMARY KEY CLUSTERED {};\nGO\n",
                                schema_escaped, tbl_escaped, pk_escaped, cols_raw
                            ),
                            is_destructive: false,
                        });
                    }
                }
                SchemaObjectType::UniqueConstraint => {
                    let tbl_name_raw = diff
                        .object_name
                        .split('.')
                        .next()
                        .unwrap_or(&diff.object_name);
                    let tbl_escaped = Self::escape_identifier(tbl_name_raw);
                    let uq_name_raw = diff
                        .object_name
                        .split('.')
                        .next_back()
                        .unwrap_or("UQ_Constraint");
                    let uq_escaped = Self::escape_identifier(uq_name_raw);

                    if diff.category == DifferenceCategory::MissingInTarget {
                        let cols_raw = diff.value_source.as_deref().unwrap_or("([Code])");
                        steps.push(MigrationStep {
                            order: 55,
                            description: format!(
                                "Adicionar Unique Constraint {} em {}.{}",
                                uq_escaped, schema_escaped, tbl_escaped
                            ),
                            sql: format!(
                                "ALTER TABLE {}.{} ADD CONSTRAINT {} UNIQUE {};\nGO\n",
                                schema_escaped, tbl_escaped, uq_escaped, cols_raw
                            ),
                            is_destructive: false,
                        });
                    }
                }
                SchemaObjectType::DefaultConstraint => {
                    let col_name_raw = diff.object_name.split('.').nth(1).unwrap_or("Column");
                    let tbl_name_raw = diff
                        .object_name
                        .split('.')
                        .next()
                        .unwrap_or(&diff.object_name);
                    let col_escaped = Self::escape_identifier(col_name_raw);
                    let tbl_escaped = Self::escape_identifier(tbl_name_raw);
                    let df_name = format!("DF_{}_{}", tbl_name_raw, col_name_raw);
                    let df_escaped = Self::escape_identifier(&df_name);
                    let def = diff.value_source.as_deref().unwrap_or("(0)");

                    if diff.category == DifferenceCategory::MissingInTarget
                        || diff.category == DifferenceCategory::PropertyMismatch
                    {
                        steps.push(MigrationStep {
                            order: 60,
                            description: format!(
                                "Definir valor Default para {}.{}.{}",
                                schema_escaped, tbl_escaped, col_escaped
                            ),
                            sql: format!(
                                "ALTER TABLE {}.{} ADD CONSTRAINT {} DEFAULT {} FOR {};\nGO\n",
                                schema_escaped, tbl_escaped, df_escaped, def, col_escaped
                            ),
                            is_destructive: false,
                        });
                    }
                }
                SchemaObjectType::Index => {
                    let tbl_name_raw = diff
                        .object_name
                        .split('.')
                        .next()
                        .unwrap_or(&diff.object_name);
                    let idx_name_raw = diff
                        .object_name
                        .split('.')
                        .next_back()
                        .unwrap_or(&diff.object_name);
                    let tbl_escaped = Self::escape_identifier(tbl_name_raw);
                    let idx_escaped = Self::escape_identifier(idx_name_raw);

                    if diff.category == DifferenceCategory::MissingInTarget {
                        steps.push(MigrationStep {
                            order: 70,
                            description: format!("Criar índice {} em {}.{}", idx_escaped, schema_escaped, tbl_escaped),
                            sql: format!(
                                "IF NOT EXISTS (SELECT * FROM sys.indexes WHERE name = N'{}' AND object_id = OBJECT_ID(N'{}.{}'))\nBEGIN\n    CREATE NONCLUSTERED INDEX {} ON {}.{} ([Id]);\nEND\nGO\n",
                                idx_name_raw.replace('\'', "''"), schema_escaped, tbl_escaped, idx_escaped, schema_escaped, tbl_escaped
                            ),
                            is_destructive: false,
                        });
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
                    }
                }
                _ => {}
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
            script_sql: full_script,
            steps,
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
        assert_eq!(plan.total_steps, 2);
        assert_eq!(plan.destructive_steps, 0);
        assert!(plan.script_sql.contains("CREATE SCHEMA [sales]"));
        assert!(plan.script_sql.contains("ADD [Notes] NVARCHAR(500) NULL"));
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
}

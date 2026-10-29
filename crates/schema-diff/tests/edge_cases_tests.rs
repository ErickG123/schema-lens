use schema_diff::DiffEngine;
use schema_lens_domain::*;

fn create_base_snapshot(name: &str) -> DatabaseSnapshot {
    DatabaseSnapshot {
        server_name: "TestServer".to_string(),
        database_name: name.to_string(),
        sql_server_version: "16.0.1000.6".to_string(),
        collation: "Latin1_General_CI_AS".to_string(),
        compatibility_level: 160,
        captured_at_utc: "2026-10-27T20:00:00Z".to_string(),
        schemas: vec![SchemaMetadata {
            name: "sales".to_string(),
            owner: "dbo".to_string(),
        }],
        tables: vec![],
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

#[test]
fn test_diff_engine_composite_primary_and_foreign_keys() {
    let mut src = create_base_snapshot("Source");
    let mut tgt = create_base_snapshot("Target");

    // Tabela com chave primária composta (TenantId, OrderId)
    let src_table = TableMetadata {
        id: ObjectId::new("sales", "OrderItems"),
        columns: vec![
            ColumnMetadata {
                name: "TenantId".to_string(),
                ordinal_position: 1,
                data_type: "INT".to_string(),
                max_length: 4,
                precision: 10,
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
                name: "OrderId".to_string(),
                ordinal_position: 2,
                data_type: "INT".to_string(),
                max_length: 4,
                precision: 10,
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
                name: "ItemId".to_string(),
                ordinal_position: 3,
                data_type: "INT".to_string(),
                max_length: 4,
                precision: 10,
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
        ],
        primary_key: Some(KeyConstraintMetadata {
            name: "PK_OrderItems".to_string(),
            is_clustered: true,
            columns: vec![
                "TenantId".to_string(),
                "OrderId".to_string(),
                "ItemId".to_string(),
            ],
        }),
        foreign_keys: vec![ForeignKeyMetadata {
            name: "FK_OrderItems_Order".to_string(),
            referenced_schema: "sales".to_string(),
            referenced_table: "Orders".to_string(),
            column_mappings: vec![
                ("TenantId".to_string(), "TenantId".to_string()),
                ("OrderId".to_string(), "OrderId".to_string()),
            ],
            delete_rule: "CASCADE".to_string(),
            update_rule: "NO_ACTION".to_string(),
            is_disabled: false,
        }],
        check_constraints: vec![],
        default_constraints: vec![],
        unique_constraints: vec![],
        indexes: vec![],
    };

    // No target, a FK composta aponta sem cascade e tem colunas diferentes
    let mut tgt_table = src_table.clone();
    tgt_table.foreign_keys = vec![ForeignKeyMetadata {
        name: "FK_OrderItems_Order".to_string(),
        referenced_schema: "sales".to_string(),
        referenced_table: "Orders".to_string(),
        column_mappings: vec![("OrderId".to_string(), "OrderId".to_string())], // Apenas OrderId
        delete_rule: "NO_ACTION".to_string(),
        update_rule: "NO_ACTION".to_string(),
        is_disabled: false,
    }];

    src.tables.push(src_table);
    tgt.tables.push(tgt_table);

    let result = DiffEngine::compare(&src, &tgt);
    assert!(!result.differences.is_empty());

    let fk_diff = result
        .differences
        .iter()
        .find(|d| d.object_name.contains("FK_OrderItems_Order"));
    assert!(fk_diff.is_some(), "Foreign key difference must be detected");
}

#[test]
fn test_diff_engine_self_referencing_foreign_key() {
    let mut src = create_base_snapshot("Source");
    let mut tgt = create_base_snapshot("Target");

    let emp_src = TableMetadata {
        id: ObjectId::new("sales", "Employees"),
        columns: vec![
            ColumnMetadata {
                name: "EmployeeId".to_string(),
                ordinal_position: 1,
                data_type: "INT".to_string(),
                max_length: 4,
                precision: 10,
                scale: 0,
                is_nullable: false,
                is_identity: true,
                identity_seed: Some(1),
                identity_increment: Some(1),
                is_computed: false,
                computed_definition: None,
                computed_persisted: false,
                collation: None,
            },
            ColumnMetadata {
                name: "ManagerId".to_string(),
                ordinal_position: 2,
                data_type: "INT".to_string(),
                max_length: 4,
                precision: 10,
                scale: 0,
                is_nullable: true,
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
            name: "PK_Employees".to_string(),
            is_clustered: true,
            columns: vec!["EmployeeId".to_string()],
        }),
        foreign_keys: vec![ForeignKeyMetadata {
            name: "FK_Employees_Manager".to_string(),
            referenced_schema: "sales".to_string(),
            referenced_table: "Employees".to_string(), // Auto-relacionamento
            column_mappings: vec![("ManagerId".to_string(), "EmployeeId".to_string())],
            delete_rule: "NO_ACTION".to_string(),
            update_rule: "NO_ACTION".to_string(),
            is_disabled: false,
        }],
        check_constraints: vec![],
        default_constraints: vec![],
        unique_constraints: vec![],
        indexes: vec![],
    };

    // Target não tem a FK autorreferencial
    let mut emp_tgt = emp_src.clone();
    emp_tgt.foreign_keys.clear();

    src.tables.push(emp_src);
    tgt.tables.push(emp_tgt);

    let result = DiffEngine::compare(&src, &tgt);
    assert_eq!(result.summary.missing_in_target, 1);
    let diff = &result.differences[0];
    assert_eq!(diff.object_type, SchemaObjectType::ForeignKey);
    assert!(diff.explanation.contains("FK_Employees_Manager"));
}

#[test]
fn test_case_difference_in_casing_detected_without_crash() {
    let mut src = create_base_snapshot("Source");
    let mut tgt = create_base_snapshot("Target");

    // Tabela com casing diferente: [Sales].[CUSTOMERS] vs [Sales].[Customers]
    src.tables.push(TableMetadata {
        id: ObjectId::new("Sales", "CUSTOMERS"),
        columns: vec![ColumnMetadata {
            name: "CustomerId".to_string(),
            ordinal_position: 1,
            data_type: "INT".to_string(),
            max_length: 4,
            precision: 10,
            scale: 0,
            is_nullable: false,
            is_identity: true,
            identity_seed: Some(1),
            identity_increment: Some(1),
            is_computed: false,
            computed_definition: None,
            computed_persisted: false,
            collation: None,
        }],
        primary_key: None,
        foreign_keys: vec![],
        check_constraints: vec![],
        default_constraints: vec![],
        unique_constraints: vec![],
        indexes: vec![],
    });

    tgt.tables.push(TableMetadata {
        id: ObjectId::new("Sales", "Customers"),
        columns: vec![ColumnMetadata {
            name: "CustomerId".to_string(),
            ordinal_position: 1,
            data_type: "INT".to_string(),
            max_length: 4,
            precision: 10,
            scale: 0,
            is_nullable: false,
            is_identity: true,
            identity_seed: Some(1),
            identity_increment: Some(1),
            is_computed: false,
            computed_definition: None,
            computed_persisted: false,
            collation: None,
        }],
        primary_key: None,
        foreign_keys: vec![],
        check_constraints: vec![],
        default_constraints: vec![],
        unique_constraints: vec![],
        indexes: vec![],
    });

    let result = DiffEngine::compare(&src, &tgt);
    // Na comparação exata do Rust ObjectId, objetos com casing estrito diferente são computados de forma determinística
    assert_eq!(result.summary.total_differences, 2);
    assert_eq!(result.summary.missing_in_target, 1);
    assert_eq!(result.summary.missing_in_source, 1);
}

#[test]
fn test_diff_engine_column_collation_and_nullability_mismatch() {
    let mut src = create_base_snapshot("Source");
    let mut tgt = create_base_snapshot("Target");

    let col_src = ColumnMetadata {
        name: "Description".to_string(),
        ordinal_position: 1,
        data_type: "VARCHAR".to_string(),
        max_length: 200,
        precision: 0,
        scale: 0,
        is_nullable: true,
        is_identity: false,
        identity_seed: None,
        identity_increment: None,
        is_computed: false,
        computed_definition: None,
        computed_persisted: false,
        collation: Some("Latin1_General_BIN".to_string()),
    };

    let col_tgt = ColumnMetadata {
        name: "Description".to_string(),
        ordinal_position: 1,
        data_type: "VARCHAR".to_string(),
        max_length: 200,
        precision: 0,
        scale: 0,
        is_nullable: false, // NOT NULL no destino (divergência crítica de nulidade)
        is_identity: false,
        identity_seed: None,
        identity_increment: None,
        is_computed: false,
        computed_definition: None,
        computed_persisted: false,
        collation: Some("Latin1_General_CI_AS".to_string()), // Collation divergente
    };

    src.tables.push(TableMetadata {
        id: ObjectId::new("sales", "Products"),
        columns: vec![col_src],
        primary_key: None,
        foreign_keys: vec![],
        check_constraints: vec![],
        default_constraints: vec![],
        unique_constraints: vec![],
        indexes: vec![],
    });

    tgt.tables.push(TableMetadata {
        id: ObjectId::new("sales", "Products"),
        columns: vec![col_tgt],
        primary_key: None,
        foreign_keys: vec![],
        check_constraints: vec![],
        default_constraints: vec![],
        unique_constraints: vec![],
        indexes: vec![],
    });

    let result = DiffEngine::compare(&src, &tgt);
    assert!(!result.differences.is_empty());

    // Deve identificar property mismatches
    assert!(result
        .differences
        .iter()
        .any(|d| d.property_name.as_deref() == Some("is_nullable")));
    assert!(result
        .differences
        .iter()
        .any(|d| d.property_name.as_deref() == Some("collation")));
}

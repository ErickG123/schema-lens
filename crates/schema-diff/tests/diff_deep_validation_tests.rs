use schema_diff::DiffEngine;
use schema_lens_domain::*;

#[test]
fn test_identical_schemas_with_different_orders_produce_zero_diffs() {
    let t1 = TableMetadata {
        id: ObjectId::new("dbo", "Users"),
        columns: vec![
            ColumnMetadata {
                name: "Id".to_string(),
                ordinal_position: 1,
                data_type: "INT".to_string(),
                max_length: 4,
                precision: 0,
                scale: 0,
                is_nullable: false,
                is_identity: true,
                identity_seed: Some(1),
                identity_increment: Some(1),
                is_computed: false,
                computed_definition: None,
                collation: None,
            },
            ColumnMetadata {
                name: "Email".to_string(),
                ordinal_position: 2,
                data_type: "VARCHAR".to_string(),
                max_length: 100,
                precision: 0,
                scale: 0,
                is_nullable: false,
                is_identity: false,
                identity_seed: None,
                identity_increment: None,
                is_computed: false,
                computed_definition: None,
                collation: None,
            },
        ],
        primary_key: Some(KeyConstraintMetadata {
            name: "PK_Users".to_string(),
            is_clustered: true,
            columns: vec!["Id".to_string()],
        }),
        foreign_keys: vec![],
        check_constraints: vec![],
        default_constraints: vec![],
        unique_constraints: vec![],
        indexes: vec![IndexMetadata {
            name: "IX_Users_Email".to_string(),
            is_unique: true,
            is_clustered: false,
            key_columns: vec!["Email".to_string()],
            included_columns: vec![],
            filter_definition: None,
        }],
    };

    let t2 = TableMetadata {
        id: ObjectId::new("sales", "Orders"),
        columns: vec![ColumnMetadata {
            name: "OrderId".to_string(),
            ordinal_position: 1,
            data_type: "BIGINT".to_string(),
            max_length: 8,
            precision: 0,
            scale: 0,
            is_nullable: false,
            is_identity: false,
            identity_seed: None,
            identity_increment: None,
            is_computed: false,
            computed_definition: None,
            collation: None,
        }],
        primary_key: None,
        foreign_keys: vec![],
        check_constraints: vec![],
        default_constraints: vec![],
        unique_constraints: vec![],
        indexes: vec![],
    };

    // Snapshot A: Tabela Users depois Orders
    let snap_a = DatabaseSnapshot {
        server_name: "SRV_A".to_string(),
        database_name: "DB_A".to_string(),
        sql_server_version: "2022".to_string(),
        collation: "Latin1_General_CI_AS".to_string(),
        compatibility_level: 160,
        captured_at_utc: "".to_string(),
        schemas: vec![
            SchemaMetadata {
                name: "dbo".to_string(),
                owner: "dbo".to_string(),
            },
            SchemaMetadata {
                name: "sales".to_string(),
                owner: "dbo".to_string(),
            },
        ],
        tables: vec![t1.clone(), t2.clone()],
        views: vec![],
        procedures: vec![],
        functions: vec![],
        triggers: vec![],
        sequences: vec![],
        synonyms: vec![],
        custom_types: vec![],
        collection_warnings: vec![],
    };

    // Snapshot B: Mesmas tabelas e schemas, mas ordem invertida
    let snap_b = DatabaseSnapshot {
        server_name: "SRV_B".to_string(),
        database_name: "DB_B".to_string(),
        sql_server_version: "2022".to_string(),
        collation: "Latin1_General_CI_AS".to_string(),
        compatibility_level: 160,
        captured_at_utc: "".to_string(),
        schemas: vec![
            SchemaMetadata {
                name: "sales".to_string(),
                owner: "dbo".to_string(),
            },
            SchemaMetadata {
                name: "dbo".to_string(),
                owner: "dbo".to_string(),
            },
        ],
        tables: vec![t2, t1],
        views: vec![],
        procedures: vec![],
        functions: vec![],
        triggers: vec![],
        sequences: vec![],
        synonyms: vec![],
        custom_types: vec![],
        collection_warnings: vec![],
    };

    let result = DiffEngine::compare(&snap_a, &snap_b);
    assert_eq!(
        result.summary.total_differences, 0,
        "Schemas equivalentes com ordem invertida devem resultar em zero diferenças!"
    );
}

#[test]
fn test_same_name_in_different_schemas_does_not_collide() {
    let t_dbo = TableMetadata {
        id: ObjectId::new("dbo", "Config"),
        columns: vec![ColumnMetadata {
            name: "Id".to_string(),
            ordinal_position: 1,
            data_type: "INT".to_string(),
            max_length: 4,
            precision: 0,
            scale: 0,
            is_nullable: false,
            is_identity: false,
            identity_seed: None,
            identity_increment: None,
            is_computed: false,
            computed_definition: None,
            collation: None,
        }],
        primary_key: None,
        foreign_keys: vec![],
        check_constraints: vec![],
        default_constraints: vec![],
        unique_constraints: vec![],
        indexes: vec![],
    };

    let t_audit = TableMetadata {
        id: ObjectId::new("audit", "Config"),
        columns: vec![ColumnMetadata {
            name: "AuditId".to_string(),
            ordinal_position: 1,
            data_type: "BIGINT".to_string(),
            max_length: 8,
            precision: 0,
            scale: 0,
            is_nullable: false,
            is_identity: false,
            identity_seed: None,
            identity_increment: None,
            is_computed: false,
            computed_definition: None,
            collation: None,
        }],
        primary_key: None,
        foreign_keys: vec![],
        check_constraints: vec![],
        default_constraints: vec![],
        unique_constraints: vec![],
        indexes: vec![],
    };

    let snap_src = DatabaseSnapshot {
        server_name: "A".to_string(),
        database_name: "Source".to_string(),
        sql_server_version: "2022".to_string(),
        collation: "Latin1_General_CI_AS".to_string(),
        compatibility_level: 160,
        captured_at_utc: "".to_string(),
        schemas: vec![
            SchemaMetadata {
                name: "dbo".to_string(),
                owner: "dbo".to_string(),
            },
            SchemaMetadata {
                name: "audit".to_string(),
                owner: "dbo".to_string(),
            },
        ],
        tables: vec![t_dbo, t_audit],
        views: vec![],
        procedures: vec![],
        functions: vec![],
        triggers: vec![],
        sequences: vec![],
        synonyms: vec![],
        custom_types: vec![],
        collection_warnings: vec![],
    };

    let snap_tgt = DatabaseSnapshot {
        server_name: "B".to_string(),
        database_name: "Target".to_string(),
        sql_server_version: "2022".to_string(),
        collation: "Latin1_General_CI_AS".to_string(),
        compatibility_level: 160,
        captured_at_utc: "".to_string(),
        schemas: vec![SchemaMetadata {
            name: "dbo".to_string(),
            owner: "dbo".to_string(),
        }],
        tables: vec![], // Inexistente no destino
        views: vec![],
        procedures: vec![],
        functions: vec![],
        triggers: vec![],
        sequences: vec![],
        synonyms: vec![],
        custom_types: vec![],
        collection_warnings: vec![],
    };

    let diff = DiffEngine::compare(&snap_src, &snap_tgt);
    let table_diffs: Vec<_> = diff
        .differences
        .iter()
        .filter(|d| d.object_type == SchemaObjectType::Table)
        .collect();
    assert_eq!(table_diffs.len(), 2);
    assert!(table_diffs
        .iter()
        .any(|d| d.schema_name == "dbo" && d.object_name == "Config"));
    assert!(table_diffs
        .iter()
        .any(|d| d.schema_name == "audit" && d.object_name == "Config"));
}

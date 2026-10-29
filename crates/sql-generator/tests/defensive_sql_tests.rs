use schema_lens_domain::*;
use sql_generator::SqlGenerator;

#[test]
fn test_sql_generator_handles_transaction_and_idempotency_blocks() {
    let diffs = vec![
        SchemaDifference {
            id: "diff-sch-1".to_string(),
            object_type: SchemaObjectType::Schema,
            schema_name: "audit".to_string(),
            object_name: "audit".to_string(),
            category: DifferenceCategory::MissingInTarget,
            property_name: None,
            value_source: Some("audit".to_string()),
            value_target: None,
            severity: DifferenceSeverity::Medium,
            confidence: ConfidenceStatus::High,
            explanation: "Schema ausente no destino".to_string(),
            risk_assessment: None,
            can_generate_sql: true,
        },
        SchemaDifference {
            id: "diff-tbl-1".to_string(),
            object_type: SchemaObjectType::Table,
            schema_name: "audit".to_string(),
            object_name: "ChangeLog".to_string(),
            category: DifferenceCategory::MissingInTarget,
            property_name: None,
            value_source: Some("Table (2 cols)".to_string()),
            value_target: None,
            severity: DifferenceSeverity::Medium,
            confidence: ConfidenceStatus::High,
            explanation: "Tabela ChangeLog ausente no destino".to_string(),
            risk_assessment: None,
            can_generate_sql: true,
        },
    ];

    let plan = SqlGenerator::plan_and_generate(&diffs, "ProductionTarget");
    assert_eq!(plan.total_steps, 1);
    assert_eq!(plan.unsupported_operations.len(), 1);
    assert_eq!(plan.destructive_steps, 0);

    // Validar proteções defensivas do script
    assert!(plan.script_sql.contains("USE [ProductionTarget];"));
    assert!(plan
        .script_sql
        .contains("IF NOT EXISTS (SELECT * FROM sys.schemas WHERE name = N'audit')"));
    assert!(!plan
        .script_sql
        .contains("IF OBJECT_ID(N'[audit].[ChangeLog]', 'U') IS NULL"));
    assert!(!plan.script_sql.contains("PlaceholderId"));
    assert!(plan.unsupported_operations[0].contains("ChangeLog"));
}

#[test]
fn test_sql_generator_refuses_to_generate_ddl_for_destructive_drops_without_confirmation() {
    let diffs = vec![SchemaDifference {
        id: "drop-1".to_string(),
        object_type: SchemaObjectType::Table,
        schema_name: "dbo".to_string(),
        object_name: "CustomerBillingArchive".to_string(),
        category: DifferenceCategory::MissingInSource, // Existe no target, ausente na origem
        property_name: None,
        value_source: None,
        value_target: Some("Table (10 cols)".to_string()),
        severity: DifferenceSeverity::Critical,
        confidence: ConfidenceStatus::High,
        explanation: "Tabela existe apenas no destino".to_string(),
        risk_assessment: Some("Possível perda irreversível de dados".to_string()),
        can_generate_sql: false, // Política estrita do SchemaLens
    }];

    let plan = SqlGenerator::plan_and_generate(&diffs, "ProductionTarget");
    // Destructive drops não devem gerar DROP TABLE silencioso
    assert!(!plan.script_sql.contains("DROP TABLE"));
}

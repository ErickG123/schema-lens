use dotnet_conn_parser::DotnetConnParser;
use schema_diff::DiffEngine;
use schema_lens_domain::*;
use schema_reader::SqlServerReader;
use sql_generator::SqlGenerator;

#[tokio::test]
async fn test_real_sqlserver_docker_end_to_end_diff_and_readonly() {
    let source_conn_str = "Server=127.0.0.1,14331;Database=SchemaSource;User Id=sa;Password=\"SchemaLens#2026!LocalDev\";TrustServerCertificate=True;Connect Timeout=15;";
    let target_conn_str = "Server=127.0.0.1,14332;Database=SchemaTarget;User Id=sa;Password=\"SchemaLens#2026!LocalDev\";TrustServerCertificate=True;Connect Timeout=15;";

    // 1. Parse de connection strings
    let parsed_src =
        DotnetConnParser::parse(source_conn_str).expect("Failed to parse source connection string");
    let parsed_tgt =
        DotnetConnParser::parse(target_conn_str).expect("Failed to parse target connection string");

    // 2. Testar conectividade real com os dois servidores Docker
    let ver_src = SqlServerReader::test_connection(&parsed_src)
        .await
        .expect("Failed to connect to Docker SchemaSource");
    assert!(
        ver_src.contains("Microsoft SQL Server"),
        "Unexpected version: {}",
        ver_src
    );

    let ver_tgt = SqlServerReader::test_connection(&parsed_tgt)
        .await
        .expect("Failed to connect to Docker SchemaTarget");
    assert!(
        ver_tgt.contains("Microsoft SQL Server"),
        "Unexpected version: {}",
        ver_tgt
    );

    // 3. Listar bancos disponíveis
    let dbs_src = SqlServerReader::list_databases(&parsed_src)
        .await
        .expect("Failed to list databases from source");
    assert!(
        dbs_src.iter().any(|d| d == "SchemaSource"),
        "Database SchemaSource not listed"
    );

    let dbs_tgt = SqlServerReader::list_databases(&parsed_tgt)
        .await
        .expect("Failed to list databases from target");
    assert!(
        dbs_tgt.iter().any(|d| d == "SchemaTarget"),
        "Database SchemaTarget not listed"
    );

    // 4. Introspecção do catálogo real (sys.tables, sys.columns, sys.objects, etc.)
    let snap_source_1 = SqlServerReader::read_snapshot(&parsed_src)
        .await
        .expect("Failed to read snapshot from Source");
    let snap_target_1 = SqlServerReader::read_snapshot(&parsed_tgt)
        .await
        .expect("Failed to read snapshot from Target");

    assert!(
        snap_source_1.tables.len() >= 3,
        "Source must have at least 3 tables (Customers, Discounts, Orders)"
    );
    assert!(
        snap_target_1.tables.len() >= 3,
        "Target must have at least 3 tables (Customers, LegacyLogs, Orders)"
    );

    // 5. Executar motor de diff determinístico
    let diff_result = DiffEngine::compare(&snap_source_1, &snap_target_1);

    // Verificar se as divergências deliberadas foram identificadas:
    // - Schema sales ausente no Target
    // - Tabela sales.Discounts ausente no Target
    // - Tabela dbo.LegacyLogs ausente na Origem (exclusiva do Target)
    // - Coluna Notes ausente em Orders no Target
    // - Coluna Status com divergência de tamanho (VARCHAR(20) vs VARCHAR(10))
    // - View vw_ActiveCustomers divergente
    assert!(
        diff_result.summary.total_differences >= 5,
        "Expected at least 5 differences, got: {}",
        diff_result.summary.total_differences
    );
    assert!(
        diff_result.summary.critical_severity >= 1,
        "Expected critical differences (truncation / table drop)"
    );

    let has_discounts_missing = diff_result.differences.iter().any(|d| {
        d.object_type == SchemaObjectType::Table
            && d.object_name == "Discounts"
            && d.category == DifferenceCategory::MissingInTarget
    });
    assert!(
        has_discounts_missing,
        "sales.Discounts missing in Target difference not found"
    );

    let has_notes_missing = diff_result.differences.iter().any(|d| {
        d.object_type == SchemaObjectType::Column
            && d.object_name.contains("Notes")
            && d.category == DifferenceCategory::MissingInTarget
    });
    assert!(
        has_notes_missing,
        "Orders.Notes missing in Target difference not found"
    );

    // 6. Geração de Script DDL defensivo
    let plan =
        SqlGenerator::plan_and_generate(&diff_result.differences, &snap_target_1.database_name);
    assert!(
        plan.total_steps >= 2,
        "Expected migration steps to be generated"
    );
    assert!(
        plan.script_sql.contains("USE [SchemaTarget]"),
        "Script must target SchemaTarget"
    );
    assert!(
        plan.script_sql.contains("CREATE SCHEMA [sales]"),
        "Script must create missing schema"
    );

    // 7. COMPROVAÇÃO DE COMPORTAMENTO SOMENTE LEITURA:
    // Captura do snapshot 2 após toda a operação de comparação e geração de scripts
    let snap_source_2 = SqlServerReader::read_snapshot(&parsed_src)
        .await
        .expect("Failed to re-read Source snapshot");
    let snap_target_2 = SqlServerReader::read_snapshot(&parsed_tgt)
        .await
        .expect("Failed to re-read Target snapshot");

    // O número de tabelas, schemas e colunas deve ser IDÊNTICO ao anterior
    assert_eq!(
        snap_source_1.tables.len(),
        snap_source_2.tables.len(),
        "Source database altered by schema inspection!"
    );
    assert_eq!(
        snap_target_1.tables.len(),
        snap_target_2.tables.len(),
        "Target database altered by schema inspection!"
    );
    assert_eq!(
        snap_source_1.schemas.len(),
        snap_source_2.schemas.len(),
        "Source schemas altered!"
    );
    assert_eq!(
        snap_target_1.schemas.len(),
        snap_target_2.schemas.len(),
        "Target schemas altered!"
    );

    println!("SUCCESS: Real SQL Server Docker End-to-End and Read-Only verified!");
}

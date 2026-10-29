use dotnet_conn_parser::DotnetConnParser;
use schema_diff::DiffEngine;
use schema_reader::SqlServerReader;

fn required_env(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .filter(|value| !value.trim().is_empty())
}

#[tokio::test]
async fn real_source_and_target_tables_are_read_from_their_configured_databases() {
    let (Some(source_connection), Some(target_connection), Some(source_table), Some(target_table)) = (
        required_env("SCHEMALENS_TEST_SOURCE_CONNECTION_STRING"),
        required_env("SCHEMALENS_TEST_TARGET_CONNECTION_STRING"),
        required_env("SCHEMALENS_TEST_SOURCE_TABLE"),
        required_env("SCHEMALENS_TEST_TARGET_TABLE"),
    ) else {
        eprintln!("Skipping provenance integration: configure test connection strings and distinct expected table identifiers.");
        return;
    };

    let source_info =
        DotnetConnParser::parse(&source_connection).expect("source profile must parse");
    let target_info =
        DotnetConnParser::parse(&target_connection).expect("target profile must parse");
    let source_snapshot = SqlServerReader::read_snapshot(&source_info)
        .await
        .expect("source catalog must be readable");
    let target_snapshot = SqlServerReader::read_snapshot(&target_info)
        .await
        .expect("target catalog must be readable");
    let source_expected = parse_table_id(&source_table);
    let target_expected = parse_table_id(&target_table);

    assert!(source_snapshot
        .tables
        .iter()
        .any(|table| table.id == source_expected));
    assert!(!target_snapshot
        .tables
        .iter()
        .any(|table| table.id == source_expected));
    assert!(target_snapshot
        .tables
        .iter()
        .any(|table| table.id == target_expected));
    assert!(!source_snapshot
        .tables
        .iter()
        .any(|table| table.id == target_expected));

    let result = DiffEngine::compare(&source_snapshot, &target_snapshot);
    assert!(result
        .differences
        .iter()
        .any(|diff| diff.object_name == source_expected.name
            && diff.category == schema_lens_domain::DifferenceCategory::MissingInTarget));
    assert!(result
        .differences
        .iter()
        .any(|diff| diff.object_name == target_expected.name
            && diff.category == schema_lens_domain::DifferenceCategory::MissingInSource));
}

#[tokio::test]
async fn empty_database_produces_no_artificial_tables() {
    let Some(connection) = required_env("SCHEMALENS_TEST_EMPTY_CONNECTION_STRING") else {
        eprintln!("Skipping empty database integration: SCHEMALENS_TEST_EMPTY_CONNECTION_STRING is not configured.");
        return;
    };
    let info = DotnetConnParser::parse(&connection).expect("empty database profile must parse");
    let snapshot = SqlServerReader::read_snapshot(&info)
        .await
        .expect("empty database catalog must be readable");
    assert!(snapshot.tables.is_empty());
}

#[tokio::test]
async fn unavailable_endpoint_returns_an_error_instead_of_a_snapshot() {
    let info = DotnetConnParser::parse("Server=127.0.0.1,1;Database=master;User Id=invalid;Password=invalid;Connect Timeout=1;TrustServerCertificate=True")
        .expect("test connection string must parse");
    assert!(SqlServerReader::read_snapshot(&info).await.is_err());
}

fn parse_table_id(value: &str) -> schema_lens_domain::ObjectId {
    let (schema, table) = value
        .split_once('.')
        .expect("expected table identifier in schema.table form");
    schema_lens_domain::ObjectId::new(schema, table)
}

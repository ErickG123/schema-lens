#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use dotnet_conn_parser::DotnetConnParser;
use schema_diff::DiffEngine;
use schema_lens_app::{
    AppService, DiffItemDto, RunComparisonResult, SaveFileResult, TestConnectionPayload,
    TestConnectionResult,
};
use schema_reader::SqlServerReader;
use serde::Serialize;
use sql_generator::SqlGenerator;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Mutex,
};
use tauri::State;

#[derive(Default)]
struct AppState {
    latest_comparison: Mutex<Option<schema_lens_domain::ComparisonRunResult>>,
    generation: AtomicU64,
}

#[derive(Serialize)]
pub struct HealthStatus {
    pub status: String,
    pub version: String,
    pub platform: String,
}

#[tauri::command]
fn health_check() -> HealthStatus {
    HealthStatus {
        status: "healthy".into(),
        version: env!("CARGO_PKG_VERSION").into(),
        platform: std::env::consts::OS.into(),
    }
}

#[tauri::command]
async fn test_connection(payload: TestConnectionPayload) -> Result<TestConnectionResult, String> {
    if payload.mode == "file" {
        return AppService::test_connection(payload).await;
    }
    let identity = SqlServerReader::identify_connection(&connection_info(&payload)?).await?;
    tracing::info!(server = %identity.server_name, database = %identity.database_name, "SQL Server connection identity confirmed");
    Ok(TestConnectionResult {
        success: true,
        message: format!(
            "Connected to {} / {}",
            identity.server_name, identity.database_name
        ),
        server_name: Some(identity.server_name),
        database_name: Some(identity.database_name),
        machine_name: identity.machine_name,
    })
}

#[tauri::command]
async fn run_comparison(
    source: TestConnectionPayload,
    target: TestConnectionPayload,
    state: State<'_, AppState>,
) -> Result<RunComparisonResult, String> {
    let generation = state.generation.fetch_add(1, Ordering::SeqCst) + 1;
    *state
        .latest_comparison
        .lock()
        .map_err(|_| "Comparison state is unavailable".to_string())? = None;
    let source_snapshot = SqlServerReader::read_snapshot(&connection_info(&source)?).await?;
    let target_snapshot = SqlServerReader::read_snapshot(&connection_info(&target)?).await?;
    let source_identity = format!(
        "{} / {}",
        source_snapshot.server_name, source_snapshot.database_name
    );
    let target_identity = format!(
        "{} / {}",
        target_snapshot.server_name, target_snapshot.database_name
    );
    let source_tables = source_snapshot.tables.clone();
    let target_tables = target_snapshot.tables.clone();
    let result = DiffEngine::compare(&source_snapshot, &target_snapshot);
    let run_id = result.run_id.clone();
    tracing::info!(run_id = %run_id, source = %source_identity, target = %target_identity, source_tables = source_snapshot.tables.len(), target_tables = target_snapshot.tables.len(), differences = result.differences.len(), "SQL Server comparison completed");
    let diffs = result
        .differences
        .iter()
        .cloned()
        .map(|d| DiffItemDto {
            object_type: Some(format!("{:?}", d.object_type)),
            severity: match d.severity {
                schema_lens_domain::DifferenceSeverity::Critical => "CR\u{00cd}TICA",
                schema_lens_domain::DifferenceSeverity::Medium => "M\u{00c9}DIA",
                _ => "BAIXA",
            }
            .into(),
            schema: d.schema_name,
            object_name: d.object_name,
            category: match d.category {
                schema_lens_domain::DifferenceCategory::MissingInTarget => "Ausente no Destino",
                schema_lens_domain::DifferenceCategory::MissingInSource => "Ausente na Origem",
                schema_lens_domain::DifferenceCategory::PropertyMismatch => {
                    "Diverg\u{00ea}ncia de Propriedade"
                }
                schema_lens_domain::DifferenceCategory::DefinitionMismatch => {
                    "Diverg\u{00ea}ncia de Defini\u{00e7}\u{00e3}o"
                }
                schema_lens_domain::DifferenceCategory::Indeterminate => "Indeterminado",
                _ => "Diferen\u{00e7}a",
            }
            .into(),
            property_name: d.property_name,
            value_source: d.value_source.unwrap_or_else(|| "Absent".into()),
            value_target: d.value_target.unwrap_or_else(|| "Absent".into()),
            details: d.explanation,
            risk_assessment: d.risk_assessment,
            confidence: Some(format!("{:?}", d.confidence).to_lowercase()),
        })
        .collect::<Vec<_>>();
    if state.generation.load(Ordering::SeqCst) == generation {
        *state
            .latest_comparison
            .lock()
            .map_err(|_| "Comparison state is unavailable".to_string())? = Some(result);
    }
    Ok(RunComparisonResult {
        success: true,
        message: format!(
            "Comparison {run_id} completed: {} difference(s).",
            diffs.len()
        ),
        comparison_id: Some(run_id),
        source_identity: Some(source_identity),
        target_identity: Some(target_identity),
        diffs,
        source_tables,
        target_tables,
    })
}

fn connection_info(
    payload: &TestConnectionPayload,
) -> Result<dotnet_conn_parser::ParsedConnectionInfo, String> {
    if payload.mode != "online" {
        return Err("File-based comparison is not available in this flow.".into());
    }
    let server = payload
        .server
        .as_deref()
        .filter(|s| !s.trim().is_empty())
        .ok_or("Server is required")?;
    let raw_server = if server.contains(',') || server.contains('\\') {
        server.to_string()
    } else {
        format!("{},{}", server, payload.port.unwrap_or(1433))
    };
    let database = payload
        .database
        .as_deref()
        .filter(|s| !s.trim().is_empty())
        .ok_or("Database is required")?
        .replace('\'', "''");
    let auth = if payload.auth_type.as_deref() == Some("windows") {
        "Integrated Security=true".to_string()
    } else {
        format!(
            "User Id='{}';Password='{}'",
            payload
                .username
                .as_deref()
                .unwrap_or("")
                .replace('\'', "''"),
            payload
                .password
                .as_deref()
                .unwrap_or("")
                .replace('\'', "''")
        )
    };
    let trust = if payload.trust_server_certificate.unwrap_or(false) {
        "true"
    } else {
        "false"
    };
    DotnetConnParser::parse(&format!(
        "Server='{}';Database='{}';{};TrustServerCertificate={};Encrypt=true",
        raw_server.replace('\'', "''"),
        database,
        auth,
        trust
    ))
}

#[tauri::command]
async fn generate_migration_sql(state: State<'_, AppState>) -> Result<String, String> {
    let latest = state
        .latest_comparison
        .lock()
        .map_err(|_| "Comparison state is unavailable".to_string())?
        .clone()
        .ok_or_else(|| "Run a real comparison before generating a script.".to_string())?;
    if latest.differences.is_empty() {
        return Err("The current comparison has no differences to export.".into());
    }
    Ok(SqlGenerator::plan_and_generate(&latest.differences, &latest.target_database).script_sql)
}

#[tauri::command]
async fn save_exported_file(path: String, content: String) -> Result<SaveFileResult, String> {
    AppService::write_exported_file(&path, &content)
}

fn main() {
    tracing_subscriber::fmt::init();
    tauri::Builder::default()
        .manage(AppState::default())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            health_check,
            test_connection,
            run_comparison,
            generate_migration_sql,
            save_exported_file
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Tauri application");
}

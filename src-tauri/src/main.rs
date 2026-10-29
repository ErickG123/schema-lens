#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use dotnet_conn_parser::DotnetConnParser;
use local_storage::{ConnectionSelection, ProfileStore, SavedProfile};
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
use tauri::{Manager, State};

#[derive(Clone)]
struct LatestComparison {
    result: schema_lens_domain::ComparisonRunResult,
    source_snapshot: schema_lens_domain::DatabaseSnapshot,
    target_snapshot: schema_lens_domain::DatabaseSnapshot,
}

struct AppState {
    latest_comparison: Mutex<Option<LatestComparison>>,
    generation: AtomicU64,
    profile_store: Mutex<ProfileStore>,
}

impl AppState {
    fn new(profile_store: ProfileStore) -> Self {
        Self {
            latest_comparison: Mutex::new(None),
            generation: AtomicU64::new(0),
            profile_store: Mutex::new(profile_store),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SavedConnectionDto {
    id: String,
    name: String,
    server: String,
    port: Option<u16>,
    instance: Option<String>,
    database: Option<String>,
    auth_type: String,
    username: Option<String>,
    trust_server_certificate: bool,
    connect_timeout_seconds: u32,
    sql_file_path: Option<String>,
    created_at_utc: String,
    has_stored_credential: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SavedConnectionSettingsDto {
    connections: Vec<SavedConnectionDto>,
    selection: ConnectionSelection,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct SaveConnectionPayload {
    profile: SavedProfile,
    password: Option<String>,
    #[serde(default)]
    remove_password: bool,
}

fn connection_dto(
    store: &ProfileStore,
    profile: SavedProfile,
) -> Result<SavedConnectionDto, String> {
    let has_stored_credential = store.has_profile_credential(&profile.id)?;
    Ok(SavedConnectionDto {
        id: profile.id,
        name: profile.name,
        server: profile.server,
        port: profile.port,
        instance: profile.instance,
        database: profile.database,
        auth_type: profile.auth_type,
        username: profile.username,
        trust_server_certificate: profile.trust_server_certificate,
        connect_timeout_seconds: profile.connect_timeout_seconds,
        sql_file_path: profile.sql_file_path,
        created_at_utc: profile.created_at_utc,
        has_stored_credential,
    })
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
async fn test_connection(
    mut payload: TestConnectionPayload,
    credential_ref: Option<String>,
    state: State<'_, AppState>,
) -> Result<TestConnectionResult, String> {
    if payload.mode == "file" {
        return AppService::test_connection(payload).await;
    }
    if payload.password.as_deref().unwrap_or_default().is_empty() {
        if let Some(credential_ref) = credential_ref {
            let store = state
                .profile_store
                .lock()
                .map_err(|_| "Connection settings are unavailable".to_string())?;
            payload.password = store
                .get_profile_for_connection(&credential_ref)?
                .and_then(|(_, password)| password);
        }
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
fn load_connection_settings(
    state: State<'_, AppState>,
) -> Result<SavedConnectionSettingsDto, String> {
    let store = state
        .profile_store
        .lock()
        .map_err(|_| "Connection settings are unavailable".to_string())?;
    let connections = store
        .list_profiles()?
        .into_iter()
        .map(|profile| connection_dto(&store, profile))
        .collect::<Result<Vec<_>, _>>()?;
    let selection = store.get_selection()?;
    Ok(SavedConnectionSettingsDto {
        connections,
        selection,
    })
}

#[tauri::command]
fn save_connection(
    payload: SaveConnectionPayload,
    state: State<'_, AppState>,
) -> Result<SavedConnectionDto, String> {
    let mut store = state
        .profile_store
        .lock()
        .map_err(|_| "Connection settings are unavailable".to_string())?;
    let has_credential = store.has_profile_credential(&payload.profile.id)?;
    if payload.profile.auth_type == "sql"
        && payload.password.as_deref().unwrap_or_default().is_empty()
        && (payload.remove_password || !has_credential)
    {
        return Err("Enter a SQL password or keep the password already stored.".into());
    }
    store.save_profile(
        &payload.profile,
        if payload.profile.auth_type == "sql" {
            payload.password.as_deref()
        } else {
            None
        },
        payload.remove_password || payload.profile.auth_type != "sql",
    )?;
    connection_dto(&store, payload.profile)
}

#[tauri::command]
fn delete_connection(
    connection_id: String,
    state: State<'_, AppState>,
) -> Result<ConnectionSelection, String> {
    let mut store = state
        .profile_store
        .lock()
        .map_err(|_| "Connection settings are unavailable".to_string())?;
    store.delete_profile(&connection_id)?;
    store.get_selection()
}

#[tauri::command]
fn save_connection_selection(
    selection: ConnectionSelection,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state
        .profile_store
        .lock()
        .map_err(|_| "Connection settings are unavailable".to_string())?
        .save_selection(&selection)
}

#[tauri::command]
async fn test_saved_connection(
    connection_id: String,
    state: State<'_, AppState>,
) -> Result<TestConnectionResult, String> {
    let payload = {
        let store = state
            .profile_store
            .lock()
            .map_err(|_| "Connection settings are unavailable".to_string())?;
        saved_profile_payload(&store, &connection_id)?
    };
    if payload.mode == "file" {
        return AppService::test_connection(payload).await;
    }
    let identity = SqlServerReader::identify_connection(&connection_info(&payload)?).await?;
    tracing::info!(server = %identity.server_name, database = %identity.database_name, "Saved SQL Server connection identity confirmed");
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
    source_connection_id: String,
    target_connection_id: String,
    state: State<'_, AppState>,
) -> Result<RunComparisonResult, String> {
    let generation = state.generation.fetch_add(1, Ordering::SeqCst) + 1;
    *state
        .latest_comparison
        .lock()
        .map_err(|_| "Comparison state is unavailable".to_string())? = None;
    let (source, target) = {
        let store = state
            .profile_store
            .lock()
            .map_err(|_| "Connection settings are unavailable".to_string())?;
        (
            saved_profile_payload(&store, &source_connection_id)?,
            saved_profile_payload(&store, &target_connection_id)?,
        )
    };
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
        .map(|d| {
            let generated = SqlGenerator::generate_for_difference(
                &d,
                &target_snapshot.database_name,
                &source_snapshot,
                &target_snapshot,
            );
            let (generated_sql, unavailable_reason, sql_status, is_destructive) = match generated {
                Ok(operation) => (
                    Some(operation.sql),
                    None,
                    "available".to_string(),
                    operation.is_destructive,
                ),
                Err(reason) => (
                    None,
                    Some(reason),
                    if !d.can_generate_sql {
                        "unsupported".to_string()
                    } else {
                        "unavailable".to_string()
                    },
                    false,
                ),
            };
            let operation_type = d.operation_type().as_str().to_string();
            let object_type_code = d.object_type.as_str().to_string();
            let difference_kind = d.category.as_str().to_string();
            let risk_level = match d.severity {
                schema_lens_domain::DifferenceSeverity::Critical => "critical",
                schema_lens_domain::DifferenceSeverity::Medium => "medium",
                schema_lens_domain::DifferenceSeverity::Low => "low",
                schema_lens_domain::DifferenceSeverity::Info => "unclassified",
            }
            .to_string();
            DiffItemDto {
                id: d.id,
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
                operation_type,
                object_type_code,
                difference_kind,
                risk_level,
                property_name: d.property_name,
                value_source: d.value_source.unwrap_or_else(|| "Absent".into()),
                value_target: d.value_target.unwrap_or_else(|| "Absent".into()),
                details: d.explanation,
                risk_assessment: d.risk_assessment,
                confidence: Some(format!("{:?}", d.confidence).to_lowercase()),
                generated_sql,
                sql_unavailable_reason: unavailable_reason,
                sql_status,
                is_destructive,
            }
        })
        .collect::<Vec<_>>();
    if state.generation.load(Ordering::SeqCst) == generation {
        *state
            .latest_comparison
            .lock()
            .map_err(|_| "Comparison state is unavailable".to_string())? = Some(LatestComparison {
            result: result.clone(),
            source_snapshot: source_snapshot.clone(),
            target_snapshot: target_snapshot.clone(),
        });
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

fn saved_profile_payload(
    store: &ProfileStore,
    connection_id: &str,
) -> Result<TestConnectionPayload, String> {
    let (profile, password) = store
        .get_profile_for_connection(connection_id)?
        .ok_or_else(|| "Selected connection no longer exists".to_string())?;
    Ok(TestConnectionPayload {
        mode: if profile.auth_type == "sql_file" {
            "file".into()
        } else {
            "online".into()
        },
        server: Some(profile.server),
        port: profile.port,
        database: profile.database,
        auth_type: Some(profile.auth_type),
        username: profile.username,
        password,
        trust_server_certificate: Some(profile.trust_server_certificate),
        file_path: profile.sql_file_path,
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
    if latest.result.differences.is_empty() {
        return Err("The current comparison has no differences to export.".into());
    }
    let plan = SqlGenerator::plan_and_generate_with_snapshots(
        &latest.result.differences,
        &latest.target_snapshot.database_name,
        Some(&latest.source_snapshot),
        Some(&latest.target_snapshot),
    );
    if !plan.unsupported_operations.is_empty() {
        return Err(format!(
            "Complete SQL export is unavailable because some operations are unsupported: {}",
            plan.unsupported_operations.join("; ")
        ));
    }
    if plan.steps.is_empty() {
        return Err("No supported SQL operations are available for export.".into());
    }
    Ok(plan.script_sql)
}

#[tauri::command]
async fn save_exported_file(path: String, content: String) -> Result<SaveFileResult, String> {
    AppService::write_exported_file(&path, &content)
}

fn main() {
    tracing_subscriber::fmt::init();
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&app_data_dir)?;
            let profile_store = ProfileStore::new(app_data_dir.join("schemalens.sqlite"))
                .map_err(std::io::Error::other)?;
            app.manage(AppState::new(profile_store));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            health_check,
            test_connection,
            load_connection_settings,
            save_connection,
            delete_connection,
            save_connection_selection,
            test_saved_connection,
            run_comparison,
            generate_migration_sql,
            save_exported_file
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Tauri application");
}

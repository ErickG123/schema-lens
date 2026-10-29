use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct SaveFilePayload {
    #[serde(rename = "defaultPath")]
    pub default_path: String,
    pub title: String,
    pub filter_name: String,
    pub filter_extension: String,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SaveFileResult {
    pub saved: bool,
    pub path: Option<String>,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TestConnectionPayload {
    pub mode: String,
    #[serde(rename = "server", default)]
    pub server: Option<String>,
    #[serde(rename = "port", default)]
    pub port: Option<u16>,
    #[serde(rename = "database", default)]
    pub database: Option<String>,
    #[serde(rename = "authType", default)]
    pub auth_type: Option<String>,
    #[serde(rename = "username", default)]
    pub username: Option<String>,
    #[serde(rename = "password", default)]
    pub password: Option<String>,
    #[serde(rename = "trustServerCertificate", default)]
    pub trust_server_certificate: Option<bool>,
    #[serde(rename = "filePath", default)]
    pub file_path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TestConnectionResult {
    pub success: bool,
    pub message: String,
    #[serde(default, rename = "serverName")]
    pub server_name: Option<String>,
    #[serde(default, rename = "databaseName")]
    pub database_name: Option<String>,
    #[serde(default, rename = "machineName")]
    pub machine_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RunComparisonResult {
    pub success: bool,
    pub message: String,
    #[serde(default, rename = "comparisonId")]
    pub comparison_id: Option<String>,
    #[serde(default, rename = "sourceIdentity")]
    pub source_identity: Option<String>,
    #[serde(default, rename = "targetIdentity")]
    pub target_identity: Option<String>,
    pub diffs: Vec<DiffItemDto>,
    #[serde(default, rename = "sourceTables")]
    pub source_tables: Vec<schema_lens_domain::TableMetadata>,
    #[serde(default, rename = "targetTables")]
    pub target_tables: Vec<schema_lens_domain::TableMetadata>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DiffItemDto {
    #[serde(default, rename = "objectType")]
    pub object_type: Option<String>,
    pub severity: String,
    pub schema: String,
    #[serde(rename = "objectName")]
    pub object_name: String,
    pub category: String,
    #[serde(default, rename = "propertyName")]
    pub property_name: Option<String>,
    #[serde(rename = "valueSource")]
    pub value_source: String,
    #[serde(rename = "valueTarget")]
    pub value_target: String,
    pub details: String,
    #[serde(default, rename = "riskAssessment")]
    pub risk_assessment: Option<String>,
    #[serde(default)]
    pub confidence: Option<String>,
}

pub struct AppService;

impl AppService {
    pub async fn test_connection(
        payload: TestConnectionPayload,
    ) -> Result<TestConnectionResult, String> {
        if payload.mode != "file" {
            return Err(
                "SQL Server connection checks are handled by the Tauri SQL reader.".to_string(),
            );
        }
        let path = payload.file_path.unwrap_or_default();
        if path.is_empty() {
            return Err("DDL file path was not provided".to_string());
        }
        if Path::new(&path).exists() {
            Ok(TestConnectionResult {
                success: true,
                message: "DDL file found.".to_string(),
                server_name: None,
                database_name: None,
                machine_name: None,
            })
        } else {
            Err("DDL file was not found.".to_string())
        }
    }
    pub fn write_exported_file(target_path: &str, content: &str) -> Result<SaveFileResult, String> {
        if target_path.is_empty() {
            return Ok(SaveFileResult {
                saved: false,
                path: None,
                message: "Operação cancelada pelo usuário.".to_string(),
            });
        }

        let path_obj = Path::new(target_path);
        if let Some(parent) = path_obj.parent() {
            if !parent.as_os_str().is_empty() && !parent.exists() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("Falha ao criar diretório: {}", e))?;
            }
        }

        fs::write(path_obj, content.as_bytes())
            .map_err(|e| format!("Falha ao gravar arquivo '{}': {}", target_path, e))?;

        Ok(SaveFileResult {
            saved: true,
            path: Some(target_path.to_string()),
            message: format!("Arquivo gravado com sucesso em '{}'.", target_path),
        })
    }
}

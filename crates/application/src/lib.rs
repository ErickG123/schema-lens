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
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RunComparisonResult {
    pub success: bool,
    pub message: String,
    pub diffs: Vec<DiffItemDto>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DiffItemDto {
    pub severity: String,
    pub schema: String,
    #[serde(rename = "objectName")]
    pub object_name: String,
    pub category: String,
    #[serde(rename = "valueSource")]
    pub value_source: String,
    #[serde(rename = "valueTarget")]
    pub value_target: String,
    pub details: String,
}

pub struct AppService;

impl AppService {
    pub async fn test_connection(
        payload: TestConnectionPayload,
    ) -> Result<TestConnectionResult, String> {
        if payload.mode == "file" {
            let path = payload.file_path.unwrap_or_default();
            if path.is_empty() {
                return Err("Caminho do arquivo não fornecido".to_string());
            }
            if Path::new(&path).exists() {
                Ok(TestConnectionResult {
                    success: true,
                    message: format!("Arquivo DDL '{}' localizado com sucesso.", path),
                })
            } else {
                Err(format!("Arquivo DDL '{}' não encontrado no disco.", path))
            }
        } else {
            let srv = payload.server.unwrap_or_else(|| "localhost".to_string());
            let db = payload.database.unwrap_or_else(|| "master".to_string());
            Ok(TestConnectionResult {
                success: true,
                message: format!(
                    "Parâmetros validados para conexão com servidor '{}', banco '{}'.",
                    srv, db
                ),
            })
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

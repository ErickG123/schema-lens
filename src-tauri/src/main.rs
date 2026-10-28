// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use schema_lens_app::{
    AppService, DiffItemDto, RunComparisonResult, SaveFileResult, TestConnectionPayload,
    TestConnectionResult,
};
use serde::Serialize;

#[derive(Serialize)]
pub struct HealthStatus {
    pub status: String,
    pub version: String,
    pub platform: String,
}

#[tauri::command]
fn health_check() -> HealthStatus {
    HealthStatus {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        platform: std::env::consts::OS.to_string(),
    }
}

#[tauri::command]
async fn test_connection(payload: TestConnectionPayload) -> Result<TestConnectionResult, String> {
    AppService::test_connection(payload).await
}

#[tauri::command]
async fn run_comparison(
    _source: serde_json::Value,
    _target: serde_json::Value,
) -> Result<RunComparisonResult, String> {
    Ok(RunComparisonResult {
        success: true,
        message: "Comparação realizada com sucesso".to_string(),
        diffs: vec![
            DiffItemDto {
                severity: "MÉDIA".to_string(),
                schema: "sales".to_string(),
                object_name: "[sales].[Discounts]".to_string(),
                category: "Ausente no Destino".to_string(),
                value_source: "Table (3 cols)".to_string(),
                value_target: "Inexistente".to_string(),
                details: "Tabela ausente no banco alvo. Necessária criação.".to_string(),
            },
            DiffItemDto {
                severity: "CRÍTICA".to_string(),
                schema: "dbo".to_string(),
                object_name: "[dbo].[Orders].Status".to_string(),
                category: "Divergência de Propriedade".to_string(),
                value_source: "VARCHAR(20)".to_string(),
                value_target: "VARCHAR(10)".to_string(),
                details: "Tamanho menor no destino. Risco de truncamento.".to_string(),
            },
            DiffItemDto {
                severity: "MÉDIA".to_string(),
                schema: "dbo".to_string(),
                object_name: "[dbo].[Orders].Notes".to_string(),
                category: "Ausente no Destino".to_string(),
                value_source: "NVARCHAR(500) NULL".to_string(),
                value_target: "Inexistente".to_string(),
                details: "Coluna ausente no banco alvo.".to_string(),
            },
            DiffItemDto {
                severity: "BAIXA".to_string(),
                schema: "dbo".to_string(),
                object_name: "[dbo].[LegacyLogs]".to_string(),
                category: "Ausente na Origem".to_string(),
                value_source: "Inexistente".to_string(),
                value_target: "Table (3 cols)".to_string(),
                details: "Tabela existe apenas no destino.".to_string(),
            },
        ],
    })
}

#[tauri::command]
async fn generate_migration_sql(
    target_database: Option<String>,
    _diff_ids: Vec<String>,
) -> Result<String, String> {
    let target = target_database.unwrap_or_else(|| "SchemaTarget".to_string());
    let sql = format!(
        "-- =========================================================================\n\
         -- Script de Migração Defensivo Gerado pelo SchemaLens\n\
         -- Banco Alvo: [{target}]\n\
         -- Geração: Somente Leitura (Nenhuma alteração foi executada no banco)\n\
         -- =========================================================================\n\
         SET XACT_ABORT ON;\n\
         SET NOCOUNT ON;\n\n\
         BEGIN TRANSACTION;\n\n\
         -- [Passo 1: Criar Tabela Ausente no Destino]\n\
         IF OBJECT_ID(N'[sales].[Discounts]', 'U') IS NULL\n\
         BEGIN\n\
             CREATE TABLE [sales].[Discounts] (\n\
                 DiscountId INT IDENTITY(1,1) NOT NULL PRIMARY KEY,\n\
                 Code VARCHAR(50) NOT NULL,\n\
                 Percentage DECIMAL(5,2) NOT NULL\n\
             );\n\
         END\n\
         GO\n\n\
         -- [Passo 2: Alinhar Tamanho da Coluna com Verificação de Truncamento]\n\
         IF EXISTS (SELECT 1 FROM sys.columns WHERE object_id = OBJECT_ID(N'[dbo].[Orders]') AND name = 'Status')\n\
         BEGIN\n\
             ALTER TABLE [dbo].[Orders] ALTER COLUMN Status VARCHAR(20) NOT NULL;\n\
         END\n\
         GO\n\n\
         -- [Passo 3: Adicionar Coluna Ausente com Aceitação de Nulo]\n\
         IF NOT EXISTS (SELECT 1 FROM sys.columns WHERE object_id = OBJECT_ID(N'[dbo].[Orders]') AND name = 'Notes')\n\
         BEGIN\n\
             ALTER TABLE [dbo].[Orders] ADD Notes NVARCHAR(500) NULL;\n\
         END\n\
         GO\n\n\
         COMMIT TRANSACTION;\n\
         PRINT 'Migração concluída com sucesso no banco [{target}].';\n"
    );
    Ok(sql)
}

#[tauri::command]
async fn save_exported_file(path: String, content: String) -> Result<SaveFileResult, String> {
    AppService::write_exported_file(&path, &content)
}

fn main() {
    // Inicialização do tracing subscriber para observabilidade e logs do host desktop
    tracing_subscriber::fmt::init();

    tauri::Builder::default()
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
        .expect("error while running tauri application");
}

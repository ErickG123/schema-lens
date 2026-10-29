use schema_lens_app::{AppService, TestConnectionPayload};
use std::env;
use std::fs;

#[tokio::test]
async fn test_app_service_test_connection_file_mode() {
    let temp_path = env::temp_dir().join(format!("test_schema_{}.sql", uuid::Uuid::new_v4()));
    fs::write(&temp_path, "CREATE TABLE Dummy (Id INT);").expect("Failed to write temp sql");

    let payload = TestConnectionPayload {
        mode: "file".to_string(),
        server: None,
        port: None,
        database: None,
        auth_type: None,
        username: None,
        password: None,
        trust_server_certificate: None,
        file_path: Some(temp_path.to_str().unwrap().to_string()),
    };

    let res = AppService::test_connection(payload)
        .await
        .expect("File mode test connection should succeed");
    assert!(res.success);
    assert!(res.message.contains("DDL file found"));

    let _ = fs::remove_file(&temp_path);
}

#[tokio::test]
async fn test_app_service_test_connection_file_missing_fails() {
    let payload = TestConnectionPayload {
        mode: "file".to_string(),
        server: None,
        port: None,
        database: None,
        auth_type: None,
        username: None,
        password: None,
        trust_server_certificate: None,
        file_path: Some("C:\\non_existent_schema_xyz_123.sql".to_string()),
    };

    let res = AppService::test_connection(payload).await;
    assert!(res.is_err());
    assert!(res.unwrap_err().contains("not found"));
}

#[tokio::test]
async fn test_app_service_test_connection_sql_mode() {
    let payload = TestConnectionPayload {
        mode: "online".to_string(),
        server: Some("localhost".to_string()),
        port: Some(1433),
        database: Some("SchemaTarget".to_string()),
        auth_type: Some("sql".to_string()),
        username: Some("sa".to_string()),
        password: Some("Secret".to_string()),
        trust_server_certificate: Some(true),
        file_path: None,
    };

    let res = AppService::test_connection(payload).await;
    assert!(res.is_err());
}

#[test]
fn test_app_service_write_exported_file() {
    let temp_path = env::temp_dir().join(format!("test_export_{}.json", uuid::Uuid::new_v4()));
    let content = "{\"status\": \"OK\"}";

    let res = AppService::write_exported_file(temp_path.to_str().unwrap(), content)
        .expect("Write file should succeed");
    assert!(res.saved);
    assert!(temp_path.exists());

    let read_back = fs::read_to_string(&temp_path).expect("Failed to read back file");
    assert_eq!(read_back, content);

    let _ = fs::remove_file(&temp_path);
}

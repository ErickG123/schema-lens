use local_storage::{ProfileStore, SavedProfile};

#[test]
fn test_sqlite_profile_crud_and_credential_isolation() {
    let store = ProfileStore::new(":memory:").expect("Failed to initialize in-memory SQLite");

    let profile1 = SavedProfile {
        id: "prof-prod-01".to_string(),
        name: "Producao Primary".to_string(),
        server: "sql-cluster.corp.internal".to_string(),
        port: Some(1433),
        instance: Some("MSSQLSERVER".to_string()),
        database: Some("EnterpriseDb".to_string()),
        auth_type: "sql".to_string(),
        username: Some("lens_reader".to_string()),
        trust_server_certificate: false,
        connect_timeout_seconds: 30,
        sql_file_path: None,
        created_at_utc: "2026-10-27T12:00:00Z".to_string(),
    };

    let profile2 = SavedProfile {
        id: "prof-stage-02".to_string(),
        name: "Stage Offline DDL".to_string(),
        server: "localhost".to_string(),
        port: None,
        instance: None,
        database: None,
        auth_type: "sql_file".to_string(),
        username: None,
        trust_server_certificate: true,
        connect_timeout_seconds: 10,
        sql_file_path: Some("C:\\schemas\\staging.sql".to_string()),
        created_at_utc: "2026-10-27T12:30:00Z".to_string(),
    };

    // Salvar perfis
    store
        .save_profile(&profile1, None)
        .expect("Failed to save profile 1");
    store
        .save_profile(&profile2, None)
        .expect("Failed to save profile 2");

    // Listar perfis
    let profiles = store.list_profiles().expect("Failed to list profiles");
    assert_eq!(profiles.len(), 2);

    let p1 = profiles
        .iter()
        .find(|p| p.id == "prof-prod-01")
        .expect("Profile 1 must be present");
    assert_eq!(p1.name, "Producao Primary");
    assert_eq!(p1.auth_type, "sql");
    assert!(!p1.trust_server_certificate);

    let p2 = profiles
        .iter()
        .find(|p| p.id == "prof-stage-02")
        .expect("Profile 2 must be present");
    assert_eq!(p2.auth_type, "sql_file");
    assert_eq!(
        p2.sql_file_path.as_deref(),
        Some("C:\\schemas\\staging.sql")
    );

    // Atualização idempotente (upsert)
    let mut updated_p1 = profile1.clone();
    updated_p1.connect_timeout_seconds = 60;
    store
        .save_profile(&updated_p1, None)
        .expect("Failed to update profile 1");

    let profiles_after_update = store
        .list_profiles()
        .expect("Failed to list profiles after update");
    assert_eq!(
        profiles_after_update.len(),
        2,
        "Count must remain 2 after upsert"
    );
    let fetched = profiles_after_update
        .iter()
        .find(|p| p.id == "prof-prod-01")
        .unwrap();
    assert_eq!(fetched.connect_timeout_seconds, 60);

    // Deletar um perfil
    store
        .delete_profile("prof-stage-02")
        .expect("Failed to delete profile 2");
    let profiles_after_delete = store
        .list_profiles()
        .expect("Failed to list profiles after delete");
    assert_eq!(profiles_after_delete.len(), 1);
    assert_eq!(profiles_after_delete[0].id, "prof-prod-01");
}

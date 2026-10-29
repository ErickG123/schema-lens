pub mod exporter;
pub use exporter::*;

use keyring::{Entry, Error as KeyringError};
use rusqlite::{params, Connection, OptionalExtension, Result as SqliteResult};
use serde::{Deserialize, Serialize};
use std::path::Path;

const KEYRING_SERVICE: &str = "SchemaLens_ConnectionProfile";
const SCHEMA_VERSION: i64 = 1;

pub trait CredentialVault: Send + Sync {
    fn set_password(&self, credential_ref: &str, password: &str) -> Result<(), String>;
    fn get_password(&self, credential_ref: &str) -> Result<Option<String>, String>;
    fn delete_password(&self, credential_ref: &str) -> Result<(), String>;
}

#[derive(Default)]
pub struct WindowsCredentialVault;

impl CredentialVault for WindowsCredentialVault {
    fn set_password(&self, credential_ref: &str, password: &str) -> Result<(), String> {
        let entry = Entry::new(KEYRING_SERVICE, credential_ref)
            .map_err(|_| "Windows Credential Manager is unavailable".to_string())?;
        entry
            .set_password(password)
            .map_err(|_| "Could not save credential in Windows Credential Manager".to_string())
    }

    fn get_password(&self, credential_ref: &str) -> Result<Option<String>, String> {
        let entry = Entry::new(KEYRING_SERVICE, credential_ref)
            .map_err(|_| "Windows Credential Manager is unavailable".to_string())?;
        match entry.get_password() {
            Ok(password) => Ok(Some(password)),
            Err(KeyringError::NoEntry) => Ok(None),
            Err(_) => Err("Could not read credential from Windows Credential Manager".into()),
        }
    }

    fn delete_password(&self, credential_ref: &str) -> Result<(), String> {
        let entry = Entry::new(KEYRING_SERVICE, credential_ref)
            .map_err(|_| "Windows Credential Manager is unavailable".to_string())?;
        match entry.delete_password() {
            Ok(()) | Err(KeyringError::NoEntry) => Ok(()),
            Err(_) => Err("Could not remove credential from Windows Credential Manager".into()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SavedProfile {
    pub id: String,
    pub name: String,
    pub server: String,
    pub port: Option<u16>,
    pub instance: Option<String>,
    pub database: Option<String>,
    pub auth_type: String, // "sql" | "windows" | "sql_file"
    pub username: Option<String>,
    pub trust_server_certificate: bool,
    pub connect_timeout_seconds: u32,
    pub sql_file_path: Option<String>,
    pub created_at_utc: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionSelection {
    pub source_connection_id: Option<String>,
    pub target_connection_id: Option<String>,
}

pub struct ProfileStore {
    conn: Connection,
    credential_vault: Box<dyn CredentialVault>,
}

impl ProfileStore {
    pub fn new(db_path: impl AsRef<Path>) -> SqliteResult<Self> {
        Self::with_vault(db_path, Box::<WindowsCredentialVault>::default())
    }

    pub fn with_vault(
        db_path: impl AsRef<Path>,
        credential_vault: Box<dyn CredentialVault>,
    ) -> SqliteResult<Self> {
        let conn = Connection::open(db_path)?;
        let current_version: i64 =
            conn.pragma_query_value(None, "user_version", |row| row.get(0))?;
        if current_version > SCHEMA_VERSION {
            return Err(rusqlite::Error::InvalidQuery);
        }
        if current_version < 1 {
            conn.execute_batch(
                "BEGIN IMMEDIATE;
            CREATE TABLE IF NOT EXISTS profiles (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                server TEXT NOT NULL,
                port INTEGER,
                instance TEXT,
                database TEXT,
                auth_type TEXT NOT NULL,
                username TEXT,
                trust_server_certificate INTEGER NOT NULL,
                connect_timeout_seconds INTEGER NOT NULL,
                sql_file_path TEXT,
                created_at_utc TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS connection_selections (
                singleton_id INTEGER PRIMARY KEY CHECK (singleton_id = 1),
                source_connection_id TEXT NULL,
                target_connection_id TEXT NULL,
                updated_at_utc TEXT NOT NULL
            );
            PRAGMA user_version = 1;
            COMMIT;",
            )?;
        }
        Ok(Self {
            conn,
            credential_vault,
        })
    }

    pub fn save_profile(
        &mut self,
        profile: &SavedProfile,
        password: Option<&str>,
        remove_credential: bool,
    ) -> Result<(), String> {
        validate_profile(profile)?;
        let replacement = password.filter(|value| !value.is_empty());
        let previous_credential = if replacement.is_some() || remove_credential {
            self.credential_vault.get_password(&profile.id)?
        } else {
            None
        };
        if let Some(password) = replacement {
            self.credential_vault.set_password(&profile.id, password)?;
        } else if remove_credential {
            self.credential_vault.delete_password(&profile.id)?;
        }

        let save_result = (|| {
            let tx = self
                .conn
                .transaction()
                .map_err(|_| "Could not begin connection settings transaction".to_string())?;
            tx.execute(
                "INSERT OR REPLACE INTO profiles
                (id, name, server, port, instance, database, auth_type, username, trust_server_certificate, connect_timeout_seconds, sql_file_path, created_at_utc)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
                params![
                    profile.id,
                    profile.name,
                    profile.server,
                    profile.port,
                    profile.instance,
                    profile.database,
                    profile.auth_type,
                    profile.username,
                    if profile.trust_server_certificate { 1 } else { 0 },
                    profile.connect_timeout_seconds,
                    profile.sql_file_path,
                    profile.created_at_utc
                ],
            )
            .map_err(|_| "Could not save connection settings to SQLite".to_string())?;
            tx.commit()
                .map_err(|_| "Could not commit connection settings".to_string())
        })();

        if let Err(error) = save_result {
            if let Err(rollback_error) =
                self.restore_credential(&profile.id, previous_credential.as_deref())
            {
                return Err(format!(
                    "{error}; credential rollback failed: {rollback_error}"
                ));
            }
            return Err(error);
        }
        Ok(())
    }

    pub fn get_profile(&self, profile_id: &str) -> Result<Option<SavedProfile>, String> {
        self.conn
            .query_row(
                "SELECT id, name, server, port, instance, database, auth_type, username, trust_server_certificate, connect_timeout_seconds, sql_file_path, created_at_utc FROM profiles WHERE id = ?1",
                params![profile_id],
                |row| {
                    let trust_int: i32 = row.get(8)?;
                    Ok(SavedProfile {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        server: row.get(2)?,
                        port: row.get(3)?,
                        instance: row.get(4)?,
                        database: row.get(5)?,
                        auth_type: row.get(6)?,
                        username: row.get(7)?,
                        trust_server_certificate: trust_int == 1,
                        connect_timeout_seconds: row.get(9)?,
                        sql_file_path: row.get(10)?,
                        created_at_utc: row.get(11)?,
                    })
                },
            )
            .optional()
            .map_err(|_| "Could not load saved connection".to_string())
    }

    pub fn get_profile_for_connection(
        &self,
        profile_id: &str,
    ) -> Result<Option<(SavedProfile, Option<String>)>, String> {
        let Some(profile) = self.get_profile(profile_id)? else {
            return Ok(None);
        };
        let password = self.credential_vault.get_password(profile_id)?;
        Ok(Some((profile, password)))
    }

    pub fn has_profile_credential(&self, profile_id: &str) -> Result<bool, String> {
        self.credential_vault
            .get_password(profile_id)
            .map(|secret| secret.is_some())
    }

    pub fn delete_profile(&mut self, profile_id: &str) -> Result<(), String> {
        let previous_credential = self.credential_vault.get_password(profile_id)?;
        self.credential_vault.delete_password(profile_id)?;
        let result = (|| {
            let tx = self
                .conn
                .transaction()
                .map_err(|_| "Could not begin connection deletion transaction".to_string())?;
            tx.execute(
                "UPDATE connection_selections SET source_connection_id = NULL WHERE source_connection_id = ?1",
                params![profile_id],
            )
            .map_err(|_| "Could not clear source connection selection".to_string())?;
            tx.execute(
                "UPDATE connection_selections SET target_connection_id = NULL WHERE target_connection_id = ?1",
                params![profile_id],
            )
            .map_err(|_| "Could not clear target connection selection".to_string())?;
            tx.execute("DELETE FROM profiles WHERE id = ?1", params![profile_id])
                .map_err(|_| "Could not delete connection settings from SQLite".to_string())?;
            tx.commit()
                .map_err(|_| "Could not commit connection deletion".to_string())
        })();
        if let Err(error) = result {
            if let Err(rollback_error) =
                self.restore_credential(profile_id, previous_credential.as_deref())
            {
                return Err(format!(
                    "{error}; credential rollback failed: {rollback_error}"
                ));
            }
            return Err(error);
        }
        Ok(())
    }

    pub fn save_selection(&mut self, selection: &ConnectionSelection) -> Result<(), String> {
        for profile_id in [
            selection.source_connection_id.as_deref(),
            selection.target_connection_id.as_deref(),
        ]
        .into_iter()
        .flatten()
        {
            if self.get_profile(profile_id)?.is_none() {
                return Err("A selected connection does not exist".to_string());
            }
        }
        self.conn.execute(
            "INSERT INTO connection_selections (singleton_id, source_connection_id, target_connection_id, updated_at_utc)
             VALUES (1, ?1, ?2, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
             ON CONFLICT(singleton_id) DO UPDATE SET source_connection_id = excluded.source_connection_id,
             target_connection_id = excluded.target_connection_id, updated_at_utc = excluded.updated_at_utc",
            params![selection.source_connection_id, selection.target_connection_id],
        ).map_err(|_| "Could not save selected connections".to_string())?;
        Ok(())
    }

    pub fn get_selection(&self) -> Result<ConnectionSelection, String> {
        self.conn
            .query_row(
                "SELECT source_connection_id, target_connection_id FROM connection_selections WHERE singleton_id = 1",
                [],
                |row| {
                    Ok(ConnectionSelection {
                        source_connection_id: row.get(0)?,
                        target_connection_id: row.get(1)?,
                    })
                },
            )
            .optional()
            .map(|selection| selection.unwrap_or_default())
            .map_err(|_| "Could not load selected connections".to_string())
    }

    fn restore_credential(
        &self,
        credential_ref: &str,
        previous: Option<&str>,
    ) -> Result<(), String> {
        if let Some(previous) = previous {
            self.credential_vault.set_password(credential_ref, previous)
        } else {
            self.credential_vault.delete_password(credential_ref)
        }
    }

    pub fn list_profiles(&self) -> Result<Vec<SavedProfile>, String> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, name, server, port, instance, database, auth_type, username, trust_server_certificate, connect_timeout_seconds, sql_file_path, created_at_utc FROM profiles ORDER BY name ASC")
            .map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map([], |row| {
                let trust_int: i32 = row.get(8)?;
                Ok(SavedProfile {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    server: row.get(2)?,
                    port: row.get(3)?,
                    instance: row.get(4)?,
                    database: row.get(5)?,
                    auth_type: row.get(6)?,
                    username: row.get(7)?,
                    trust_server_certificate: trust_int == 1,
                    connect_timeout_seconds: row.get(9)?,
                    sql_file_path: row.get(10)?,
                    created_at_utc: row.get(11)?,
                })
            })
            .map_err(|e| e.to_string())?;

        let mut list = Vec::new();
        for p in rows.flatten() {
            list.push(p);
        }
        Ok(list)
    }
}

fn validate_profile(profile: &SavedProfile) -> Result<(), String> {
    if profile.id.trim().is_empty() || profile.name.trim().is_empty() {
        return Err("Connection ID and name are required".to_string());
    }
    if profile.auth_type != "sql"
        && profile.auth_type != "windows"
        && profile.auth_type != "sql_file"
    {
        return Err("Unsupported connection authentication type".to_string());
    }
    if profile.auth_type == "sql_file" {
        if profile
            .sql_file_path
            .as_deref()
            .unwrap_or_default()
            .trim()
            .is_empty()
        {
            return Err("DDL file path is required".to_string());
        }
    } else if profile.server.trim().is_empty()
        || profile
            .database
            .as_deref()
            .unwrap_or_default()
            .trim()
            .is_empty()
    {
        return Err("Server and database are required".to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    #[derive(Clone, Default)]
    struct TestCredentialVault(Arc<Mutex<HashMap<String, String>>>);

    impl CredentialVault for TestCredentialVault {
        fn set_password(&self, credential_ref: &str, password: &str) -> Result<(), String> {
            self.0
                .lock()
                .map_err(|_| "test vault unavailable".to_string())?
                .insert(credential_ref.to_string(), password.to_string());
            Ok(())
        }

        fn get_password(&self, credential_ref: &str) -> Result<Option<String>, String> {
            Ok(self
                .0
                .lock()
                .map_err(|_| "test vault unavailable".to_string())?
                .get(credential_ref)
                .cloned())
        }

        fn delete_password(&self, credential_ref: &str) -> Result<(), String> {
            self.0
                .lock()
                .map_err(|_| "test vault unavailable".to_string())?
                .remove(credential_ref);
            Ok(())
        }
    }

    #[test]
    fn test_profile_sqlite_lifecycle() {
        let mut store = ProfileStore::with_vault(":memory:", Box::<TestCredentialVault>::default())
            .expect("Failed to init in-memory sqlite");
        let profile = SavedProfile {
            id: "prof-001".to_string(),
            name: "Homologacao A".to_string(),
            server: "127.0.0.1".to_string(),
            port: Some(14331),
            instance: None,
            database: Some("SchemaSource".to_string()),
            auth_type: "sql".to_string(),
            username: Some("sa".to_string()),
            trust_server_certificate: true,
            connect_timeout_seconds: 15,
            sql_file_path: None,
            created_at_utc: "2026-10-27T20:00:00Z".to_string(),
        };

        store
            .save_profile(&profile, None, false)
            .expect("Save profile failed");
        let list = store.list_profiles().expect("List profiles failed");
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].name, "Homologacao A");
        assert_eq!(list[0].port, Some(14331));

        store.delete_profile("prof-001").expect("Delete failed");
        let list_after = store.list_profiles().expect("List after delete failed");
        assert_eq!(list_after.len(), 0);
    }

    #[test]
    fn selection_is_independent_and_cleared_when_a_profile_is_deleted() {
        let mut store = ProfileStore::with_vault(":memory:", Box::<TestCredentialVault>::default())
            .expect("store should initialize");
        let source = SavedProfile {
            id: "source-id".into(),
            name: "Source".into(),
            server: "source.example".into(),
            port: Some(1433),
            instance: None,
            database: Some("SourceDb".into()),
            auth_type: "windows".into(),
            username: None,
            trust_server_certificate: false,
            connect_timeout_seconds: 15,
            sql_file_path: None,
            created_at_utc: "2026-10-29T00:00:00Z".into(),
        };
        let mut target = source.clone();
        target.id = "target-id".into();
        target.name = "Target".into();
        target.database = Some("TargetDb".into());
        store.save_profile(&source, None, false).unwrap();
        store.save_profile(&target, None, false).unwrap();
        store
            .save_selection(&ConnectionSelection {
                source_connection_id: Some(source.id),
                target_connection_id: Some(target.id.clone()),
            })
            .unwrap();

        assert_eq!(
            store.get_selection().unwrap().target_connection_id,
            Some(target.id.clone())
        );
        store.delete_profile(&target.id).unwrap();
        assert_eq!(store.get_selection().unwrap().target_connection_id, None);
        assert_eq!(
            store
                .get_selection()
                .unwrap()
                .source_connection_id
                .as_deref(),
            Some("source-id")
        );
    }

    #[test]
    fn password_is_only_kept_in_the_credential_vault() {
        let db_path = std::env::temp_dir().join(format!(
            "schemalens-profile-test-{}-{}.sqlite",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let vault = TestCredentialVault::default();
        let vault_handle = vault.clone();
        let mut store = ProfileStore::with_vault(&db_path, Box::new(vault)).unwrap();
        let profile = SavedProfile {
            id: "sql-id".into(),
            name: "SQL connection".into(),
            server: "localhost".into(),
            port: Some(1433),
            instance: None,
            database: Some("AppDb".into()),
            auth_type: "sql".into(),
            username: Some("readonly_user".into()),
            trust_server_certificate: false,
            connect_timeout_seconds: 15,
            sql_file_path: None,
            created_at_utc: "2026-10-29T00:00:00Z".into(),
        };
        store
            .save_profile(&profile, Some("unit-test-secret"), false)
            .expect("metadata and credential should be saved");
        assert!(store.has_profile_credential("sql-id").unwrap());
        assert!(store.get_profile("sql-id").unwrap().is_some());
        assert_eq!(
            vault_handle.get_password("sql-id").unwrap().as_deref(),
            Some("unit-test-secret")
        );
        drop(store);
        let db_content = std::fs::read(&db_path).unwrap();
        assert!(!db_content
            .windows("unit-test-secret".len())
            .any(|window| window == b"unit-test-secret"));
        let _ = std::fs::remove_file(db_path);
    }

    #[test]
    fn profiles_and_selections_survive_store_reinitialization() {
        let db_path = std::env::temp_dir().join(format!(
            "schemalens-reopen-test-{}-{}.sqlite",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let mut store =
            ProfileStore::with_vault(&db_path, Box::<TestCredentialVault>::default()).unwrap();
        let profile = SavedProfile {
            id: "persisted-id".into(),
            name: "Persisted profile".into(),
            server: "localhost".into(),
            port: Some(1433),
            instance: None,
            database: Some("AppDb".into()),
            auth_type: "windows".into(),
            username: None,
            trust_server_certificate: false,
            connect_timeout_seconds: 15,
            sql_file_path: None,
            created_at_utc: "2026-10-29T00:00:00Z".into(),
        };
        store.save_profile(&profile, None, false).unwrap();
        store
            .save_selection(&ConnectionSelection {
                source_connection_id: Some(profile.id.clone()),
                target_connection_id: Some(profile.id.clone()),
            })
            .unwrap();
        drop(store);

        let reopened =
            ProfileStore::with_vault(&db_path, Box::<TestCredentialVault>::default()).unwrap();
        assert_eq!(
            reopened.get_profile("persisted-id").unwrap().unwrap().name,
            profile.name
        );
        assert_eq!(
            reopened.get_selection().unwrap(),
            ConnectionSelection {
                source_connection_id: Some(profile.id.clone()),
                target_connection_id: Some(profile.id),
            }
        );
        drop(reopened);
        let _ = std::fs::remove_file(db_path);
    }

    #[test]
    fn sqlite_failure_restores_the_previously_stored_credential_state() {
        let vault = TestCredentialVault::default();
        let vault_handle = vault.clone();
        let mut store = ProfileStore::with_vault(":memory:", Box::new(vault)).unwrap();
        let profile = SavedProfile {
            id: "rollback-profile".into(),
            name: "Rollback test".into(),
            server: "localhost".into(),
            port: Some(1433),
            instance: None,
            database: Some("AppDb".into()),
            auth_type: "sql".into(),
            username: Some("reader".into()),
            trust_server_certificate: false,
            connect_timeout_seconds: 15,
            sql_file_path: None,
            created_at_utc: "2026-10-29T00:00:00Z".into(),
        };
        store
            .conn
            .execute_batch(
                "CREATE TRIGGER reject_profile_insert BEFORE INSERT ON profiles
                 BEGIN SELECT RAISE(ABORT, 'test failure'); END;",
            )
            .unwrap();

        assert!(store
            .save_profile(&profile, Some("transient-test-secret"), false)
            .is_err());
        assert!(store.list_profiles().unwrap().is_empty());
        assert_eq!(vault_handle.get_password("rollback-profile").unwrap(), None);
    }

    #[test]
    fn failed_delete_rolls_back_selection_and_restores_credential() {
        let vault = TestCredentialVault::default();
        let vault_handle = vault.clone();
        let mut store = ProfileStore::with_vault(":memory:", Box::new(vault)).unwrap();
        let profile = SavedProfile {
            id: "delete-rollback-profile".into(),
            name: "Delete rollback test".into(),
            server: "localhost".into(),
            port: Some(1433),
            instance: None,
            database: Some("AppDb".into()),
            auth_type: "sql".into(),
            username: Some("reader".into()),
            trust_server_certificate: false,
            connect_timeout_seconds: 15,
            sql_file_path: None,
            created_at_utc: "2026-10-29T00:00:00Z".into(),
        };
        store
            .save_profile(&profile, Some("stored-test-secret"), false)
            .unwrap();
        store
            .save_selection(&ConnectionSelection {
                source_connection_id: Some(profile.id.clone()),
                target_connection_id: None,
            })
            .unwrap();
        store
            .conn
            .execute_batch(
                "CREATE TRIGGER reject_profile_delete BEFORE DELETE ON profiles
                 BEGIN SELECT RAISE(ABORT, 'test failure'); END;",
            )
            .unwrap();

        assert!(store.delete_profile(&profile.id).is_err());
        assert!(store.get_profile(&profile.id).unwrap().is_some());
        assert_eq!(
            store.get_selection().unwrap().source_connection_id,
            Some(profile.id.clone())
        );
        assert_eq!(
            vault_handle.get_password(&profile.id).unwrap().as_deref(),
            Some("stored-test-secret")
        );
    }
}

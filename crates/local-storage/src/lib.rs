pub mod exporter;
pub use exporter::*;

use keyring::Entry;
use rusqlite::{params, Connection, Result as SqliteResult};
use serde::{Deserialize, Serialize};
use std::path::Path;

const KEYRING_SERVICE: &str = "SchemaLens_ConnectionProfile";

#[derive(Debug, Clone, Serialize, Deserialize)]
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

pub struct ProfileStore {
    conn: Connection,
}

impl ProfileStore {
    pub fn new(db_path: impl AsRef<Path>) -> SqliteResult<Self> {
        let conn = Connection::open(db_path)?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS profiles (
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
            )",
            [],
        )?;
        Ok(Self { conn })
    }

    pub fn save_profile(
        &self,
        profile: &SavedProfile,
        password: Option<&str>,
    ) -> Result<(), String> {
        self.conn
            .execute(
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
            .map_err(|e| format!("Database error saving profile: {}", e))?;

        if let Some(pwd) = password {
            if !pwd.is_empty() {
                let entry = Entry::new(KEYRING_SERVICE, &profile.id)
                    .map_err(|e| format!("Failed to access Windows Credential Manager: {}", e))?;
                entry
                    .set_password(pwd)
                    .map_err(|e| format!("Failed to write credential to Windows Vault: {}", e))?;
            }
        }

        Ok(())
    }

    pub fn get_profile_password(&self, profile_id: &str) -> Option<String> {
        if let Ok(entry) = Entry::new(KEYRING_SERVICE, profile_id) {
            entry.get_password().ok()
        } else {
            None
        }
    }

    pub fn delete_profile(&self, profile_id: &str) -> Result<(), String> {
        self.conn
            .execute("DELETE FROM profiles WHERE id = ?1", params![profile_id])
            .map_err(|e| format!("Failed to delete profile from db: {}", e))?;

        if let Ok(entry) = Entry::new(KEYRING_SERVICE, profile_id) {
            let _ = entry.delete_password();
        }

        Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_sqlite_lifecycle() {
        let store = ProfileStore::new(":memory:").expect("Failed to init in-memory sqlite");
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
            .save_profile(&profile, None)
            .expect("Save profile failed");
        let list = store.list_profiles().expect("List profiles failed");
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].name, "Homologacao A");
        assert_eq!(list[0].port, Some(14331));

        store.delete_profile("prof-001").expect("Delete failed");
        let list_after = store.list_profiles().expect("List after delete failed");
        assert_eq!(list_after.len(), 0);
    }
}

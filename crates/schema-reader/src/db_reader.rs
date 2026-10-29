use dotnet_conn_parser::{AuthConfig, ParsedConnectionInfo};
use schema_lens_domain::*;
use std::time::Duration;
use tiberius::{AuthMethod, Client, Config};
use tokio::net::TcpStream;
use tokio_util::compat::{Compat, TokioAsyncWriteCompatExt};

pub struct SqlServerReader;

#[derive(Debug, Clone, serde::Serialize)]
pub struct ConnectionIdentity {
    pub server_name: String,
    pub database_name: String,
    pub machine_name: Option<String>,
}

impl SqlServerReader {
    pub async fn identify_connection(
        info: &ParsedConnectionInfo,
    ) -> Result<ConnectionIdentity, String> {
        let mut client = Self::connect(info).await?;
        let row = client.simple_query("SELECT DB_NAME() AS database_name, CAST(SERVERPROPERTY('ServerName') AS nvarchar(128)) AS server_name, CAST(SERVERPROPERTY('MachineName') AS nvarchar(128)) AS machine_name")
            .await.map_err(|e| format!("Identity query failed: {e}"))?
            .into_row().await.map_err(|e| e.to_string())?
            .ok_or_else(|| "SQL Server returned no connection identity".to_string())?;
        Ok(ConnectionIdentity {
            server_name: row
                .get::<&str, _>("server_name")
                .unwrap_or("unknown server")
                .to_string(),
            database_name: row
                .get::<&str, _>("database_name")
                .unwrap_or("unknown database")
                .to_string(),
            machine_name: row.get::<&str, _>("machine_name").map(str::to_string),
        })
    }

    pub async fn test_connection(info: &ParsedConnectionInfo) -> Result<String, String> {
        let mut client = Self::connect(info).await?;
        let stream = client
            .simple_query("SELECT @@VERSION AS ver")
            .await
            .map_err(|e| format!("Query failed: {}", e))?;
        let row = stream
            .into_row()
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "No version returned".to_string())?;
        let version: &str = row.get("ver").unwrap_or("Unknown");
        Ok(version.to_string())
    }

    pub async fn list_databases(info: &ParsedConnectionInfo) -> Result<Vec<String>, String> {
        let mut client = Self::connect(info).await?;
        let stream = client
            .simple_query("SELECT name FROM sys.databases WHERE state = 0 ORDER BY name")
            .await
            .map_err(|e| format!("List databases failed: {}", e))?;

        let rows = stream
            .into_first_result()
            .await
            .map_err(|e| e.to_string())?;
        let mut dbs = Vec::new();
        for r in rows {
            if let Some(name) = r.get::<&str, _>("name") {
                dbs.push(name.to_string());
            }
        }
        Ok(dbs)
    }

    pub async fn read_snapshot(info: &ParsedConnectionInfo) -> Result<DatabaseSnapshot, String> {
        let mut client = Self::connect(info).await?;

        // Fail closed when database-wide metadata visibility cannot be confirmed.
        let permission_row = client
            .simple_query("SELECT CAST(CASE WHEN HAS_PERMS_BY_NAME(DB_NAME(), 'DATABASE', 'VIEW DEFINITION') = 1 OR IS_MEMBER('db_owner') = 1 THEN 1 ELSE 0 END AS bit) AS can_view_definition")
            .await
            .map_err(|e| format!("Failed to verify catalog permissions: {e}"))?
            .into_row()
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "Catalog permission check returned no result".to_string())?;
        let can_view_definition: bool = permission_row.get("can_view_definition").unwrap_or(false);
        if !can_view_definition {
            return Err("Catalog snapshot is indeterminate: database-level VIEW DEFINITION permission is required to confirm completeness".to_string());
        }

        // 1. Database Info
        let db_row = client
            .simple_query("SELECT DB_NAME() AS db_name, CAST(SERVERPROPERTY('ServerName') AS nvarchar(128)) AS server_name, collation_name, compatibility_level FROM sys.databases WHERE name = DB_NAME()")
            .await
            .map_err(|e| format!("Failed to read database metadata: {}", e))?
            .into_row()
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "Database not found in sys.databases".to_string())?;

        let db_name: &str = db_row.get("db_name").ok_or_else(|| {
            "SQL Server did not return DB_NAME(); snapshot identity is indeterminate".to_string()
        })?;
        let effective_server_name: &str = db_row.get("server_name").unwrap_or(&info.server);
        let collation: &str = db_row.get("collation_name").ok_or_else(|| {
            "Database collation is not visible; snapshot is incomplete".to_string()
        })?;
        let comp_level: u8 = db_row.get("compatibility_level").ok_or_else(|| {
            "Database compatibility level is not visible; snapshot is incomplete".to_string()
        })?;

        // 2. Version
        let ver_row = client
            .simple_query("SELECT @@VERSION AS ver")
            .await
            .map_err(|e| e.to_string())?
            .into_row()
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "Version query failed".to_string())?;
        let version: &str = ver_row.get("ver").unwrap_or("Unknown");

        // 3. Schemas
        let schema_rows = client
            .simple_query("SELECT s.name, dp.name AS owner FROM sys.schemas s LEFT JOIN sys.database_principals dp ON s.principal_id = dp.principal_id WHERE s.schema_id < 16384 AND s.name NOT IN ('sys', 'INFORMATION_SCHEMA')")
            .await
            .map_err(|e| e.to_string())?
            .into_first_result()
            .await
            .map_err(|e| e.to_string())?;

        let mut schemas = Vec::new();
        for r in schema_rows {
            schemas.push(SchemaMetadata {
                name: r.get::<&str, _>("name").unwrap_or("").to_string(),
                owner: r.get::<&str, _>("owner").unwrap_or("dbo").to_string(),
            });
        }

        // 4. Tables and Columns
        let table_rows = client
            .simple_query("SELECT t.object_id, s.name AS schema_name, t.name AS table_name FROM sys.tables t INNER JOIN sys.schemas s ON t.schema_id = s.schema_id WHERE t.is_ms_shipped = 0 ORDER BY s.name, t.name")
            .await
            .map_err(|e| e.to_string())?
            .into_first_result()
            .await
            .map_err(|e| e.to_string())?;

        let mut tables = Vec::new();
        for tr in table_rows {
            let obj_id: i32 = tr.get("object_id").unwrap_or(0);
            let s_name = tr
                .get::<&str, _>("schema_name")
                .unwrap_or("dbo")
                .to_string();
            let t_name = tr.get::<&str, _>("table_name").unwrap_or("").to_string();

            // Fetch columns for table
            let col_query = format!(
                "SELECT c.name, c.column_id, tp.name AS type_name, c.max_length, c.precision, c.scale, c.is_nullable, c.is_identity, TRY_CONVERT(bigint, idc.seed_value) AS identity_seed, TRY_CONVERT(bigint, idc.increment_value) AS identity_increment, c.is_computed, cc.definition AS computed_def, cc.is_persisted AS computed_persisted, c.collation_name
                 FROM sys.columns c 
                 INNER JOIN sys.types tp ON c.user_type_id = tp.user_type_id 
                 LEFT JOIN sys.identity_columns idc ON c.object_id = idc.object_id AND c.column_id = idc.column_id
                 LEFT JOIN sys.computed_columns cc ON c.object_id = cc.object_id AND c.column_id = cc.column_id 
                 WHERE c.object_id = {} 
                 ORDER BY c.column_id",
                obj_id
            );

            let col_rows = client
                .simple_query(&col_query)
                .await
                .map_err(|e| e.to_string())?
                .into_first_result()
                .await
                .map_err(|e| e.to_string())?;

            let mut columns = Vec::new();
            for cr in col_rows {
                columns.push(ColumnMetadata {
                    name: cr.get::<&str, _>("name").unwrap_or("").to_string(),
                    ordinal_position: cr.get::<i32, _>("column_id").unwrap_or(0) as u32,
                    data_type: cr
                        .get::<&str, _>("type_name")
                        .unwrap_or("varchar")
                        .to_string(),
                    max_length: cr.get::<i16, _>("max_length").unwrap_or(0),
                    precision: cr.get::<u8, _>("precision").unwrap_or(0),
                    scale: cr.get::<u8, _>("scale").unwrap_or(0),
                    is_nullable: cr.get::<bool, _>("is_nullable").unwrap_or(false),
                    is_identity: cr.get::<bool, _>("is_identity").unwrap_or(false),
                    identity_seed: cr.get::<i64, _>("identity_seed"),
                    identity_increment: cr.get::<i64, _>("identity_increment"),
                    is_computed: cr.get::<bool, _>("is_computed").unwrap_or(false),
                    computed_definition: cr.get::<&str, _>("computed_def").map(|s| s.to_string()),
                    computed_persisted: cr.get::<bool, _>("computed_persisted").unwrap_or(false),
                    collation: cr.get::<&str, _>("collation_name").map(|s| s.to_string()),
                });
            }

            let mut table = TableMetadata {
                id: ObjectId::new(s_name, t_name),
                columns,
                primary_key: None,
                foreign_keys: Vec::new(),
                check_constraints: Vec::new(),
                default_constraints: Vec::new(),
                unique_constraints: Vec::new(),
                indexes: Vec::new(),
            };
            Self::populate_table_constraints(&mut client, obj_id, &mut table).await?;
            tables.push(table);
        }

        // 5. Views, Procedures, Functions
        let module_rows = client
            .simple_query("SELECT o.name, s.name AS schema_name, o.type AS obj_type, m.definition, CASE WHEN m.definition IS NULL THEN 1 ELSE 0 END AS is_encrypted FROM sys.objects o INNER JOIN sys.schemas s ON o.schema_id = s.schema_id LEFT JOIN sys.sql_modules m ON o.object_id = m.object_id WHERE o.type IN ('V', 'P', 'FN', 'IF', 'TF') AND o.is_ms_shipped = 0")
            .await
            .map_err(|e| e.to_string())?
            .into_first_result()
            .await
            .map_err(|e| e.to_string())?;

        let mut views = Vec::new();
        let mut procedures = Vec::new();
        let mut functions = Vec::new();

        for mr in module_rows {
            let s_name = mr
                .get::<&str, _>("schema_name")
                .unwrap_or("dbo")
                .to_string();
            let m_name = mr.get::<&str, _>("name").unwrap_or("").to_string();
            let o_type = mr.get::<&str, _>("obj_type").unwrap_or("").trim();
            let def = mr.get::<&str, _>("definition").map(|s| s.to_string());
            let is_enc: i32 = mr.get("is_encrypted").unwrap_or(0);

            match o_type {
                "V" => views.push(ModuleMetadata {
                    id: ObjectId::new(s_name, m_name),
                    object_type: SchemaObjectType::View,
                    definition: def,
                    is_encrypted: is_enc == 1,
                }),
                "P" => procedures.push(ModuleMetadata {
                    id: ObjectId::new(s_name, m_name),
                    object_type: SchemaObjectType::StoredProcedure,
                    definition: def,
                    is_encrypted: is_enc == 1,
                }),
                "FN" | "IF" | "TF" => functions.push(ModuleMetadata {
                    id: ObjectId::new(s_name, m_name),
                    object_type: SchemaObjectType::Function,
                    definition: def,
                    is_encrypted: is_enc == 1,
                }),
                _ => {}
            }
        }

        Ok(DatabaseSnapshot {
            server_name: effective_server_name.to_string(),
            database_name: db_name.to_string(),
            sql_server_version: version.to_string(),
            collation: collation.to_string(),
            compatibility_level: comp_level as u16,
            captured_at_utc: chrono::Utc::now().to_rfc3339(),
            schemas,
            tables,
            views,
            procedures,
            functions,
            triggers: Vec::new(),
            sequences: Vec::new(),
            synonyms: Vec::new(),
            custom_types: Vec::new(),
            collection_warnings: Vec::new(),
        })
    }

    async fn populate_table_constraints(
        client: &mut Client<Compat<TcpStream>>,
        object_id: i32,
        table: &mut TableMetadata,
    ) -> Result<(), String> {
        let key_rows = client.simple_query(&format!(
            "SELECT kc.name AS constraint_name, kc.type AS constraint_type, CAST(CASE WHEN i.type = 1 THEN 1 ELSE 0 END AS bit) AS is_clustered, c.name AS column_name FROM sys.key_constraints kc JOIN sys.indexes i ON i.object_id = kc.parent_object_id AND i.index_id = kc.unique_index_id JOIN sys.index_columns ic ON ic.object_id = i.object_id AND ic.index_id = i.index_id JOIN sys.columns c ON c.object_id = ic.object_id AND c.column_id = ic.column_id WHERE kc.parent_object_id = {object_id} AND ic.key_ordinal > 0 ORDER BY kc.name, ic.key_ordinal"
        )).await.map_err(|e| format!("Failed to read key constraints: {e}"))?
            .into_first_result().await.map_err(|e| e.to_string())?;
        for row in key_rows {
            let name = row
                .get::<&str, _>("constraint_name")
                .unwrap_or("")
                .to_string();
            let kind = row.get::<&str, _>("constraint_type").unwrap_or("");
            let column = row.get::<&str, _>("column_name").unwrap_or("").to_string();
            let clustered = row.get::<bool, _>("is_clustered").unwrap_or(false);
            let keys = if kind == "PK" {
                &mut table.primary_key
            } else {
                continue;
            };
            let key = keys.get_or_insert(KeyConstraintMetadata {
                name,
                is_clustered: clustered,
                columns: Vec::new(),
            });
            key.columns.push(column);
        }

        let unique_rows = client.simple_query(&format!(
            "SELECT kc.name AS constraint_name, CAST(CASE WHEN i.type = 1 THEN 1 ELSE 0 END AS bit) AS is_clustered, c.name AS column_name FROM sys.key_constraints kc JOIN sys.indexes i ON i.object_id = kc.parent_object_id AND i.index_id = kc.unique_index_id JOIN sys.index_columns ic ON ic.object_id = i.object_id AND ic.index_id = i.index_id JOIN sys.columns c ON c.object_id = ic.object_id AND c.column_id = ic.column_id WHERE kc.parent_object_id = {object_id} AND kc.type = 'UQ' AND ic.key_ordinal > 0 ORDER BY kc.name, ic.key_ordinal"
        )).await.map_err(|e| format!("Failed to read unique constraints: {e}"))?
            .into_first_result().await.map_err(|e| e.to_string())?;
        for row in unique_rows {
            let name = row
                .get::<&str, _>("constraint_name")
                .unwrap_or("")
                .to_string();
            let clustered = row.get::<bool, _>("is_clustered").unwrap_or(false);
            let column = row.get::<&str, _>("column_name").unwrap_or("").to_string();
            if let Some(key) = table
                .unique_constraints
                .iter_mut()
                .find(|key| key.name == name)
            {
                key.columns.push(column);
            } else {
                table.unique_constraints.push(KeyConstraintMetadata {
                    name,
                    is_clustered: clustered,
                    columns: vec![column],
                });
            }
        }

        let default_rows = client.simple_query(&format!(
            "SELECT dc.name AS constraint_name, c.name AS column_name, dc.definition FROM sys.default_constraints dc JOIN sys.columns c ON c.object_id = dc.parent_object_id AND c.column_id = dc.parent_column_id WHERE dc.parent_object_id = {object_id} ORDER BY c.column_id"
        )).await.map_err(|e| format!("Failed to read default constraints: {e}"))?
            .into_first_result().await.map_err(|e| e.to_string())?;
        for row in default_rows {
            table.default_constraints.push(DefaultConstraintMetadata {
                name: row
                    .get::<&str, _>("constraint_name")
                    .unwrap_or("")
                    .to_string(),
                column_name: row.get::<&str, _>("column_name").unwrap_or("").to_string(),
                definition: row.get::<&str, _>("definition").unwrap_or("").to_string(),
            });
        }

        let check_rows = client.simple_query(&format!(
            "SELECT name, definition, is_disabled FROM sys.check_constraints WHERE parent_object_id = {object_id} ORDER BY name"
        )).await.map_err(|e| format!("Failed to read check constraints: {e}"))?
            .into_first_result().await.map_err(|e| e.to_string())?;
        for row in check_rows {
            table.check_constraints.push(CheckConstraintMetadata {
                name: row.get::<&str, _>("name").unwrap_or("").to_string(),
                definition: row.get::<&str, _>("definition").unwrap_or("").to_string(),
                is_disabled: row.get::<bool, _>("is_disabled").unwrap_or(false),
            });
        }

        let fk_rows = client.simple_query(&format!(
            "SELECT fk.name AS constraint_name, rs.name AS referenced_schema, rt.name AS referenced_table, pc.name AS parent_column, rc.name AS referenced_column, fk.delete_referential_action_desc AS delete_rule, fk.update_referential_action_desc AS update_rule, fk.is_disabled FROM sys.foreign_keys fk JOIN sys.foreign_key_columns fkc ON fkc.constraint_object_id = fk.object_id JOIN sys.tables rt ON rt.object_id = fkc.referenced_object_id JOIN sys.schemas rs ON rs.schema_id = rt.schema_id JOIN sys.columns pc ON pc.object_id = fkc.parent_object_id AND pc.column_id = fkc.parent_column_id JOIN sys.columns rc ON rc.object_id = fkc.referenced_object_id AND rc.column_id = fkc.referenced_column_id WHERE fk.parent_object_id = {object_id} ORDER BY fk.name, fkc.constraint_column_id"
        )).await.map_err(|e| format!("Failed to read foreign keys: {e}"))?
            .into_first_result().await.map_err(|e| e.to_string())?;
        for row in fk_rows {
            let name = row
                .get::<&str, _>("constraint_name")
                .unwrap_or("")
                .to_string();
            let referenced_schema = row
                .get::<&str, _>("referenced_schema")
                .unwrap_or("")
                .to_string();
            let referenced_table = row
                .get::<&str, _>("referenced_table")
                .unwrap_or("")
                .to_string();
            let mapping = (
                row.get::<&str, _>("parent_column")
                    .unwrap_or("")
                    .to_string(),
                row.get::<&str, _>("referenced_column")
                    .unwrap_or("")
                    .to_string(),
            );
            if let Some(fk) = table.foreign_keys.iter_mut().find(|fk| fk.name == name) {
                fk.column_mappings.push(mapping);
            } else {
                table.foreign_keys.push(ForeignKeyMetadata {
                    name,
                    referenced_schema,
                    referenced_table,
                    column_mappings: vec![mapping],
                    delete_rule: row
                        .get::<&str, _>("delete_rule")
                        .unwrap_or("NO_ACTION")
                        .to_string(),
                    update_rule: row
                        .get::<&str, _>("update_rule")
                        .unwrap_or("NO_ACTION")
                        .to_string(),
                    is_disabled: row.get::<bool, _>("is_disabled").unwrap_or(false),
                });
            }
        }

        let index_rows = client.simple_query(&format!(
            "SELECT i.name AS index_name, i.is_unique, CAST(CASE WHEN i.type = 1 THEN 1 ELSE 0 END AS bit) AS is_clustered, i.filter_definition, ic.is_included_column, ic.is_descending_key, ic.key_ordinal, c.name AS column_name FROM sys.indexes i JOIN sys.index_columns ic ON ic.object_id = i.object_id AND ic.index_id = i.index_id JOIN sys.columns c ON c.object_id = ic.object_id AND c.column_id = ic.column_id WHERE i.object_id = {object_id} AND i.name IS NOT NULL AND i.is_primary_key = 0 AND i.is_unique_constraint = 0 AND i.type IN (1, 2) ORDER BY i.name, ic.is_included_column, ic.key_ordinal, ic.index_column_id"
        )).await.map_err(|e| format!("Failed to read indexes: {e}"))?
            .into_first_result().await.map_err(|e| e.to_string())?;
        for row in index_rows {
            let name = row.get::<&str, _>("index_name").unwrap_or("").to_string();
            let column = row.get::<&str, _>("column_name").unwrap_or("").to_string();
            let included = row.get::<bool, _>("is_included_column").unwrap_or(false);
            let descending = row.get::<bool, _>("is_descending_key").unwrap_or(false);
            if let Some(index) = table.indexes.iter_mut().find(|index| index.name == name) {
                if included {
                    index.included_columns.push(column);
                } else {
                    index.key_columns.push(column);
                    index.key_columns_descending.push(descending);
                }
            } else {
                table.indexes.push(IndexMetadata {
                    name,
                    is_unique: row.get::<bool, _>("is_unique").unwrap_or(false),
                    is_clustered: row.get::<bool, _>("is_clustered").unwrap_or(false),
                    key_columns: if included {
                        Vec::new()
                    } else {
                        vec![column.clone()]
                    },
                    key_columns_descending: if included {
                        Vec::new()
                    } else {
                        vec![descending]
                    },
                    included_columns: if included { vec![column] } else { Vec::new() },
                    filter_definition: row.get::<&str, _>("filter_definition").map(str::to_string),
                });
            }
        }
        Ok(())
    }

    async fn connect(info: &ParsedConnectionInfo) -> Result<Client<Compat<TcpStream>>, String> {
        let mut config = Config::new();
        config.host(&info.server);
        config.port(info.port.unwrap_or(1433));
        if let Some(db) = &info.database {
            config.database(db);
        }

        match &info.auth {
            AuthConfig::Sql { username, password } => {
                config.authentication(AuthMethod::sql_server(username, password));
            }
            AuthConfig::WindowsIntegrated => {
                #[cfg(windows)]
                {
                    config.authentication(AuthMethod::Integrated);
                }
                #[cfg(not(windows))]
                {
                    return Err("Windows Integrated Authentication requires Windows".to_string());
                }
            }
        }

        if info.trust_server_certificate {
            config.trust_cert();
        }

        let tcp_addr = format!("{}:{}", info.server, info.port.unwrap_or(1433));
        let connect_timeout = Duration::from_secs(info.connect_timeout_seconds as u64);

        let tcp = tokio::time::timeout(connect_timeout, TcpStream::connect(&tcp_addr))
            .await
            .map_err(|_| format!("Connection timeout to {}", tcp_addr))?
            .map_err(|e| format!("TCP connection error to {}: {}", tcp_addr, e))?;

        tcp.set_nodelay(true).map_err(|e| e.to_string())?;

        let client = Client::connect(config, tcp.compat_write())
            .await
            .map_err(|e| format!("TDS handshake failed: {}", e))?;

        Ok(client)
    }
}

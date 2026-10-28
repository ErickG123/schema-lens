use dotnet_conn_parser::{AuthConfig, ParsedConnectionInfo};
use schema_lens_domain::*;
use std::time::Duration;
use tiberius::{AuthMethod, Client, Config};
use tokio::net::TcpStream;
use tokio_util::compat::{Compat, TokioAsyncWriteCompatExt};

pub struct SqlServerReader;

impl SqlServerReader {
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

        // 1. Database Info
        let db_row = client
            .simple_query("SELECT DB_NAME() AS db_name, collation_name, compatibility_level FROM sys.databases WHERE name = DB_NAME()")
            .await
            .map_err(|e| format!("Failed to read database metadata: {}", e))?
            .into_row()
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "Database not found in sys.databases".to_string())?;

        let db_name: &str = db_row.get("db_name").unwrap_or("master");
        let collation: &str = db_row
            .get("collation_name")
            .unwrap_or("Latin1_General_CI_AS");
        let comp_level: u8 = db_row.get("compatibility_level").unwrap_or(150);

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
                "SELECT c.name, c.column_id, tp.name AS type_name, c.max_length, c.precision, c.scale, c.is_nullable, c.is_identity, c.is_computed, cc.definition AS computed_def, c.collation_name 
                 FROM sys.columns c 
                 INNER JOIN sys.types tp ON c.user_type_id = tp.user_type_id 
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
                    identity_seed: None,
                    identity_increment: None,
                    is_computed: cr.get::<bool, _>("is_computed").unwrap_or(false),
                    computed_definition: cr.get::<&str, _>("computed_def").map(|s| s.to_string()),
                    collation: cr.get::<&str, _>("collation_name").map(|s| s.to_string()),
                });
            }

            tables.push(TableMetadata {
                id: ObjectId::new(s_name, t_name),
                columns,
                primary_key: None,
                foreign_keys: Vec::new(),
                check_constraints: Vec::new(),
                default_constraints: Vec::new(),
                unique_constraints: Vec::new(),
                indexes: Vec::new(),
            });
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
            server_name: info.server.clone(),
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

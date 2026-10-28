mod parser;

use parser::{AuthConfig, DotnetConnParser, ParsedConnectionInfo};
use std::time::Duration;
use tiberius::{AuthMethod, Client, Config};
use tokio::net::TcpStream;
use tokio_util::compat::TokioAsyncWriteCompatExt;

pub async fn build_tiberius_config(info: &ParsedConnectionInfo) -> Result<Config, String> {
    let mut config = Config::new();
    config.host(&info.server);
    if let Some(port) = info.port {
        config.port(port);
    } else {
        config.port(1433);
    }

    if let Some(instance) = &info.instance {
        config.instance_name(instance);
    }

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
                return Err("Windows Integrated Authentication requires Windows OS".to_string());
            }
        }
    }

    if info.trust_server_certificate {
        config.trust_cert();
    }

    Ok(config)
}

pub async fn test_connection(info: &ParsedConnectionInfo) -> Result<(String, String), String> {
    let config = build_tiberius_config(info).await?;
    let tcp_addr = format!("{}:{}", info.server, info.port.unwrap_or(1433));

    let connect_timeout = Duration::from_secs(info.connect_timeout_seconds as u64);
    let tcp = tokio::time::timeout(connect_timeout, TcpStream::connect(&tcp_addr))
        .await
        .map_err(|_| format!("Connection timeout to {}", tcp_addr))?
        .map_err(|e| format!("TCP connection error to {}: {}", tcp_addr, e))?;

    tcp.set_nodelay(true).map_err(|e| e.to_string())?;

    let mut client = Client::connect(config, tcp.compat_write())
        .await
        .map_err(|e| format!("TDS handshake failed: {}", e))?;

    let stream = client
        .simple_query("SELECT @@VERSION AS ver, DB_NAME() AS dbname")
        .await
        .map_err(|e| format!("Query failed: {}", e))?;

    let row = stream
        .into_row()
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "No rows returned".to_string())?;

    let version: &str = row.get("ver").unwrap_or("Unknown Version");
    let dbname: &str = row.get("dbname").unwrap_or("master");

    Ok((version.to_string(), dbname.to_string()))
}

#[tokio::main]
async fn main() {
    println!("=== SchemaLens Connection POC ===");
    println!("Validating .NET Connection String Parser...");

    let sample_conn = "Server=localhost,14331;Database=SchemaSource;User Id=sa;Password=\"SchemaLens#2026!LocalDev;special=val\";TrustServerCertificate=True;Encrypt=True;";
    let parsed = DotnetConnParser::parse(sample_conn).expect("Parsing sample conn string failed");

    println!("Parsed Server: {}:{:?}", parsed.server, parsed.port);
    println!("Parsed Database: {:?}", parsed.database);
    println!("Parsed TrustCert: {}", parsed.trust_server_certificate);

    // Redaction check
    match &parsed.auth {
        AuthConfig::Sql { username, password: _ } => {
            println!("Parsed User: {} | Password: [REDACTED]", username);
        }
        AuthConfig::WindowsIntegrated => {
            println!("Parsed Auth: Windows Integrated");
        }
    }

    println!("All initial tests defined in integration tests.");
}

#[cfg(test)]
mod tests {
    use super::parser::*;

    #[test]
    fn test_parser_aliases_and_casing() {
        let conn = "Data Source=127.0.0.1,14331;Initial Catalog=MyDb;UID=admin;PWD=my_pwd;TrustServerCertificate=True;";
        let parsed = DotnetConnParser::parse(conn).unwrap();
        assert_eq!(parsed.server, "127.0.0.1");
        assert_eq!(parsed.port, Some(14331));
        assert_eq!(parsed.database, Some("MyDb".to_string()));
        assert_eq!(
            parsed.auth,
            AuthConfig::Sql {
                username: "admin".to_string(),
                password: "my_pwd".to_string()
            }
        );
        assert!(parsed.trust_server_certificate);
    }

    #[test]
    fn test_parser_quoted_password_with_semicolon_and_equals() {
        let conn = "Server=remote.db.local;Database=TestDb;User Id=test;Password=\"p@ss;word=123\";";
        let parsed = DotnetConnParser::parse(conn).unwrap();
        assert_eq!(parsed.server, "remote.db.local");
        assert_eq!(
            parsed.auth,
            AuthConfig::Sql {
                username: "test".to_string(),
                password: "p@ss;word=123".to_string()
            }
        );
    }

    #[test]
    fn test_parser_windows_auth_aliases() {
        let conn1 = "Server=my-pc\\SQLEXPRESS;Database=Empresa;Integrated Security=SSPI;";
        let p1 = DotnetConnParser::parse(conn1).unwrap();
        assert_eq!(p1.server, "my-pc");
        assert_eq!(p1.instance, Some("SQLEXPRESS".to_string()));
        assert_eq!(p1.auth, AuthConfig::WindowsIntegrated);

        let conn2 = "Server=localhost;Database=Empresa;Trusted_Connection=True;";
        let p2 = DotnetConnParser::parse(conn2).unwrap();
        assert_eq!(p2.auth, AuthConfig::WindowsIntegrated);
    }

    #[test]
    fn test_parser_options_and_unknowns() {
        let conn = "Server=dbserver;Connect Timeout=45;Application Name=CustomApp;CustomOption=Val;";
        let parsed = DotnetConnParser::parse(conn).unwrap();
        assert_eq!(parsed.connect_timeout_seconds, 45);
        assert_eq!(parsed.application_name, "CustomApp");
        assert_eq!(
            parsed.unknown_options.get("CustomOption").map(|s| s.as_str()),
            Some("Val")
        );
    }
}

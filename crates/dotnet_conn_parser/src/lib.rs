pub mod sanitizer;
pub use sanitizer::*;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuthConfig {
    Sql { username: String, password: String },
    WindowsIntegrated,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParsedConnectionInfo {
    pub server: String,
    pub port: Option<u16>,
    pub instance: Option<String>,
    pub database: Option<String>,
    pub auth: AuthConfig,
    pub encrypt: bool,
    pub trust_server_certificate: bool,
    pub connect_timeout_seconds: u32,
    pub application_name: String,
    pub unknown_options: HashMap<String, String>,
}

impl std::fmt::Debug for ParsedConnectionInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let auth_masked = match &self.auth {
            AuthConfig::Sql { username, .. } => {
                format!("Sql(user: {}, password: [REDACTED])", username)
            }
            AuthConfig::WindowsIntegrated => "WindowsIntegrated".to_string(),
        };
        f.debug_struct("ParsedConnectionInfo")
            .field("server", &self.server)
            .field("port", &self.port)
            .field("instance", &self.instance)
            .field("database", &self.database)
            .field("auth", &auth_masked)
            .field("encrypt", &self.encrypt)
            .field("trust_server_certificate", &self.trust_server_certificate)
            .field("connect_timeout_seconds", &self.connect_timeout_seconds)
            .field("application_name", &self.application_name)
            .finish()
    }
}

pub struct DotnetConnParser;

impl DotnetConnParser {
    pub fn parse(conn_str: &str) -> Result<ParsedConnectionInfo, String> {
        let pairs = Self::tokenize_key_values(conn_str)?;

        let mut server_raw = String::new();
        let mut database = None;
        let mut username = None;
        let mut password = None;
        let mut integrated_security = false;
        let mut encrypt = true;
        let mut trust_server_certificate = false;
        let mut connect_timeout_seconds = 30;
        let mut application_name = "SchemaLens".to_string();
        let mut unknown_options = HashMap::new();

        for (key, value) in pairs {
            let key_norm = key.trim().to_lowercase();
            match key_norm.as_str() {
                "server" | "data source" | "address" | "addr" | "network address" => {
                    server_raw = value;
                }
                "database" | "initial catalog" => {
                    database = Some(value);
                }
                "user id" | "uid" | "user" => {
                    username = Some(value);
                }
                "password" | "pwd" => {
                    password = Some(value);
                }
                "integrated security" | "trusted_connection" => {
                    let v = value.to_lowercase();
                    if v == "true" || v == "sspi" || v == "yes" || v == "1" {
                        integrated_security = true;
                    }
                }
                "encrypt" => {
                    let v = value.to_lowercase();
                    encrypt = v == "true" || v == "yes" || v == "mandatory" || v == "strict";
                }
                "trustservercertificate" | "trust_server_certificate" => {
                    let v = value.to_lowercase();
                    trust_server_certificate = v == "true" || v == "yes" || v == "1";
                }
                "connect timeout" | "connection timeout" | "timeout" => {
                    if let Ok(secs) = value.parse::<u32>() {
                        connect_timeout_seconds = secs;
                    }
                }
                "application name" | "app" => {
                    application_name = value;
                }
                _ => {
                    unknown_options.insert(key, value);
                }
            }
        }

        if server_raw.is_empty() {
            return Err("Missing 'Server' or 'Data Source' in connection string".to_string());
        }

        let (server, port, instance) = Self::parse_server_target(&server_raw);

        let auth = if integrated_security {
            AuthConfig::WindowsIntegrated
        } else {
            match (username, password) {
                (Some(u), Some(p)) => AuthConfig::Sql {
                    username: u,
                    password: p,
                },
                (Some(u), None) => AuthConfig::Sql {
                    username: u,
                    password: String::new(),
                },
                _ => AuthConfig::WindowsIntegrated,
            }
        };

        Ok(ParsedConnectionInfo {
            server,
            port,
            instance,
            database,
            auth,
            encrypt,
            trust_server_certificate,
            connect_timeout_seconds,
            application_name,
            unknown_options,
        })
    }

    fn parse_server_target(raw: &str) -> (String, Option<u16>, Option<String>) {
        if let Some((host_part, port_part)) = raw.split_once(',') {
            let port = port_part.trim().parse::<u16>().ok();
            return (host_part.trim().to_string(), port, None);
        }

        if let Some((host_part, instance_part)) = raw.split_once('\\') {
            return (
                host_part.trim().to_string(),
                None,
                Some(instance_part.trim().to_string()),
            );
        }

        (raw.trim().to_string(), None, None)
    }

    fn tokenize_key_values(input: &str) -> Result<Vec<(String, String)>, String> {
        let mut results = Vec::new();
        let chars: Vec<char> = input.chars().collect();
        let len = chars.len();
        let mut i = 0;

        while i < len {
            while i < len && (chars[i].is_whitespace() || chars[i] == ';') {
                i += 1;
            }
            if i >= len {
                break;
            }

            let mut key = String::new();
            while i < len && chars[i] != '=' && chars[i] != ';' {
                key.push(chars[i]);
                i += 1;
            }

            if i >= len || chars[i] != '=' {
                if !key.trim().is_empty() {
                    return Err(format!("Malformed connection string segment: '{}'", key));
                }
                break;
            }

            i += 1;

            while i < len && chars[i].is_whitespace() {
                i += 1;
            }

            let mut value = String::new();
            if i < len && (chars[i] == '"' || chars[i] == '\'') {
                let quote = chars[i];
                i += 1;
                while i < len {
                    if chars[i] == quote {
                        if i + 1 < len && chars[i + 1] == quote {
                            value.push(quote);
                            i += 2;
                        } else {
                            i += 1;
                            break;
                        }
                    } else {
                        value.push(chars[i]);
                        i += 1;
                    }
                }
            } else {
                while i < len && chars[i] != ';' {
                    value.push(chars[i]);
                    i += 1;
                }
                value = value.trim_end().to_string();
            }

            results.push((key.trim().to_string(), value));
        }

        Ok(results)
    }
}

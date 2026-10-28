pub struct SecretSanitizer;

impl SecretSanitizer {
    pub fn sanitize_connection_string(conn: &str) -> String {
        let mut sanitized = String::new();
        let segments = conn.split(';');

        for seg in segments {
            let seg_trim = seg.trim();
            if seg_trim.is_empty() {
                continue;
            }

            if let Some((k, _)) = seg_trim.split_once('=') {
                let k_lower = k.trim().to_lowercase();
                if k_lower == "password" || k_lower == "pwd" {
                    sanitized.push_str(&format!("{}={};", k, "[REDACTED]"));
                } else {
                    sanitized.push_str(seg_trim);
                    sanitized.push(';');
                }
            } else {
                sanitized.push_str(seg_trim);
                sanitized.push(';');
            }
        }

        sanitized
    }

    pub fn escape_sql_identifier(identifier: &str) -> String {
        let escaped = identifier.replace(']', "]]");
        format!("[{}]", escaped)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_connection_string() {
        let raw = "Server=localhost;Database=Test;User Id=sa;Password=SuperSecret123!;";
        let clean = SecretSanitizer::sanitize_connection_string(raw);
        assert!(!clean.contains("SuperSecret123!"));
        assert!(clean.contains("Password=[REDACTED]"));
    }

    #[test]
    fn test_escape_sql_identifier_injection() {
        let malformed = "Orders]; DROP TABLE Users; --";
        let safe = SecretSanitizer::escape_sql_identifier(malformed);
        assert_eq!(safe, "[Orders]]; DROP TABLE Users; --]");
    }
}

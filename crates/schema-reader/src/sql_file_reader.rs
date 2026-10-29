use schema_lens_domain::*;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub struct SqlFileReader;

impl SqlFileReader {
    pub fn read_from_file(path: impl AsRef<Path>) -> Result<DatabaseSnapshot, String> {
        let content = fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read SQL script file: {}", e))?;
        Self::parse_sql_content(&content, path.as_ref().to_string_lossy().as_ref())
    }

    pub fn parse_sql_content(content: &str, file_label: &str) -> Result<DatabaseSnapshot, String> {
        let mut schemas = vec![SchemaMetadata {
            name: "dbo".to_string(),
            owner: "dbo".to_string(),
        }];
        let mut tables: HashMap<ObjectId, TableMetadata> = HashMap::new();
        let mut views = Vec::new();
        let mut procedures = Vec::new();
        let mut warnings = Vec::new();

        // 1. Remover comentários de bloco /* ... */ e comentários de linha --
        let cleaned_sql = Self::strip_comments(content);

        // 2. Quebrar por instruções DDL usando batches 'GO' ou separadores ';'
        let statements = Self::split_statements(&cleaned_sql);

        for stmt in statements {
            let trimmed = stmt.trim();
            if trimmed.is_empty() {
                continue;
            }

            let upper = trimmed.to_uppercase();

            // --- CREATE SCHEMA ---
            if upper.contains("CREATE SCHEMA") {
                if let Some(pos) = upper.find("CREATE SCHEMA") {
                    let after = &trimmed[pos + 13..].trim();
                    let schema_name = after
                        .split_whitespace()
                        .next()
                        .unwrap_or("")
                        .trim_matches(|c| c == '[' || c == ']' || c == ';');
                    if !schema_name.is_empty()
                        && !schemas
                            .iter()
                            .any(|s| s.name.eq_ignore_ascii_case(schema_name))
                    {
                        schemas.push(SchemaMetadata {
                            name: schema_name.to_string(),
                            owner: "dbo".to_string(),
                        });
                    }
                }
            }
            // --- CREATE TABLE ---
            else if upper.contains("CREATE TABLE") {
                if let Some(table_meta) = Self::parse_create_table(trimmed) {
                    // Garantir que o schema seja registrado se não for padrão
                    if !schemas
                        .iter()
                        .any(|s| s.name.eq_ignore_ascii_case(&table_meta.id.schema))
                    {
                        schemas.push(SchemaMetadata {
                            name: table_meta.id.schema.clone(),
                            owner: "dbo".to_string(),
                        });
                    }
                    tables.insert(table_meta.id.clone(), table_meta);
                } else {
                    warnings.push(format!(
                        "Falha ao interpretar declaração CREATE TABLE: {}",
                        trimmed.lines().next().unwrap_or("")
                    ));
                }
            }
            // --- CREATE [UNIQUE] [CLUSTERED|NONCLUSTERED] INDEX ---
            else if upper.contains("CREATE INDEX")
                || upper.contains("CREATE UNIQUE INDEX")
                || upper.contains("CREATE NONCLUSTERED INDEX")
                || upper.contains("CREATE CLUSTERED INDEX")
                || upper.contains("CREATE UNIQUE NONCLUSTERED INDEX")
                || upper.contains("CREATE UNIQUE CLUSTERED INDEX")
            {
                if let Some((target_table_id, index_meta)) = Self::parse_create_index(trimmed) {
                    if let Some(tbl) = tables.get_mut(&target_table_id) {
                        tbl.indexes.push(index_meta);
                    } else {
                        // Tabela ainda não existe no batch ou foi referenciada antes: criar stub
                        let stub = TableMetadata {
                            id: target_table_id.clone(),
                            columns: Vec::new(),
                            primary_key: None,
                            foreign_keys: Vec::new(),
                            check_constraints: Vec::new(),
                            default_constraints: Vec::new(),
                            unique_constraints: Vec::new(),
                            indexes: vec![index_meta],
                        };
                        tables.insert(target_table_id, stub);
                    }
                } else {
                    warnings.push(format!(
                        "Índice com sintaxe complexa não interpretado: {}",
                        trimmed.lines().next().unwrap_or("")
                    ));
                }
            }
            // --- ALTER TABLE ADD CONSTRAINT ---
            else if upper.contains("ALTER TABLE") && upper.contains("ADD CONSTRAINT") {
                Self::parse_alter_table_add_constraint(trimmed, &mut tables, &mut warnings);
            }
            // --- CREATE VIEW ---
            else if upper.contains("CREATE VIEW") {
                if let Some(pos) = upper.find("CREATE VIEW") {
                    let after = &trimmed[pos + 11..].trim();
                    let name_part = after.split_whitespace().next().unwrap_or("").trim();
                    let (schema, name) = Self::parse_object_identifier(name_part);
                    views.push(ModuleMetadata {
                        id: ObjectId::new(schema, name),
                        object_type: SchemaObjectType::View,
                        definition: Some(trimmed.to_string()),
                        is_encrypted: false,
                    });
                }
            }
            // --- CREATE PROCEDURE ---
            else if upper.contains("CREATE PROCEDURE") || upper.contains("CREATE PROC") {
                let kw_len = if upper.contains("CREATE PROCEDURE") {
                    16
                } else {
                    11
                };
                if let Some(pos) = upper.find("CREATE PROC") {
                    let after = &trimmed[pos + kw_len..].trim();
                    let name_part = after.split_whitespace().next().unwrap_or("").trim();
                    let (schema, name) = Self::parse_object_identifier(name_part);
                    procedures.push(ModuleMetadata {
                        id: ObjectId::new(schema, name),
                        object_type: SchemaObjectType::StoredProcedure,
                        definition: Some(trimmed.to_string()),
                        is_encrypted: false,
                    });
                }
            }
        }

        let mut table_list: Vec<TableMetadata> = tables.into_values().collect();
        table_list.sort_by(|a, b| {
            a.id.schema
                .cmp(&b.id.schema)
                .then(a.id.name.cmp(&b.id.name))
        });

        Ok(DatabaseSnapshot {
            server_name: "SQL_SCRIPT_FILE".to_string(),
            database_name: file_label.to_string(),
            sql_server_version: "SQL Server Script DDL".to_string(),
            collation: "Latin1_General_CI_AS".to_string(),
            compatibility_level: 160,
            captured_at_utc: chrono::Utc::now().to_rfc3339(),
            schemas,
            tables: table_list,
            views,
            procedures,
            functions: Vec::new(),
            triggers: Vec::new(),
            sequences: Vec::new(),
            synonyms: Vec::new(),
            custom_types: Vec::new(),
            collection_warnings: warnings,
        })
    }

    fn strip_comments(sql: &str) -> String {
        let mut result = String::new();
        let chars: Vec<char> = sql.chars().collect();
        let len = chars.len();
        let mut i = 0;

        while i < len {
            // Comentário de bloco /* ... */
            if chars[i] == '/' && i + 1 < len && chars[i + 1] == '*' {
                i += 2;
                while i + 1 < len && !(chars[i] == '*' && chars[i + 1] == '/') {
                    i += 1;
                }
                i += 2; // Pula */
                result.push(' ');
            }
            // Comentário de linha -- ...
            else if chars[i] == '-' && i + 1 < len && chars[i + 1] == '-' {
                i += 2;
                while i < len && chars[i] != '\n' && chars[i] != '\r' {
                    i += 1;
                }
                result.push('\n');
            }
            // String literal '...'
            else if chars[i] == '\'' {
                result.push(chars[i]);
                i += 1;
                while i < len {
                    result.push(chars[i]);
                    if chars[i] == '\'' {
                        if i + 1 < len && chars[i + 1] == '\'' {
                            // Aspas duplicadas
                            result.push(chars[i + 1]);
                            i += 2;
                        } else {
                            i += 1;
                            break;
                        }
                    } else {
                        i += 1;
                    }
                }
            } else {
                result.push(chars[i]);
                i += 1;
            }
        }

        result
    }

    fn split_statements(sql: &str) -> Vec<String> {
        let mut stmts = Vec::new();
        let lines: Vec<&str> = sql.lines().collect();
        let mut current = String::new();

        for line in lines {
            let trimmed = line.trim();
            if trimmed.eq_ignore_ascii_case("GO") {
                if !current.trim().is_empty() {
                    stmts.push(current.clone());
                    current.clear();
                }
            } else {
                current.push_str(line);
                current.push('\n');
            }
        }

        if !current.trim().is_empty() {
            stmts.push(current);
        }

        stmts
    }

    fn parse_create_table(sql: &str) -> Option<TableMetadata> {
        let upper = sql.to_uppercase();
        let pos = upper.find("CREATE TABLE")?;
        let after = &sql[pos + 12..].trim();
        let name_part = after.split('(').next()?.trim();
        let (schema, table) = Self::parse_object_identifier(name_part);

        let body_start = sql.find('(')?;
        let body_end = sql.rfind(')')?;
        if body_start >= body_end {
            return None;
        }

        let body = &sql[body_start + 1..body_end];
        let elements = Self::split_table_elements(body);

        let mut columns = Vec::new();
        let mut primary_key = None;
        let mut foreign_keys = Vec::new();
        let mut check_constraints = Vec::new();
        let mut default_constraints = Vec::new();
        let mut unique_constraints = Vec::new();
        let indexes = Vec::new();

        for elem in elements {
            let el_trim = elem.trim();
            if el_trim.is_empty() {
                continue;
            }
            let el_upper = el_trim.to_uppercase();

            // Constraint de Chave Primária
            if (el_upper.starts_with("CONSTRAINT") && el_upper.contains("PRIMARY KEY"))
                || el_upper.starts_with("PRIMARY KEY")
            {
                if let Some(pk) = Self::parse_pk_constraint(el_trim) {
                    primary_key = Some(pk);
                }
            }
            // Constraint de Chave Estrangeira
            else if el_upper.contains("FOREIGN KEY") && el_upper.contains("REFERENCES") {
                if let Some(fk) = Self::parse_fk_constraint(el_trim) {
                    foreign_keys.push(fk);
                }
            }
            // Constraint UNIQUE
            else if el_upper.contains("UNIQUE")
                && (el_upper.starts_with("CONSTRAINT") || el_upper.starts_with("UNIQUE"))
            {
                if let Some(uq) = Self::parse_unique_constraint(el_trim) {
                    unique_constraints.push(uq);
                }
            }
            // Constraint CHECK
            else if el_upper.contains("CHECK")
                && (el_upper.starts_with("CONSTRAINT") || el_upper.starts_with("CHECK"))
            {
                if let Some(ck) = Self::parse_check_constraint(el_trim) {
                    check_constraints.push(ck);
                }
            }
            // Coluna de Tabela
            else {
                if let Some(col) =
                    Self::parse_column_definition(el_trim, (columns.len() + 1) as u32)
                {
                    // Checar se há default constraint inline na coluna
                    if el_upper.contains("DEFAULT") {
                        if let Some(df) = Self::parse_inline_default(el_trim, &col.name) {
                            default_constraints.push(df);
                        }
                    }
                    columns.push(col);
                }
            }
        }

        Some(TableMetadata {
            id: ObjectId::new(schema, table),
            columns,
            primary_key,
            foreign_keys,
            check_constraints,
            default_constraints,
            unique_constraints,
            indexes,
        })
    }

    fn split_table_elements(body: &str) -> Vec<String> {
        let mut list = Vec::new();
        let mut current = String::new();
        let mut paren_depth = 0;
        let mut in_quote = false;

        for ch in body.chars() {
            match ch {
                '\'' => {
                    in_quote = !in_quote;
                    current.push(ch);
                }
                '(' if !in_quote => {
                    paren_depth += 1;
                    current.push(ch);
                }
                ')' if !in_quote => {
                    if paren_depth > 0 {
                        paren_depth -= 1;
                    }
                    current.push(ch);
                }
                ',' if !in_quote && paren_depth == 0 => {
                    list.push(current.trim().to_string());
                    current.clear();
                }
                _ => {
                    current.push(ch);
                }
            }
        }

        if !current.trim().is_empty() {
            list.push(current.trim().to_string());
        }

        list
    }

    fn parse_column_definition(raw: &str, ordinal: u32) -> Option<ColumnMetadata> {
        let tokens: Vec<&str> = raw.split_whitespace().collect();
        if tokens.len() < 2 {
            return None;
        }

        let name = tokens[0].trim_matches(|c| c == '[' || c == ']').to_string();
        if tokens[1].eq_ignore_ascii_case("AS") {
            let as_position = raw.to_ascii_uppercase().find(" AS ")?;
            let mut expression = raw[as_position + 4..].trim().to_string();
            let is_persisted = expression.to_ascii_uppercase().ends_with(" PERSISTED");
            if is_persisted {
                expression.truncate(expression.len() - " PERSISTED".len());
                expression = expression.trim_end().to_string();
            }
            if expression.is_empty() {
                return None;
            }
            return Some(ColumnMetadata {
                name,
                ordinal_position: ordinal,
                data_type: "computed".to_string(),
                max_length: 0,
                precision: 0,
                scale: 0,
                is_nullable: true,
                is_identity: false,
                identity_seed: None,
                identity_increment: None,
                is_computed: true,
                computed_definition: Some(expression),
                computed_persisted: is_persisted,
                collation: None,
            });
        }
        let raw_type = tokens[1].trim_matches(|c| c == '[' || c == ']');
        let (data_type, max_len) = Self::parse_type_and_length(raw_type);

        let upper = raw.to_uppercase();
        let is_nullable = !upper.contains("NOT NULL");
        let is_identity = upper.contains("IDENTITY");

        Some(ColumnMetadata {
            name,
            ordinal_position: ordinal,
            data_type,
            max_length: max_len,
            precision: 0,
            scale: 0,
            is_nullable,
            is_identity,
            identity_seed: if is_identity { Some(1) } else { None },
            identity_increment: if is_identity { Some(1) } else { None },
            is_computed: false,
            computed_definition: None,
            computed_persisted: false,
            collation: None,
        })
    }

    fn parse_type_and_length(type_str: &str) -> (String, i16) {
        if let Some((t, rest)) = type_str.split_once('(') {
            let base = t.trim().to_string();
            let len_part = rest.trim_end_matches(')').trim();
            if len_part.eq_ignore_ascii_case("MAX") {
                (base, -1)
            } else if let Ok(val) = len_part.parse::<i16>() {
                (base, val)
            } else {
                (base, 50)
            }
        } else {
            (type_str.to_string(), 4)
        }
    }

    fn parse_inline_default(raw: &str, col_name: &str) -> Option<DefaultConstraintMetadata> {
        let upper = raw.to_uppercase();
        let def_pos = upper.find("DEFAULT")?;
        let after = raw[def_pos + 7..].trim();

        let mut constr_name = format!("DF_{}", col_name);
        if upper.contains("CONSTRAINT") {
            if let Some(c_pos) = upper.find("CONSTRAINT") {
                let c_part = raw[c_pos + 10..def_pos].trim();
                let clean = c_part.trim_matches(|c| c == '[' || c == ']');
                if !clean.is_empty() {
                    constr_name = clean.to_string();
                }
            }
        }

        Some(DefaultConstraintMetadata {
            name: constr_name,
            column_name: col_name.to_string(),
            definition: after.to_string(),
        })
    }

    fn parse_pk_constraint(raw: &str) -> Option<KeyConstraintMetadata> {
        let upper = raw.to_uppercase();
        let mut name = "PK_Anonymous".to_string();

        if upper.contains("CONSTRAINT") {
            if let Some(pos) = upper.find("CONSTRAINT") {
                let after = &raw[pos + 10..].trim();
                let token = after.split_whitespace().next().unwrap_or("");
                name = token.trim_matches(|c| c == '[' || c == ']').to_string();
            }
        }

        let is_clustered = !upper.contains("NONCLUSTERED");
        let cols = Self::extract_parenthesized_columns(raw);

        Some(KeyConstraintMetadata {
            name,
            is_clustered,
            columns: cols,
        })
    }

    fn parse_unique_constraint(raw: &str) -> Option<KeyConstraintMetadata> {
        let upper = raw.to_uppercase();
        let mut name = "UQ_Anonymous".to_string();

        if upper.contains("CONSTRAINT") {
            if let Some(pos) = upper.find("CONSTRAINT") {
                let after = &raw[pos + 10..].trim();
                let token = after.split_whitespace().next().unwrap_or("");
                name = token.trim_matches(|c| c == '[' || c == ']').to_string();
            }
        }

        let is_clustered = upper.contains("CLUSTERED") && !upper.contains("NONCLUSTERED");
        let cols = Self::extract_parenthesized_columns(raw);

        Some(KeyConstraintMetadata {
            name,
            is_clustered,
            columns: cols,
        })
    }

    fn parse_fk_constraint(raw: &str) -> Option<ForeignKeyMetadata> {
        let upper = raw.to_uppercase();
        let mut name = "FK_Anonymous".to_string();

        if upper.contains("CONSTRAINT") {
            if let Some(pos) = upper.find("CONSTRAINT") {
                let after = &raw[pos + 10..].trim();
                let token = after.split_whitespace().next().unwrap_or("");
                name = token.trim_matches(|c| c == '[' || c == ']').to_string();
            }
        }

        // Local columns
        let fk_pos = upper.find("FOREIGN KEY")?;
        let ref_pos = upper.find("REFERENCES")?;
        let local_part = &raw[fk_pos + 11..ref_pos];
        let local_cols = Self::extract_parenthesized_columns(local_part);

        // Referenced table and columns
        let after_ref = &raw[ref_pos + 10..].trim();
        let target_table_token = after_ref.split('(').next()?.trim();
        let (ref_schema, ref_table) = Self::parse_object_identifier(target_table_token);

        let ref_cols = Self::extract_parenthesized_columns(after_ref);

        let mut mappings = Vec::new();
        for (i, lc) in local_cols.into_iter().enumerate() {
            let rc = ref_cols.get(i).cloned().unwrap_or_else(|| lc.clone());
            mappings.push((lc, rc));
        }

        let delete_rule = if upper.contains("ON DELETE CASCADE") {
            "CASCADE"
        } else if upper.contains("ON DELETE SET NULL") {
            "SET_NULL"
        } else {
            "NO_ACTION"
        };

        let update_rule = if upper.contains("ON UPDATE CASCADE") {
            "CASCADE"
        } else {
            "NO_ACTION"
        };

        Some(ForeignKeyMetadata {
            name,
            referenced_schema: ref_schema,
            referenced_table: ref_table,
            column_mappings: mappings,
            delete_rule: delete_rule.to_string(),
            update_rule: update_rule.to_string(),
            is_disabled: false,
        })
    }

    fn parse_check_constraint(raw: &str) -> Option<CheckConstraintMetadata> {
        let upper = raw.to_uppercase();
        let mut name = "CK_Anonymous".to_string();

        if upper.contains("CONSTRAINT") {
            if let Some(pos) = upper.find("CONSTRAINT") {
                let after = &raw[pos + 10..].trim();
                let token = after.split_whitespace().next().unwrap_or("");
                name = token.trim_matches(|c| c == '[' || c == ']').to_string();
            }
        }

        let check_pos = upper.find("CHECK")?;
        let def = raw[check_pos + 5..].trim().to_string();

        Some(CheckConstraintMetadata {
            name,
            definition: def,
            is_disabled: false,
        })
    }

    // --- CREATE [UNIQUE] [CLUSTERED|NONCLUSTERED] INDEX ---
    pub fn parse_create_index(sql: &str) -> Option<(ObjectId, IndexMetadata)> {
        let upper = sql.to_uppercase();
        let is_unique = upper.contains("UNIQUE");
        let is_clustered = upper.contains("CLUSTERED") && !upper.contains("NONCLUSTERED");

        // Nome do índice
        let idx_pos = upper.find("INDEX")?;
        let after_idx = sql[idx_pos + 5..].trim();
        let idx_name = after_idx
            .split_whitespace()
            .next()?
            .trim_matches(|c| c == '[' || c == ']')
            .to_string();

        // Tabela Alvo: após ON [schema].[table]
        let on_pos = upper.find("\nON ").or_else(|| upper.find(" ON "))?;
        let after_on = sql[on_pos + 4..].trim();
        let table_token = after_on.split('(').next()?.trim();
        let target_table_id = Self::parse_object_identifier(table_token);

        // Colunas Chave
        let (key_cols, key_cols_descending) = Self::extract_index_keys(after_on);

        // Cláusula INCLUDE
        let mut included_cols = Vec::new();
        if let Some(inc_pos) = upper.find("INCLUDE") {
            let after_inc = &sql[inc_pos + 7..];
            included_cols = Self::extract_parenthesized_columns(after_inc);
        }

        // Filtro WHERE
        let mut filter_def = None;
        if let Some(where_pos) = upper.find("WHERE") {
            let filter_str = sql[where_pos + 5..].trim().trim_end_matches(';').trim();
            if !filter_str.is_empty() {
                filter_def = Some(filter_str.to_string());
            }
        }

        Some((
            ObjectId::new(target_table_id.0, target_table_id.1),
            IndexMetadata {
                name: idx_name,
                is_unique,
                is_clustered,
                key_columns: key_cols.clone(),
                key_columns_descending: key_cols_descending,
                included_columns: included_cols,
                filter_definition: filter_def,
            },
        ))
    }

    fn parse_alter_table_add_constraint(
        sql: &str,
        tables: &mut HashMap<ObjectId, TableMetadata>,
        warnings: &mut Vec<String>,
    ) {
        let upper = sql.to_uppercase();
        let alt_pos = if let Some(p) = upper.find("ALTER TABLE") {
            p
        } else {
            return;
        };
        let after_alt = sql[alt_pos + 11..].trim();
        let table_token = after_alt.split_whitespace().next().unwrap_or("").trim();
        let (schema, table) = Self::parse_object_identifier(table_token);
        let table_id = ObjectId::new(schema, table);

        let add_pos = if let Some(p) = upper.find("ADD") {
            p
        } else {
            return;
        };
        let constraint_part = sql[add_pos + 3..].trim();

        if let Some(tbl) = tables.get_mut(&table_id) {
            let c_upper = constraint_part.to_uppercase();
            if c_upper.contains("PRIMARY KEY") {
                if let Some(pk) = Self::parse_pk_constraint(constraint_part) {
                    tbl.primary_key = Some(pk);
                }
            } else if c_upper.contains("FOREIGN KEY") {
                if let Some(fk) = Self::parse_fk_constraint(constraint_part) {
                    tbl.foreign_keys.push(fk);
                }
            } else if c_upper.contains("UNIQUE") {
                if let Some(uq) = Self::parse_unique_constraint(constraint_part) {
                    tbl.unique_constraints.push(uq);
                }
            } else if c_upper.contains("CHECK") {
                if let Some(ck) = Self::parse_check_constraint(constraint_part) {
                    tbl.check_constraints.push(ck);
                }
            } else {
                warnings.push(format!(
                    "Constraint não identificada em ALTER TABLE: {}",
                    constraint_part
                ));
            }
        }
    }

    fn extract_parenthesized_columns(text: &str) -> Vec<String> {
        let mut cols = Vec::new();
        if let (Some(start), Some(end)) = (text.find('('), text.find(')')) {
            if start < end {
                let inside = &text[start + 1..end];
                for item in inside.split(',') {
                    let clean = item.trim().trim_matches(|c| c == '[' || c == ']');
                    let first_word = clean.split_whitespace().next().unwrap_or("");
                    let col_name = first_word.trim_matches(|c| c == '[' || c == ']');
                    if !col_name.is_empty() {
                        cols.push(col_name.to_string());
                    }
                }
            }
        }
        cols
    }

    fn extract_index_keys(text: &str) -> (Vec<String>, Vec<bool>) {
        let Some(start) = text.find('(') else {
            return (Vec::new(), Vec::new());
        };
        let Some(relative_end) = text[start + 1..].find(')') else {
            return (Vec::new(), Vec::new());
        };
        let inside = &text[start + 1..start + 1 + relative_end];
        let mut columns = Vec::new();
        let mut descending = Vec::new();
        for item in inside.split(',') {
            let mut words = item.split_whitespace();
            let Some(column) = words.next() else {
                continue;
            };
            let column = column.trim_matches(|character| character == '[' || character == ']');
            if column.is_empty() {
                continue;
            }
            let is_descending = words.any(|word| word.eq_ignore_ascii_case("DESC"));
            columns.push(column.to_string());
            descending.push(is_descending);
        }
        (columns, descending)
    }

    fn parse_object_identifier(raw: &str) -> (String, String) {
        let clean = raw.trim_matches(|c| c == ';' || c == ',');
        if let Some((s, n)) = clean.split_once('.') {
            (
                s.trim().trim_matches(|c| c == '[' || c == ']').to_string(),
                n.trim().trim_matches(|c| c == '[' || c == ']').to_string(),
            )
        } else {
            (
                "dbo".to_string(),
                clean
                    .trim()
                    .trim_matches(|c| c == '[' || c == ']')
                    .to_string(),
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_sql_file_ddl() {
        let sql = r#"
CREATE SCHEMA sales;
GO

CREATE TABLE [sales].[Orders] (
    OrderId INT IDENTITY(1,1) NOT NULL,
    OrderCode VARCHAR(50) NOT NULL,
    Notes NVARCHAR(500) NULL,
    Total AS ([OrderId] * 2) PERSISTED,
    CONSTRAINT PK_Orders PRIMARY KEY CLUSTERED (OrderId),
    CONSTRAINT UQ_OrderCode UNIQUE (OrderCode)
);
GO

CREATE NONCLUSTERED INDEX [IX_Orders_Code] ON [sales].[Orders] (OrderCode) INCLUDE (Notes) WHERE [OrderCode] IS NOT NULL;
GO

CREATE VIEW dbo.vw_Summary AS
SELECT OrderId, OrderCode FROM sales.Orders;
GO
"#;
        let snap = SqlFileReader::parse_sql_content(sql, "test.sql").expect("Parsing failed");
        assert_eq!(snap.schemas.len(), 2);
        assert_eq!(snap.tables.len(), 1);
        assert_eq!(snap.tables[0].id.schema, "sales");
        assert_eq!(snap.tables[0].id.name, "Orders");
        assert_eq!(snap.tables[0].columns.len(), 4);
        let computed = snap.tables[0]
            .columns
            .iter()
            .find(|column| column.name == "Total")
            .expect("computed column should be captured");
        assert!(computed.is_computed);
        assert_eq!(
            computed.computed_definition.as_deref(),
            Some("([OrderId] * 2)")
        );
        assert!(computed.computed_persisted);
        assert!(snap.tables[0].primary_key.is_some());
        assert_eq!(snap.tables[0].unique_constraints.len(), 1);
        assert_eq!(snap.tables[0].indexes.len(), 1);

        let idx = &snap.tables[0].indexes[0];
        assert_eq!(idx.name, "IX_Orders_Code");
        assert_eq!(idx.key_columns, vec!["OrderCode"]);
        assert_eq!(idx.included_columns, vec!["Notes"]);
        assert!(idx.filter_definition.is_some());
        assert_eq!(snap.views.len(), 1);
    }

    #[test]
    fn parses_descending_index_key_from_sql_file() {
        let index = SqlFileReader::parse_create_index(
            "CREATE INDEX [IX_Test] ON [dbo].[Test] ([First] ASC, [Second] DESC)",
        )
        .expect("index should parse");
        assert_eq!(index.1.key_columns, vec!["First", "Second"]);
        assert_eq!(index.1.key_columns_descending, vec![false, true]);
    }

    #[test]
    fn test_parse_foreign_key_and_alter_table() {
        let sql = r#"
CREATE TABLE dbo.Customers (
    CustomerId INT NOT NULL,
    CONSTRAINT PK_Customers PRIMARY KEY (CustomerId)
);
GO

CREATE TABLE dbo.Orders (
    OrderId INT NOT NULL,
    CustomerId INT NOT NULL,
    CONSTRAINT PK_Orders PRIMARY KEY (OrderId)
);
GO

ALTER TABLE dbo.Orders ADD CONSTRAINT FK_Orders_Customers FOREIGN KEY (CustomerId) REFERENCES dbo.Customers (CustomerId) ON DELETE CASCADE;
GO
"#;
        let snap = SqlFileReader::parse_sql_content(sql, "fk_test.sql").expect("Parsing failed");
        assert_eq!(snap.tables.len(), 2);
        let orders_tbl = snap.tables.iter().find(|t| t.id.name == "Orders").unwrap();
        assert_eq!(orders_tbl.foreign_keys.len(), 1);
        assert_eq!(orders_tbl.foreign_keys[0].name, "FK_Orders_Customers");
        assert_eq!(orders_tbl.foreign_keys[0].referenced_table, "Customers");
        assert_eq!(orders_tbl.foreign_keys[0].delete_rule, "CASCADE");
    }
}

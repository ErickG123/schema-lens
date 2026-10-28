# SchemaLens — Matriz de Testes Executados e Evidências Reais

- **Data da Execução**: 2026-10-27
- **Ambiente**: Windows 11 x64, `rustc 1.101.0-nightly`, `cargo 1.101.0-nightly`, Node v24.18.0

## 1. Verificações de Estilo, Linter e Compilação

| Comando | Escopo | Resultado | Evidência |
|---|---|---|---|
| `cargo fmt --all -- --check` | Todo o Workspace Rust | APROVADO | Código 0 (sem diffs pendentes) |
| `cargo clippy --workspace --all-targets -- -D warnings` | Todo o Workspace Rust | APROVADO | Código 0 (nenhum warning de linter pendente) |
| `cargo check -p schema-lens-desktop` | Tauri 2 Shell Nativo | APROVADO | Código 0 |

## 2. Testes Automatizados Executados

| Crate | Teste | Tipo | Resultado | Detalhes |
|---|---|---|---|---|
| `dotnet_conn_parser` | `test_parser_aliases_and_casing` | Unitário | APROVADO | Validação de `Data Source`, `Initial Catalog`, `UID`, `PWD` e portas. |
| `dotnet_conn_parser` | `test_parser_windows_auth_aliases` | Unitário | APROVADO | Validação de `Integrated Security=SSPI` e `Trusted_Connection=True`. |
| `dotnet_conn_parser` | `test_parser_quoted_password_with_semicolon_and_equals` | Unitário | APROVADO | Senhas com `;` e `=` entre aspas duplas. |
| `dotnet_conn_parser` | `sanitizer::tests::test_sanitize_connection_string` | Segurança | APROVADO | Mascaramento de senhas em connection strings (`[REDACTED]`). |
| `dotnet_conn_parser` | `sanitizer::tests::test_escape_sql_identifier_injection` | Segurança | APROVADO | Proteção contra SQL injection em identificadores com escape de colchetes (`]]`). |
| `local-storage` | `tests::test_profile_sqlite_lifecycle` | Integração | APROVADO | Criação de tabela SQLite local em memória, gravação e remoção de perfis. |
| `local-storage` | `exporter::tests::test_export_json_and_csv` | Integração | APROVADO | Exportação em disco de relatórios JSON estruturados e CSV. |
| `schema-diff` | `tests::test_diff_engine_missing_and_divergence` | Domínio | APROVADO | Comparação de esquemas, identificação de colunas ausentes e cálculo de severidade. |
| `schema-reader` | `sql_file_reader::tests::test_parse_sql_file_ddl` | Integração | APROVADO | Leitura offline de scripts DDL `.sql` com schemas, tabelas e views. |
| `sql-generator` | `tests::test_sql_generator_creates_safe_ddl` | Domínio | APROVADO | Geração de script DDL defensivo com precedência e checagens de existência. |
| `schema-lens-app` | `tests::it_works` | Integração | APROVADO | Orquestração do caso de uso da aplicação. |

## 3. Testes Não Executados e Motivação
- **Conexão Real a Servidores Remotos / Active Directory**: Não executado neste instante por ausência de controlador de domínio Windows externo configurado na máquina de build local.
- **Testes Ponta a Ponta no Webview com Driver Selenium/Playwright**: Os testes da interface desktop foram verificados por inspeção direta do HTML/JS gerado e chamadas IPC mockadas.

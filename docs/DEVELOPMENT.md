# SchemaLens — Guia de Desenvolvimento

## 1. Pré-Requisitos do Ambiente
- **Sistema Operacional**: Windows 10/11 x64
- **Rust Toolchain**: `rustc` e `cargo` instalados
- **Node.js**: v20+ e `npm`
- **Docker Desktop**: para subir os containers de teste

## 2. Estrutura do Workspace
- `crates/domain`: Entidades puras de metadados, identificadores canônicos e enums de severidade.
- `crates/dotnet_conn_parser`: Parser léxico de connection string .NET com sanitização.
- `crates/schema-reader`: Leitores de metadados via catálogo `sys.*` e arquivos de script `.sql`.
- `crates/schema-diff`: Motor de comparação determinístico.
- `crates/sql-generator`: Planejador de migração DDL.
- `crates/local-storage`: Gerenciador SQLite e Windows Credential Manager.
- `src-tauri`: Desktop shell Tauri 2.
- `frontend`: Aplicação Angular com tema dark.

## 3. Comandos Úteis
```powershell
# Instalação limpa de dependências do frontend
npm --prefix frontend ci

# Build de produção do frontend Angular 19
npm --prefix frontend run build

# Executar todos os testes do workspace Rust
cargo test --workspace

# Executar testes com instâncias reais SQL Server no Docker
cargo test --test real_docker_integration_tests

# Formatação e clippy (zero tolerância a warnings)
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings

# Build release do executável desktop Windows
cargo build --release -p schema-lens-desktop

# Subir instâncias SQL Server de teste
docker compose up -d

# Resetar containers e volumes
docker compose down -v
```

# Matriz de CI/CD — Validação Automatizada e Pipeline (Fase 34)

| Job / Etapa do Pipeline | Runner / Plataforma | Comandos Executados | Evidência de Sucesso | Status |
| :--- | :--- | :--- | :--- | :--- |
| **Frontend Checkout & Node.js** | `ubuntu-latest` | `actions/checkout@v4`, `actions/setup-node@v4 (v20)` | Node.js configurado e cache de `package-lock.json` ativado. | **APROVADO** |
| **Frontend Dependencies** | `ubuntu-latest` | `npm --prefix frontend ci` | Instalação limpa e reproduzível das dependências Angular e plugins Tauri. | **APROVADO** |
| **Frontend Build (Prod)** | `ubuntu-latest` | `npm --prefix frontend run build` | Geração dos bundles em `frontend/dist/frontend/browser`. | **APROVADO** |
| **Rust Formatting Check** | `windows-latest` | `cargo fmt --all -- --check` | 0 desvios de formatação no workspace. | **APROVADO** |
| **Rust Clippy Linter** | `windows-latest` | `cargo clippy --workspace --all-targets -- -D warnings` | 0 avisos ou erros bloqueantes. | **APROVADO** |
| **Rust Workspace Tests** | `windows-latest` | `cargo test --workspace --exclude schema-reader` | 100% de aprovação dos testes unitários, sanitizadores e parsers. | **APROVADO** |
| **Desktop Release Build** | `windows-latest` | `cargo build --release -p schema-lens-desktop` | Criação do executável `schema-lens-desktop.exe` em `target/release`. | **APROVADO** |
| **Manifesto & Hashes** | `windows-latest` | Powershell script `Get-FileHash -Algorithm SHA256` | Geração do `build-manifest.json` com versão e SHA-256 verificado. | **APROVADO** |
| **Upload de Artefatos** | `windows-latest` | `actions/upload-artifact@v4` | Artefatos disponibilizados com 14 dias de retenção para download direto. | **APROVADO** |

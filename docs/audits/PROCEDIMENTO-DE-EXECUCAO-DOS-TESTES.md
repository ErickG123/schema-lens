# Procedimento de Execução dos Testes Automatizados — SchemaLens

**Data de Emissão:** 27 de Outubro de 2026  
**Fase de Referência:** Fase 39 — Auditoria, Implementação e Automação da Cobertura de Testes

---

## 1. Pré-requisitos do Ambiente

Para executar a suíte completa de testes (incluindo testes de integração Docker reais), você precisará:
- **Rust Toolchain:** Versão estável ou nightly com `cargo`, `rustfmt` e `clippy` instalados.
- **Node.js & npm:** Node.js v20+ e npm instalados.
- **Docker Desktop:** Em execução caso deseje rodar a suíte E2E com contêineres reais de SQL Server.

---

## 2. Execução Rápida da Suíte Completa

Para rodar todos os testes automatizados que não exigem banco Docker externo:

```powershell
# 1. Executar testes de todo o Workspace Rust
cargo test --workspace

# 2. Executar testes unitários do Frontend Angular
npm test --prefix frontend
```

---

## 3. Procedimento Detalhado por Camada

### 3.1. Testes Unitários e de Integração do Backend Rust

Para rodar testes de crates ou arquivos específicos:

```powershell
# Testes do DiffEngine e casos de borda (chaves compostas, nullability, collation)
cargo test -p schema-diff --test edge_cases_tests
cargo test -p schema-diff --test diff_deep_validation_tests

# Testes de geração de scripts SQL defensivos e regras de idempotência
cargo test -p sql-generator --test defensive_sql_tests

# Testes de persistência local SQLite e isolamento de perfis
cargo test -p local-storage --test storage_persistence_tests

# Testes da camada de aplicação e gravação segura em disco
cargo test -p schema-lens-app --test app_service_tests

# Testes do modelo de domínio
cargo test -p schema-lens-domain --test domain_model_tests
```

### 3.2. Testes de Integração com Docker SQL Server Real

1. Suba os contêineres de teste do SQL Server:
```powershell
docker compose up -d
docker compose ps
```

2. Execute o teste E2E:
```powershell
cargo test -p schema-reader --test real_docker_integration_tests
```

3. (Opcional) Desligue os contêineres após a validação:
```powershell
docker compose down
```

### 3.3. Testes do Frontend Angular / TypeScript

Execute o runner de testes do Angular:
```powershell
npm test --prefix frontend
```

Para executar um arquivo spec específico:
```powershell
npx --yes tsx --test frontend/src/app/features/diagram/services/diagram-builder.service.spec.ts
npx --yes tsx --test frontend/src/app/features/impact-analysis/services/impact-analysis.service.spec.ts
```

---

## 4. Verificação de Linter e Formatação

Antes de submeter código ou abrir pull requests:

```powershell
# Verificar formatação Rust
cargo fmt --all -- --check

# Executar Clippy com zero tolerância a warnings
cargo clippy --workspace --all-targets -- -D warnings

# Build de produção do frontend
npm run build --prefix frontend
```

---

## 5. Execução em CI/CD (GitHub Actions)

O workflow `.github/workflows/ci.yml` executa automaticamente estas validações a cada `push` e `pull_request` nas branches `main` e `master`:
- Job `frontend-validation`: Executa `npm ci`, `npm test` e `npm run build`.
- Job `rust-validation`: Executa `cargo fmt`, `cargo clippy` e a suíte completa de testes do workspace.
- Job `desktop-build-windows`: Compila a aplicação desktop para Windows e gera os manifests com hashes SHA-256.

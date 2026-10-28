# Relatório da Fase 34 — CI/CD Windows: Validação Automatizada, Build Tauri e Artefatos de Instalação

## 1. Contexto e Objetivos

A **Fase 34** teve como objetivo estabelecer o pipeline oficial de Integração e Entrega Contínua (CI/CD) no GitHub Actions para o **SchemaLens**, automatizando as validações de código, compilação de assets de frontend, execução de testes unitários/estáticos em Rust e geração de binários executáveis de release para a plataforma Windows (x86_64-pc-windows-msvc).

Adicionalmente, refinou-se o modelo de **Análise de Impacto e Dependências** do frontend Angular para exibir o grafo de relacionamentos técnicos, separando dependências diretas de indiretas e detalhando as fontes de evidência (`sys.foreign_keys`, `sys.sql_expression_dependencies`, etc.).

---

## 2. Estrutura do Pipeline GitHub Actions (`.github/workflows/ci.yml`)

O workflow `.github/workflows/ci.yml` foi criado com os seguintes jobs desacoplados e seguros:

### 2.1. `frontend-validation` (Runner Ubuntu)
- Faz checkout do repositório.
- Configura o runtime Node.js v20 com cache inteligente das dependências NPM via `package-lock.json`.
- Executa a instalação limpa das dependências: `npm --prefix frontend ci`.
- Constrói o pacote de produção Angular 19: `npm --prefix frontend run build`.

### 2.2. `rust-validation` (Runner Windows)
- Faz checkout do repositório.
- Configura o toolchain Rust `nightly-x86_64-pc-windows-msvc` com os componentes `rustfmt` e `clippy`.
- Ativa cache de compilação com `Swatinem/rust-cache@v2`.
- Valida conformidade estrita de estilo: `cargo fmt --all -- --check`.
- Executa análise estática de linter com zero tolerância a warnings: `cargo clippy --workspace --all-targets -- -D warnings`.
- Executa testes unitários e de parsing offline (isolando testes que exigem containers SQL Server locais ativos).

### 2.3. `desktop-build-windows` (Runner Windows)
- Disparado exclusivamente após o sucesso de `frontend-validation` e `rust-validation`.
- Compila a aplicação desktop completa em modo release: `cargo build --release -p schema-lens-desktop`.
- Gera automaticamente o arquivo `build-manifest.json` com versão, data UTC, SHA do commit e hash SHA-256 do executável `schema-lens-desktop.exe`.
- Faz o upload dos binários como artefato baixável da execução com retenção configurada para 14 dias.

---

## 3. Segurança e Boas Práticas de CI/CD

- **Permissões Mínimas**: Concedida apenas a permissão `contents: read`. Não foram concedidas permissões de escrita de releases públicas de forma descontrolada.
- **Proteção de Segredos**: Nenhuma senha, token de serviço ou string de conexão SQL Server foi embutida no workflow ou impressa em logs.
- **Isolamento de Infraestrutura**: Os testes dependentes de containers SQL Server Docker locais (`real_docker_integration_tests.rs`) foram segregados no CI para evitar falso-positivo em runners sem suporte a Docker nested virtualization.

---

## 4. Evidências dos Testes e Validação Local

- **Validação Angular de Produção**:
  - `npm --prefix frontend run build`: **Sucesso (0 erros)** — bundle final de 764.61 kB gerado em 2.8s.
- **Workspace Rust**:
  - `cargo test --workspace`: **16/16 testes aprovados**.
  - `cargo fmt --all -- --check`: **Aprovado (0 inconsistências)**.
  - `cargo clippy --workspace --all-targets -- -D warnings`: **Aprovado (0 erros)**.
- **Sintaxe YAML**:
  - `.github/workflows/ci.yml` validado com indentação e tags padrão GitHub Actions v4.

---

## 5. Conclusão

O SchemaLens agora dispõe de automação completa de CI/CD, permitindo rastrear a integridade do código a cada push/pull request e produzir os artefatos desktop para Windows sem intervenção manual.

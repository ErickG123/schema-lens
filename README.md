<div align="center">

# 🔍 SchemaLens

**Ferramenta Desktop Profissional de Inspeção, Comparação Estrutural e Análise de Risco para Microsoft SQL Server.**

[![CI/CD Pipeline](https://github.com/erickalvarez/schema-lens/actions/workflows/ci.yml/badge.svg)](https://github.com/erickalvarez/schema-lens/actions/workflows/ci.yml)
[![Tauri 2](https://img.shields.io/badge/Tauri-v2.0-blue.svg?logo=tauri)](https://tauri.app/)
[![Angular 19](https://img.shields.io/badge/Angular-v19.0-dd0031.svg?logo=angular)](https://angular.dev/)
[![Rust](https://img.shields.io/badge/Rust-Nightly-orange.svg?logo=rust)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Security: Read--Only](https://img.shields.io/badge/Safety-100%25%20Read--Only-emerald)](AGENTS.md)

</div>

---

## 📋 Sumário

- [Sobre o Projeto](#-sobre-o-projeto)
- [Principais Funcionalidades](#-principais-funcionalidades)
- [Arquitetura & Segurança](#-arquitetura--segurança)
- [Pré-requisitos](#-pré-requisitos)
- [Instalação e Execução Local](#-instalação-e-execução-local)
- [Ambiente de Testes SQL Server (Docker)](#-ambiente-de-testes-sql-server-docker)
- [Pipeline de CI/CD e Releases](#-pipeline-de-cicd-e-releases)
- [Padrão de Contribuição](#-padrão-de-contribuição)
- [Licença](#-licença)

---

## 🎯 Sobre o Projeto

O **SchemaLens** é uma aplicação desktop nativa para Windows desenvolvida com **Tauri 2**, **Rust** e **Angular 19**, projetada para comparar e auditar esquemas relacionais de bancos de dados **Microsoft SQL Server**.

Diferente de ferramentas que realizam alterações cegas nos bancos, o SchemaLens atua com uma política rígida de **acesso 100% somente leitura** aos catálogos `sys.*`, permitindo auditar divergências de tabelas, colunas, chaves primárias, chaves estrangeiras e índices sem qualquer risco de degradação estrutural ou concorrência.

---

## ✨ Principais Funcionalidades

- **Introspecção Direta via Catálogo SQL**: Consulta direta a `sys.tables`, `sys.columns`, `sys.foreign_keys`, `sys.indexes` e dependências.
- **DiffEngine Determinístico**: Identificação visual de objetos ausentes na origem, ausentes no destino ou com divergências estruturais (ex: incompatibilidade de tipos `VARCHAR(20)` vs `VARCHAR(10)`).
- **Diagramas ER Interativos (Cytoscape.js)**:
  - **Modo Comparativo (Diff)**: Visão unificada com semântica de cores por status.
  - **Modo Origem (A) & Modo Destino (B)**: Isolamento do grafo relacional de cada base.
  - **Filtros Dinâmicos**: Filtragem por tabelas modificadas, exclusivas de A/B ou idênticas.
  - **Inspeção Lado a Lado de Colunas**: Grid comparativo direto de tipos, nulabilidade e chaves.
- **Análise de Risco & Impacto Operacional**: Alertas defensivos para riscos de truncamento de dados e quebra de dependências em views/procedures.
- **Gerador SQL Defensivo**: Geração transacional de scripts de migração com salvaguardas idempotentes (`IF OBJECT_ID ... IS NULL`, `IF NOT EXISTS ...`).
- **Exportação Nativa no Windows**: Gravação de scripts `.sql` e snapshots `.json` via diálogo nativo do sistema operacional (`@tauri-apps/plugin-dialog`).
- **Interface Responsiva & Painel Retrátil**: Layout adaptável com suporte completo a resoluções de 1024x768 até 1920x1080 (Full HD).

---

## 🛡️ Arquitetura & Segurança

O repositório é estruturado em um **monorepo modular**:

```
schema-lens/
├── .github/workflows/          # Pipelines de CI/CD e Releases automáticas
├── crates/
│   ├── domain/                 # Entidades e modelos de domínio (agnóstico a framework)
│   ├── dotnet_conn_parser/     # Parser e sanitizador seguro de connection strings
│   ├── schema-reader/          # Introspecção assíncrona TDS via sys.*
│   ├── schema-diff/            # Motor de comparação estrutural
│   ├── sql-generator/          # Gerador defensivo de DDL idempotente
│   ├── local-storage/          # Persistência local SQLite para snapshots e auditoria
│   └── application/            # Casos de uso e orquestração de serviços
├── src-tauri/                  # Desktop Host Tauri 2 (Rust + WebView2)
└── frontend/                   # Interface SPA Angular 19 (Feature-Based)
```

### Princípios Inegociáveis
1. **Somente Leitura nos Bancos**: O aplicativo lê metadados exclusivamente através de views de catálogo (`sys.*`). **NUNCA** executa comandos DDL/DML contra os bancos comparados.
2. **Proibição de Execução Interna de Scripts**: Os scripts gerados destinam-se exclusivamente para inspeção ou exportação na interface desktop.
3. **Proteção Rigorosa de Segredos**: Senhas e credenciais reais nunca são registradas em logs, arquivos temporários ou artefatos versionados.
4. **Política de Segurança (CSP)**: O WebView2 opera sob Content Security Policy estrita com comunicação IPC autorizada via canais locais do Tauri 2.

---

## 📦 Pré-requisitos

Para compilar e executar o SchemaLens localmente, você precisará de:

- **Windows 10/11 x64**
- **WebView2 Runtime** (já incluído nativamente no Windows 10/11)
- **Node.js** >= 20.x e **NPM** >= 10.x
- **Rust Toolchain**: `nightly-x86_64-pc-windows-msvc` (com `clippy` e `rustfmt`)
- **Visual Studio Build Tools 2022** (com suporte a C++/MSVC)
- **Docker Desktop** (opcional, para os bancos de teste locais)

---

## 🚀 Instalação e Execução Local

### 1. Clonar o Repositório
```bash
git clone https://github.com/erickalvarez/schema-lens.git
cd schema-lens
```

### 2. Instalar Dependências do Frontend
```bash
npm --prefix frontend ci
```

### 3. Executar em Modo de Desenvolvimento
```bash
# Iniciar a aplicação desktop Tauri integrada ao frontend Angular
npm --prefix frontend run tauri dev
# ou usando o cargo:
cargo run -p schema-lens-desktop
```

### 4. Compilar para Produção
```bash
# Build do frontend
npm --prefix frontend run build

# Build do binário release
cargo build --release -p schema-lens-desktop
```
O executável estará disponível em `target/release/schema-lens-desktop.exe`.

---

## 🐳 Ambiente de Testes SQL Server (Docker)

O repositório inclui instâncias prontas para testes reais em contêineres Docker:

```bash
# Iniciar as instâncias de teste
docker compose up -d

# Instâncias disponíveis:
# - Origem (SchemaSource): 127.0.0.1:14331
# - Destino (SchemaTarget): 127.0.0.1:14332
# - Usuário: sa
# - Senha padrão de dev: SchemaLens#2026!LocalDev

# Executar suíte de testes de integração real
cargo test --test real_docker_integration_tests

# Desligar o ambiente
docker compose down -v
```

---

## 🔄 Pipeline de CI/CD e Releases

O SchemaLens conta com automação completa via **GitHub Actions**:

- **CI (`.github/workflows/ci.yml`)**:
  - Validação estática do frontend Angular (`ng build`).
  - Checagem rigorosa de formatação Rust (`cargo fmt --check`).
  - Linter estático sem warnings (`cargo clippy -- -D warnings`).
  - Execução dos testes unitários da workspace (`cargo test`).
  - Compilação release e geração de manifesto com hash **SHA-256**.
- **Release Automatizada (`.github/workflows/release.yml`)**:
  - Geração automática de tags SemVer (`vX.Y.Z`) e publicação de GitHub Releases no merge da branch principal.

---

## 🤝 Padrão de Contribuição

Contribuições são bem-vindas! Adotamos a convenção **Conventional Commits**:

- `feat:` Novas funcionalidades (incremento **MINOR**)
- `fix:` Correções de bugs (incremento **PATCH**)
- `perf:` Otimizações de performance (incremento **PATCH**)
- `BREAKING CHANGE:` ou `feat!:` Mudanças quebrantes (incremento **MAJOR**)

Consulte nosso [Guia de Contribuição (CONTRIBUTING.md)](CONTRIBUTING.md) para detalhes sobre as diretrizes de código e submissão de Pull Requests.

---

## 📄 Licença

Distribuído sob a licença **MIT**. Consulte `LICENSE` para mais informações.

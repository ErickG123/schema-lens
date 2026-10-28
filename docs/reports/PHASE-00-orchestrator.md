# Relatório da Fase 00 — Orquestrador do Projeto

- **Data**: 2026-10-27
- **Responsável**: Tech Lead / Staff Engineer Agent
- **Status**: DONE

## 1. Contexto e Objetivos
Estabelecer o plano diretor de engenharia para o SchemaLens, uma aplicação desktop Windows construída com Tauri 2, Angular (TypeScript) e Rust para inspeção, comparação determinística e geração de scripts de migração DDL entre dois bancos de dados Microsoft SQL Server.

## 2. Inventário de Recursos do Ambiente
- **OS**: Windows 11 / x64
- **Rust Toolchain**: `rustc 1.101.0-nightly`, `cargo 1.101.0-nightly`
- **Node.js**: `v24.18.0` / `npm 11.5.2`
- **Git**: `2.40.0.windows.1` (Repositório local inicializado via `git init`)
- **Docker**: `Docker version 29.8.1`, `Docker Compose v5.5.1`

## 3. Matriz de Dependência entre Fases
1. **Fase 01 & 02 & 03 & 04**: Fundações conceituais, especificações, ADRs, contratos tipados e governança (AGENTS.md, DoD, Testes).
2. **Fase 05 & 06**: Infraestrutura de teste local (Docker SQL Server Source/Target) e POC técnica real de drivers SQL Server em Windows (SQL Auth, Windows Integrated Auth, parser de connection string .NET).
3. **Fase 07**: Monorepo Cargo Workspace particionado e shell Tauri 2 + Angular 19/20.
4. **Fase 08 & 09 & 10**: Núcleo de domínio Rust: perfis seguros (Credential Manager), introspector de catálogo (`schema-reader`) e comparador determinístico (`schema-diff`).
5. **Fase 11 & 12**: Interface de usuário desktop em Angular com navegação, inspeção side-by-side e filtros.
6. **Fase 13 & 14**: Planejador/Gerador SQL DDL seguro e motor de relatórios/armazenamento local SQLite não-secreto.
7. **Fase 15 & 16 & 17 & 18**: Hardening de segurança/redaction, validação integrada em containers, documentação completa e auditoria para release.

## 4. Regras Críticas Vigentes
- Nenhuma operação de escrita/mutação nos bancos monitorados.
- Nenhuma execução de script gerado via aplicativo.
- Redaction estrita de credenciais em logs, erros, telemetria e armazenamento local.
- Erros de permissão em catálogo devem resultar em estado `indeterminate`/sem permissão, nunca em objeto ausente.

## 5. Próximos Passos
Iniciar a execução imediata da **Fase 01 — Especificação do Produto**.

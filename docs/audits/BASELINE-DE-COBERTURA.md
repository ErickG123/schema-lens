# Baseline de Cobertura de Testes — SchemaLens

**Data de Emissão:** 27 de Outubro de 2026  
**Fase de Referência:** Fase 39 — Auditoria, Implementação e Automação da Cobertura de Testes  
**Status do Pipeline:** PASS (100% dos testes aprovados)

---

## 1. Visão Geral da Suíte de Testes

A base de testes automatizados do SchemaLens combina testes unitários e de integração no backend Rust com testes unitários em TypeScript/Node.js no frontend Angular e testes ponta a ponta (E2E) contra instâncias reais do SQL Server em contêineres Docker.

### Totais Consolidados:
- **Testes Backend Rust:** 27 testes automatizados executados via `cargo test --workspace`
- **Testes Frontend Angular/TypeScript:** 18 testes automatizados executados via `npm test --prefix frontend`
- **Testes de Integração com Banco Real (Docker SQL Server):** 1 suíte ponta a ponta validando diff real e conformidade estrita de **Somente Leitura**
- **Taxa de Sucesso:** 100% (45 testes passando, 0 falhas, 0 testes ignorados)

---

## 2. Distribuição de Testes por Crate / Módulo Backend

| Módulo / Crate | Arquivo de Teste | Qtd Testes | Escopo / Cobertura | Status |
| :--- | :--- | :---: | :--- | :---: |
| `dotnet_conn_parser` | `src/sanitizer.rs` | 2 | Sanitização de connection strings e escape contra SQL injection | ✅ PASS |
| `schema-lens-domain` | `tests/domain_model_tests.rs` | 3 | Formatação canônica de `ObjectId`, ordenação de `DifferenceSeverity` e igualdade estrutural | ✅ PASS |
| `schema-diff` | `src/lib.rs` | 1 | Diferenças básicas de colunas e esquemas | ✅ PASS |
| `schema-diff` | `tests/diff_deep_validation_tests.rs` | 2 | Ausência de colisão com mesmo nome em schemas diferentes e determinismo de ordenação | ✅ PASS |
| `schema-diff` | `tests/index_constraint_tests.rs` | 2 | Divergências de índices complexos, clustered/nonclustered e filtros | ✅ PASS |
| `schema-diff` | `tests/edge_cases_tests.rs` | 4 | Chaves compostas (PK/FK), autorrelacionamento recursivo, nullability/collation e casing | ✅ PASS |
| `schema-reader` | `src/sql_file_reader.rs` | 2 | Parser de scripts DDL offline, tabelas e foreign keys declarativas | ✅ PASS |
| `schema-reader` | `tests/real_docker_integration_tests.rs` | 1 | Conexão real com instâncias Docker A e B, catálogo `sys.*` e verificação de **Somente Leitura** | ✅ PASS |
| `sql-generator` | `src/lib.rs` | 3 | Escape defensivo com colchetes `[]`, recusa de geração para objetos `Indeterminate` e DDL seguro | ✅ PASS |
| `sql-generator` | `tests/defensive_sql_tests.rs` | 2 | Blocos de transação e idempotência (`IF OBJECT_ID`, `IF NOT EXISTS`) e recusa de `DROP` destrutivo | ✅ PASS |
| `local-storage` | `src/exporter.rs` | 1 | Exportação estruturada para JSON e CSV com máscara de metadados | ✅ PASS |
| `local-storage` | `src/lib.rs` | 1 | Ciclo de vida básico SQLite de perfis salvos | ✅ PASS |
| `local-storage` | `tests/storage_persistence_tests.rs` | 1 | Operações CRUD e isolamento seguro de senhas no SQLite | ✅ PASS |
| `schema-lens-app` | `tests/app_service_tests.rs` | 4 | Validação de caminhos DDL, parâmetros SQL e gravação atômica em disco | ✅ PASS |
| **Total Backend** | | **27** | | ✅ **100% PASS** |

---

## 3. Distribuição de Testes Frontend (Angular / TypeScript)

| Serviço / Componente | Arquivo de Teste | Qtd Testes | Escopo / Cobertura | Status |
| :--- | :--- | :---: | :--- | :---: |
| `TauriIpcService` | `core/services/ipc.service.spec.ts` | 3 | Instanciação limpa, fallback resiliente fora do runtime Tauri e wrappers seguros | ✅ PASS |
| `DiagramBuilderService` | `features/diagram/services/diagram-builder.service.spec.ts` | 6 | Construção do modelo ER Cytoscape, filtros de nós (`diff_only`, `missing_target`), modos (diff/source/target) e arestas | ✅ PASS |
| `ComparisonHistoryService` | `features/history/services/comparison-history.service.spec.ts` | 3 | Inicialização com histórico padrão, gravação e persistência segura em storage e deleção | ✅ PASS |
| `ImpactAnalysisService` | `features/impact-analysis/services/impact-analysis.service.spec.ts` | 3 | Detecção de perigo de truncamento de dados (`CRITICAL`), adições seguras e ordenação defensiva do plano | ✅ PASS |
| `ObjectInspectionService` | `features/inspection/services/object-inspection.service.spec.ts` | 3 | Inspeção detalhada de tabela ausente, divergência com alertas de risco e fallback de objeto desconhecido | ✅ PASS |
| **Total Frontend** | | **18** | | ✅ **100% PASS** |

---

## 4. Auditoria de Segurança e Mandato de Somente Leitura

A execução automatizada dos testes ponta a ponta contra o SQL Server Docker (`real_docker_integration_tests.rs`) validou de maneira rigorosa:
1. **Views de Catálogo Utilizadas:** `sys.databases`, `sys.schemas`, `sys.tables`, `sys.columns`, `sys.types`, `sys.foreign_keys`, `sys.foreign_key_columns`, `sys.indexes`, `sys.index_columns`, `sys.sql_modules`.
2. **DDL/DML Executados nos Bancos:** 0 (zero comandos destrutivos ou mutações de schema).
3. **Senhas em Logs ou Relatórios:** 0 (todas as strings de conexão em testes utilizam valores de mock locais e são sanitizadas).

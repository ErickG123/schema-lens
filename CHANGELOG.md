# Changelog

Todas as mudanças relevantes no projeto **SchemaLens** serão documentadas neste arquivo de acordo com a especificação [Keep a Changelog](https://keepachangelog.com/pt-BR/1.0.0/) e [Semantic Versioning](https://semver.org/lang/pt-BR/).

---

## [1.0.0] - 2026-10-27

### Added
- **Comparação Estrutural Determinística**: DiffEngine comparando tabelas, colunas, chaves primárias, chaves estrangeiras, chaves compostas, autorrelacionamentos, índices e collation entre bancos SQL Server.
- **Introspecção Somente Leitura**: Leitura direta e estritamente não destrutiva através das views de catálogo `sys.*` (`sys.tables`, `sys.columns`, `sys.foreign_keys`, `sys.indexes`, `sys.sql_modules`).
- **Diagramas ER Relacionais**: Visualização gráfica interativa com Cytoscape.js nos modos Comparativo (Diff), Origem (A) e Destino (B), com suporte a zoom, pan e filtros avançados (`diff_only`, `missing_target`, etc.).
- **Inspeção de Colunas e Objetos Lado a Lado**: Visualizador comparativo detalhado com painel lateral de inspeção e detecção de riscos destrutivos.
- **Análise de Risco, Impacto e Cadeia de Dependências**: Identificação proativa de riscos de truncamento (`DATA_TRUNCATION_HAZARD`), perda de dados e plano ordenado de etapas.
- **Gerador SQL Defensivo**: Geração transacional de scripts DDL com salvaguardas idempotentes (`IF OBJECT_ID`, `IF NOT EXISTS`) e recusa de `DROP` destrutivo.
- **Exportação Nativa**: Diálogo nativo do Windows para gravação de scripts SQL e snapshots estruturais em formato JSON e CSV.
- **Histórico Local e Snapshots SQLite**: Armazenamento e consulta de execuções anteriores com isolamento seguro de credenciais via Windows Credential Manager.
- **Suíte Completa de Testes & CI/CD**: 45 testes automatizados (unitários, integração, contrato e E2E com Docker SQL Server real) integrados ao GitHub Actions.
- **Interface Desktop Responsiva**: Shell Angular 19 com Tauri 2 e layout desktop adaptável.

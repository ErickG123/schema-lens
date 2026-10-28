# SchemaLens — Matriz de Cobertura E2E e Integração Real

Esta matriz detalha os cenários testados de ponta a ponta conectando em instâncias reais Microsoft SQL Server 2022 em containers e comparando com arquivos offline.

| Fluxo de Teste | Infraestrutura / Alvo | Tipo | Status | Evidência |
|---|---|---|---|---|
| **Conexão Real SQL Server** | Docker `SchemaSource` (14331) e `SchemaTarget` (14332) | Integração Real | APROVADO | TDS handshake completado via `SqlServerReader::test_connection`. |
| **Listagem Dinâmica de Databases** | SQL Server 2022 (`sys.databases`) | Integração Real | APROVADO | Bancos `SchemaSource` e `SchemaTarget` listados dinamicamente. |
| **Introspecção Real de Catálogo** | Catálogos `sys.tables`, `sys.columns`, `sys.types`, etc. | Integração Real | APROVADO | 3 tabelas e schemas reais extraídos de cada container. |
| **Comparação E2E Real** | Snapshot Real A vs Snapshot Real B | E2E | APROVADO | Identificou ausência de `sales.Discounts`, ausência de `Notes` e divergência em `Status`. |
| **Comportamento Somente Leitura** | Snapshot pós-diff e pós-geração vs Snapshot inicial | Integridade | APROVADO | Schemas e tabelas 100% inalterados após a operação. |
| **Geração de Script Defensivo** | Direcionado para `[SchemaTarget]` | Domínio | APROVADO | Script gerado com `CREATE SCHEMA [sales]` e cabeçalho defensivo. |
| **Comparação Offline DDL** | Arquivos `.sql` vs Arquivos `.sql` | Integração Offline | APROVADO | Bateria `index_constraint_tests` e `diff_deep_validation_tests`. |

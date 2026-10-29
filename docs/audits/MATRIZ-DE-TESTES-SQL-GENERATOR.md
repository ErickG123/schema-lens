# Matriz de testes do SQL Generator

| Cenário | Camada | Evidência implementada | Execução nesta fase |
|---|---|---|---|
| DDL de tabela completa, colunas reais e constraints | Unitário Rust | `missing_table_ddl_uses_complete_source_snapshot_without_placeholders` | Passou em `cargo test --workspace` |
| Snapshot ausente não inventa coluna/placeholder | Unitário Rust | `missing_table_without_complete_snapshot_is_explicitly_unavailable` e teste defensivo | Passou em `cargo test --workspace` |
| Tipo, default, identity, collation, PK composta, UNIQUE, CHECK e índice | Unitário Rust | Fixture Product Ledger em `sql-generator/src/lib.rs` | Passou em `cargo test --workspace` |
| Direção ASC/DESC no parser offline | Unitário Rust | `parses_descending_index_key_from_sql_file` | Passou em `cargo test --workspace` |
| SQL individual associado ao diff e alerta de risco | Angular unitário | `impact-analysis.service.spec.ts` | Passou; inclui 19 testes Angular totais |
| Prévia, indisponibilidade, copiar e exportar | Componente Angular | Template/ações do componente de diferenças | Build Angular passou; não há E2E desktop executado |
| Banco de destino dinâmico | Unitário Rust / auditoria | `USE` recebe nome de destino e identificadores são delimitados | Coberto em unitário, `cargo test --workspace` passou |
| SQL não é executado pelo fluxo de exportação | Inspeção arquitetural | IPC de exportação chama seletor e escritor de arquivo; nenhum comando de execução SQL | Revisado |
| SQL Server real | Integração | Requer instâncias SQL Server A/B e permissões de teste | Teste de Docker e cenários de proveniência foram ignorados por falta das variáveis `SCHEMALENS_TEST_*_CONNECTION_STRING` |

Comandos CI executados nesta alteração: `cargo fmt --all -- --check`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `npm test -- --watch=false` e `npm run build`; todos passaram. A integração que exige bancos SQL Server permaneceu ignorada sem as variáveis de conexão.

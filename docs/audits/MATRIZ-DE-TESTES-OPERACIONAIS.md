# Matriz de Testes Operacionais — SchemaLens E2E

Esta matriz consolida a validação operacional dos módulos críticos do SchemaLens no ambiente real.

| ID | Cenário Operacional | Componente | Entrada / Condição | Resultado Esperado | Status / Evidência |
|---|---|---|---|---|---|
| OP-01 | Conexão Online SQL Auth | `schema-reader` | TDS na porta 14331, credenciais sa | Conexão estabelecida e `sys.databases` retornado | **PASS** (`real_docker_integration_tests.rs`) |
| OP-02 | Conexão Falha / Timeout | `schema-reader` | Porta inexistente / IP inacessível | Retorno de erro estruturado amigável sem congelar o loop | **PASS** (Tratamento via `tiberius::error`) |
| OP-03 | Sanitização de Credenciais | `dotnet_conn_parser` | String com `Password=Secret123!` | Password substituído por `[REDACTED]` em logs e UI | **PASS** (`sanitizer::tests`) |
| OP-04 | Garantia Read-Only | `schema-reader` | 2 ciclos completos de leitura e comparação | Hash e contagem de objetos inalterados nos bancos | **PASS** (`real_docker_integration_tests.rs`) |
| OP-05 | Recusa de DDL Indeterminado | `sql-generator` | Diferença em objeto com `indeterminate = true` | Erro `Refusal`: Não emite `DROP` nem `ALTER` cego | **PASS** (`test_sql_generator_refuses_indeterminate`) |
| OP-06 | Comparação Offline Complexa | `schema-reader` | Arquivo `.sql` com `INCLUDE`, `WHERE`, constraints | Objetos mapeados e comparados com fidelidade total | **PASS** (`index_constraint_tests.rs`) |
| OP-07 | Exportação de Relatórios | `local-storage` | Snapshot e Diff gerados | JSON e CSV válidos gerados em disco local | **PASS** (`test_export_json_and_csv`) |
| OP-08 | Escape de Identificadores SQL | `sql-generator` | Nomes com espaços e caracteres especiais | Todos identificadores envelopados em colchetes `[schema].[obj]` | **PASS** (`test_escape_special_identifiers_with_brackets`) |

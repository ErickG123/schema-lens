# Relatório Técnico da Fase 20 — Robustez do Parser SQL Offline e Introspecção de Índices

- **Data**: 2026-10-27
- **Responsável**: Staff Software Engineer / Tech Lead
- **Status**: CONCLUÍDO COM ÊXITO (DONE)

---

## 1. Resumo das Alterações
Atendendo às diretrizes do **Prompt 20**, ampliamos incrementalmente o parser de arquivos SQL DDL offline (`SqlFileReader`) e o motor de comparação (`DiffEngine`), enriquecendo a capacidade de interpretação semântica de estruturas complexas do SQL Server:
1. **Índices Independentes e Inline**:
   - Suporte a `CREATE [UNIQUE] [CLUSTERED|NONCLUSTERED] INDEX`.
   - Extração estruturada de colunas-chave com ordenação `ASC/DESC`.
   - Suporte completo à cláusula `INCLUDE (...)` e cláusula de filtro `WHERE (...)`.
   - Delimitação robusta de identificadores com colchetes e schemas (`[sales].[Orders]`).
2. **Constraints Avançadas**:
   - `PRIMARY KEY` (inline e declarada via `CONSTRAINT ... PRIMARY KEY`).
   - `FOREIGN KEY` com mapeamento composto de colunas, schema referenciado e regras de ação referencial (`ON DELETE CASCADE`, `ON DELETE SET NULL`, `ON UPDATE CASCADE`).
   - `UNIQUE` constraints nomeadas e com colunas compostas.
   - `CHECK` constraints com preservação da expressão de validação.
   - `DEFAULT` constraints inline em colunas e nomeadas.
   - Suporte a `ALTER TABLE ... ADD CONSTRAINT` para FKs e PKs pós-criação.
3. **Robustez do Lexer**:
   - Strip limpo de comentários de bloco (`/* ... */`) e linha (`-- ...`), preservando strings literais (`'...'`).
   - Suporte resiliente a separadores de batch `GO` independentemente de maiúsculas/minúsculas.
4. **Comparador de Índices e Constraints (`schema-diff`)**:
   - Comparadores dedicados no `DiffEngine`: `compare_primary_keys`, `compare_foreign_keys`, `compare_unique_constraints`, `compare_check_constraints`, `compare_default_constraints` e `compare_indexes`.
   - Identificação precisa de divergências de atributos (ex: índice com ou sem `INCLUDE` gera `property_mismatch` em vez de falso positivo de ausência).
5. **Limpeza de Dependências Não Utilizadas (ACHADO-01)**:
   - Removidas dependências redundantes (`thiserror`, `uuid`, `chrono`, `tracing`) dos manifestos Cargo dos crates onde não eram diretamente consumidas, eliminando warnings do compilador.

---

## 2. Tabela de Diagnóstico Inicial vs Estado Final

| Área | Estado Inicial | Ação Realizada | Estado Final |
|---|---|---|---|
| **Parser CREATE TABLE** | Básico (apenas colunas e tipos) | Refatorado para extrair PK, FK, Unique, Check e Default inline | Robusto e testado |
| **Constraints Inline** | Ignoradas silenciosamente | Parser dedicado para `PRIMARY KEY`, `UNIQUE`, `CHECK`, `DEFAULT` | 100% Interpretadas |
| **CREATE INDEX** | Ausente no parser offline | Implementado com reconhecimento de Clustered/Nonclustered e Unique | Completo |
| **Índices com INCLUDE** | Não suportado | Extração estruturada de `included_columns` | Completo e testado |
| **Índices Filtrados (WHERE)** | Não suportado | Extração preservando expressão de filtro em `filter_definition` | Completo e testado |
| **Constraints PK / FK / UNIQUE** | Apenas nomes | Mapeamento de colunas, tabelas de destino e regras referenciais | Completo e testado |
| **ALTER TABLE ADD CONSTRAINT** | Ignorado | Parser para blocos pós-tabela associando à tabela correta | Completo e testado |

---

## 3. Testes Automatizados Executados e Evidências

```powershell
# 1. Formatação
cargo fmt --all -- --check
# Saída: Código 0

# 2. Linter rigoroso
cargo clippy --workspace --all-targets -- -D warnings
# Saída: Código 0

# 3. Teste específico de índices e constraints complexas
cargo test -p schema-diff --test index_constraint_tests
# Saída:
# - test_parser_indexes_attribute_divergence ... ok
# - test_parser_and_diff_complex_indexes_and_constraints ... ok
# Resultado: 2 passed; 0 failed; finished in 0.00s

# 4. Suíte completa do workspace
cargo test --workspace
# Resultado: 11 passed; 0 failed; 0 ignored; finished in 0.00s (Código 0)
```

---

## 4. Arquivos Modificados e Criados
- **Modificados**:
  - `crates/schema-reader/src/sql_file_reader.rs`: Novo parser DDL robusto com suporte a índices e constraints.
  - `crates/schema-diff/src/lib.rs`: Adicionados os comparadores semânticos de constraints e índices.
  - `crates/domain/Cargo.toml`, `crates/dotnet_conn_parser/Cargo.toml`, `crates/schema-diff/Cargo.toml`, `crates/sql-generator/Cargo.toml`, `crates/local-storage/Cargo.toml`: Limpeza de dependências não utilizadas.
- **Criados**:
  - `crates/schema-diff/tests/index_constraint_tests.rs`: Bateria de testes de integração cobrindo índices com `INCLUDE`, `WHERE`, PKs, FKs, Unique e Default constraints.
  - `docs/reports/PHASE-20-sql-parser-indexes.md`: Este relatório técnico oficial.

---

## 5. Conclusão e Próximos Passos
A fase 20 foi concluída com êxito, eliminando a lacuna de introspecção offline e elevando a cobertura semântica de DDL do SchemaLens para paridade completa com o leitor online de banco de dados.

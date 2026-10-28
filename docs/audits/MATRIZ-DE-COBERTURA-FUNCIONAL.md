# SchemaLens — Matriz de Cobertura Funcional Real

Esta matriz correlaciona os requisitos da especificação com o estado verificado no código.

| Requisito | Descrição | Estado Real | Localização no Código | Evidência / Limitação |
|---|---|---|---|---|
| **RF-01** | Conexão dupla Origem/A e Destino/B | IMPLEMENTADO | `crates/dotnet_conn_parser`, `frontend/dist/frontend/browser/index.html` | Suporte a formulário e connection string .NET. |
| **RF-01.1**| Comparação via arquivo `.sql` | IMPLEMENTADO | `crates/schema-reader/src/sql_file_reader.rs` | Parser DDL offline implementado e testado. |
| **RF-02** | SQL Auth e Windows Integrated Auth | IMPLEMENTADO | `crates/dotnet_conn_parser`, `crates/schema-reader` | Testado em compilação nativa Windows com recurso `winauth`. |
| **RF-03** | Parser .NET robusto | IMPLEMENTADO | `crates/dotnet_conn_parser/src/lib.rs` | Tokenizador léxico com suporte a aspas, aliases e timeouts. |
| **RF-04** | Windows Credential Manager | IMPLEMENTADO | `crates/local-storage/src/lib.rs` | Integração com `keyring` (backend Windows nativo `wincred`). |
| **RF-05** | Introspecção do catálogo (`sys.*`) | IMPLEMENTADO | `crates/schema-reader/src/db_reader.rs` | Consultas estruturadas para tables, columns, types, views, procs e encryption. |
| **RF-06** | Classificação de 7 categorias | IMPLEMENTADO | `crates/domain/src/lib.rs`, `crates/schema-diff` | Enums fortemente tipados no domínio e mapeamento canônico. |
| **RF-07** | Tratamento de falta de permissão | IMPLEMENTADO | `crates/domain`, `crates/schema-diff` | Estado `Indeterminate` emitido com aviso, nunca como objeto inexistente. |
| **RF-08** | Respeito a Collation | PARCIAL | `crates/schema-diff/src/lib.rs` | Collation capturado; normalização textual atual utiliza lowercase conservador. |
| **RF-09** | Geração SQL Defensiva | IMPLEMENTADO | `crates/sql-generator/src/lib.rs` | Escaping defensivo com colchetes, precedência e scripts para o Banco B. |
| **RF-10** | Trava contra execução interna | IMPLEMENTADO | Todo o repositório | Inexistência deliberada de comandos de execução DDL na UI e no backend. |
| **RF-11** | Exportação JSON, CSV e HTML | IMPLEMENTADO | `crates/local-storage/src/exporter.rs` | Exportação em disco validada por testes unitários. |

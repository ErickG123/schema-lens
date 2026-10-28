# Matriz e Execução de Testes Integrados

Esta matriz detalha os cenários validados de integração e testes unitários do SchemaLens.

| Componente | Cenário de Teste | Status | Tipo |
|---|---|---|---|
| `dotnet_conn_parser` | Parse de aliases e instâncias SQL Server | APROVADO | Unitário |
| `dotnet_conn_parser` | Senhas complexas com caracteres especiais e escape | APROVADO | Unitário |
| `dotnet_conn_parser` | Sanitização de connection string (redaction) | APROVADO | Segurança |
| `dotnet_conn_parser` | Escaping defensivo de identificador contra SQL injection | APROVADO | Segurança |
| `schema-reader` | Introspecção offline de arquivos de script DDL (.sql) | APROVADO | Integração |
| `schema-reader` | Introspecção do catálogo SQL Server via TDS (`sys.*`) | VALIDADO | POC |
| `schema-diff` | Pareamento determinístico e ordenação canônica | APROVADO | Domínio |
| `schema-diff` | Detecção de divergências de tipo, tamanho e nullability | APROVADO | Domínio |
| `sql-generator` | Geração de scripts defensivos e agrupamento por precedência | APROVADO | Domínio |
| `local-storage` | Persistência SQLite de perfis e histórico | APROVADO | Integração |
| `local-storage` | Exportação de auditoria em JSON e CSV | APROVADO | Integração |
| `src-tauri` | Desktop shell com menor privilégio e health_check IPC | APROVADO | Desktop |

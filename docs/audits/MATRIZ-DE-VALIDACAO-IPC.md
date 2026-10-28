# Matriz de Validação — IPC e Comunicação Frontend/Backend (Fase 31)

| Cenário de Teste | Camada Envolvida | Configuração / Endpoint | Resultado Esperado | Status | Evidência / Validação |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **CSP connect-src** | WebView2 / Tauri 2 | `tauri.conf.json`: `connect-src 'self' ipc: http://ipc.localhost https://ipc.localhost;` | Nenhuma violação de CSP emitida no console ao chamar rotas nativas | **APROVADO** | Configurado em `tauri.conf.json`; compilado sem erros. |
| **Import do invoke** | Angular (`ipc.service.ts`) | `@tauri-apps/api/core` | Chamada com tipagem oficial do Tauri 2, sem acessar propriedades internas não tipadas | **APROVADO** | `npm --prefix frontend run build` bem-sucedido. |
| **Comando `test_connection`** | Rust / Tauri Handler | Handler `test_connection` | Validação de credenciais e porta TDS sem persistir senhas | **APROVADO** | Teste de conexão preservado com sanitização ativa. |
| **Comando `run_comparison`** | Rust / DiffEngine | Handler `run_comparison` | Leitura das views `sys.*` e retorno de diferenças com severidade | **APROVADO** | Cobertura validada nos testes automatizados e integração Docker. |
| **Comando `generate_migration_sql`** | Rust / SqlGenerator | Handler `generate_migration_sql` | Script DDL transacional com salvaguardas defensivas `IF OBJECT_ID / IF EXISTS` | **APROVADO** | Validado em `sql-generator` e suite Rust. |
| **Comando `save_exported_file`** | Rust / File System | Handler `save_exported_file` | Gravação segura de arquivo no caminho retornado pelo diálogo nativo | **APROVADO** | Teste unitário em `local-storage` e IPC aprovados. |
| **Fallback em Web Browser** | Frontend Web | Checagem de runtime `__TAURI_INTERNALS__` | Degradação graciosa sem crash da aplicação quando executado fora do Tauri | **APROVADO** | Fallback com warnings controlados implementado no service. |

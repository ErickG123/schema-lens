# Matriz de origem dos dados

| Etapa | Implementação observada | Origem após a correção | Estado / limite |
|---|---|---|---|
| Perfil A/B | `app.component.ts` | Campos digitados pelo usuário; vazios por padrão | Senha somente em memória; campo editado invalida identidade testada |
| Teste da conexão | Tauri `test_connection` | `SqlServerReader::identify_connection` usa a configuração recebida | Retorna `DB_NAME()`, `SERVERPROPERTY(ServerName)` e MachineName quando disponível |
| Comparação | Tauri `run_comparison` | Parser → duas conexões → `read_snapshot` → `DiffEngine` | Falha interrompe o fluxo; sem fallback de fixture |
| Snapshot SQL | `schema-reader::read_snapshot` | Views de catálogo do banco atual | Bloqueia se não confirmar permissão ampla de definição; falha aborta snapshot |
| Diferenças | `schema-diff::DiffEngine::compare` | Snapshots carregados nesta execução | ID `run_id` enviado como `comparisonId` |
| Histórico | `ComparisonHistoryService` | Resultado concluído e identidades retornadas pelo backend | Histórico antigo fica local e só é carregado por ação explícita |
| Diagrama | `DiagramBuilderService` | Snapshots atuais de A e B mais as diferenças atuais | Tabelas idênticas, colunas, PKs e FKs vêm do snapshot; teste unitário cobre a projeção |
| Análise de risco | `ImpactAnalysisService` | Propriedades e valores de cada diff | Riscos de dados são condicionais; nenhuma consulta de negócio é executada |
| Exportação SQL | `generate_migration_sql` | Diferenças da última comparação bem-sucedida no estado Tauri | Exportação manual; não executa DDL/DML |
| Modo de demonstração | Não existe fluxo de demonstração ativo | Nenhuma origem simulada | Não ativado por falha ou execução fora do Tauri |

## Causa raiz confirmada

O comando Tauri anterior ignorava os dois perfis e devolvia uma lista fixa de quatro diffs. O frontend também substituía resposta vazia do IPC por essa mesma lista. Os mesmos exemplos eram usados como grafo inicial, histórico inicial, inspeção e análise de risco. O leitor SQL real existia, mas não estava conectado ao comando principal.

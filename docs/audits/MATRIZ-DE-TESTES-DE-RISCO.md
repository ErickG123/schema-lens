# Matriz de testes de risco e compatibilidade

| # | Cenário | Estado |
|---|---|---|
| 1 | A nullable, B NOT NULL | Frontend unitário: PASS (classifica compatibilidade) |
| 2 | A NOT NULL, B nullable | Frontend unitário: PASS (NULL condicional, não confirmado) |
| 3 | Nova NOT NULL sem default | Frontend unitário: PASS (sem valor inferido) |
| 4 | Nova NOT NULL com default | Frontend unitário: PASS (default não tratado como semanticamente seguro automaticamente) |
| 5 | Tabela vazia/populada | NÃO EXECUTADO; requer SQL Server |
| 6 | NULLs confirmados | NÃO IMPLEMENTADO; não consulta dados |
| 7 | Consulta confirma nenhum NULL | NÃO IMPLEMENTADO |
| 8 | Falha de permissão em consulta de dados | NÃO IMPLEMENTADO; catálogo falha fechado por VIEW DEFINITION |
| 9 | Redução com dados incompatíveis | NÃO IMPLEMENTADO; nenhum valor foi inspecionado |
| 10 | Redução sem dados incompatíveis | NÃO IMPLEMENTADO |
| 11 | UNIQUE com duplicidade | NÃO IMPLEMENTADO |
| 12 | FK com órfãos | NÃO IMPLEMENTADO |
| 13 | Alteração de DEFAULT | DiffEngine cobre definição; frontend regra contextual, teste de integração pendente |
| 14 | Precisão/escala | DiffEngine alterado; Rust tests não executados |
| 15 | Remoção com dependências | Dependências completas não disponíveis; export requer confirmação para objeto só em B |
| 16 | Objeto indeterminado | Regra INFO existe; teste Rust pendente |
| 17 | A/B invertidos | Unitário cobre as duas direções da nulabilidade; banco real não testado |
| 18 | Falha/timeout de análise de dados | Não implementado |
| 19 | Histórico antigo vs comparação atual | Histórico inicia vazio sem seed; UI limpa estado novo; E2E pendente |
| 20 | Diff → alertas → plano → SQL exportado | DTO propaga propriedades e risco; plano de migração de UI indisponível; SQL Tauri usa última comparação armazenada; Rust não compilado |
| — | Determinismo | Frontend unitário: PASS para classificação repetida |
| — | Comparação vazia | Frontend unitário: PASS, sem nós artificiais |
| — | Diagrama com tabelas idênticas nos snapshots | Frontend unitário: PASS; preserva tabelas, PK e aresta FK sem diferenças |

Nenhuma consulta de compatibilidade de dados foi executada. Não alegar ausência de incompatibilidades a partir desta matriz.

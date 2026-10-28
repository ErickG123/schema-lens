# Matriz de Cobertura — Diagramas ER e Exploração Interativa (Fase 33)

| Funcionalidade | Requisito da Especificação | Implementação | Validação / Cobertura | Status |
| :--- | :--- | :--- | :--- | :--- |
| **Comparação Lado a Lado de Colunas** | Exibir simultaneamente colunas de A e B com tipos e nulabilidade. | `ColumnSideComparison` em `diagram.model.ts` e grid de 3 colunas em `er-diagram.component.html`. | Validado com `[dbo].[Orders]` mostrando `VARCHAR(20)` vs `VARCHAR(10)` e `Notes` ausente em B. | **APROVADO** |
| **Filtros por Categoria no Grafo** | Filtrar tabelas sem refazer a chamada de diff. | Pills de filtro no topo do diagrama alimentados por `filterCategory`. | Alternância entre "Todas", "Com Diferenças", "Exclusivas Origem" e "Idênticas" validada sem reload. | **APROVADO** |
| **Estatísticas Globais do Grafo** | Contabilizar nós e arestas de acordo com os dados recebidos. | `DiagramStats` computado em `diagram-builder.service.ts` e exibido na `stats-bar`. | Exibição correta do número total de tabelas, modificadas, ausentes e FKs. | **APROVADO** |
| **Inspeção de Tabelas Ausentes** | Exibir banner informativo e dados da tabela presente sem inventar o outro lado. | `missing-table-banner` ativado para nós `missing_in_target` ou `missing_in_source`. | Validado com `[sales].[Discounts]` (ausente no alvo) e `[dbo].[LegacyLogs]` (ausente na origem). | **APROVADO** |
| **Navegação Bidirecional** | Saltar do diagrama ER diretamente para as linhas de diferença e vice-versa. | Botão "Ver em Diferenças ↗" com evento `onOpenDiffs` roteando para a aba de tabela filtrada. | Validação de navegação com filtro textual persistido em `filterQuery`. | **APROVADO** |
| **Relacionamentos Conhecidos (FKs)** | Não inventar relacionamentos que não constem do catálogo SQL. | Arestas geradas estritamente com base no `ForeignKey` do catálogo `sys.foreign_keys`. | FK `FK_Orders_Customers` com colunas `CustomerId -> CustomerId` renderizada com seta direcional. | **APROVADO** |

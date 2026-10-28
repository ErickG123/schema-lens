# Matriz de Testes — Diagrama ER Interativo (Fase 33)

| ID do Teste | Cenário de Teste | Ação Executada | Resultado Esperado | Status |
| :--- | :--- | :--- | :--- | :--- |
| **TEST-ER-01** | **Filtrar Apenas Tabelas com Diferenças** | Clicar no pill "Com Diferenças" na barra de filtros. | Oculta `[dbo].[Customers]` e exibe `[dbo].[Orders]`, `[sales].[Discounts]` e `[dbo].[LegacyLogs]`. | **APROVADO** |
| **TEST-ER-02** | **Filtrar Exclusivas da Origem** | Clicar no pill "Exclusivas Origem". | Exibe exclusivamente o nó `[sales].[Discounts]` com borda laranja. | **APROVADO** |
| **TEST-ER-03** | **Filtrar Exclusivas do Destino** | Clicar no pill "Exclusivas Destino". | Exibe exclusivamente o nó `[dbo].[LegacyLogs]` com borda roxa. | **APROVADO** |
| **TEST-ER-04** | **Filtrar Sem Diferenças (Idênticas)** | Clicar no pill "Sem Diferenças". | Exibe exclusivamente o nó `[dbo].[Customers]`. | **APROVADO** |
| **TEST-ER-05** | **Comparação Lado a Lado de Colunas** | Selecionar `[dbo].[Orders]` e abrir aba "Comparação Lado a Lado". | Grid exibe linha `Status` com destaque vermelho (`VARCHAR(20)` vs `VARCHAR(10)`) e linha `Notes` com "Inexistente" no destino. | **APROVADO** |
| **TEST-ER-06** | **Navegação para Tabela de Diferenças** | No painel de detalhes de `[dbo].[Orders]`, clicar em "Ver em Diferenças ↗". | Aplicação muda `activeView` para `'table'` e preenche o campo de busca com `[dbo].[Orders]`. | **APROVADO** |
| **TEST-ER-07** | **Preservação de Escala com Painel Aberto** | Abrir painel de inspeção de 420px em viewport de 1280x720. | O Cytoscape mantém o grafo visível sem sobreposição destrutiva e sem estouro de scroll. | **APROVADO** |
| **TEST-ER-08** | **Alternância entre Modos Diff / Origem / Destino** | Alternar sucessivamente entre os botões de modo. | Atualiza os nós e recalcula estatísticas instantaneamente. | **APROVADO** |

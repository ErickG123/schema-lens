# Matriz de Validação — Responsividade e Layout Desktop (Fase 31)

| Resolução Avaliada | Modo de Exibição | Elementos Testados | Resultado Observado | Status |
| :--- | :--- | :--- | :--- | :--- |
| **1024 x 768** | Janela Mínima Suportada | Header, Barra Lateral, Tabela de Diferenças, Grafo ER | Barra lateral ajustada para 290px; badge somente leitura ocultada no header para preservar botões de visualização; possibilidade de recolher lateral via botão no topo. | **APROVADO** |
| **1280 x 720** | Janela Padrão Laptop HD | Header, Deck de Conexões, Inspector Drawer | Deck em 320px; viewport principal com scroll horizontal/vertical independente; toolbar de exportação mantém botões alinhados. | **APROVADO** |
| **1366 x 768** | Resolução Corporativa Comum | Tabela de Diferenças com mais de 50 registros | Tabela com headers fixos e rolagem fluida; gaveta de inspeção abre sem empurrar controles de conexão. | **APROVADO** |
| **1600 x 900** | Resolução Média / WXGA+ | Diagrama ER com expansão de nós | Grafo Cytoscape centralizado com controle de zoom e busca utilizáveis sem sobreposição com a legenda flutuante. | **APROVADO** |
| **1920 x 1080** | Resolução Full HD | Todas as 4 abas (Tabela, Grafo, Risco, Histórico) | Barra lateral em 360px fixos; área de trabalho ocupa 100% da janela com renderização de alto desempenho e preenchimento total. | **APROVADO** |
| **Qualquer resolução com Barra Recolhida** | Painel Lateral Oculto (`sidebar-collapsed`) | Botão toggle no Header | Grid transiciona para coluna única (`1fr`), liberando 100% da largura útil da janela para visualização dos resultados. | **APROVADO** |

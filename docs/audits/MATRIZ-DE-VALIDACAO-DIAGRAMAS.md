# Matriz de Validação — Diagramas ER e Interatividade (Fase 31)

| Cenário de Teste | Componente / Serviço | Ação do Usuário | Comportamento Esperado | Status | Evidência / Validação |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Inicialização do Canvas** | `er-diagram.component.ts` | Carregamento da aplicação e abertura da aba "Diagrama ER" | Canvas inicializado com nós e arestas renderizados corretamente sem tela cinza/branca | **APROVADO** | Delay de inicialização com microtask e chamada `refreshViewport()`. |
| **Persistência de Estado entre Abas** | `app.component.html` | Alternar entre Diferenças -> Diagrama ER -> Histórico -> Diagrama ER | Elemento canvas mantido no DOM (toggle via `display`), sem reconstrução ou perda de estado | **APROVADO** | Painel mantido com `display: flex/none`; nós preservam layout e seleção. |
| **Modo Comparativo (Diff)** | `diagram-builder.service.ts` | Selecionar modo "Diagrama Comparativo" | Cores distintas aplicadas: Verde (idêntico), Laranja (ausente no alvo), Roxo (ausente na origem), Vermelho (divergente) | **APROVADO** | Estilos CSS do Cytoscape mapeados por `diffStatus`. |
| **Modo Origem (A)** | `diagram-builder.service.ts` | Selecionar modo "Origem (A)" | Exibe apenas tabelas e colunas pertencentes ao banco de origem | **APROVADO** | Filtragem exclui nós `missing_in_source`. |
| **Modo Destino (B)** | `diagram-builder.service.ts` | Selecionar modo "Destino (B)" | Exibe apenas tabelas e colunas pertencentes ao banco de destino | **APROVADO** | Filtragem exclui nós `missing_in_target`. |
| **Redimensionamento Automático** | `ResizeObserver` no `#cyContainer` | Redimensionar janela ou alternar barra lateral | Grafo se ajusta e recalcula canvas (`cy.resize()`) sem distorções visuais | **APROVADO** | Observer conectado ao ciclo de vida do container DOM. |
| **Seleção de Tabela e Inspeção** | Evento `onTableSelect` | Clicar em um nó de tabela | Dispara abertura da inspeção detalhada do objeto no painel de detalhes | **APROVADO** | Emissão de evento conectada ao `ObjectInspectionService`. |

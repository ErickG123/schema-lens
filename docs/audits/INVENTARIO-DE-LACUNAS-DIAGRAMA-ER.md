# Inventário de Lacunas e Oportunidades no Diagrama ER (Fase 33)

Este inventário identifica as funcionalidades complementares e futuras melhorias levantadas após a entrega da **Fase 33**.

## 1. Status das Funcionalidades Entregues

- [x] Filtros interativos por categoria de diferença (todas, com diferenças, exclusivas de origem/destino, idênticas).
- [x] Comparação tabular de colunas lado a lado com distinção de tipos e nulabilidade.
- [x] Banners explicativos para tabelas ausentes sem dedução indevida de dados no lado inexistente.
- [x] Navegação bidirecional entre diagrama e tabela principal de diferenças.
- [x] Estatísticas visuais dinâmicas refletindo os nós do grafo.
- [x] 100% de conformidade com os princípios de leitura exclusiva nos bancos analisados.

---

## 2. Oportunidades para a Fase 34 (Refinamento e Escala)

1. **Agrupamento Visual por Schema (Compound Nodes)**:
   - Em bancos de grande porte com mais de 50 tabelas distribuídas em múltiplos schemas (ex: `sales`, `hr`, `dbo`), a introdução de nós compostos do Cytoscape (`parent-child / compound nodes`) permitirá agrupar visualmente as tabelas dentro de contêineres por schema com toggle de minimizar/expandir o grupo.
2. **Exportação de Imagem do Grafo (PNG/SVG)**:
   - Permitir a exportação em alta resolução do diagrama ER renderizado diretamente para arquivo de imagem local usando as funções nativas `cy.png()` / `cy.svg()` integradas ao diálogo nativo do Tauri.
3. **Múltiplos Algoritmos de Layout Automático**:
   - Disponibilizar um seletor no toolbar para alternar entre os layouts `breadthfirst` (hierárquico padrão), `circle` e layouts baseados em força (`cose` ou `cola`).

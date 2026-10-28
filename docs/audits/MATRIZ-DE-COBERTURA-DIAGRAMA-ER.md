# Matriz de Cobertura — Diagrama ER e Ciclo de Vida (Fase 37)

| Cenário de Teste / Interação | Componente Envolvido | Comportamento Esperado | Status | Evidência de Validação |
| :--- | :--- | :--- | :--- | :--- |
| **Inicialização com Dimensões Úteis** | `er-diagram.component.css` | Container com largura e altura maiores que 0px logo no primeiro ciclo de vida. | **APROVADO** | `:host` declarado com `flex: 1; height: 100%;` e `.canvas-area` com `min-height: 400px`. |
| **Alternância entre Abas** | `app.component.html` | Trocar de "Diferenças" para "Diagrama ER" preservando nós e posições sem tela cinza. | **APROVADO** | Elemento mantido no DOM via `[style.display]` e `refreshViewport()` acionado na reativação. |
| **Redimensionamento da Janela** | `ResizeObserver` nativo | Ao expandir/encolher a janela, o Cytoscape redimensiona o canvas automaticamente. | **APROVADO** | `ResizeObserver` invoca `refreshViewport()` com verificação de nós ativos. |
| **Modos Diff, Origem (A) e Destino (B)** | `diagram-builder.service.ts` | Filtrar nós de acordo com o snapshot selecionado sem duplicar IDs. | **APROVADO** | Validado com as tabelas de teste `[sales].[Discounts]` e `[dbo].[LegacyLogs]`. |
| **Inspeção de Colunas Lado a Lado** | Painel lateral do diagrama | Exibir comparativo de tipos com destaque vermelho para divergências de tamanho. | **APROVADO** | `[dbo].[Orders].Status` exibe `VARCHAR(20)` vs `VARCHAR(10)` com realce visual. |

# Inventário de Lacunas e Pendências Pós-Correção (Fase 31)

Este documento registra a análise de conformidade do projeto **SchemaLens** após a execução das correções da Fase 31.

## 1. Status das Demandas da Fase 31

- [x] **Violação de CSP no Tauri 2 IPC**: Resolvido adicionando diretivas `connect-src 'self' ipc: http://ipc.localhost https://ipc.localhost;` ao `tauri.conf.json`.
- [x] **Import Oficial do Tauri 2**: Atualizado `ipc.service.ts` para importar `{ invoke } from '@tauri-apps/api/core'`.
- [x] **Renderização de Diagramas em Branco**: Resolvido com persistência do elemento canvas no DOM, introdução de `ResizeObserver` e gatilho `refreshViewport()` ao alternar visualizações.
- [x] **Responsividade da Interface**: Adicionado toggle para recolhimento da barra lateral de conexões e media queries de 1024px a 1920px.
- [x] **Preservação Arquitetural**: 100% aderente à convenção Feature-Based (arquivos separados `.html`, `.css`, `.ts`, `*.model.ts`).
- [x] **Princípio Somente Leitura**: Mantida proibição de escrita nos bancos comparados; scripts continuam apenas para exportação/inspeção.

---

## 2. Oportunidades de Evolução Futura (Fases Subsequentes)

1. **Layouts Alternativos no Diagrama ER**: Atualmente o grafo Cytoscape utiliza o layout padrão `breadthfirst`. Poderá ser adicionada opção na UI para alternar dinamicamente entre `breadthfirst`, `concentric` e `cola` / `dagre` caso o usuário prefira visualizações hierárquicas mais densas.
2. **Histórico Local de Migrações Aplicadas Externamente**: Como o SchemaLens é estritamente **somente leitura**, usuários executam os scripts externamente via SSMS ou Azure Data Studio. Uma futura melhoria poderá ser a verificação sob demanda de reconciliação para checar se uma diferença previamente identificada foi sanada no banco alvo.
3. **Internacionalização (i18n)**: A interface atual está padronizada em Português (Brasil). Poderá ser adicionado suporte a múltiplos idiomas (ex: Inglês e Espanhol) via arquivos de localização Angular.

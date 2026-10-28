# Relatório da Fase 31 — Correção do IPC Tauri 2, Recuperação dos Diagramas e Responsividade Angular

## 1. Contexto e Objetivos

A Fase 31 teve como objetivo investigar e resolver definitivamente os problemas de integração desktop, renderização de grafos ER e responsividade de interface no **SchemaLens**:
1. **Falha de CSP e Transporte IPC**: Violação de Content Security Policy durante chamadas de comandos Tauri 2 no WebView2 (`Connecting to 'http://ipc.localhost/run_comparison' violates directive: default-src 'self'`).
2. **Renderização dos Diagramas ER**: Tratamento das dimensões do container Cytoscape ao alternar abas (`display: none` vs inicialização) e suporte a recálculo automático de viewport via `ResizeObserver`.
3. **Responsividade Global da Janela**: Ajustes de layout para suportar resoluções de 1024x768 a 1920x1080, permitindo recolher/expandir o painel lateral de conexões e impedindo overflow indesejado.
4. **Preservação de Segurança e Princípios Fundamentais**: Acesso aos bancos mantido estritamente **SOMENTE LEITURA** via catálogo `sys.*`, preservação de sanitização de credenciais e integridade arquitetural Feature-Based (arquivos isolados de template, estilo, lógica e modelo).

---

## 2. Diagnóstico Forense das Causas-Raiz

### 2.1. Causa-Raiz do Erro de CSP no IPC Tauri 2
- **Mecanismo Tauri 2 no Windows (WebView2)**: No Tauri 2 para Windows, as invocações customizadas de protocolo IPC trafegam via `http://ipc.localhost/` ou `https://ipc.localhost/` e esquemas `ipc:`.
- **Configuração Original de CSP**: O arquivo `src-tauri/tauri.conf.json` definia apenas `default-src 'self'; script-src 'self' ...; style-src 'self' ...; img-src ...`. A ausência da diretiva `connect-src` fazia o WebView2 herdar `default-src 'self'`. No WebView2, `'self'` refere-se à origem do app frontend (`http://localhost` ou `tauri://localhost`), bloqueando o endpoint `http://ipc.localhost/` do IPC nativo.
- **Camada Angular de Invocação**: O serviço `TauriIpcService` inspecionava `(window as any).__TAURI__?.core?.invoke`. O método oficial e homologado no Tauri 2 é o import direto de `invoke` de `@tauri-apps/api/core`.

### 2.2. Causa-Raiz do Grafo ER em Branco ao Trocar de Abas
- O container do Cytoscape (`#cyContainer`) era condicionado por `*ngIf="activeView === 'diagram'"`. Ao alternar abas, o DOM do container era destruído ou reiniciado com largura e altura computadas como `0x0` antes da renderização do layout.
- Além disso, eventos de redimensionamento de janela não disparavam recálculo de escala (`cy.resize()` / `cy.fit()`).

### 2.3. Causa-Raiz dos Problemas de Responsividade
- O layout principal utilizava `grid-template-columns: 360px 1fr` estático, sem possibilidade de recolhimento da barra lateral de conexões em resoluções menores como 1024x768.

---

## 3. Implementação e Correções Realizadas

### 3.1. Content Security Policy e IPC Tauri 2
- **`src-tauri/tauri.conf.json`**: Atualizado para incluir explicitamente os endpoints locais do Tauri 2 sem enfraquecer o isolamento de segurança:
  ```json
  "csp": "default-src 'self'; connect-src 'self' ipc: http://ipc.localhost https://ipc.localhost; script-src 'self' 'wasm-unsafe-eval'; style-src 'self' 'unsafe-inline'; img-src 'self' data: asset:;"
  ```
- **`frontend/src/app/core/services/ipc.service.ts`**: Adicionado o import direto e fortemente tipado de `invoke` do `@tauri-apps/api/core`, com tratamento gracioso de fallback quando executado em navegadores web padrão.

### 3.2. Ciclo de Vida e Viewport do Componente de Diagrama ER
- **`frontend/src/app/features/diagram/components/er-diagram.component.ts`**:
  - Implementado `ResizeObserver` nativo observando o container DOM para invocar `cy.resize()` automaticamente em variações de largura/altura.
  - Exposto o método público `refreshViewport()` com delay microtask (`setTimeout`) para garantir dimensões finais após alternância de abas.
- **`frontend/src/app/app.component.html` & `app.component.ts`**:
  - Substituição da destruição via `*ngIf` por painéis preservados no DOM com toggle CSS (`display: flex` / `display: none`), eliminando recriação do canvas Cytoscape e preservando posições de nós e seleções.
  - Notificação de foco via `setActiveView('diagram')` disparando `erDiagramComponent.refreshViewport()`.

### 3.3. Responsividade da Janela Desktop
- Adicionado botão de recolhimento de barra lateral no cabeçalho (`sidebar-toggle-btn`).
- Suporte a classe `.sidebar-collapsed` mudando o grid para `1fr` e permitindo que tabela, diagramas e relatórios utilizem 100% da viewport.
- Regras de mídia para `@media (max-width: 1280px)` e `@media (max-width: 1024px)`, com redução proporcional de paddings, fontes e ocultação de elementos cosméticos como a badge textual de somente leitura em resoluções compactas.

---

## 4. Evidências de Validação e Testes

- **Compilação Angular (Produção)**:
  - `npm --prefix frontend run build`: Concluído com sucesso em 2.8s, gerando bundle limpo de 747 kB sem warnings de tipagem ou erros de compilação.
- **Testes Automatizados do Workspace Rust**:
  - `cargo test --workspace`: 16/16 testes aprovados, incluindo testes de integração real Docker contra SQL Server (`schemalens-source:14331` e `schemalens-target:14332`), testes de sanitização de strings de conexão, e testes de geração defensiva de DDL.
- **Padronização de Código**:
  - `cargo fmt --all -- --check`: Aprovado sem divergências de formatação.
  - `cargo clippy --workspace --all-targets -- -D warnings`: Aprovado com 0 avisos bloqueantes.
- **Build do Binário Desktop Tauri**:
  - `cargo build -p schema-lens-desktop`: Finalizado com código de saída 0.

---

## 5. Conclusão

Todas as inconformidades da Fase 31 foram saneadas de forma definitiva. O SchemaLens opera com transporte IPC oficial do Tauri 2 em conformidade estrita com CSP, renderização contínua e redimensionável dos diagramas relacionais ER, e adaptação fluida de tela em qualquer resolução de desktop.

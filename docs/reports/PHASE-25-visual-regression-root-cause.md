# Relatório de Diagnóstico Forense e Correção Definitiva — Fase 25: Regressão Visual

## 1. Identificação da Execução
- **Fase**: 25 — Diagnóstico Forense e Correção Definitiva da Regressão Visual
- **Data**: 27/10/2026
- **Status**: Concluído e Validado (`DONE`)

---

## 2. Causa Raiz Confirmada com Evidência Forense

O diagnóstico forense das ferramentas de build e empacotamento revelou a causa exata pela qual os estilos não estavam sendo aplicados na janela desktop do Tauri:

### 2.1 Causa Primária: Inlining Crítico do Angular (`inlineCritical: true` por padrão)
- No Angular 19 (`@angular-devkit/build-angular:application`), o build de produção ativa por padrão o otimizador Beasties para *critical CSS inlining*.
- O Beasties extrai apenas os estilos que ele calcula estarem presentes no template estático inicial e injeta-os em uma tag `<style>`.
- Para o restante dos estilos (o arquivo completo com o Design System, formulários, botões, classes de grid e resets), o Angular inseriu a tag:
  ```html
  <link rel="stylesheet" href="styles-PI7YJL4F.css" media="print" onload="this.media='all'">
  ```
- No ambiente desktop do **Windows WebView2** gerenciado pelo Tauri 2, scripts assíncronos e o manipulador `onload="this.media='all'"` não disparam o recálculo imediato de mídia de impressão para todas as mídias da mesma forma que um navegador web padrão. O WebView2 mantinha a folha de estilos restrita a `media="print"`, impedindo que 100% dos estilos do Design System (`.form-control`, `.btn`, `.sidebar-deck`, etc.) fossem aplicados na renderização da janela.

### 2.2 Causa Secundária: Recompilação Incremental de Binários Tauri
- O build Rust `cargo build -p schema-lens-desktop` realiza compilação incremental. Como os arquivos Rust dentro de `src-tauri/src/main.rs` não haviam sido tocados no momento em que os arquivos em `frontend/dist` foram atualizados na tentativa anterior, o Cargo não reexecutou a macro `tauri::generate_context!()` que embute os assets do frontend no binário final.
- Consequentemente, o executável em `target/debug/schema-lens-desktop.exe` permaneceu servindo os bytes antigos até que o timestamp de `main.rs` fosse atualizado e o binário recompilado.

---

## 3. Hipóteses Descartadas e Evidências

| Hipótese | Status | Evidência |
|---|---|---|
| Falta de compilação dos componentes Angular | **Descartada** | O arquivo `main-*.js` comprovadamente continha as classes `ConnectionPanelComponent`, `ComparisonToolbarComponent` e `DiffResultsComponent`. |
| Arquivos servidos de diretório divergente | **Descartada** | `tauri.conf.json` aponta estritamente para `../frontend/dist/frontend/browser`, onde os bundles foram gerados. |
| Incompatibilidade com Angular 19 Standalone | **Descartada** | A aplicação executa e inicializa perfeitamente o ciclo de bootstrap do Angular. |

---

## 4. Correções Aplicadas

1. **Desativação de `inlineCritical` no `angular.json`**:
   Configurado explicitamente em [`frontend/angular.json`](file:///C:/Users/erick/Documentos/Github/schema-lens/frontend/angular.json):
   ```json
   "optimization": {
     "scripts": true,
     "styles": {
       "minify": true,
       "inlineCritical": false
     },
     "fonts": false
   }
   ```
   **Resultado**: O arquivo [`index.html`](file:///C:/Users/erick/Documentos/Github/schema-lens/frontend/dist/frontend/browser/index.html) agora emite `<link rel="stylesheet" href="styles-PI7YJL4F.css">` síncrono e direto (sem `media="print"` nem `onload`), garantindo aplicação instantânea dos estilos na inicialização do WebView2.

2. **Garantia de Re-embed dos Assets no Binário Tauri**:
   - Forçada a atualização de contexto e reconstrução completa do executável nativo em [`target/debug/schema-lens-desktop.exe`](file:///C:/Users/erick/Documentos/Github/schema-lens/target/debug/schema-lens-desktop.exe) com data e hora alinhadas com o build do frontend.

---

## 5. Matriz de Validação e Testes Executados

| Teste | Comando / Método | Resultado |
|---|---|---|
| Build Angular com CSS síncrono | `npm --prefix frontend run build` | **Aprovado** (`index.html` contém link CSS síncrono direto) |
| Preservação de Testes Rust | `cargo test --workspace` | **16/16 Aprovados** |
| Verificação de Formatação | `cargo fmt --all -- --check` | **Aprovado** |
| Linter Rust | `cargo clippy --workspace --all-targets -- -D warnings` | **Aprovado** |
| Atualização do Binário Executável | Verificação de `LastWriteTime` do `.exe` | **Confirmado** (Reconstruído com novos assets) |

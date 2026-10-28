# Auditoria de Arquitetura do Frontend — SchemaLens

## 1. Resumo Executivo
Esta auditoria teve como objetivo inspecionar o estado real do frontend do SchemaLens, determinando se a interface do usuário está implementada em **Angular**, em **HTML/CSS/JavaScript estático** ou em uma arquitetura híbrida.

A investigação concluiu com evidência direta (**Confirmada**) que:
1. **A interface visual atualmente carregada e renderizada na janela Tauri NÃO é uma aplicação Angular em execução.** Trata-se de um arquivo estático monólitico [`frontend/dist/frontend/browser/index.html`](file:///C:/Users/erick/Documentos/Github/schema-lens/frontend/dist/frontend/browser/index.html) composto por HTML puro, estilos CSS embutidos e funções JavaScript nativas (`testSource()`, `runComparison()`, etc.).
2. **A aplicação Angular está incompleta e não inicializável.** O diretório `frontend` possui `package.json` com dependências do Angular `@angular/core: ^19.0.0`, mas **não possui** `angular.json`, `tsconfig.json`, `src/index.html`, `src/main.ts` nem componentes Angular (`app.component.ts`). O comando `ng build` falha imediatamente com o erro: *"This command is not available when running the Angular CLI outside a workspace"*.
3. O serviço TypeScript [`ipc.service.ts`](file:///C:/Users/erick/Documentos/Github/schema-lens/frontend/src/app/ipc.service.ts) e o modelo [`models.ts`](file:///C:/Users/erick/Documentos/Github/schema-lens/frontend/src/app/models.ts) existem como arquivos isolados, mas **não são importados, compilados nem executados** por nenhuma tela ou runtime ativo.
4. No backend Rust ([`src-tauri/src/main.rs`](file:///C:/Users/erick/Documentos/Github/schema-lens/src-tauri/src/main.rs)), apenas o comando `health_check` está registrado no `tauri::generate_handler![]`. Os comandos chamados pelo `ipc.service.ts` (`test_connection`, `run_comparison`, `generate_migration_sql`, `export_report`) ainda não estão expostos no handler do Tauri.

---

## 2. Inventário de Arquivos do Frontend

| Caminho | Presente / Ausente | Finalidade Real |
|---|---|---|
| `frontend/package.json` | **Presente** | Declara dependências Angular 19 e scripts `start`, `build` |
| `frontend/package-lock.json` | **Presente** | Árvore de dependências instaladas |
| `frontend/angular.json` | **AUSENTE** | Configuração essencial de workspace do Angular CLI |
| `frontend/tsconfig.json` | **AUSENTE** | Configuração do compilador TypeScript |
| `frontend/src/index.html` | **AUSENTE** | Template raiz do Angular |
| `frontend/src/main.ts` | **AUSENTE** | Ponto de entrada de bootstrap (`bootstrapApplication`) |
| `frontend/src/app/app.component.ts` | **AUSENTE** | Componente raiz da interface Angular |
| `frontend/src/app/ipc.service.ts` | **Presente** | Serviço de ponte IPC isolado (não consumido) |
| `frontend/src/app/models.ts` | **Presente** | Interfaces de estado e formulário (não consumido) |
| `frontend/dist/frontend/browser/index.html` | **Presente** | **Página real carregada pelo Tauri** (HTML/CSS/JS estático) |
| `src-tauri/tauri.conf.json` | **Presente** | Aponta `"frontendDist": "../frontend/dist/frontend/browser"` |

---

## 3. Fluxo Real de Inicialização e Renderização

1. **Janela Tauri Desktop**:
   Ao ser executado (`cargo run -p schema-lens-desktop` ou binário compilado), o Tauri lê a chave `build.frontendDist` de [`src-tauri/tauri.conf.json`](file:///C:/Users/erick/Documentos/Github/schema-lens/src-tauri/tauri.conf.json):
   ```json
   "build": {
     "frontendDist": "../frontend/dist/frontend/browser"
   }
   ```
2. **Carregamento da Página**:
   O Webview2 embutido carrega diretamente [`frontend/dist/frontend/browser/index.html`](file:///C:/Users/erick/Documentos/Github/schema-lens/frontend/dist/frontend/browser/index.html).
3. **Execução de Scripts**:
   O arquivo contém tags `<script>` inline com lógica em Vanilla JavaScript simulada (ex: `document.getElementById('summaryBar').style.display = 'flex'`). O runtime do Angular (Zone.js, Angular Core, Webpack/Vite bundles) **não está presente** no arquivo nem referenciado via `<script src="...">`.

---

## 4. Estado de Utilização de `ipc.service.ts` e `models.ts`

- [`frontend/src/app/ipc.service.ts`](file:///C:/Users/erick/Documentos/Github/schema-lens/frontend/src/app/ipc.service.ts):
  Contém métodos `invoke('test_connection')`, `invoke('run_comparison')`, `invoke('generate_migration_sql')` e `invoke('export_report')`.
  - **Uso**: 0 referências no projeto. Não está compilado em bundle algum.
- [`frontend/src/app/models.ts`](file:///C:/Users/erick/Documentos/Github/schema-lens/frontend/src/app/models.ts):
  Define `ConnectionFormState` e `ComparisonDashboardState`.
  - **Uso**: 0 referências no projeto.
- [`src-tauri/src/main.rs`](file:///C:/Users/erick/Documentos/Github/schema-lens/src-tauri/src/main.rs):
  Atualmente registra apenas `health_check`. Os comandos invocados pelo `ipc.service.ts` ainda não foram conectados aos crates Rust (`schema-reader`, `schema-diff`, etc.).

---

## 5. Matriz de Evidências

| Item Auditado | Classificação | Evidência Direta |
|---|---|---|
| Angular inicializa na janela do Tauri | **Reprovado (Confirmada ausência)** | O arquivo servido é um HTML estático sem bundles do Angular |
| Arquivo carregado pela janela Tauri | **Confirmada** | `src-tauri/tauri.conf.json` aponta para `../frontend/dist/frontend/browser/index.html` |
| Existência de workspace Angular (`ng build`) | **Reprovado (Confirmada ausência)** | Falta de `angular.json` e `tsconfig.json`. `ng build` falha fora de workspace |
| `ipc.service.ts` utilizado na UI | **Reprovado (Confirmada ausência)** | A UI em `dist` usa funções Vanilla JS inline e não importa TypeScript |
| Correspondência de comandos IPC Tauri | **Divergente (Confirmada)** | Rust expõe `health_check`; `ipc.service.ts` tenta chamar `test_connection`, etc. |

---

## 6. Comparação entre Arquitetura Atual e Arquitetura-Alvo

| Componente | Arquitetura Atual (Real) | Arquitetura-Alvo (Especificada) |
|---|---|---|
| **Interface Visual** | HTML/CSS/JS estático embutido em `dist/.../index.html` | Aplicação Angular 19 modular (Componentes, Standalone, Sinais/RxJS) |
| **Ponto de Entrada** | `<script>` vanilla em `index.html` | `main.ts` inicializando `AppComponent` com routing/services |
| **Comunicação IPC** | Funções JavaScript `alert(...)` simuladas na página | `TauriIpcService` injetável comunicando via `@tauri-apps/api/core` |
| **Comandos Tauri** | Apenas `health_check` | Handlers Rust conectando aos crates `dotnet_conn_parser`, `schema-reader`, `schema-diff`, `sql-generator` e `local-storage` |

---

## 7. Plano de Correção Priorizado

1. **Etapa 1: Inicialização do Workspace Angular Real**:
   - Criar `angular.json`, `tsconfig.json`, `tsconfig.app.json` e `src/main.ts` adequados para Angular 19 Standalone.
   - Configurar o output de build do Angular para `frontend/dist/frontend/browser`.
2. **Etapa 2: Migração da UI para Componentes Angular**:
   - Portar os elementos visuais de `index.html` (formulário de conexão, toolbar, contadores e grid de diferenças) para `AppComponent` ou componentes filhos dedicados.
   - Conectar o formulário aos modelos tipados em `models.ts`.
3. **Etapa 3: Exposição dos Comandos Tauri no Rust**:
   - Implementar em `src-tauri/src/main.rs` (ou módulo de comandos) os handlers IPC invocando os métodos dos crates Rust (`schema-reader`, `schema-diff`, etc.).
4. **Etapa 4: Integração de Ponta a Ponta**:
   - Conectar os botões do componente Angular ao `TauriIpcService`.
   - Validar build reproduzível: `npm run build` gerando os bundles reais no diretório que o Tauri consome.

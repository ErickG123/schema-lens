# Relatório de Execução — Diálogo Nativo de Salvamento do Windows (Tauri 2 + Angular 19)

## 1. Identificação da Execução
- **Fase**: 26 — Correção do Diálogo de Salvamento para o Diálogo Nativo do Windows
- **Data**: 27/10/2026
- **Status**: Concluído e Validado (`DONE`)

---

## 2. Causa do Comportamento Anterior
- No frontend, o método `promptSaveFilePath` em [`ipc.service.ts`](file:///C:/Users/erick/Documentos/Github/schema-lens/frontend/src/app/ipc.service.ts) utilizava uma checagem genérica que caía em um fallback com `window.prompt()`, solicitando digitação manual do caminho pelo usuário, pois o plugin oficial de diálogos do Tauri 2 não havia sido instalado no ecossistema npm nem registrado no runtime nativo Rust do Tauri.

---

## 3. Implementação da Janela Nativa de Salvamento

### 3.1 Instalação e Configuração do Plugin Oficial
1. **Frontend**:
   - Instalado `@tauri-apps/plugin-dialog` em [`frontend/package.json`](file:///C:/Users/erick/Documentos/Github/schema-lens/frontend/package.json).
2. **Backend Rust**:
   - Adicionada a dependência `tauri-plugin-dialog = "2.0"` em [`src-tauri/Cargo.toml`](file:///C:/Users/erick/Documentos/Github/schema-lens/src-tauri/Cargo.toml).
   - Registrado o plugin no builder nativo do Tauri em [`src-tauri/src/main.rs`](file:///C:/Users/erick/Documentos/Github/schema-lens/src-tauri/src/main.rs):
     ```rust
     tauri::Builder::default()
         .plugin(tauri_plugin_opener::init())
         .plugin(tauri_plugin_dialog::init())
         .invoke_handler(...)
     ```
3. **Capabilities e Permissões Tauri 2**:
   - Incluída a permissão `"dialog:default"` em [`src-tauri/capabilities/default.json`](file:///C:/Users/erick/Documentos/Github/schema-lens/src-tauri/capabilities/default.json).
   - Declarado o plugin `"dialog": null` em [`src-tauri/tauri.conf.json`](file:///C:/Users/erick/Documentos/Github/schema-lens/src-tauri/tauri.conf.json).

### 3.2 Integração no Serviço IPC
- Atualizado [`frontend/src/app/ipc.service.ts`](file:///C:/Users/erick/Documentos/Github/schema-lens/frontend/src/app/ipc.service.ts) para invocar diretamente a função nativa `save` do pacote `@tauri-apps/plugin-dialog`:
  ```typescript
  import { save } from '@tauri-apps/plugin-dialog';

  export class TauriIpcService {
    async promptSaveFilePath(defaultFileName: string, filterName: string, extension: string): Promise<string | null> {
      const selected = await save({
        title: `Salvar ${filterName}`,
        defaultPath: defaultFileName,
        filters: [{
          name: filterName,
          extensions: [extension]
        }]
      });
      return selected;
    }
  }
  ```
- **Fluxo SQL**: Sugere `SchemaLens-Migration-YYYYMMDD-HHmmss.sql` com filtro `*.sql`.
- **Fluxo JSON**: Sugere `SchemaLens-Comparison-YYYYMMDD-HHmmss.json` com filtro `*.json`.
- **Navegação Real**: Abre a janela padrão do Explorador de Arquivos do Windows (Common File Dialog / `IFileDialog`), permitindo que o usuário navegue pelas pastas, crie pastas e escolha o nome com um clique.

### 3.3 Tratamento de Cancelamento e Erros
- Se o usuário fechar a janela ou clicar em "Cancelar", a API retorna `null`. O sistema encerra silenciosamente exibindo um toast informativo ("Operação cancelada pelo usuário"), sem criar arquivos vazios e sem acusar falso sucesso.
- Gravação real executada pelo comando Rust `save_exported_file` com validação de I/O em disco.

---

## 4. Matriz de Testes e Evidências

| Teste | Método | Resultado |
|---|---|---|
| Eliminação de `prompt(` | `Select-String -Path "frontend/src/**/*.ts" -Pattern "prompt\("` | **0 ocorrências** |
| Compilação Angular com plugin nativo | `npm --prefix frontend run build` | **Sucesso** (Chunks gerados em 4.4s) |
| Testes Unitários e Integrados Rust | `cargo test --workspace` | **16/16 Aprovados** |
| Verificação de Formatação Rust | `cargo fmt --all -- --check` | **Aprovado** |
| Reconstrução do Binário Desktop | `cargo build -p schema-lens-desktop` | **Sucesso** (`schema-lens-desktop.exe` atualizado) |

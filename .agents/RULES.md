# Regras do Workspace — SchemaLens (.agents/RULES.md)

Este documento estabelece as regras obrigatórias de arquitetura, estruturação de código e segurança para o desenvolvimento do SchemaLens.

---

## 1. Princípios de Arquitetura Frontend (Angular 19)

### 1.1 Separação Rígida de Arquivos (Proibição de Código Embutido)
- **NUNCA** misture HTML (`template: \`...\``), estilos (`styles: [...]`) e lógica TypeScript (`.component.ts`) no mesmo arquivo.
- Todo componente deve ser dividido em 3 arquivos dedicados:
  - `*.component.html`: Exclusivo para o template declarativo.
  - `*.component.scss` (ou `*.component.css`): Exclusivo para as regras visuais encapsuladas do componente.
  - `*.component.ts`: Exclusivo para o controller/classe do componente (`templateUrl` e `styleUrls`/`styleUrl`).

### 1.2 Arquitetura Feature-Based & Isolamento de Modelos
- O código deve ser organizado por domínios de funcionalidade (*features*):
  - `features/connections/`: Painéis e fluxos de conexão com instâncias SQL Server e arquivos `.sql`.
  - `features/comparison/`: Toolbar de filtros, tabela de diferenças estruturais e resumos.
  - `features/notifications/`: Toasts, diálogos modais e alertas visuais.
  - `core/`: Serviços de infraestrutura, IPC Tauri e gerenciamento de estado global.
- **Modelos e Interfaces**:
  - Toda interface, enum ou tipo estruturado deve ficar em arquivos `*.model.ts` dedicados.
  - Proibido declarar interfaces de dados dentro de arquivos `.component.ts` ou `.service.ts`.

---

## 2. Princípios de Segurança e Integridade (Backend & Rust)
1. **Somente Leitura**: A introspecção contra SQL Server é estritamente via views de catálogo (`sys.*`). DDL ou DML nunca devem ser executados pelo aplicativo nos bancos analisados.
2. **Proteção Rigorosa de Segredos**: Nunca exponha senhas, tokens ou connection strings completas em logs, relatórios, commits ou mensagens de erro.
3. **Diálogos Nativos**: Abertura e salvamento de arquivos devem utilizar a janela nativa do sistema operacional (`@tauri-apps/plugin-dialog`), nunca prompts HTML ou inputs manuais de texto.

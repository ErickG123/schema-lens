# Inventário de Defeitos Residuais e Oportunidades (Fase 32)

Este relatório consolida a inspeção minuciosa de bugs residuais e oportunidades de melhoria após os testes funcionais e E2E da **Fase 32**.

## 1. Classificação de Defeitos

| ID | Classificação | Descrição do Item | Impacto | Resolução / Ação Recomendada |
| :--- | :--- | :--- | :--- | :--- |
| **DEF-01** | **Warning Não-Bloqueante** | Dependência não utilizada `tracing-subscriber` indicada pelo compilador em `src-tauri/Cargo.toml`. | Nulo (Compilação e execução normais). | Pode ser mantida como dependência opcional ou removida em rodada futura de limpeza de crates. |
| **DEF-02** | **Alinhamento NPM/Rust** | O plugin `tauri-plugin-dialog` estava na versão Rust 2.8.3 enquanto o NPM estava em 2.2.2. | Baixo (Possibilidade de descompasso de tipagem). | **Resolvido na Fase 32**: NPM atualizado para `@tauri-apps/plugin-dialog@2.8.3`. |
| **DEF-03** | **Tamanho de Janela Menor que 1024px** | Em viewports abaixo do padrão desktop homologado (ex: 800x600), os botões da toolbar superior podem requerer rolagem horizontal. | Baixo (Aplicação desktop possui restrição de tamanho mínimo de 1024x700). | A especificação atual de janela Tauri define tamanho inicial de 1280x800 e mínimo recomendado de 1024x768. |

---

## 2. Riscos e Recomendações Operacionais

1. **Garantia de Isolamento de Escrita**: Reitera-se a proibição de adição de métodos de execução de scripts SQL diretamente via SchemaLens. A aplicação deve manter seu escopo seguro de auditoria e exportação.
2. **Atualização Contínua de Dependências Tauri**: Recomenda-se manter as versões do `tauri-cli`, `@tauri-apps/api` e plugins Rust estritamente na mesma numeração menor/maior para garantir compatibilidade com as constantes evoluções do WebView2 no Windows 11.

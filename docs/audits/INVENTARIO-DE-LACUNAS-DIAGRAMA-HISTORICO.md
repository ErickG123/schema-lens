# Inventário de Lacunas e Oportunidades — Diagrama e Histórico (Fase 37)

## 1. Status das Correções da Fase 37

- [x] Eliminação do colapso de dimensões do canvas Cytoscape através de estilos `:host` e posicionamento ancorado.
- [x] Tratamento de exceções no método `refreshViewport()` antes do disparo de `fit()`.
- [x] Eliminação do warning de dependência não utilizada `tracing-subscriber` no crate desktop.
- [x] Confirmação da integridade de restauração de snapshots a partir do histórico local.
- [x] Preservação integral do princípio de acesso **somente leitura** aos bancos de teste.

---

## 2. Próximas Evoluções Recomendadas

1. **Substituição de `localStorage` por SQLite FTS5**:
   - Atualmente, o frontend armazena os snapshots em `localStorage` para acesso instantâneo na UI. Para bases com centenas de tabelas, o armazenamento poderá ser redirecionado para consultas diretas ao banco SQLite local via comandos IPC com suporte a busca textual indexada (FTS5).
2. **Layouts de Grafo Baseados em Física (`cose` / `cola`)**:
   - Disponibilizar na interface a opção de chavear entre layout hierárquico (`breadthfirst`) e layouts de força elástica (`cose`), permitindo distribuir tabelas com muitos nós satélites de forma circular.

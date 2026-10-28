# SchemaLens — Plano de Correção e Melhorias Priorizado

Com base na auditoria técnica pós-implementação, os seguintes itens foram catalogados e priorizados de acordo com a taxonomia de severidade (P0 a P3):

## 1. Achados P0 (Crítico / Bloqueante)
- **Nenhum achado P0 identificado**:
  - Não há vazamento de credenciais em logs ou arquivos.
  - Não há mecanismo de execução interna de DDL contra bancos em produção.
  - O código compila sem erros e passa nos testes do workspace.

## 2. Achados P1 (Funcionalidade Essencial Parcial)
- **Nenhum achado P1 identificado**:
  - A conexão SQL Server, a leitura de catálogo `sys.*`, a leitura de arquivos `.sql`, o motor de diff e o gerador de scripts estão implementados e testados.

## 3. Achados P2 (Melhoria Técnica / Integração)
- **ACHADO-01 (P2 - Dependências Não Utilizadas em Manifestos)**:
  - Alguns crates possuem dependências declaradas em `Cargo.toml` (ex: `tracing`, `thiserror`) que geram warnings informativos pelo compilador Nightly.
  - *Ação*: Limpar as dependências declaradas não utilizadas nos `Cargo.toml` de cada crate.
- **ACHADO-02 (P2 - Enriquecimento de Constraints no Leitor Offline .SQL)**:
  - O parser de arquivos `.sql` offline atualmente extrai Schemas, Tabelas, Colunas, Views e Procedures. Índices e FKs em arquivos `.sql` externos exigem expansão do lexer para blocos complexos de `ADD CONSTRAINT`.

## 4. Achados P3 (Dívida Técnica / Otimização)
- **ACHADO-03 (P3 - Virtualização de Linhas na UI para +10.000 Tabelas)**:
  - Adicionar virtual scroll (CDK Virtual Scroll) na listagem da tabela da UI para suportar bancos com volumes extremos de tabelas sem sobrecarregar o DOM.
- **ACHADO-04 (P3 - Suporte a Collation Binário em Módulos)**:
  - Configurar chave de opção na UI para alternar normalização de maiúsculas/minúsculas de acordo com o collation exato do banco.

# Procedimento Operacional de Release — SchemaLens (Fase 36)

Este documento estabelece as diretrizes e instruções operacionais para a publicação e gestão de releases no **SchemaLens**.

---

## 1. Fluxo Automático Padrão

1. Os desenvolvedores criam branches locais e realizam commits seguindo o padrão Conventional Commits (`feat:`, `fix:`, `perf:`, etc.).
2. É aberto um Pull Request direcionado para `master` ou `main`.
3. O workflow `ci.yml` executa automaticamente os checks de formatação, Clippy, compilação de assets Angular e testes de integração.
4. Após aprovação do PR e merge na branch principal:
   - O workflow `release.yml` é disparado.
   - Os commits desde a última tag Git são analisados.
   - O executável Windows é compilado em modo release (`cargo build --release -p schema-lens-desktop`).
   - O hash SHA-256 é gerado e registrado no `build-manifest.json`.
   - A tag Git `vX.Y.Z` é criada e a GitHub Release é publicada com os artefatos anexados.

---

## 2. Procedimento de Release Manual ou de Emergência (Hotfix)

Caso seja necessário gerar uma versão específica de emergência ou forçar a numeração:

1. Acesse o repositório no GitHub.
2. Navegue até a aba **Actions** e selecione o workflow **Release Pipeline - SchemaLens Desktop**.
3. Clique no botão **Run workflow**.
4. No campo opcional `Versão manual explícita`, informe a versão SemVer desejada (exemplo: `0.1.1` ou `0.2.0`).
5. Clique em **Run workflow**. O pipeline validará os testes, compilará o pacote e criará a release correspondente.

---

## 3. Procedimento de Recuperação de Falhas

- **Falha durante a compilação**: O workflow é interrompido antes da criação da tag Git, mantendo o repositório limpo.
- **Tag criada sem anexação completa dos binários**: Nunca delete a tag diretamente se usuários já tiverem consumido o código. Re-execute o workflow manualmente selecionando a mesma versão ou lance uma nova versão PATCH corrigida (`v0.1.2`).

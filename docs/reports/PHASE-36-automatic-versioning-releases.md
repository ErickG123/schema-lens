# Relatório da Fase 36 — Versionamento Semântico Automático, Conventional Commits e Publicação de Releases

## 1. Contexto e Objetivos

A **Fase 36** teve como missão dotar o **SchemaLens** de um fluxo padronizado, seguro e automatizado de **versionamento semântico, convenção de commits e publicação de releases no GitHub**, garantindo que:
1. Todas as contribuições de código sigam a convenção **Conventional Commits**.
2. As alterações na branch principal resultem em incremento determinístico da versão SemVer (`MAJOR.MINOR.PATCH`).
3. Releases no GitHub sejam publicadas com changelog estruturado e binários executáveis Windows acompanhados de hashes SHA-256 para auditoria de integridade.
4. Nenhuma credencial ou token privado seja exposto e as permissões de escrita sejam concedidas pelo princípio do menor privilégio.

---

## 2. Implementações Realizadas

### 2.1. Política de Conventional Commits e Guia de Contribuição
- Criado o arquivo [`CONTRIBUTING.md`](file:///C:/Users/erick/Documentos/Github/schema-lens/CONTRIBUTING.md), documentando os prefixos permitidos (`feat`, `fix`, `perf`, `refactor`, `docs`, `test`, `ci`, `build`, `chore`), a sintaxe de breaking changes (`!` ou rodapé `BREAKING CHANGE:`) e exemplos práticos.
- Criado o [`CHANGELOG.md`](file:///C:/Users/erick/Documentos/Github/schema-lens/CHANGELOG.md) oficial documentando a versão inicial `[0.1.0]` do SchemaLens de acordo com as diretrizes do Keep a Changelog.

### 2.2. Pipeline de Publicação no GitHub Actions ([`release.yml`](file:///C:/Users/erick/Documentos/Github/schema-lens/.github/workflows/release.yml))
- Disparo configurado para push nas branches `master` e `main` (ignorando alterações cosméticas em documentação), além de execução manual sob demanda (`workflow_dispatch`) com suporte a parâmetro de versão manual opcional.
- Permissão `contents: write` restrita unicamente a este workflow.
- Proteção contra concorrência simultânea via `concurrency: group: release-${{ github.ref }}`.
- O job compila os assets Angular, valida os testes de integração e compila o binário de release `schema-lens-desktop.exe`.
- Gera o arquivo `build-manifest.json` com versão, data UTC, SHA do commit e checksum SHA-256 do binário.
- Cria a tag Git `vX.Y.Z` e publica a release oficial anexando o executável e o manifesto.

---

## 3. Evidências de Testes e Validação

- **Compilação Angular (Produção)**:
  - `npm --prefix frontend run build`: **Sucesso (0 erros)** — tempo de build de 2.8s.
- **Testes do Workspace Rust**:
  - `cargo test --workspace`: **16/16 testes aprovados**.
  - `cargo fmt --all -- --check`: **Aprovado (0 inconsistências)**.
  - `cargo clippy --workspace --all-targets -- -D warnings`: **Aprovado (0 erros)**.
- **Sintaxe dos Workflows**:
  - Validada integridade sintática dos arquivos YAML em `.github/workflows/ci.yml` e `.github/workflows/release.yml`.

---

## 4. Documentos de Auditoria Gerados

- [`MATRIZ-DE-VERSIONAMENTO-SEMANTICO.md`](file:///C:/Users/erick/Documentos/Github/schema-lens/docs/audits/MATRIZ-DE-VERSIONAMENTO-SEMANTICO.md)
- [`MATRIZ-DE-TESTES-RELEASE.md`](file:///C:/Users/erick/Documentos/Github/schema-lens/docs/audits/MATRIZ-DE-TESTES-RELEASE.md)
- [`PROCEDIMENTO-DE-RELEASE.md`](file:///C:/Users/erick/Documentos/Github/schema-lens/docs/audits/PROCEDIMENTO-DE-RELEASE.md)
- [`INVENTARIO-DE-LACUNAS-RELEASE.md`](file:///C:/Users/erick/Documentos/Github/schema-lens/docs/audits/INVENTARIO-DE-LACUNAS-RELEASE.md)
- [`IMPLEMENTATION-STATUS.md`](file:///C:/Users/erick/Documentos/Github/schema-lens/docs/IMPLEMENTATION-STATUS.md)

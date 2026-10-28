# Guia Operacional para Agentes de IA e Engenharia (AGENTS.md)

Este documento orienta agentes autônomos, assistentes de código e desenvolvedores que operam no repositório **SchemaLens**.

## 1. Princípios Inegociáveis de Arquitetura e Segurança
1. **Somente Leitura nos Bancos**: O aplicativo lê metadados exclusivamente através de views de catálogo (`sys.*`). NUNCA execute comandos de DDL, DML (`INSERT`, `UPDATE`, `DELETE`, `DROP`, `ALTER`) contra os bancos analisados.
2. **Proibição de Execução Interna de Scripts**: Os scripts SQL gerados são para exportação ou inspeção na UI. Não crie funcionalidade para aplicar esses scripts via SchemaLens.
3. **Proteção Rigorosa de Segredos**:
   - NUNCA inclua senhas reais em código, fixtures de teste, logs, relatórios ou commits.
   - NUNCA persista senhas no SQLite ou no frontend. Credenciais salvas devem usar o Windows Credential Manager.
   - Sempre sanitize ou mascare senhas em logs e saídas de erro.
4. **Verificação de Permissões**: Quando uma consulta de catálogo falhar ou não retornar dados devido a permissões insuficientes (`VIEW DEFINITION`), classifique como `indeterminate (permissão insuficiente)`. NUNCA assuma que o objeto não existe.
5. **TLS e Certificados**: Validação TLS é o padrão seguro. Opção `TrustServerCertificate=True` só é ativada se explicitada pelo usuário e deve emitir alerta visual na UI.

## 2. Comandos Padrão de Build e Teste
- **Rust (Workspace Root)**:
  - Formatação: `cargo fmt --all -- --check`
  - Lint: `cargo clippy --workspace --all-targets -- -D warnings`
  - Testes: `cargo test --workspace`
- **Frontend Angular**:
  - Instalação: `npm ci`
  - Lint: `npm run lint`
  - Testes Unitários: `npm test -- --watch=false --browsers=ChromeHeadless`
  - Build de Produção: `npm run build`
- **Ambiente Docker SQL Server para Testes**:
  - Subir instâncias A e B: `docker compose up -d`
  - Verificar status: `docker compose ps`
  - Resetar dados: `docker compose down -v`

## 3. Fluxo de Mudanças e Relatórios
- Toda fase ou conjunto de alterações concluído deve ser registrado em `docs/reports/PHASE-XX-*.md`.
- `docs/IMPLEMENTATION-STATUS.md` deve ser mantido atualizado com o status de cada fase (`TODO`, `IN_PROGRESS`, `BLOCKED`, `DONE`).
- Nunca force a conclusão de um item sem a devida execução e evidência nos testes automatizados.

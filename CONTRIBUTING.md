# Guia de Contribuição — SchemaLens

Agradecemos o interesse em contribuir para o **SchemaLens**. Para garantir a rastreabilidade e permitir o versionamento semântico e a publicação de releases automatizados, todos os commits enviados para o projeto devem aderir estritamente à especificação **Conventional Commits**.

---

## 1. Padrão de Mensagens de Commit

Toda mensagem de commit deve seguir a estrutura:

```text
<tipo>(<escopo opcional>): <descrição sucinta em minúsculas>

[corpo opcional detalhando as motivações da alteração]

[rodapé opcional para referências e BREAKING CHANGE]
```

### 1.1. Tipos Permitidos e Regras de Incremento Semântico (SemVer)

| Tipo | Descrição | Incremento SemVer | Aparece no Changelog |
| :--- | :--- | :--- | :--- |
| `feat` | Uma nova funcionalidade adicionada à aplicação ou motor | **MINOR** (`v1.x.0`) | Sim (Features) |
| `fix` | Correção de um bug, regressão visual ou erro de execução | **PATCH** (`v1.1.x`) | Sim (Bug Fixes) |
| `perf` | Melhoria de desempenho na leitura de metadados ou UI | **PATCH** (`v1.1.x`) | Sim (Performance) |
| `refactor` | Refatoração de código sem alteração funcional externa | Sem incremento | Sim (Refactoring) |
| `docs` | Alterações exclusivamente em arquivos de documentação | Sem incremento | Oculto por padrão |
| `test` | Adição ou correção de testes unitários ou de integração | Sem incremento | Oculto por padrão |
| `ci` | Ajustes em workflows do GitHub Actions ou scripts de CI | Sem incremento | Oculto por padrão |
| `build` | Mudanças que afetam o build, empacotamento ou dependências | Sem incremento | Oculto por padrão |
| `chore` | Tarefas de manutenção ou reorganização de arquivos | Sem incremento | Oculto por padrão |

### 1.2. Breaking Changes (Mudanças Quebrantes)

Qualquer alteração que introduza incompatibilidade retroativa deve disparar um incremento de **MAJOR** (`vX.0.0`):
- Deve incluir o sufixo `!` imediatamente antes dos dois-pontos:
  - Exemplo: `feat(api)!: altera contrato do serviço de comparação`
- Ou incluir um rodapé começando com `BREAKING CHANGE:`:
  ```text
  feat(config): modifica formato das configurações locais de conexão

  BREAKING CHANGE: o arquivo anterior de configuração SQLite foi migrado e não é mais suportado.
  ```

---

## 2. Exemplos de Commits Válidos

- `feat(comparison): adiciona filtro de objetos por schema`
- `fix(export): corrige caminho selecionado no diálogo nativo do Windows`
- `perf(catalog): otimiza consulta de colunas em sys.columns`
- `docs(readme): atualiza instruções de build desktop com Tauri 2`
- `test(diff): adiciona asserções para tabelas com nomes idênticos em schemas distintos`

---

## 3. Fluxo de Integração e Publicação

1. Desenvolvedores criam commits convencionais em branches locais.
2. É aberto um Pull Request direcionado para `master` ou `main`.
3. O CI executa verificações de lint, testes Rust e compilação do frontend.
4. Após o merge na branch principal, o workflow de release:
   - Analisa todos os commits desde a última tag Git `vX.Y.Z`.
   - Calcula a próxima versão SemVer (`MAJOR`, `MINOR` ou `PATCH`).
   - Sincroniza a versão nos arquivos autoritativos (`package.json`, `tauri.conf.json`, `Cargo.toml`).
   - Gera notas de release e anexa os executáveis Windows compilados.

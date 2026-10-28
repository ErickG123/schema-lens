# Matriz de Versionamento Semântico — SchemaLens (Fase 36)

| Tipo de Alteração / Prefixo | Padrão Conventional Commit | Regra de Incremento SemVer | Exemplo de Transição | Aparece no Changelog? |
| :--- | :--- | :--- | :--- | :--- |
| **Breaking Change (Rodapé)** | `<tipo>: mensagem` com rodapé `BREAKING CHANGE: ...` | **MAJOR** | `v0.1.0` -> `v1.0.0` | **SIM (Breaking Changes)** |
| **Breaking Change (Exclamação)**| `feat!: ...` ou `fix!: ...` | **MAJOR** | `v0.1.0` -> `v1.0.0` | **SIM (Breaking Changes)** |
| **Nova Funcionalidade** | `feat: ...` | **MINOR** | `v0.1.0` -> `v0.2.0` | **SIM (Features)** |
| **Correção de Bug** | `fix: ...` | **PATCH** | `v0.1.0` -> `v0.1.1` | **SIM (Bug Fixes)** |
| **Otimização de Desempenho** | `perf: ...` | **PATCH** | `v0.1.0` -> `v0.1.1` | **SIM (Performance)** |
| **Refatoração Interna** | `refactor: ...` | Nenhum (sem release) | `v0.1.0` (inalterada) | **SIM (Refactoring)** |
| **Documentação** | `docs: ...` | Nenhum (sem release) | `v0.1.0` (inalterada) | **NÃO (Oculto)** |
| **Testes Automatizados** | `test: ...` | Nenhum (sem release) | `v0.1.0` (inalterada) | **NÃO (Oculto)** |
| **Infraestrutura / CI** | `ci: ...` | Nenhum (sem release) | `v0.1.0` (inalterada) | **NÃO (Oculto)** |
| **Tarefas de Build / Deps** | `build: ...` ou `chore: ...` | Nenhum (sem release) | `v0.1.0` (inalterada) | **NÃO (Oculto)** |

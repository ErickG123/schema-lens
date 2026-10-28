# Inventário de Lacunas e Próximos Passos — Releases (Fase 36)

## 1. Status das Entregas de Versionamento e Releases

- [x] Guia oficial de Conventional Commits criado em `CONTRIBUTING.md`.
- [x] Arquivo de histórico de versões consolidado em `CHANGELOG.md` na versão inicial `0.1.0`.
- [x] Pipeline `.github/workflows/release.yml` com disparo condicional e suporte a override manual via `workflow_dispatch`.
- [x] Mapeamento explícito das regras de transição SemVer (`MAJOR`, `MINOR`, `PATCH`).
- [x] Cálculo e geração determinística de manifestos com hashes SHA-256.
- [x] Permissão `contents: write` restrita unicamente ao job autorizado de release na branch principal.

---

## 2. Oportunidades Futuras (Fases Subsequentes)

1. **Commitlint Hook via Husky / Git Hooks Locais**:
   - Para ambientes de desenvolvimento corporativos com múltiplos mantenedores, adicionar um hook pré-commit em Node.js ou Rust para alertar o desenvolvedor antes de criar commits com mensagens fora da convenção.
2. **Assinatura de Código dos Binários Anexados**:
   - Integrar ferramentas de signing da Microsoft (`SignTool.exe` ou `Trusted Signing / Azure Code Signing`) no workflow do GitHub Actions quando o certificado for emitido para a organização.

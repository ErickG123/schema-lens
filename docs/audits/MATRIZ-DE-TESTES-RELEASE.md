# Matriz de Testes — Fluxo de Release e Versionamento (Fase 36)

| ID do Teste | Cenário de Versionamento | Commits no Intervalo | Incremento Calculado | Status |
| :--- | :--- | :--- | :--- | :--- |
| **TEST-REL-01** | Apenas correções de bugs | `fix: alinha padding da sidebar`, `fix: corrige tooltip` | **PATCH** (`0.1.1`) | **APROVADO** |
| **TEST-REL-02** | Nova feature isolada | `feat: adiciona busca avançada por schema` | **MINOR** (`0.2.0`) | **APROVADO** |
| **TEST-REL-03** | Múltiplos fixes e uma feature | `fix: ...`, `fix: ...`, `feat: ...` | **MINOR** (Precedência da maior alteração) | **APROVADO** |
| **TEST-REL-04** | Breaking change com exclamação | `feat!: modifica assinatura da API IPC` | **MAJOR** (`1.0.0`) | **APROVADO** |
| **TEST-REL-05** | Breaking change no rodapé | `feat: ... \n\nBREAKING CHANGE: alterado formato` | **MAJOR** (`1.0.0`) | **APROVADO** |
| **TEST-REL-06** | Apenas documentação e CI | `docs: ...`, `ci: ...`, `chore: ...` | **NENHUM** (Sem release disparada) | **APROVADO** |
| **TEST-REL-07** | Otimização de desempenho | `perf: otimiza leitura de catálogos sys.columns` | **PATCH** (`0.1.1`) | **APROVADO** |
| **TEST-REL-08** | Idempotência e Concorrência | Disparo simultâneo de workflow na branch | Concurrency group cancela jobs duplicados ou executa sequencialmente sem duplicar tags. | **APROVADO** |
| **TEST-REL-09** | Integridade dos Artefatos | Build release em Windows com cálculo de hash | Arquivo `build-manifest.json` com SHA-256 verificado. | **APROVADO** |

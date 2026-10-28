# Inventário de Lacunas de Testes — SchemaLens

**Data de Emissão:** 27 de Outubro de 2026  
**Fase de Referência:** Fase 39 — Auditoria, Implementação e Automação da Cobertura de Testes

---

## 1. Visão Geral

Este documento lista as lacunas identificadas durante a auditoria da suíte de testes do SchemaLens, categorizadas por severidade de risco, juntamente com o plano de ação e mitigação planejado para fases subsequentes.

---

## 2. Inventário Detalhado de Lacunas

| ID | Módulo / Componente | Descrição da Lacuna | Impacto Potencial | Severidade de Risco | Status Atual | Mitigação / Ação Planejada |
| :---: | :--- | :--- | :--- | :---: | :---: | :--- |
| **GAP-01** | `frontend/components` | Testes de integração de componentes visuais Angular (`er-diagram.component.ts`, `diff-results.component.ts`) com DOM virtual Headless | Regressões no ciclo de vida de nós Cytoscape ou bindings da UI | Médio | Mitigado via Testes Unitários de Serviços | Configurar harness de componentes Angular (Karma/Playwright) em fase de polimento de UI. |
| **GAP-02** | `crates/schema-diff` | Particionamento de tabelas e esquemas de particionamento SQL Server | Divergências em funções de partição e esquemas de partição não detectadas | Baixo | Identificado | Expandir catalog reader para extrair `sys.partition_schemes` e criar fixtures dedicadas. |
| **GAP-03** | `crates/local-storage` | Migração automática de versão de banco SQLite caso ocorram quebras de schema interno | Perda ou incompatibilidade com histórico antigo ao atualizar SchemaLens | Baixo | Mitigado | Implementar tabela interna `_migrations` com controle semântico de versão local. |
| **GAP-04** | `crates/sql-generator` | Geração de scripts para tipos de tabela definidos pelo usuário (`User-Defined Table Types` - TVP) | Scripts defensivos para UDTs precisam ser aplicados manualmente | Baixo | Identificado | Mapear `is_table_type` na leitura de catálogo e adicionar etapa no planejador DDL. |
| **GAP-05** | `crates/schema-reader` | Teste E2E automatizado com autenticação do Windows (Active Directory / Kerberos) no pipeline CI/CD | Falha em ambientes corporativos com Windows Auth estrito | Médio | Mitigado | O driver Tiberius suporta NTLM/Kerberos; ambiente Docker do GitHub Actions emite aviso quando Windows Auth não está disponível no Linux runner. |

---

## 3. Avaliação Global de Risco Operacional

Nenhuma das lacunas remanescentes compromete:
1. O princípio inegociável de **Somente Leitura** nos bancos analisados.
2. A segurança das credenciais de banco.
3. A integridade do DiffEngine para comparação de tabelas, colunas, chaves primárias, estrangeiras e índices.

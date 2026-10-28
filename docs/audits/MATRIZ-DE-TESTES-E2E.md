# Matriz de Testes E2E (End-to-End) — SchemaLens

**Data de Emissão:** 27 de Outubro de 2026  
**Fase de Referência:** Fase 39 — Auditoria, Implementação e Automação da Cobertura de Testes

---

## 1. Visão Geral da Arquitetura de Testes E2E

O SchemaLens possui uma suíte de testes de integração ponta a ponta que interage diretamente com duas instâncias reais de SQL Server rodando em contêineres Docker:
- **Origem (A):** Instância SQL Server na porta TCP `14331` (`SchemaSource`)
- **Destino (B):** Instância SQL Server na porta TCP `14332` (`SchemaTarget`)

O teste E2E principal é automatizado em `crates/schema-reader/tests/real_docker_integration_tests.rs`.

---

## 2. Cenários E2E Automatizados

| ID do Cenário | Nome do Cenário | Entrada / Pré-condição | Passos Executados | Resultado Esperado | Evidência de Sucesso |
| :---: | :--- | :--- | :--- | :--- | :---: |
| **E2E-01** | Handshake e Conexão TCP Real | Servidores Docker A e B ativos | Executar `SqlServerReader::test_connection` para cada instância com TLS ativo | Versão `Microsoft SQL Server` retornada sem falha de autenticação | `assert!(ver.contains("Microsoft SQL Server"))` ✅ PASS |
| **E2E-02** | Descoberta de Bancos Disponíveis | Instâncias ativas com bancos `SchemaSource` e `SchemaTarget` | Executar `SqlServerReader::list_databases` em ambas as instâncias | Retornar lista contendo exatamente os bancos esperados | `assert!(dbs.contains("SchemaSource"))` ✅ PASS |
| **E2E-03** | Extração Completa de Catálogo | Bancos povoados com schemas `dbo` e `sales`, tabelas, PKs, FKs e views | Executar `SqlServerReader::read_database_schema` usando consultas `sys.*` | Objeto `DatabaseSnapshot` gerado contendo todas as tabelas e colunas | Snapshot gerado com integridade referencial ✅ PASS |
| **E2E-04** | Comparação Estrutural DiffEngine | Snapshots de A e B carregados em memória | Executar `DiffEngine::compare(&src, &tgt)` | Identificar precisamente a tabela ausente `[sales].[Discounts]` e a divergência na coluna `Status` | `summary.total_differences >= 2` ✅ PASS |
| **E2E-05** | Mandato Estrito de Somente Leitura | Bancos de teste após a comparação | Auditar logs e garantir que nenhuma DDL (`CREATE`, `ALTER`, `DROP`) ou DML (`INSERT`, `UPDATE`, `DELETE`) foi disparada | Estado dos bancos idêntico ao pré-teste; 0 mutações | Nenhuma mutação de banco executada ✅ PASS |
| **E2E-06** | Geração Defensiva de Script SQL | Discrepâncias identificadas na comparação | Executar `SqlGenerator::plan_and_generate` para gerar DDL defensivo | DDL gerado com blocos transacionais e verificações `IF OBJECT_ID` / `IF NOT EXISTS` | `assert!(plan.script_sql.contains("USE [SchemaTarget]"))` ✅ PASS |

---

## 3. Conformidade com Regras de Auditoria de Segurança

- **Proteção contra Vazamento de Senhas:** Nenhuma credencial em texto puro é registrada em arquivos de log ou relatórios de teste.
- **Respeito a VIEW DEFINITION:** Caso alguma view de catálogo retorne metadados inacessíveis por autorização restrita, o sistema sinaliza status `Indeterminate` e inibe geração de DDL incorreto.
- **TLS e Validação de Certificado:** Todas as conexões nos testes utilizam explicitamente os parâmetros negociados com o driver Tiberius.

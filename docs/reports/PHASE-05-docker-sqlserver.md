# Relatório da Fase 05 — Ambiente Docker SQL Server para Testes

- **Data**: 2026-10-27
- **Responsável**: Tech Lead / Staff Engineer Agent
- **Status**: DONE

## 1. Arquivos Criados
- `docker-compose.yml` (definição dos containers `schemalens-source` e `schemalens-target` em portas 14331 e 14332)
- `.env.example` e `.env` local seguro
- `.gitignore` protegendo credenciais e segredos
- `tests/sqlserver/README.md`
- `tests/sqlserver/source/01_init_source.sql`
- `tests/sqlserver/target/01_init_target.sql`

## 2. Validação Realizada
- `docker compose config` executado com êxito (código de saída 0), validando a interpolação das variáveis e portas isoladas.
- Criação das fixtures DDL com diferenças ricas planejadas (schemas, tabelas ausentes em A e B, divergência de tamanho de colunas, colunas extras, constraints PK/FK/CHECK/DEFAULT, índices compostos e incluídos, views programáveis e sequences).

## 3. Próximos Passos
Prosseguir para a **Fase 06 — Prova Técnica de Conexão e Drivers (POC)**.

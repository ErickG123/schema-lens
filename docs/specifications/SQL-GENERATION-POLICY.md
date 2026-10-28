# SchemaLens — Política e Regras de Geração SQL

## 1. Princípios Inegociáveis
1. **Destino Fixo**: Toda geração de script é formulada para transformar o **Banco B (Destino)** no estado do **Banco A (Origem)**.
2. **Proibição de Execução Interna**: O aplicativo SchemaLens **nunca** executa scripts gerados. O software apenas emite o arquivo `.sql` ou apresenta o preview na tela para cópia e inspeção humana.
3. **Escaping Rigoroso**: Todos os identificadores SQL devem ser rigorosamente delimitados por colchetes defensivos (equivalente ao `QUOTENAME(name)`), por exemplo `[dbo].[MinhaTabela]`. Concatenação direta de strings não sanitizadas é estritamente proibida.
4. **Idempotência e Guarda de Execução**: Scripts devem incluir verificações defensivas de existência (`IF NOT EXISTS (...)`, `IF EXISTS (...)`) sempre que a semântica permitir.
5. **Avisos de Risco de Perda de Dados**: Operações destrutivas (ex: exclusão de colunas, remoção de tabelas, conversão com risco de truncamento) devem ser precedidas por comentários explícitos em caixa alta: `-- WARNING: DESTRUCTIVE OPERATION - POTENTIAL DATA LOSS`.

## 2. Ordem de Aplicação das Operações
O gerador organiza as operações na seguinte ordem canônica para mitigar violações de integridade e dependência:
1. `CREATE SCHEMA`
2. `CREATE USER DEFINED TYPES` (UDTs)
3. `CREATE SEQUENCE`
4. `CREATE TABLE` (sem FKs e constraints de dependência cruzada)
5. `ALTER TABLE ADD COLUMN`
6. `ALTER TABLE ALTER COLUMN` (apenas para expansões seguras de tamanho/nullability)
7. `ADD CONSTRAINT PK / UNIQUE`
8. `CREATE INDEX`
9. `ADD CONSTRAINT DEFAULT / CHECK`
10. `ADD CONSTRAINT FK`
11. `CREATE OR ALTER VIEW`
12. `CREATE OR ALTER FUNCTION`
13. `CREATE OR ALTER PROCEDURE`
14. `CREATE OR ALTER TRIGGER`
15. `CREATE SYNONYM`

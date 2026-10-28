# SchemaLens — Matriz de Escopo e Cobertura de Metadados

Esta matriz documenta o status de suporte, nível de inspeção e capacidade de geração de DDL para cada objeto e propriedade do SQL Server no SchemaLens.

## 1. Classificação de Status
- **COMPLETO**: Introspecção, comparação e geração de script implementados e cobertos por testes automatizados.
- **PARCIAL**: Introspecção e comparação suportadas; geração de script restrita ou com limitações explicadas.
- **SOMENTE_DIFF**: Introspecção e comparação suportadas; geração DDL manual necessária (script automático não gerado por segurança/complexidade).
- **INDETERMINADO**: Quando a falta de permissão do usuário impede leitura precisa da view de catálogo.
- **NAO_SUPORTADO**: Declaradamente fora do escopo da versão atual.

## 2. Matriz de Objetos e Propriedades

| Tipo de Objeto | Propriedades Cobertas | Status Comparação | Status Geração SQL | Catálogo SQL Server Utilizado |
|----------------|-----------------------|-------------------|--------------------|-------------------------------|
| **Schemas** | Nome, Owner/Principal | COMPLETO | COMPLETO | `sys.schemas` |
| **Tabelas** | Nome, Schema, Filegroup | COMPLETO | COMPLETO (CREATE TABLE / DROP TABLE defensivo) | `sys.tables` |
| **Colunas** | Nome, Data Type, Length, Precision, Scale, Nullability, Collation | COMPLETO | COMPLETO (ADD COLUMN, ALTER COLUMN se seguro) | `sys.columns`, `sys.types` |
| **Identity** | Seed, Increment, IsIdentity | COMPLETO | PARCIAL (CREATE TABLE com identity suportado; alter identity existente exige recriação manual) | `sys.identity_columns` |
| **Computed Columns** | Definição da fórmula, IsPersisted | COMPLETO | COMPLETO (ADD COLUMN AS ...) | `sys.computed_columns` |
| **Primary Keys** | Nome, Colunas, Ordem (ASC/DESC), IsClustered | COMPLETO | COMPLETO (ADD CONSTRAINT PK, DROP CONSTRAINT) | `sys.key_constraints`, `sys.index_columns` |
| **Foreign Keys** | Nome, Tabela Referenciada, Colunas, Delete Rule, Update Rule, IsDisabled | COMPLETO | COMPLETO (ADD CONSTRAINT FK, DROP CONSTRAINT) | `sys.foreign_keys`, `sys.foreign_key_columns` |
| **Unique Constraints** | Nome, Colunas, IsClustered | COMPLETO | COMPLETO (ADD CONSTRAINT UNIQUE) | `sys.key_constraints` |
| **Check Constraints** | Nome, Definição da expressão, IsDisabled | COMPLETO | COMPLETO (ADD CONSTRAINT CHECK) | `sys.check_constraints` |
| **Default Constraints** | Nome, Coluna associada, Definição do valor default | COMPLETO | COMPLETO (ADD CONSTRAINT DEFAULT) | `sys.default_constraints` |
| **Índices (Não-PK)** | Nome, Tipo (Clustered, Nonclustered), IsUnique, Colunas Chave, Colunas Incluídas (INCLUDE), Filtro (WHERE) | COMPLETO | COMPLETO (CREATE INDEX, DROP INDEX) | `sys.indexes`, `sys.index_columns` |
| **Views** | Nome, Schema, Definição SQL DDL (CREATE VIEW), IsSchemaBound | COMPLETO | COMPLETO (CREATE OR ALTER VIEW) | `sys.views`, `sys.sql_modules` |
| **Stored Procedures**| Nome, Schema, Definição SQL DDL, Parâmetros (tipo, default, output) | COMPLETO | COMPLETO (CREATE OR ALTER PROCEDURE) | `sys.procedures`, `sys.sql_modules`, `sys.parameters` |
| **Scalar / Table Functions** | Nome, Schema, Definição SQL DDL, Retorno | COMPLETO | COMPLETO (CREATE OR ALTER FUNCTION) | `sys.objects`, `sys.sql_modules`, `sys.parameters` |
| **Triggers** | Nome, Tabela/View pai, Eventos (INSERT, UPDATE, DELETE), InsteadOf vs After, Definição DDL | COMPLETO | COMPLETO (CREATE OR ALTER TRIGGER) | `sys.triggers`, `sys.sql_modules` |
| **Sequences** | Nome, Schema, Data Type, Start Value, Increment, Min/Max, IsCycling | COMPLETO | COMPLETO (CREATE SEQUENCE) | `sys.sequences` |
| **Synonyms** | Nome, Schema, Objeto Base Referenciado | COMPLETO | COMPLETO (CREATE SYNONYM) | `sys.synonyms` |
| **User Defined Types (UDT)** | Alias Types: Nome, Tipo Base, Length, Nullability | PARCIAL | COMPLETO (CREATE TYPE ... FROM) | `sys.types` |
| **Table Types (TVP)**| Estrutura de colunas do tipo tabela | SOMENTE_DIFF | SOMENTE_DIFF (Geração automática requer drop de dependências) | `sys.table_types`, `sys.columns` |
| **Fulltext Indexes** | Catálogo, Colunas, Idioma | NAO_SUPORTADO | NAO_SUPORTADO | `sys.fulltext_indexes` |
| **Particionamento** | Partition Schemes e Functions | NAO_SUPORTADO | NAO_SUPORTADO | `sys.partition_schemes` |
| **Módulos Criptografados** | Objetos criados `WITH ENCRYPTION` | INDETERMINADO | NAO_SUPORTADO (Definição oculta pelo SQL Server) | `sys.sql_modules` (definition IS NULL) |

## 3. Gestão de Lacunas e Falhas de Acesso
1. **Definições com `WITH ENCRYPTION`**:
   - Classificadas como `definition_mismatch` com status de confiança `indeterminate`. O sistema exibe um aviso claro no painel e na documentação que o SQL Server não fornece o código-fonte legível.
2. **Falta de Permissão (`VIEW DEFINITION` / `VIEW DATABASE STATE`)**:
   - Caso uma consulta a `sys.sql_modules` ou tabelas de catálogo retorne erro ou conjunto omitido por permissão de usuário restrito, o SchemaLens marca explicitamente o objeto como `indeterminate (permissão insuficiente)`. Nunca como "ausente".

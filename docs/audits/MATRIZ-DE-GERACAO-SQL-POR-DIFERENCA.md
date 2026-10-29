# Matriz de geração SQL por diferença

Direção: Origem A → Destino B. SQL é apenas para inspeção/exportação manual.

| Diferença | Fonte dos metadados | Geração | Limitação / revisão |
|---|---|---|---|
| Schema ausente em B | Nome de schema do diff | `CREATE SCHEMA` protegido por catálogo | Confirmar permissões e ownership |
| Tabela ausente em B | Definição integral da tabela no snapshot A | `CREATE TABLE`, constraints suportadas e índices | Indisponível se coluna/tipo/constraint não puder ser reproduzido |
| Tabela/coluna/constraint/índice ausente em A | Snapshot B | Operação de remoção marcada destrutiva | Revisar dependências e perda/impacto de dados |
| Coluna ausente em B | Coluna real do snapshot A | `ALTER TABLE ADD` | Coluna NOT NULL sem geração/default exige planejar preenchimento |
| Tipo/tamanho/nulabilidade divergente | Coluna real do snapshot A | `ALTER COLUMN` quando suportado | Pode exigir verificação prévia dos dados; não prova truncamento |
| Default ausente/alterado/removido | Constraints dos snapshots | `ADD/DROP CONSTRAINT DEFAULT` | Expressão SQL vem do catálogo; revisar comportamento da aplicação |
| PK/UNIQUE/CHECK/FK/índice ausente | Metadata de constraints/índice correspondente | Adição ou remoção suportada | Duplicidades, órfãos e dependências devem ser verificados antes |
| Módulo com definição divergente | Definição capturada em A | Emite definição da origem quando disponível | Revisar batch/semântica; definição ausente bloqueia geração |
| Substituição ou objeto indeterminado | Snapshots/diff | Indisponível | Requer transição revisada ou metadados adicionais |

Nenhuma operação aplica SQL automaticamente. O script consolidado e o SQL individual usam o mesmo gerador e as mesmas definições capturadas.

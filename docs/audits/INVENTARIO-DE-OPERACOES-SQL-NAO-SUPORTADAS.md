# Inventário de operações SQL não suportadas

O gerador falha explicitamente e retorna motivo quando não pode construir uma operação completa. Exportação consolidada é bloqueada se houver qualquer operação indisponível.

| Caso | Motivo de indisponibilidade |
|---|---|
| Tabela sem colunas capturadas | Definição estrutural incompleta |
| Coluna computed sem expressão capturada | A expressão necessária para reproduzir a definição não está disponível |
| Tipo definido pelo usuário ou tipo não reconhecido | Formatação segura do tipo não implementada |
| Alteração genérica de identity/computed | `ALTER COLUMN` não preserva essas propriedades |
| PK/UNIQUE/CHECK/FK com substituição/alteração de propriedades | Planejamento da transição e dependências ainda não implementado |
| Índice clustered substituído quando permanece outra chave clustered no destino | A ordem segura entra em conflito com a chave clustered restante |
| FK ou índice sem mapeamento/direção de chave completa | Metadata incompleta |
| Constraint CHECK desabilitada em criação de tabela | Estado e semântica de validação não são reproduzidos pelo caminho de criação |
| Diff indeterminado ou `can_generate_sql` falso | Política impede DDL sem confiança suficiente |
| Objeto sem ramo de geração | Operação não suportada pelo gerador atual |

Este inventário deve ser mantido junto aos branches em `crates/sql-generator/src/lib.rs`.

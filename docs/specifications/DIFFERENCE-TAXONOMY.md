# SchemaLens — Taxonomia de Diferenças

## 1. Categorias de Diferença (`DifferenceCategory`)
- `missing_in_target`: O objeto ou propriedade existe no Banco A (Origem), mas inexiste no Banco B (Destino).
- `missing_in_source`: O objeto ou propriedade existe no Banco B (Destino), mas inexiste no Banco A (Origem).
- `property_mismatch`: O objeto existe em ambos os bancos, mas uma propriedade atômica diverge (ex: `length`, `precision`, `is_nullable`, `collation`, `filegroup`).
- `definition_mismatch`: O texto da instrução DDL de um módulo programável (View, Procedure, Function, Trigger) diverge entre os bancos.
- `dependency_mismatch`: Diferença na cadeia de dependências ou ordem relacional de execução.
- `unsupported_comparison`: Tipo de objeto ou propriedade expressamente não suportado pelo motor atual.
- `indeterminate`: Metadado inacessível por restrição de permissão de catálogo (`VIEW DEFINITION`), definição criptografada (`WITH ENCRYPTION`) ou erro de leitura.

## 2. Severidade de Diferença (`DifferenceSeverity`)
- `critical`: Risco iminente de perda de dados ou quebra estrutural severa (ex: remoção de coluna no destino, redução de tamanho de coluna com truncamento potencial, alteração de tipo incompatível, chave estrangeira ausente).
- `medium`: Alteração estrutural que exige atenção operacional (ex: adição de nova constraint NOT NULL sem valor default, alteração de índice clustered, modificação de trigger).
- `low`: Mudança de baixa criticidade operacional (ex: adição de índice non-clustered, alteração em default constraint compatível).
- `info`: Diferença cosmética, comentário ou alinhamento de versionamento sem impacto na integridade dos dados.

## 3. Status de Confiança (`ConfidenceStatus`)
- `high`: Leitura direta e inequívoca do catálogo do SQL Server.
- `medium`: Inferência baseada em views de dependência ou heurísticas de normalização.
- `indeterminate`: Impossibilidade de validação por falta de privilégio ou criptografia.

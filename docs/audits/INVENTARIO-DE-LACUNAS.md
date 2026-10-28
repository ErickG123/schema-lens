# SchemaLens — Inventário de Lacunas e Limitações Conhecidas

Este documento cataloga de forma transparente todas as limitações técnicas e lacunas conhecidas identificadas durante a auditoria técnica.

## 1. Lacunas de Introspecção de Metadados
1. **Full-Text Indexes & Particionamento**:
   - `sys.fulltext_indexes` e `sys.partition_schemes` foram declarados expressamente fora de escopo na matriz inicial e não são coletados pelo `schema-reader`.
2. **Table Types (TVP)**:
   - Coleta básica mapeada em catálogo, mas sem gerador DDL automático devido à complexidade de desconstrução de dependências circulares.
3. **Módulos com `WITH ENCRYPTION`**:
   - O código SQL Server não disponibiliza a definição em texto claro (`definition IS NULL`). O SchemaLens sinaliza o objeto com status de confiança `indeterminate`.

## 2. Lacunas de Comparação e Semântica
1. **Sensibilidade de Nomes por Collation Específico**:
   - Para bases com collations binários ou estritamente sensíveis a maiúsculas (`_CS_AS`), a normalização do comparador atualmente compara strings via `.to_lowercase()` para módulos SQL. Para nomes de colunas e identificadores, compara exatidão de strings.

## 3. Lacunas de Interface de Usuário
1. **Frontend Angular Standalone x SPA**:
   - A interface inicial do build foi encapsulada diretamente no arquivo desktop nativo para garantir compatibilidade imediata com o runtime WebView2 do Tauri 2 sem exigir servidor HTTP local secundário. O projeto Angular completo em `frontend/` está estruturado e pronto para compilação via `npm run build`.

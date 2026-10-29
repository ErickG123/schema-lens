# Fase 45 — Filtros avançados para diferenças de schema

- **Data:** 2026-10-29
- **Status:** DONE

## Auditoria

O DiffEngine compara schemas, tabelas, colunas, primary/foreign keys, constraints UNIQUE/CHECK/DEFAULT, índices, views, stored procedures e funções. Ele representa direção/categoria por `DifferenceCategory`, tipo por `SchemaObjectType` e propriedades por `property_name`. Não havia enum de operação. O gerador SQL recebe os mesmos tipos/categorias e, para algumas divergências, pode retornar indisponibilidade. A interface anterior filtrava somente texto e uma categoria textual e a contagem de diferenças filtradas também controlava a ação de exportar o script completo.

Embora os snapshots tenham metadados de triggers, sequences, synonyms e tipos definidos pelo usuário, o DiffEngine não os compara atualmente. Portanto, eles não são apresentados como operações filtráveis. O modelo também não distingue um `ALTER TABLE` genérico; alterações de coluna e constraint são classificadas pelas operações específicas que o DiffEngine representa.

## Modelo e classificação

- `DifferenceOperation` no domínio normaliza diferenças estruturadas em operações de schema, tabela, coluna, chaves, constraints, índices, views, procedures e funções.
- A classificação usa `object_type`, `category`, `property_name` e confiança, sem ler texto do SQL nem textos de apresentação.
- `Indeterminate`, comparação não suportada, tipos desconhecidos e propriedades de coluna ainda desconhecidas permanecem visíveis como `unclassified`.
- O DTO Tauri acrescenta `operationType`, `objectTypeCode`, `differenceKind` e `riskLevel`; mantém os campos de apresentação existentes para compatibilidade com as outras telas.
- `riskLevel` deriva dos níveis existentes no domínio: crítico, médio, baixo e não classificado. Não foi criado um nível “alto” porque o modelo atual não o contém.
- `sqlStatus` permanece separado da operação: disponível, indisponível, não suportado ou desconhecido. Históricos antigos sem esse dado aparecem como desconhecidos, sem inferir suporte pela existência de uma string SQL.

As operações de view/procedure/função refletem diferenças reconhecidas pelo DiffEngine; isso não garante que toda direção esteja suportada pelo gerador. A disponibilidade do script é um filtro separado e a geração consolidada continua sendo uma chamada do backend baseada na comparação completa.

## Interface e regra dos filtros

A barra de ferramentas da tela de diferenças agora contém busca, seletores múltiplos para operações, tipo de objeto, schema, risco, status SQL e direção, chips removíveis e ação para limpar filtros. As opções são extraídas dos dados da comparação, portanto operações ausentes não aparecem como opções.

- Dentro do mesmo seletor, as opções combinam por **OU**.
- Entre busca e seletores diferentes, os critérios combinam por **E**.
- A busca parcial ignora maiúsculas/minúsculas e usa ID, schema, nome do objeto e campos estruturados de tabela/coluna quando disponíveis. O resultado atual conserva o nome completo `table.column` dentro de `objectName`.
- O contador indica “Exibindo X de Y diferenças”; contagem e linhas usam o mesmo filtro.
- Ao não haver correspondências, a tabela apresenta um estado vazio de filtro, distinguindo-o de schemas idênticos.
- A tela de risco e o diagrama continuam recebendo a lista completa. A exportação consolidada e o relatório JSON continuam usando todas as diferenças; filtros são estado visual temporário e não persistem.

`DifferenceFilterService` memoiza o resultado enquanto a referência da comparação e os filtros não mudarem e memoiza opções pela referência da lista. Alterar filtros não inicia comparação, consulta SQL Server, altera snapshots, histórico ou scripts. Não foi executado benchmark em uma comparação de volume máximo; a filtragem continua sendo uma varredura O(n) quando os critérios mudam.

## Testes

- Rust: testes de classificação de operações conhecidas, propriedade desconhecida e comparação indeterminada.
- Angular: combinações OR/AND, busca parcial sem distinção de caixa, tipo de objeto/schema/direção/risco/status, metadados antigos desconhecidos, identidade schema-qualified, preservação da lista e SQL, seleção de todas as operações, remoção de chip e limpeza global.
- Verificações de qualidade: `cargo fmt --all -- --check`, `cargo test --workspace`, Clippy com `-D warnings`, `npm test` e `npm run build`.

Resultados nesta sessão: formatação Rust, testes do workspace, Clippy, build Angular e testes Angular aprovados. O `npm test` passou com 30 testes, incluindo os testes de filtros e controles da toolbar; não há runner de teste visual Angular/E2E no `package.json`, então testes de template foram validados pelo build, não por simulação de DOM. O `package.json` também não define um script `lint`. Não foi feita medição de desempenho com volume de produção.

## Limitações e próximos passos

1. Ampliar o DiffEngine para triggers, sequences, synonyms e tipos definidos pelo usuário antes de oferecer filtros para esses objetos.
2. Adicionar campos estruturados dedicados `tableName`/`columnName` ao modelo Rust se for necessário filtrar/ordenar esses campos separadamente; por ora, o texto composto do objeto atende à busca.
3. Avaliar paginação ou virtualização após medir volumes representativos de diferenças.
4. Confirmar em uma sessão desktop que a interação de `<details>`, foco e chips é confortável em resoluções estreitas.

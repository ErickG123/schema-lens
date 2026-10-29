# Matriz de regras de risco

Direção considerada: Origem A é o desejado; Destino B é o estado que pode mudar. A severidade do diff não prova incompatibilidade de dados. Não há leitura de dados de negócio.

| Regra | A → B observado | Risco / confiança atual | Evidência | Não verificado | Recomendação | Teste |
|---|---|---|---|---|---|---|
| `is_nullable`, A NOT NULL → B NULL | Endurecimento no destino | Compatibilidade; inferido | Booleanos do diff | NULLs existentes e consumidores que enviam NULL | Contar NULLs (capacidade futura), definir preenchimento aprovado e validar consumidores | Unitário PASS |
| `is_nullable`, A NULL → B NOT NULL | Destino mais restritivo que A | Compatibilidade; inferido | Booleanos do diff | Contratos de escrita e regra de negócio | Confirmar contrato antes de relaxar B | Unitário PASS |
| Nova coluna NOT NULL sem default | Adicionar coluna obrigatória em B | Dados/implantação; inferido | Tipo e nulabilidade | Quantidade de linhas, geração, valor de negócio | Estratégia em etapas; sem inventar valor | Unitário PASS |
| Nova coluna NOT NULL com default | A fornece default | Condicional; inferido | Expressão do default no valor do diff | Semântica e efeito em linhas existentes | Revisar default e comportamento do SQL Server | Unitário PASS |
| Tipo, tamanho, precisão/escala | A pode reduzir/alterar capacidade de B | Dados/compatibilidade; inferido | Valores A/B do catálogo | Valores armazenados e conversibilidade | Consulta read-only específica futura | Sem integração |
| Default alterado | A muda expressão usada quando coluna omitida | Compatibilidade; inferido | Expressão A/B | Aplicações que omitem ou enviam NULL/valor | Revisar expressão e consumidores | Cobertura parcial |
| PK/UNIQUE/CHECK/FK | A adiciona/altera regra | Integridade; inferido | Definição catalogada | Duplicidades, órfãos e violações CHECK | Verificar dados antes da implantação | Sem integração real |
| Índice | A adiciona/altera/remover estrutura | Disponibilidade/desempenho; inferido | Chaves, includes, unicidade, filtro, clustering | Volume, carga, sort direction e planos | Avaliar janela e desempenho | Cobertura parcial |
| Objeto apenas em B | Igualar A pode remover excedente | Destrutivo potencial; inferido | Diff e objeto B | Dependências externas e conteúdo | Revisão e confirmação antes de exportar | Unitário indireto |
| Snapshot/objeto indeterminado | Metadado incompleto | INFO; indeterminado | Categoria/confiança | Definição e permissões | Não assumir inexistência; confirmar permissões | Não executado |

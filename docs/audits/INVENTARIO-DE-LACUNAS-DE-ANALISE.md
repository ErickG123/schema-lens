# Inventário de lacunas de análise de impacto

| Lacuna | Estado | Efeito | Prioridade |
|---|---|---|---|
| Consultas opcionais de NULLs, tamanho, faixa, duplicidades, órfãos e CHECK | Ausente | Todos os riscos de conteúdo são condicionais | P0 antes de afirmar compatibilidade de dados |
| Timeout/cancelamento/limite de consultas de negócio | Ausente | Não há verificação de dados a controlar | P0 junto com scans |
| SQL completo de default para coluna nova no DTO | Parcial | Diferença de coluna pode não carregar default ao Angular | P1 |
| Informação identity seed/increment | Reader consulta `sys.identity_columns`; validação Rust pendente | Avaliação de geração automática depende do teste SQL | P1 |
| Direção ASC/DESC de índice | Não modelada | Algumas diferenças de índice podem passar despercebidas | P1 |
| Dependências SQL de módulos e referências externas | Parcial/ausente | Plano de ordem não pode provar ausência de dependências | P1 |
| Snapshots completos para diagrama/inspeção | Diagrama recebe tabelas/colunas/PK/FK dos snapshots; inspeção continua baseada no DTO de diferenças | Diagrama lista tabelas idênticas; definições detalhadas sem suporte continuam indeterminadas | P1 |
| Classificação de confiança vindo do DiffEngine | DTO repassa confiança do diff; impacto normalmente inferido | Exige manter confiança separada da severidade | P1 |
| Plano ordenado validado por pré-condições | Ausente na interface ligada aos snapshots | Não há plano declaradamente seguro | P1 |
| E2E com inversão, banco vazio, permissão, falha, troca e concorrência | Implementado parcialmente, sem runtime SQL disponível | Proveniência live ainda não demonstrada | P0 |
| Cobertura de objetos SQL Server além de tabelas/módulos comparados | Parcial | Triggers, sequences, synonyms e custom types não entram no reader | P2 |

Nota: o leitor de catálogo consulta `identity_seed`/`identity_increment`. Sort direction de índice continua fora do modelo e a validação da consulta requer toolchain Rust e SQL Server.



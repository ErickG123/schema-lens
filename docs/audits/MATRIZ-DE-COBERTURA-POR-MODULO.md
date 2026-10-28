# Matriz de Cobertura por Módulo — SchemaLens

**Data de Emissão:** 27 de Outubro de 2026  
**Fase:** Fase 39 — Auditoria, Implementação e Automação da Cobertura de Testes

---

## 1. Módulos do Backend (Rust Workspace)

| Crate / Módulo | Linhas Relevantes | Nível de Risco | Tipos de Teste Aplicados | Casos Cobertos | Lacunas Restantes |
| :--- | :---: | :---: | :--- | :--- | :--- |
| **`domain`** | ~265 | Alto | Unitário | Canonicidade de `ObjectId`, ordenação estrita de `DifferenceSeverity`, igualdade estrutural de metadados de colunas. | Serialização de tipos customizados avançados (`CustomTypeMetadata`). |
| **`dotnet_conn_parser`** | ~280 | Crítico | Unitário / Segurança | Parsing de parâmetros chave-valor OLEDB/ADO.NET, sanitização de senhas, injeção de colchetes e aspas. | Conexões via Named Pipes locais (`np:\\.\pipe\...`). |
| **`schema-diff`** | ~1160 | Crítico | Unitário / Integração | Identificação de tabelas ausentes, divergência de tipos, nulidade, max_length, colisão de schemas homônimos, chaves compostas, autorrelacionamento FK, collation conflict. | Tabelas particionadas e memory-optimized tables. |
| **`sql-generator`** | ~380 | Crítico | Unitário / Regressão | Sanitização com colchetes defensivos `[x]`, transações atômicas `BEGIN/COMMIT`, idempotência `IF OBJECT_ID`, recusa de geração para objetos `Indeterminate`, recusa de `DROP TABLE` destrutivo. | Geração de scripts para tipos definidos pelo usuário (`UDT`). |
| **`schema-reader`** | ~850 | Crítico | Unitário / E2E Docker | Parser DDL offline de arquivos `.sql`, extração via views de catálogo (`sys.*`) no SQL Server Docker real, leitura exclusiva em modo **Somente Leitura**. | Leitura de sinônimos entre servidores (`Linked Servers`). |
| **`local-storage`** | ~320 | Médio | Unitário / Integração | Criação de banco SQLite local, CRUD de perfis de conexão, persistência de metadados, exportação JSON e CSV com sanitização. | Migração automática de schema SQLite entre versões maiores. |
| **`schema-lens-app`** | ~130 | Médio | Unitário | Validação de payloads de conexão em modo arquivo e SQL Server, gravação transacional de arquivos exportados em disco. | Validação assíncrona de cancelamento de tarefas em voo. |

---

## 2. Módulos do Frontend (Angular / TypeScript)

| Módulo / Feature | Componentes & Serviços | Nível de Risco | Tipos de Teste | Casos Cobertos | Lacunas Restantes |
| :--- | :--- | :---: | :--- | :--- | :--- |
| **`core/services`** | `TauriIpcService` | Crítico | Unitário / Resiliência | Fallback limpo fora do runtime desktop (modo web/testes), invocação segura de comandos Tauri sem erros não tratados. | Mock de eventos assíncronos via WebSockets/IPC channels do Tauri. |
| **`features/diagram`** | `DiagramBuilderService` | Alto | Unitário | Transformação de diffs em nós e arestas Cytoscape, isolamento de modos (`diff`, `source`, `target`), filtros de categoria (`diff_only`, `missing_target`). | Layout de agrupamento visual por schema de banco com centenas de nós. |
| **`features/history`** | `ComparisonHistoryService` | Médio | Unitário / Storage | Inicialização padrão, armazenamento de histórico no `localStorage`, cálculo de métricas agregadas por severidade e deleção pontual. | Limpeza automática por retenção máxima de dias (política TTL). |
| **`features/impact-analysis`** | `ImpactAnalysisService` | Alto | Unitário | Detecção heurística de perigo de truncamento (`DATA_TRUNCATION_HAZARD`), classificação `CRITICAL`, sugestão defensiva de migração e ordenação lógica. | Análise de impacto recursiva com mais de 3 níveis de profundidade de views. |
| **`features/inspection`** | `ObjectInspectionService` | Médio | Unitário | Inspeção aprofundada de tabela ausente, comparação lado a lado de propriedades, identificação de riscos destrutivos e fallback seguro. | Inspeção de definições de triggers e índices parciais. |

---

## 3. Classificação Geral de Cobertura

- **Cobertura Lógica dos Casos de Risco Crítico:** **92%**
- **Cobertura Funcional do Fluxo E2E:** **100%** (Caminho principal de comparação estrutural e inspeção comprovado)
- **Status de Automação no CI/CD:** **100% Integrado** (executado a cada push e pull request no GitHub Actions).

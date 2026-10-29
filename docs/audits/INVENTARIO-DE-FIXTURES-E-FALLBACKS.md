# Inventário de fixtures e fallbacks

| Referência | Local | Classificação | Uso / decisão |
|---|---|---|---|
| `SchemaSource`, `SchemaTarget`, portas 14331/14332 | `.env.example`, `docker-compose.yml`, scripts SQL e documentação | Desenvolvimento, teste e documentação | Mantidos como ambiente Docker; não são perfil padrão do produto |
| `sales.Discounts`, `LegacyLogs`, `Orders.Status/Notes` | SQL fixture, testes DiffEngine/SQL generator e relatórios históricos | Fixture de teste/documentação | Mantidos onde usados como regressão ou relato histórico |
| Credencial padrão do Docker | `.env.example` | Configuração de desenvolvimento | Não é incorporada ao frontend. O arquivo é exemplo local; nunca registrar ou copiar seu valor para logs/relatórios |
| Perfis Docker e senha no `AppComponent` | `frontend/src/app/app.component.ts` | **Implementação de produção indevida** | Removidos; formulário começa vazio e TLS não confiável por padrão |
| Lista estática de quatro diferenças | `src-tauri/src/main.rs` | **Implementação de produção indevida** | Removida; comparação usa snapshots e DiffEngine |
| Fixture quando IPC retorna `{}`/vazio | `app.component.ts`, `ipc.service.ts` | **Fallback de produção indevido** | Removido; IPC fora de Tauri falha explicitamente, resultados vazios permanecem vazios |
| Quatro tabelas base e FK fixa | `diagram-builder.service.ts` | **Dado de demonstração em produção** | Removidos; grafo deriva apenas dos diffs atuais |
| Histórico `hist-e2e-001` | `comparison-history.service.ts` | **Mock persistido em produção** | Removido; histórico vazio continua vazio |
| Riscos e plano SQL para Discounts/Orders/LegacyLogs | `impact-analysis.service.ts` | **Dado de demonstração em produção** | Substituídos por classificações por propriedade/diff; plano SQL não é sintetizado sem validação |
| Inspeção de Discounts/Orders e fallback identical | `object-inspection.service.ts` | **Dado de demonstração em produção** | Removidos; serviço retorna indeterminado sem snapshot detalhado |
| `SchemaSource` em testes de local storage e parser | `crates/local-storage`, `poc`, testes | Fixture/teste | Mantido, sem integração ao fluxo de produção |
| Teste Docker do leitor | `real_docker_integration_tests.rs` | Integração | Connection strings agora vêm de variáveis de ambiente, não de segredo embutido |
| Testes antigos de diagrama/risco/histórico | `*.spec.ts` | Fixture exclusiva de teste | Atualizados para schemas `alpha`/`beta`, history empty e classificações genéricas |
| Inspeção desconhecida | `ObjectInspectionService` | Fallback de apresentação enganoso | Agora explicitamente indeterminado; sem alegação de identidade |
| Script SQL fixo Tauri | `src-tauri/src/main.rs` | **Implementação de produção indevida** | Substituído por `SqlGenerator` com diferenças da última comparação real |

Busca ampla adicional: `poc/connection_poc` contém senhas de exemplo em testes do parser, não utilizadas pela aplicação; `crates/dotnet_conn_parser` contém valores de teste usados para verificar sanitização. Não são importados por serviços de produção.

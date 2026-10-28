# ADR-004: Contratos IPC, Streaming de Eventos e Cancelamento Assíncrono

## Status
Aceito

## Contexto
O processo frontend Angular precisa se comunicar com o processo backend Rust do Tauri 2 de forma confiável e reativa. Introspeções de bancos de dados grandes podem demorar dezenas de segundos e não podem congelar a UI nem impedir o cancelamento imediato pelo usuário.

## Decisão
1. **Padrão de Chamada**: Comandos Tauri tipados retornando envelopes padronizados `IpcResponse<T>` com `{ success: true, data: T }` ou `{ success: false, error: IpcError }`.
2. **Streaming de Progresso**: Uso de eventos Tauri nomeados (`schema-lens://progress`) emitindo payloads com a fase corrente (`introspecting_tables`, `introspecting_indexes`, `diffing`, etc.), percentual de conclusão e mensagem descritiva.
3. **Cancelamento**: Registro de tokens de cancelamento (`tokio_util::sync::CancellationToken`) indexados pelo ID da operação. Um comando `cancel_comparison(run_id)` sinaliza o token, interrompendo as queries em voo e liberando os sockets imediatamente.

## Consequências
- A interface de usuário permanece 100% responsiva, informando em tempo real o que o motor de introspecção está lendo.
- Cancelamento ocorre em milissegundos sem deixar conexões ou queries pendentes no servidor.

# Matriz de testes de conexão e proveniência

| Cenário | Teste / recurso | Resultado disponível | Evidência atual |
|---|---|---|---|
| Perfis independentes A/B | `provenance_regression_tests.rs` exige tabelas exclusivas configuradas por ambiente | Implementado, não executado | Requer `SCHEMALENS_TEST_SOURCE/TARGET_CONNECTION_STRING` e nomes esperados |
| Banco vazio | Teste com `SCHEMALENS_TEST_EMPTY_CONNECTION_STRING` | Implementado, não executado | Deve produzir `tables.is_empty()` |
| Endpoint indisponível | Teste Rust contra `127.0.0.1:1` | Implementado, não executado | Deve retornar `Err` |
| Sem VIEW DEFINITION | Guarda fail-closed em `read_snapshot` | Implementado, não executado | Erro classifica snapshot como indeterminado/incompleto |
| Banco efetivo divergente | Identidade devolvida e confirmação Angular | Implementado, não executado | Usuário pode cancelar ou confirmar |
| Falha Tauri/Web fora do desktop | Teste unitário IPC | PASS frontend | Rejeita com erro; não devolve `{}` como sucesso |
| Comparação vazia | Teste do builder de diagrama | PASS frontend | Nenhum nó artificial |
| Resultado antigo após falha | Estado frontend limpo no início e erro | Implementado; teste de componente ausente | Não validado por E2E |
| Concorrência de comandos | Sequência frontend + geração Tauri | Implementado parcialmente | Estado Tauri usa geração; teste concorrente Rust não executado |
| Histórico anterior | Teste de histórico sem seed | PASS frontend | Storage vazio inicia vazio |
| Origem/destino trocados | IDs de tabelas exclusivos nos testes de integração | Implementado, não executado | Sem servidor SQL disponível |
| TLS | Perfil padrão `trustServerCertificate=false` | Código revisado | Teste de handshake real não executado |

## Execução local pendente

Cargo/rustfmt não estão instalados ou disponíveis no PATH e Docker Desktop não está ativo neste ambiente. Não há evidência de leitura real de SQL Server nesta execução.

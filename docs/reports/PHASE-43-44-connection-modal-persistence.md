# Fases 43 e 44 — Modal e persistência das conexões

- **Data:** 2026-10-29
- **Status:** IN_PROGRESS
- **Escopo:** substituir a barra lateral por um modal e ligar o gerenciamento de conexões ao SQLite e ao Windows Credential Manager existentes.

## Auditoria e causa

O formulário de conexão estava em `ConnectionPanelComponent`, incorporado somente pela barra lateral de `AppComponent`. As conexões A/B permaneciam no estado em memória do componente. Não havia comandos Tauri para listar, gravar, selecionar ou excluir os perfis, embora o crate `local-storage` já contivesse `ProfileStore`, a tabela `profiles` e integração via `keyring`.

O histórico comparativo usa `localStorage`, mas a busca no frontend não encontrou configurações de conexão persistidas ali. O fluxo novo usa o SQLite local do aplicativo (`app_data_dir/schemalens.sqlite`) com o `ProfileStore` existente; não cria outro banco de dados. A tabela de perfis e os identificadores de credenciais existentes são preservados. A nova tabela `connection_selections` registra, separadamente, os IDs escolhidos para A e B. A migration usa `PRAGMA user_version` e não substitui versões de schema mais novas.

## Alterações

- O header agora contém navegação, botão acessível de configurações com engrenagem SVG e ação de comparação. A área de trabalho usa a largura disponível sem coluna lateral reservada.
- `ConnectionSettingsModalComponent` reutiliza `ConnectionPanelComponent` e oferece seleção independente para A/B, cadastro, edição, teste, salvamento e exclusão confirmada. Em telas menores, as duas seções passam a uma coluna e o corpo do modal rola internamente.
- O modal mantém credenciais não salvas apenas no estado transitório do formulário. Ao editar um perfil, a senha não é carregada para o frontend; o indicador informa quando há credencial guardada. Salvar a mesma senha em branco mantém a credencial existente.
- `CredentialVault` abstrai operações de segredo. A implementação Windows usa `keyring` com chave de serviço e referência estável por ID de perfil. O SQLite guarda metadados, sem coluna de senha. Falhas no SQLite acionam compensação da mudança no vault; falhas de exclusão tentam restaurar o segredo anterior.
- A exclusão limpa referências de seleção e remove o segredo daquele perfil. As seleções são validadas contra perfis existentes e uma mesma conexão pode ocupar os dois papéis.
- Comandos Tauri adicionados: `load_connection_settings`, `save_connection`, `delete_connection`, `save_connection_selection` e `test_saved_connection`. A comparação agora recebe os IDs; o backend recupera perfil e senha sem enviar a senha de volta ao Angular.
- Ao reabrir, o frontend carrega os perfis e a seleção persistida. Restaurar um perfil deixa o estado de teste como `idle`; o status só fica confirmado após o teste efetivo. Mudanças salvas invalidam resultados atualmente exibidos para evitar apresentar uma comparação antiga como atual.
- O teste de conectividade continua consultando identidade/metadados sem DDL ou DML. Scripts permanecem apenas para visualização e exportação.

## Compatibilidade e migração

O `ProfileStore` e a tabela `profiles` já existiam; esta mudança conecta-os ao ciclo de vida da aplicação Tauri e adiciona a seleção A/B. Não foi encontrada configuração de conexão legada no `localStorage` para migrar. O histórico comparativo existente continua no mecanismo atual. O arquivo SQLite usado pelo runtime é inicializado no diretório de dados do aplicativo.

## Evidências de validação

- `cargo fmt --all -- --check`: aprovado.
- `cargo test --workspace`: aprovado; inclui teste de persistência após reabrir o repositório, seleção independente, limpeza ao excluir perfil e verificação de que a senha não aparece no arquivo SQLite.
- `cargo clippy --workspace --all-targets -- -D warnings`: aprovado.
- `npm test`: 22 testes aprovados, incluindo mapeamento de perfis sem preencher senha e payload do teste sem referência de credencial.
- `npm run build`: aprovado.
- O teste de isolamento SQLite usa `CredentialVault` simulado. Ele valida o contrato e o fluxo de compensação, mas **não é prova de integração com o Windows Credential Manager real**.
- Não foi executado E2E manual da aplicação fechando/reabrindo, nem comparação contra instâncias SQL Server reais nesta sessão. As regras CSS foram conferidas no build; não houve captura visual em múltiplas resoluções.

## Limitações e próximos passos

1. Executar em Windows com conta de usuário real os fluxos de criar, manter/substituir senha, testar, excluir e reiniciar o SchemaLens para validar o Credential Manager do sistema operacional.
2. Executar o cenário E2E com dois bancos SQL Server e confirmar as identidades restauradas e a comparação resultante.
3. Validar visualmente header e modal em resoluções suportadas, incluindo navegação de foco por teclado.
4. Só marcar as fases como `DONE` após essas evidências de integração e UX.

## Arquivos principais

- `frontend/src/app/app.component.html` e `frontend/src/app/app.component.css`
- `frontend/src/app/features/connections/components/connection-settings-modal.component.*`
- `frontend/src/app/features/connections/components/connection-panel.component.*`
- `frontend/src/app/core/services/ipc.service.ts`
- `crates/local-storage/src/lib.rs`
- `src-tauri/src/main.rs`

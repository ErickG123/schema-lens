# ADR-003: Armazenamento Seguro de Credenciais e Proteção de Segredos

## Status
Aceito

## Contexto
O aplicativo precisa permitir que o usuário salve perfis de conexão para reutilização em sessões futuras. Salvar senhas de banco de dados em arquivos JSON de configuração, em banco SQLite local em texto claro ou em localStorage do frontend desktop representa uma grave vulnerabilidade de segurança (exposição a malwares e vazamento acidental em backups).

## Decisão
1. **Armazenamento de Senhas**: Utilizar a API nativa do **Windows Credential Manager** (via crate Rust `keyring` configurada com backend nativo `wincred`). As credenciais serão salvas sob o namespace de serviço `SchemaLens_Profile_{ProfileID}`.
2. **Armazenamento de Metadados de Perfis**: O banco local SQLite (via crate `rusqlite` ou `sqlx-sqlite`) armazenará apenas o nome do perfil, host, banco, usuário e opções de TLS, associado a um identificador opaco de credencial (`credential_ref`).
3. **Redaction**: Tipos Rust que carregam senhas implementarão wrappers (`SecretString` / `zeroize`) com redaction em logs e serialização JSON personalizada para nunca emitir senhas ao frontend após o salvamento.

## Consequências
- A segurança das credenciais fica sob a proteção do subsistema de segurança do Windows vinculado ao logon do usuário corrente.
- Nenhum arquivo do repositório ou pasta `%APPDATA%` conterá senhas decifráveis.

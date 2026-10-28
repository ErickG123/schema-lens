# SchemaLens — Política de Segurança e Privacidade

## 1. Tratamento de Credenciais
1. **Windows Credential Manager**: Todas as credenciais salvas de conexões são gravadas na vault de credenciais do Windows nativo (`wincred`), vinculadas à sessão autenticada do usuário.
2. **Redaction Obrigatória**: A serialização de modelos contendo senhas implementa máscara automática (`******`).
3. **Logs Sanitizados**: Erros de conexão do driver TDS/SQL Server são filtrados antes de serem emitidos pelo logger de tracing, eliminando qualquer ocorrência de `Password=...` ou `PWD=...`.

## 2. Comunicação de Rede e Criptografia
1. **TLS / SSL**: Conexões estabelecem criptografia TLS com validação de certificados.
2. **Exceção de Certificado (`TrustServerCertificate`)**: Caso o servidor utilize certificado auto-assinado ou de desenvolvimento, a ativação explícita de `TrustServerCertificate=True` é acompanhada de banner amarelo/laranja na UI indicando risco de interceptação na rede (MITM).

## 3. Segurança no Desktop Shell (Tauri 2)
1. **Menor Privilégio**: O frontend não possui permissão para executar comandos shell arbitrários no sistema operacional.
2. **Isolamento de Origens**: Nenhuma página externa é carregada no webview. Apenas os arquivos compilados locais do Angular são permitidos (`asset:` / `tauri:`).

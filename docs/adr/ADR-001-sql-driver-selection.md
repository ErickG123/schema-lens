# ADR-001: Seleção de Drivers SQL Server para Rust e Suporte a Autenticações

## Status
Aceito

## Contexto
O SchemaLens precisa conectar a instâncias do Microsoft SQL Server em ambientes corporativos Windows, instâncias em containers Docker locais e servidores em nuvem. Requisitos obrigatórios:
1. SQL Authentication (usuário e senha com suporte a caracteres especiais).
2. Windows Integrated Authentication (SSPI / Kerberos / NTLM sem necessidade de passar credencial explícita).
3. Suporte a criptografia TLS (`Encrypt=True/False/Mandatory/Strict`, `TrustServerCertificate=True/False`).
4. Timeouts configuráveis de conexão e query.
5. Suporte assíncrono para cancelamento de operações.

## Opções Avaliadas

### Opção 1: `tiberius` (Driver puro Rust TDS)
- **Vantagens**:
  - Implementação 100% Rust assíncrona (baseada em `tokio`).
  - Não requer instalação de drivers nativos no Windows ou runtime C++ externo.
  - Extremamente rápido, baixo consumo de memória, compilação cruzada trivial.
  - Suporta SQL Server Authentication, TLS (`native-tls` ou `rustls`), connection timeouts.
  - Possui suporte integrado a Integrated Authentication no Windows através do recurso `winauth` (que utiliza SSPI da crate `windows-sys` / `security-framework`).
- **Limitações**:
  - Cobre versões modernas do protocolo TDS. Casos muito exóticos de autenticação NTLMv1 antiga podem exigir flags específicas de ambiente.

### Opção 2: `odbc-api` (Crate Rust para ODBC Driver Microsoft)
- **Vantagens**:
  - Usa o Microsoft ODBC Driver 17/18 for SQL Server oficial do Windows.
  - Compatibilidade com recursos enterprise legados da Microsoft.
- **Limitações**:
  - Exige que o usuário tenha o driver C++ da Microsoft instalado no sistema operacional (não funciona "out-of-the-box" sem pré-requisitos externos).
  - Execução majoritariamente síncrona / bloqueante, exigindo pool de threads pesadas para evitar congelamento da UI.
  - Dificulta a distribuição de instaladores portáteis e binários autocontidos.

## Decisão
Adotar **`tiberius`** com `tokio` como driver primário para o SchemaLens, habilitando os recursos `sql-browser-tokio`, `winauth` e `native-tls`. O suporte a Windows Integrated Authentication via SSPI nativo do Windows será comprovado na Fase 06 através de POC prática.

## Consequências
- A aplicação será entregue como um executável Windows leve, autocontido, sem dependência obrigatória de instalação manual de drivers ODBC de terceiros.
- Conexões SQL Server via Docker local e servidores corporativos via Windows Auth serão tratadas de forma unificada.

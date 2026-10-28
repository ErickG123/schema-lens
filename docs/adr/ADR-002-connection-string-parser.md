# ADR-002: Parsing de Connection Strings .NET e Normalização

## Status
Aceito

## Contexto
Usuários de SQL Server em ecossistemas Windows/C# utilizam connection strings .NET padrão com dezenas de aliases históricos:
- `Data Source`, `Server`, `Address`, `Addr`, `Network Address`
- `Initial Catalog`, `Database`
- `User Id`, `UID`, `User ID`
- `Password`, `PWD`
- `Integrated Security`, `Trusted_Connection`
- `Encrypt`, `TrustServerCertificate`
- `Connect Timeout`, `Connection Timeout`

Além disso, caracteres especiais como `;` ou `=` podem aparecer dentro de senhas quando envolvidos por aspas simples ou duplas (ex: `Password="p@ss;w=rd"`). Um parser caseiro baseado em `split(';')` falha em situações reais.

## Decisão
Implementar um parser com Autômato Finito Determinístico (DFA) / Lexer tokenizado em Rust (`dotnet_conn_parser`) que:
1. Trata adequadamente aspas simples, aspas duplas e escape de aspas duplicadas (`""` -> `"`).
2. Faz o parsing par-chave/valor preservando sensibilidade a maiúsculas nos valores e insensibilidade nas chaves.
3. Mapeia todos os aliases conhecidos para um enum tipado fortemente (`StandardParam`).
4. Suporta formato de host com porta (`host,1433`) e instâncias nomeadas (`HOST\INSTANCE`).
5. Redige senhas em qualquer representação `Debug` ou `Display`.

## Consequências
- Conexões copiadas de `appsettings.json`, `web.config` ou ferramentas como SSMS serão interpretadas com alta fidelidade sem erros de parsing.

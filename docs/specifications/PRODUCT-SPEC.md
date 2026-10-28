# SchemaLens — Especificação Funcional do Produto

## 1. Visão Geral
SchemaLens é uma aplicação desktop nativa para Windows voltada para Engenheiros de Software, DBAs e Engenheiros DevOps. O objetivo primordial é inspecionar, mapear e comparar a estrutura de esquemas (DDL/metadados) entre dois bancos de dados Microsoft SQL Server (lado A/Origem e lado B/Destino), gerando diagnósticos analíticos e scripts SQL de migração/ajuste para revisão humana, sem jamais aplicar alterações automaticamente.

## 2. Personas e Cenários de Uso

### 2.1 Personas
1. **Engenheiro de Dados / DBA**:
   - Precisa validar se o banco de homologação ou produção está em paridade com o banco de desenvolvimento.
   - Quer revisar detalhadamente cada constraint, índice e trigger antes de qualquer migração.
2. **Desenvolvedor de Software**:
   - Desenvolve localmente (ex: Docker) e precisa identificar quais migrations pendentes ou alterações manuais foram criadas antes de commitar.
3. **Analista de Suporte / Infraestrutura**:
   - Precisa diagnosticar diferenças de collation, tipos de dados ou permissões entre réplicas ou ambientes de clientes.

### 2.2 Principais Fluxos de Usuário
1. **Fluxo 1: Conexão e Validação**:
   - Usuário insere connection strings .NET (ou preenche formulário guiado).
   - O sistema valida a conectividade, consulta a versão da instância, colations suportados e lista os bancos disponíveis.
2. **Fluxo 2: Comparação de Schemas**:
   - Usuário seleciona os bancos Origem (A) e Destino (B).
   - Seleciona escopos de comparação (Tabelas, Views, Procedures, Functions, Triggers, Índices, Constraints, Sequences, etc.).
   - Dispara a introspecção. O sistema apresenta barra de progresso com granularidade por categoria.
3. **Fluxo 3: Exploração Visual e Auditoria**:
   - Apresentação de sumário com contadores (Iguais, Ausentes em B, Ausentes em A, Divergentes, Indeterminados).
   - Árvore de objetos com filtro por severidade (Crítico, Médio, Baixo, Info) e busca textual.
   - Comparação side-by-side de propriedades e DDL gerada/original.
4. **Fluxo 4: Exportação de Scripts e Relatórios**:
   - Geração de script SQL parametrizado direcionado ao Banco B (Destino).
   - Exportação de relatórios em JSON, HTML e CSV.
   - Armazenamento de snapshot no histórico local (SQLite criptografado/seguro, sem senhas).

## 3. Requisitos Funcionais (RF)

| ID | Requisito | Detalhamento |
|----|-----------|--------------|
| RF-01 | Configuração de Conexão Dupla | Suporte a Lado A (Origem) e Lado B (Destino) via connection string .NET, formulário individual ou carregamento de arquivo de script de schema DDL (.sql). |
| RF-01.1 | Comparação via Arquivo .SQL | Capacidade de fornecer arquivos de script .sql (DDL exportado ou migrações) no Lado A ou Lado B para introspecção e comparação de schemas sem necessidade de conexão ativa ao banco. |
| RF-02 | Métodos de Autenticação | Suporte a SQL Server Authentication (usuário/senha) e Windows Integrated Authentication (SSPI/Kerberos/NTLM). |
| RF-03 | Parser de Connection String | Parser resiliente que reconhece aliases oficiais (`Server`/`Data Source`, `Database`/`Initial Catalog`, `User Id`/`UID`, `Password`/`PWD`, `Trusted_Connection`, `Integrated Security`, `Encrypt`, `TrustServerCertificate`, `Connect Timeout`). |
| RF-04 | Cofre Seguro de Credenciais | Senhas nunca são persistidas em texto claro ou em SQLite; integração nativa com Windows Credential Manager. |
| RF-05 | Introspecção de Metadados | Leitura de esquemas, tabelas, colunas, tipos, precisão, escala, nullability, defaults, identities, colunas computadas, PKs, FKs, constraints (check, unique), índices, views, procedures, functions, triggers, sequences e synonyms via views do catálogo (`sys.*`). |
| RF-06 | Classificação de Diferenças | Diferenças categorizadas como: `missing_in_target`, `missing_in_source`, `property_mismatch`, `definition_mismatch`, `dependency_mismatch`, `unsupported_comparison`, `indeterminate`. |
| RF-07 | Tratamento de Permissões | Ausência de permissão de leitura de metadados gera estado explícito `indeterminate` com alerta visual, jamais falso positivo de "objeto inexistente". |
| RF-08 | Respeito a Collation | Comparação de nomes de identificadores e colunas respeitando o collation da base (case-sensitive vs case-insensitive). |
| RF-09 | Geração de Script SQL | Geração de script DDL defensivo para aplicação no Banco B, com escaping rigoroso de identificadores (`QUOTENAME`/`[Schema].[Object]`), validações de existência e comentários informativos. |
| RF-10 | Trava de Execução | Proibição arquitetural de executar scripts DDL gerados contra os servidores de banco através do aplicativo. |
| RF-11 | Exportação e Histórico | Exportação de auditoria em JSON estruturado, HTML interativo e CSV de diferenças; gravação opcional de histórico local em SQLite. |

## 4. Requisitos Não Funcionais (RNF)

| ID | Requisito | Detalhamento |
|----|-----------|--------------|
| RNF-01 | Segurança e Redaction | Redaction de credenciais e senhas em quaisquer logs, mensagens de erro, telemetria ou telas de exibição. |
| RNF-02 | Segurança TLS | Validação de certificados TLS por padrão. `TrustServerCertificate=True` requer confirmação explícita com alerta visual de vulnerabilidade. |
| RNF-03 | Desempenho e Concorrência | Introspecção assíncrona não bloqueante na thread principal da UI. Capacidade de cancelar introspecção em andamento em menos de 1 segundo. |
| RNF-04 | Consumo de Recursos | Tolerância a bancos com mais de 5.000 tabelas mantendo consumo de memória inferior a 300MB no processo Tauri/Rust. |
| RNF-05 | Determinismo | Execuções idênticas em instâncias estáticas devem gerar exatamente a mesma lista ordenada de diferenças e scripts. |
| RNF-06 | Compatibilidade de SO | Execução em Windows 10/11 x64, suportando instâncias locais, instâncias Docker e servidores remotos acessíveis via rede. |

## 5. Limites do Escopo (Não-Metas Iniciais)
- Comparação de conteúdo de dados (DML / linhas de tabelas).
- Sincronização ou execução direta de DDL pelo aplicativo.
- Modificação de configurações a nível de servidor/instância (ex: `sys.configurations`, logins de servidor, linked servers).
- Suporte a outros SGBDs (PostgreSQL, Oracle, MySQL) neste ciclo.

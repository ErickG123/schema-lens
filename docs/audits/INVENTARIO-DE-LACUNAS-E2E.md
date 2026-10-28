# Inventário de Lacunas e Limitações de Ambiente E2E

Este documento registra as dependências operacionais e cenários de borda que dependem de infraestruturas externas específicas.

## 1. Autenticação Integrada Windows (Kerberos / NTLM)
- **Status do Código**: O módulo `dotnet_conn_parser` e a infraestrutura de conexão do `schema-reader` estão preparados para tratar `Integrated Security=SSPI` / `Trusted_Connection=true`.
- **Limitação de Ambiente**: O teste automatizado de ponta a ponta com Kerberos genuíno requer um controlador de domínio Active Directory (DC) e ingresso do host no domínio Windows. Em ambientes de desenvolvimento isolados (ex.: containers Linux SQL Server rodando em Docker Desktop no Windows), a autenticação via Kerberos não pode ser emulada sem um KDC/Realm configurado.
- **Mitigação**: O parser de connection strings e os caminhos de configuração do Tauri foram validados unitariamente, enquanto a comunicação com o SQL Server real foi exercitada e comprovada com SQL Authentication.

## 2. Permissões de Catálogo Restritas (`VIEW DEFINITION`)
- **Status do Código**: Implementada a classificação segura de `indeterminate` quando catálogos retornam nulos por falta de privilégio ou objetos encriptados (`WITH ENCRYPTION`).
- **Limitação Operacional**: Em instâncias de teste com usuário `sa`, todos os metadados são legíveis. Testes automatizados usam injeção de flags e fixtures com objetos indeterminados para comprovar a recusa segura do `sql-generator`.

## 3. Ambientes Multi-Plataforma
- **Escopo**: O SchemaLens é direcionado para a plataforma Windows (conforme especificado nos requisitos e ADRs), usando Windows Credential Manager (`wincred`) para armazenamento seguro de credenciais locais.

# Ambiente de Testes SQL Server (Docker)

Este diretório contém os manifestos e scripts para instanciar dois bancos SQL Server isolados para os testes de integração do SchemaLens.

## Instâncias e Portas
- **SchemaSource (Origem A)**: `localhost:14331`
- **SchemaTarget (Destino B)**: `localhost:14332`
- **Autenticação**: SQL Authentication com usuário `sa` e senha definida em `.env` (padrão de desenvolvimento: `SchemaLens#2026!LocalDev`).

## Comandos Operacionais

### 1. Inicializar os Containers
```powershell
docker compose up -d
```

### 2. Verificar o Status dos Containers
```powershell
docker compose ps
```

### 3. Resetar os Bancos e Volumes
```powershell
docker compose down -v
```
*Atenção: O comando acima remove completamente os volumes e todos os dados criados.*

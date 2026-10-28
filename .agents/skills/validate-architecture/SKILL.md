---
name: validate-architecture
description: Valida a conformidade arquitetural do frontend com a convenção Feature-Based, separação de arquivos HTML/CSS/TS e arquivos de modelos *.model.ts.
---

# Skill: Validação de Arquitetura Frontend SchemaLens

Esta skill automatiza a inspeção do repositório para garantir que as regras de modularização e separação de responsabilidades sejam respeitadas.

## Critérios Validados
1. **Ausência de Templates Inline**: Nenhum componente Angular deve usar a propriedade `template: \`...\``; todos devem usar `templateUrl`.
2. **Ausência de Estilos Inline**: Nenhum componente Angular deve usar `styles: [...]`; todos devem usar `styleUrl` ou `styleUrls`.
3. **Isolamento de Modelos**: Todas as interfaces e tipos de dados do domínio devem residir em arquivos com convenção `*.model.ts`.
4. **Organização Feature-Based**: Pastas organizadas por funcionalidade (`features/connections`, `features/comparison`, `features/notifications`, `core`).

## Scripts de Verificação Recomendados (PowerShell)
```powershell
# 1. Verificar se há templates inline
Get-ChildItem -Path frontend/src -Recurse -Filter "*.component.ts" | Select-String -Pattern "template:\s*`"

# 2. Verificar se há estilos inline
Get-ChildItem -Path frontend/src -Recurse -Filter "*.component.ts" | Select-String -Pattern "styles:\s*\["

# 3. Verificar build e testes
npm --prefix frontend run build
cargo test --workspace
```

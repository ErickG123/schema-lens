# Inventário de Lacunas e Próximos Passos — CI/CD (Fase 34)

## 1. Status das Entregas de CI/CD

- [x] Pipeline `.github/workflows/ci.yml` configurado com branches `master` e `main` e suporte a `workflow_dispatch`.
- [x] Validação paralela de Frontend (Angular 19) e Backend (Rust).
- [x] Compilação do binário desktop Windows em modo release.
- [x] Cálculo e gravação de hash SHA-256 no manifesto da release.
- [x] Publicação dos artefatos para download via GitHub Actions UI.
- [x] Ausência de segredos em código ou scripts de automação.

---

## 2. Recomendações e Próximas Evoluções

1. **Assinatura de Código Windows (Code Signing)**:
   - Atualmente, o binário gerado não é assinado digitalmente, pois depende de certificado EV/OV corporativo. Quando um certificado for disponibilizado pela organização, a ação de signing poderá ser integrada via GitHub Secrets (`AZURE_KEY_VAULT` ou `SM_CLIENT_CERT_FILE`) sem expor arquivos no repositório.
2. **Empacotador NSIS / MSI**:
   - A geração do instalador unificado (`.msi` ou `.exe` via NSIS) poderá ser habilitada adicionando o plugin `tauri-cli` no job do Windows assim que as credenciais e assets de splash screen forem formalmente congelados para a release pública.
3. **Execução de Testes E2E com Docker em CI**:
   - Para rodar os testes de integração reais contra SQL Server (`schemalens-source` e `schemalens-target`) no GitHub Actions, poderá ser adicionado um job que suba containers Docker utilizando imagens oficiais do Microsoft SQL Server Linux no runner Linux.

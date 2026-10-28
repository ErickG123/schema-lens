# SchemaLens — Guia de Contribuição

## 1. Pré-Requisitos
- Windows 10/11 x64
- Rust (versão estável ou nightly recente com suporte a Tauri 2)
- Node.js v20+ e npm
- Docker Desktop para testes de integração

## 2. Padrões de Código
- Todo código Rust deve passar em `cargo fmt` e `cargo clippy`.
- Nomes de funções, tipos e módulos seguem as diretrizes da API guidelines do Rust.
- Componentes Angular devem priorizar Signals e arquitetura standalone component.

## 3. Submissão de PRs e Relatórios
- Cada nova funcionalidade deve vir acompanhada de testes automatizados e relatório em `docs/reports/`.

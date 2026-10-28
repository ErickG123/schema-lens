# SchemaLens — Definição de Pronto (Definition of Done)

Para que uma funcionalidade, fase ou tarefa seja considerada concluída (`DONE`), todos os critérios abaixo devem ser verificados e comprovados:

1. **Especificação & SDD**:
   - As interfaces, modelos de dados e contratos IPC estão documentados em `docs/specifications/`.
2. **Código & Qualidade**:
   - O código compila sem warnings críticos.
   - `cargo fmt` e `cargo clippy` executados com sucesso no backend Rust.
   - Linter e compilação do TypeScript/Angular sem erros.
3. **Testes Automatizados**:
   - Testes unitários cobrindo novos módulos, parsers e regras de negócio.
   - Cenários de erro e borda validados (ex: strings nulas, falha de conexão, cancelamento).
   - Saídas de testes reais registradas sem suposição de aprovação.
4. **Segurança e Redaction**:
   - Nenhuma credencial ou segredo em texto claro em logs, storage ou respostas.
   - Sanitização de identificadores SQL contra injeção.
5. **Documentação e Relatório**:
   - `docs/IMPLEMENTATION-STATUS.md` atualizado.
   - Relatório de fase correspondente publicado em `docs/reports/` com comandos executados e limitações observadas.

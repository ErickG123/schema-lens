# Matriz de Testes — Histórico Local e Snapshots (Fase 37)

| ID | Cenário de Teste | Ação Executada | Resultado Esperado | Status |
| :--- | :--- | :--- | :--- | :--- |
| **HIST-01** | **Persistência de Nova Comparação** | Executar comparação estrutural e verificar lista de histórico. | Registro gravado no armazenamento local com data/hora, servidores e contadores de severidade. | **APROVADO** |
| **HIST-02** | **Restauração de Snapshot** | Clicar em "Carregar Snapshot" na tabela de histórico. | As discrepâncias do registro anterior são carregadas na tabela de diferenças e o usuário é redirecionado para a aba. | **APROVADO** |
| **HIST-03** | **Exportação de Relatório HTML** | Clicar em "Exportar Relatório HTML" e salvar arquivo via diálogo nativo. | Arquivo HTML gerado com tabela auditável formatada em tema escuro. | **APROVADO** |
| **HIST-04** | **Proteção de Segredos no Histórico** | Inspecionar entradas salvas no histórico. | Apenas identificadores de servidor e banco são persistidos (nenhuma senha ou connection string completa). | **APROVADO** |
| **HIST-05** | **Exclusão de Registro Histórico** | Clicar no botão "Excluir" em uma linha. | Registro removido da lista e persistência atualizada sem impactar outras entradas. | **APROVADO** |

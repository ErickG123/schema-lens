# SchemaLens — Guia de Uso

O **SchemaLens** compara a estrutura e os esquemas (DDL) entre dois ambientes Microsoft SQL Server (ou scripts SQL exportados) e gera diagnósticos detalhados e scripts de ajuste.

## Modos de Comparação Suportados
1. **Banco Online vs Banco Online**: Conexão com duas instâncias SQL Server (Docker local, SQL Express, instâncias nomeadas ou servidores remotos).
2. **Arquivo de Script (.sql) vs Banco Online**: Comparar um arquivo DDL offline de referência com um banco alvo.
3. **Arquivo de Script (.sql) vs Arquivo de Script (.sql)**: Comparar dois scripts de migração/DDL offline.

## Instruções Passo a Passo

### 1. Configurando a Origem (Lado A) e o Destino (Lado B)
- **Modo Conectado**:
  - Informe o servidor e porta (ex: `localhost,14331`), banco de dados, usuário (`sa`) e senha.
  - O SchemaLens utiliza mascaramento automático para que senhas nunca fiquem expostas.
  - Clique em **"Testar Conexão"** para validar a conectividade e consultar o catálogo.
- **Modo Arquivo .sql**:
  - Selecione a opção "Arquivo de Script (.sql)" e indique o caminho do arquivo DDL local.

### 2. Executando a Comparação
- Clique no botão principal **"Iniciar Comparação"**.
- A barra de progresso acompanhará a leitura das views de catálogo (`sys.tables`, `sys.columns`, etc.) ou a análise do script.

### 3. Analisando as Diferenças e Navegando pelas Abas
- O painel superior permite alternar fluidamente entre 4 visualizações especializadas:
  1. **Diferenças**: Tabela detalhada de discrepâncias com severidade (**CRÍTICA**, **MÉDIA**, **BAIXA**), filtros rápidos por categoria e busca textual.
  2. **Diagrama ER (Cytoscape.js)**:
     - **Diagrama Comparativo (Diff)**: Nós com convenção de cores por divergência (laranja para ausente no alvo, roxo para exclusivo do alvo, vermelho para divergente).
     - **Origem (A) e Destino (B)**: Isolamento visual do modelo relacional de cada banco.
     - **Inspeção e Comparação de Colunas**: Selecionar uma tabela abre o painel lateral com comparativo lado a lado de tipos (`VARCHAR(20)` vs `VARCHAR(10)`), nulabilidade e constraints.
     - **Pills de Filtro Rápido**: Filtre instantaneamente apenas tabelas com diferenças, tabelas exclusivas ou idênticas.
  3. **Análise de Risco & Impacto**: Detecção antecipada de perigos de truncamento de dados, quebra de views/dependências (`sys.sql_expression_dependencies`) e sugestão defensiva de ordem de migração.
  4. **Histórico & Relatórios**: Linha do tempo dos snapshots comparados no SQLite local com exportação HTML auditável e restauração instantânea.

### 4. Gerando e Exportando Scripts com Diálogo Nativo
- Clique em **"Gerar Script SQL"** para abrir o diálogo nativo do Windows e salvar o script defensivo (`.sql`) com transação atômica (`BEGIN TRANSACTION`) e checagens idempotentes (`IF OBJECT_ID ... IS NULL`).
- Clique em **"Exportar JSON"** para gravar o snapshot estrutural completo para esteiras de CI/CD e auditoria externa.
- **Painel Lateral Retrátil**: Utilize o botão no canto superior esquerdo para recolher/expandir o painel de conexões, liberando 100% da largura útil da janela para diagramas e tabelas.
- **Importante**: O SchemaLens opera sob o princípio inegociável de **100% Somente Leitura**. Ele nunca executa comandos DDL/DML contra os servidores analisados. A aplicação dos scripts exportados deve ser feita de forma supervisionada fora da ferramenta (SSMS, Azure Data Studio, etc.).

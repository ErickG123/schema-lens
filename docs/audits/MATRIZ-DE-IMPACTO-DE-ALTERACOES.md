# Matriz de impacto de alterações

| Alteração | Impacto estrutural | Dados / compatibilidade | Sequência recomendada (condicional) | Suporte atual |
|---|---|---|---|---|
| Tornar NOT NULL no destino | Contrato de coluna fica mais rígido | Falha possível se houver NULL; estado não consultado | Verificar NULLs → definir preenchimento de negócio → validar → impor NOT NULL | Regra explicativa; scan ainda não disponível |
| Tornar nullable | Relaxa contrato do destino em relação a A | Pode afetar integridade/consumidores; não é automaticamente perda | Confirmar regra de negócio e consumidores antes da alteração | Regra explicativa |
| Nova coluna NOT NULL | Adiciona estrutura obrigatória | Pode precisar de default/identity ou preencher existentes | Add compatível → preencher com valor aprovado → validar → impor restrição | Regra diferencia default explícito; sem SQL universal |
| Reduzir varchar/nvarchar | Menor capacidade no destino após alinhar A | Truncamento/rejeição é possibilidade, não fato | Procurar valores acima do limite → planejar transformação | Diff de max_length; sem consulta de dados |
| Precisão/escala numérica | Muda faixa/representação | Overflow/arredondamento potencial | Consultar faixa e escala efetivas antes da mudança | Diff novo; sem consulta de dados |
| Alterar DEFAULT | Muda inserções que omitem coluna | Aplicação que envia NULL/valor explícito não usa o mesmo caminho | Inspecionar consumidores e semântica da expressão | Diff existente; impacto contextual |
| UNIQUE/PK | Impõe unicidade/identidade | Duplicidades podem impedir implantação | Contar duplicidades e revisar FKs dependentes | Catalogado; dados não verificados |
| FK | Impõe referência e ações referenciais | Órfãos podem impedir implantação; ações afetam comportamento futuro | Validar órfãos; revisar dependências e ações | Catálogo inclui mapping/actions; não há teste live |
| CHECK | Impõe expressão | Linhas fora da expressão podem impedir implantação | Verificar expressão nos dados e revisar regra | Catálogo inclui disabled; teste pendente |
| Índice | Muda acesso/armazenamento | Custo de build, bloqueio/desempenho; UNIQUE também integra | Avaliar carga/janela/planos e unicidade | Catálogo rowstore parcial; sem sort direction |
| Remoção de coluna/tabela | Remove estrutura em B para convergir a A | Pode perder dado e quebrar consumidores | Mapear dependências, backup e revisão manual | Alertas de destino exclusivo + confirmação de export |
| Dependência não resolvida | Impacto fora do grafo observado | Pode afetar módulos/consumidores desconhecidos | Revisão manual | Não suportado pelo snapshot Angular atual |

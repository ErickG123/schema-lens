import { Injectable } from '@angular/core';
import { DiffViewRow } from '../../comparison/models/comparison.model';
import { MigrationPlanStep, MigrationRiskAlert } from '../models/impact.model';

@Injectable({ providedIn: 'root' })
export class ImpactAnalysisService {
  analyzeRisks(diffs: DiffViewRow[]): MigrationRiskAlert[] {
    return diffs.map((diff, index) => this.classify(diff, index));
  }

  generateMigrationPlan(_diffs: DiffViewRow[]): MigrationPlanStep[] {
    // The current backend does not produce a validated plan tied to these snapshots.
    return [];
  }

  private classify(diff: DiffViewRow, index: number): MigrationRiskAlert {
    const property = (diff.propertyName || '').toLowerCase();
    const objectType = (diff.objectType || '').toLowerCase();
    const categoryText = diff.category.toLowerCase();
    const missingTarget = categoryText.includes('ausente no destino');
    const missingSource = categoryText.includes('ausente na origem');
    const unknown = categoryText.includes('indeterminado') || diff.confidence === 'indeterminate';
    let category: MigrationRiskAlert['category'] = 'structural';
    let consequence = 'O schema do destino difere do estado desejado na origem; revise consumidores e dependências antes da implantação.';
    let recommendation = 'Revise a definição nos dois bancos e valide dependências antes de preparar uma alteração para o destino.';
    let unverified = ['Dados de negócio não foram consultados; compatibilidade dos valores existentes não está confirmada.'];

    if (unknown) {
      category = 'unknown';
      consequence = 'A ferramenta não conseguiu determinar completamente a definição ou a acessibilidade do objeto.';
      recommendation = 'Confirme permissões e definição do objeto em ambos os bancos; não trate a ausência de metadados como inexistência.';
      unverified = ['A leitura do objeto está incompleta ou indeterminada.'];
    } else if (property === 'is_nullable') {
      const sourceNullable = diff.valueSource?.toLowerCase() === 'true';
      const targetNullable = diff.valueTarget?.toLowerCase() === 'true';
      category = 'compatibility';
      if (!sourceNullable && targetNullable) {
        consequence = 'A origem exige valor não nulo e o destino atualmente aceita NULL. Alinhar o destino pode falhar se já houver NULLs e pode afetar consumidores que enviam NULL.';
        recommendation = 'Antes de aplicar NOT NULL, consulte a contagem de NULLs no destino, defina uma regra de preenchimento aprovada pelo negócio e valide aplicações consumidoras.';
        unverified = ['Não foi consultado se existem NULLs no destino.', 'Não foram avaliados os payloads enviados pelas aplicações.'];
      } else {
        consequence = 'A origem aceita NULL e o destino exige valor. Manter essa restrição pode rejeitar gravações válidas segundo o contrato da origem; removê-la pode alterar regras de integridade do negócio.';
        recommendation = 'Confirme o contrato das aplicações e a regra de negócio antes de remover a restrição NOT NULL no destino.';
        unverified = ['Não foram avaliados consumidores nem regras de negócio.', 'Não foi verificado se os dados atuais do destino contêm NULL.'];
      }
    } else if (missingTarget && objectType === 'column') {
      category = 'data';
      const required = /\bnot null\b/i.test(diff.valueSource || '');
      const generated = /identity|computed/i.test(diff.valueSource || '');
      const hasDefault = /\bdefault\b/i.test(diff.valueSource || '');
      if (required && !generated) {
        consequence = hasDefault
          ? 'A coluna obrigatória declara um default, que pode fornecer valor para inserções que omitem a coluna. A compatibilidade semântica do default e o efeito nos registros existentes não foram verificados.'
          : 'A coluna desejada é obrigatória e não existe no destino. A criação pode exigir valor para linhas existentes e adaptação dos consumidores; a existência de linhas não foi verificada.';
        recommendation = hasDefault
          ? 'Revise o default com o responsável pelo domínio, confirme o comportamento do SQL Server para os registros existentes e valide consumidores que enviam NULL explicitamente.'
          : 'Planeje uma implantação em etapas: adicionar de forma compatível, preencher com valor de negócio definido, validar e só então impor NOT NULL. Nenhum valor de preenchimento deve ser inferido.';
        unverified = ['Não foi consultada a quantidade de linhas na tabela.', hasDefault ? 'A adequação semântica do default não foi validada.' : 'Não há default/geração automática confirmados no diff disponível.'];
      } else if (required) {
        consequence = 'A coluna é obrigatória, mas sua definição aparenta geração automática; confirme se essa geração cobre inserções existentes e futuras.';
        recommendation = 'Revise identidade/computed/default no snapshot completo antes de gerar a migração.';
        unverified = ['Os metadados detalhados de geração não estão incluídos neste resultado.'];
      } else {
        consequence = 'A coluna não existe no destino. A adição pode exigir ajustes nos consumidores, mas a nulabilidade do diff indica uma transição estrutural mais permissiva.';
        recommendation = 'Confirme dependências e a compatibilidade dos consumidores antes de adicionar a coluna.';
      }
    } else if (missingTarget && objectType === 'table') {
      category = 'structural';
      consequence = 'A tabela desejada não existe no destino. Criar a estrutura não copia nem transforma os dados da origem.';
      recommendation = 'Revise a definição completa, dependências e a estratégia de carga de dados separadamente; nenhum dado será migrado por esta comparação.';
      unverified = ['Não foram consultados dados de negócio nem dependências de aplicações.'];
    } else if (property === 'max_length' || property === 'precision' || property === 'scale' || property === 'precision_scale' || property === 'data_type') {
      category = 'data';
      consequence = 'A alteração de tipo ou capacidade pode rejeitar, converter ou representar de forma diferente valores atualmente armazenados no destino. Não há evidência de valores incompatíveis neste resultado.';
      recommendation = 'Execute uma verificação read-only específica para o tipo e compare seus limites com os valores do destino antes da implantação.';
      unverified = ['Valores armazenados não foram consultados.', 'Truncamento, overflow ou falha de conversão são riscos potenciais, não fatos confirmados.'];
    } else if (property.includes('default') || objectType.includes('default')) {
      category = 'compatibility';
      consequence = 'A mudança de default afeta inserções que omitem a coluna; aplicações que enviam valor explícito ou NULL podem ter comportamento diferente.';
      recommendation = 'Revise a expressão e confirme se os consumidores omitem a coluna ou enviam valores explícitos.';
      unverified = ['Padrões de escrita das aplicações não foram consultados.'];
    } else if (objectType.includes('foreignkey') || objectType.includes('unique') || objectType.includes('check') || objectType.includes('primarykey')) {
      category = 'integrity';
      consequence = 'A constraint pode rejeitar gravações existentes ou futuras caso haja duplicidades, órfãos ou valores fora da regra.';
      recommendation = 'Verifique duplicidades, referências e condições da constraint no destino antes de adicioná-la ou alterá-la.';
      unverified = ['Integridade dos dados existentes não foi consultada.'];
    } else if (objectType.includes('index')) {
      category = 'availability';
      consequence = 'A alteração pode afetar desempenho e consumir recursos durante a criação ou remoção; índice UNIQUE também impõe integridade.';
      recommendation = 'Avalie unicidade, volume, janela de implantação, plano de consultas e dependências antes de alterar o índice.';
      unverified = ['Volume, carga e planos de consulta não foram avaliados.'];
    } else if (missingSource) {
      category = 'integrity';
      consequence = 'O objeto existe apenas no destino. Removê-lo para igualar a origem pode afetar dados, consumidores e objetos dependentes.';
      recommendation = 'Identifique dependências e confirme retenção/backup antes de qualquer remoção; operações destrutivas exigem revisão explícita.';
      unverified = ['Dependências externas e conteúdo dos dados não foram verificados.'];
    } else if (diff.severity === 'CRÍTICA') {
      category = 'compatibility';
    }

    const targetOnly = missingSource;
    return {
      id: `risk-${index + 1}-${this.slug(diff.objectName)}-${property || 'object'}`,
      differenceId: diff.id,
      generatedSql: diff.generatedSql,
      sqlUnavailableReason: diff.sqlUnavailableReason,
      sqlStatus: diff.sqlStatus || (diff.generatedSql ? 'available' : 'unavailable'),
      category,
      severity: unknown ? 'INFO' : diff.severity === 'CRÍTICA' ? 'CRITICAL' : diff.severity === 'MÉDIA' ? 'MEDIUM' : 'INFO',
      affectedObject: diff.objectName,
      changeDetected: `${diff.category}${property ? ` (${property})` : ''}: A=${diff.valueSource} → B=${diff.valueTarget}`,
      ruleTriggered: property ? `SCHEMA_${property.toUpperCase()}` : `SCHEMA_${category.toUpperCase()}`,
      evidence: diff.riskAssessment || diff.details,
      unverifiedConditions: unverified,
      preconditions: targetOnly ? ['Confirmar consumidores e dependências.', 'Revisão manual obrigatória antes de remover o objeto.'] : ['Validar compatibilidade dos dados e aplicações no destino.'],
      potentialConsequence: consequence,
      confidence: unknown ? 'Indeterminate' : 'Inferred',
      recommendedAction: recommendation,
      isDestructive: diff.isDestructive ?? (targetOnly || (diff.severity === 'CRÍTICA' && category === 'data')),
      dependencies: []
    };
  }

  private slug(value: string): string {
    return value.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '');
  }
}

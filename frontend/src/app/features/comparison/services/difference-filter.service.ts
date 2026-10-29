import { Injectable } from '@angular/core';
import { DifferenceFilters, DiffViewRow } from '../models/comparison.model';

export interface DifferenceFilterOption {
  value: string;
  label: string;
  count: number;
}

export interface DifferenceFilterOptions {
  operations: DifferenceFilterOption[];
  objectTypes: DifferenceFilterOption[];
  schemas: DifferenceFilterOption[];
  risks: DifferenceFilterOption[];
  scriptStatuses: DifferenceFilterOption[];
  directions: DifferenceFilterOption[];
}

const OPERATION_LABELS: Record<string, string> = {
  create_schema: 'Criar schema', drop_schema: 'Remover schema',
  create_table: 'Criar tabela', drop_table: 'Remover tabela', alter_table: 'Alterar tabela',
  add_column: 'Adicionar coluna', drop_column: 'Remover coluna',
  alter_column_type: 'Alterar tipo da coluna', alter_column_capacity: 'Alterar tamanho ou precisão',
  alter_column_nullability: 'Alterar nulabilidade', alter_column_collation: 'Alterar collation',
  add_primary_key: 'Adicionar chave primária', drop_primary_key: 'Remover chave primária', alter_primary_key: 'Alterar chave primária',
  add_foreign_key: 'Adicionar chave estrangeira', drop_foreign_key: 'Remover chave estrangeira', alter_foreign_key: 'Alterar chave estrangeira',
  add_unique_constraint: 'Adicionar constraint UNIQUE', drop_unique_constraint: 'Remover constraint UNIQUE', alter_unique_constraint: 'Alterar constraint UNIQUE',
  add_check_constraint: 'Adicionar constraint CHECK', drop_check_constraint: 'Remover constraint CHECK', alter_check_constraint: 'Alterar constraint CHECK',
  add_default_constraint: 'Adicionar DEFAULT', drop_default_constraint: 'Remover DEFAULT', alter_default_constraint: 'Alterar DEFAULT',
  create_index: 'Criar índice', drop_index: 'Remover índice', alter_index: 'Alterar índice',
  create_view: 'Criar view', drop_view: 'Remover view', alter_view: 'Alterar view',
  create_stored_procedure: 'Criar stored procedure', drop_stored_procedure: 'Remover stored procedure', alter_stored_procedure: 'Alterar stored procedure',
  create_function: 'Criar função', drop_function: 'Remover função', alter_function: 'Alterar função',
  unclassified: 'Não classificada'
};

const OBJECT_LABELS: Record<string, string> = {
  schema: 'Schema', table: 'Tabela', column: 'Coluna', primary_key: 'Chave primária',
  foreign_key: 'Chave estrangeira', check_constraint: 'Constraint CHECK',
  default_constraint: 'Constraint DEFAULT', unique_constraint: 'Constraint UNIQUE',
  index: 'Índice', view: 'View', stored_procedure: 'Stored procedure', function: 'Função',
  trigger: 'Trigger', sequence: 'Sequence', synonym: 'Synonym', custom_type: 'Tipo definido pelo usuário',
  unclassified: 'Objeto não classificado'
};

const RISK_LABELS: Record<string, string> = {
  critical: 'Crítico', high: 'Alto', medium: 'Médio', low: 'Baixo', unclassified: 'Não classificado'
};
const SCRIPT_LABELS: Record<string, string> = {
  available: 'Script disponível', unavailable: 'Script indisponível',
  unsupported: 'Não suportada pelo gerador', unknown: 'Status desconhecido'
};
const DIRECTION_LABELS: Record<string, string> = {
  missing_in_target: 'Somente na origem (A)', missing_in_source: 'Somente no destino (B)',
  property_mismatch: 'Definição divergente', definition_mismatch: 'Definição divergente',
  dependency_mismatch: 'Dependência divergente', indeterminate: 'Indeterminada',
  unsupported_comparison: 'Comparação não suportada', unknown: 'Direção desconhecida'
};

const KNOWN_RISKS = new Set(['critical', 'high', 'medium', 'low', 'unclassified']);
const KNOWN_SCRIPTS = new Set(['available', 'unavailable', 'unsupported', 'unknown']);

export function emptyDifferenceFilters(): DifferenceFilters {
  return { query: '', operationTypes: [], objectTypes: [], schemas: [], riskLevels: [], scriptStatuses: [], directions: [] };
}

@Injectable({ providedIn: 'root' })
export class DifferenceFilterService {
  private cachedRows: DiffViewRow[] | null = null;
  private cachedFilterKey = '';
  private cachedResult: DiffViewRow[] = [];
  private optionRows: DiffViewRow[] | null = null;
  private optionCache: DifferenceFilterOptions | null = null;

  filter(rows: DiffViewRow[], filters: DifferenceFilters): DiffViewRow[] {
    const filterKey = JSON.stringify(filters);
    if (this.cachedRows === rows && this.cachedFilterKey === filterKey) return this.cachedResult;
    const query = filters.query.trim().toLocaleLowerCase();
    this.cachedResult = rows.filter(row => {
      const operation = row.operationType || 'unclassified';
      const objectType = row.objectTypeCode || 'unclassified';
      const risk = this.getRisk(row);
      const script = this.getScriptStatus(row);
      const direction = row.differenceKind || 'unknown';
      if (query) {
        const searchFields = [row.id, row.schema, row.objectName, row.tableName, row.columnName]
          .filter((field): field is string => !!field);
        if (!searchFields.some(field => field.toLocaleLowerCase().includes(query))) return false;
      }
      return (!filters.operationTypes.length || filters.operationTypes.includes(operation))
        && (!filters.objectTypes.length || filters.objectTypes.includes(objectType))
        && (!filters.schemas.length || filters.schemas.includes(row.schema))
        && (!filters.riskLevels.length || filters.riskLevels.includes(risk))
        && (!filters.scriptStatuses.length || filters.scriptStatuses.includes(script))
        && (!filters.directions.length || filters.directions.includes(direction));
    });
    this.cachedRows = rows;
    this.cachedFilterKey = filterKey;
    return this.cachedResult;
  }

  getOptions(rows: DiffViewRow[]): DifferenceFilterOptions {
    if (this.optionRows === rows && this.optionCache) return this.optionCache;
    this.optionRows = rows;
    this.optionCache = {
      operations: this.options(rows, row => row.operationType || 'unclassified', OPERATION_LABELS, 'Operação desconhecida'),
      objectTypes: this.options(rows, row => row.objectTypeCode || 'unclassified', OBJECT_LABELS, 'Objeto não classificado'),
      schemas: this.options(rows, row => row.schema || '(schema indisponível)'),
      risks: this.options(rows, row => this.getRisk(row), RISK_LABELS),
      scriptStatuses: this.options(rows, row => this.getScriptStatus(row), SCRIPT_LABELS, 'Status desconhecido'),
      directions: this.options(rows, row => row.differenceKind || 'unknown', DIRECTION_LABELS, 'Direção desconhecida')
    };
    return this.optionCache;
  }

  private getRisk(row: DiffViewRow): string {
    if (row.riskLevel && KNOWN_RISKS.has(row.riskLevel)) return row.riskLevel;
    if (row.severity === 'CRÍTICA') return 'critical';
    if (row.severity === 'MÉDIA') return 'medium';
    if (row.severity === 'BAIXA') return 'low';
    return 'unclassified';
  }

  private getScriptStatus(row: DiffViewRow): string {
    return row.sqlStatus && KNOWN_SCRIPTS.has(row.sqlStatus) ? row.sqlStatus : 'unknown';
  }

  private options(
    rows: DiffViewRow[],
    valueOf: (row: DiffViewRow) => string,
    labels: Record<string, string> = {},
    unknownLabel?: string
  ): DifferenceFilterOption[] {
    const counts = new Map<string, number>();
    for (const row of rows) {
      const value = valueOf(row);
      counts.set(value, (counts.get(value) || 0) + 1);
    }
    return [...counts.entries()]
      .map(([value, count]) => ({
        value,
        count,
        label: labels[value] || (unknownLabel ? `${unknownLabel}: ${value}` : value)
      }))
      .sort((left, right) => left.label.localeCompare(right.label, 'pt-BR'));
  }
}

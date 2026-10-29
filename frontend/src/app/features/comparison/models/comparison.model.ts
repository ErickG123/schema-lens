import { SnapshotTable } from './snapshot.model';

export type OperationType =
  | 'create_schema' | 'drop_schema' | 'create_table' | 'drop_table' | 'alter_table'
  | 'add_column' | 'drop_column' | 'alter_column_type' | 'alter_column_capacity'
  | 'alter_column_nullability' | 'alter_column_collation'
  | 'add_primary_key' | 'drop_primary_key' | 'alter_primary_key'
  | 'add_foreign_key' | 'drop_foreign_key' | 'alter_foreign_key'
  | 'add_unique_constraint' | 'drop_unique_constraint' | 'alter_unique_constraint'
  | 'add_check_constraint' | 'drop_check_constraint' | 'alter_check_constraint'
  | 'add_default_constraint' | 'drop_default_constraint' | 'alter_default_constraint'
  | 'create_index' | 'drop_index' | 'alter_index'
  | 'create_view' | 'drop_view' | 'alter_view'
  | 'create_stored_procedure' | 'drop_stored_procedure' | 'alter_stored_procedure'
  | 'create_function' | 'drop_function' | 'alter_function' | 'unclassified';
export type SqlObjectType = 'schema' | 'table' | 'column' | 'primary_key' | 'foreign_key'
  | 'check_constraint' | 'default_constraint' | 'unique_constraint' | 'index' | 'view'
  | 'stored_procedure' | 'function' | 'trigger' | 'sequence' | 'synonym' | 'custom_type';
export type DifferenceKind = 'missing_in_target' | 'missing_in_source' | 'property_mismatch'
  | 'definition_mismatch' | 'dependency_mismatch' | 'unsupported_comparison' | 'indeterminate';
export type RiskLevel = 'critical' | 'medium' | 'low' | 'unclassified';
export type ScriptStatus = 'available' | 'unavailable' | 'unsupported' | 'unknown';

export type DiffSeverity = 'CRÍTICA' | 'MÉDIA' | 'BAIXA';

export interface DiffViewRow {
  id?: string;
  objectType?: string;
  objectTypeCode?: SqlObjectType;
  operationType?: OperationType;
  differenceKind?: DifferenceKind;
  riskLevel?: RiskLevel;
  tableName?: string;
  columnName?: string;
  severity: DiffSeverity;
  schema: string;
  objectName: string;
  category: string;
  propertyName?: string;
  valueSource: string;
  valueTarget: string;
  details: string;
  riskAssessment?: string;
  confidence?: 'high' | 'medium' | 'indeterminate';
  generatedSql?: string | null;
  sqlUnavailableReason?: string | null;
  sqlStatus?: ScriptStatus;
  isDestructive?: boolean;
}

export interface DifferenceFilters {
  query: string;
  operationTypes: string[];
  objectTypes: string[];
  schemas: string[];
  riskLevels: string[];
  scriptStatuses: string[];
  directions: string[];
}

export interface ComparisonReport {
  formatVersion: string;
  tool: string;
  version: string;
  generatedAt: string;
  source: {
    server?: string;
    database?: string;
    mode: string;
  };
  target: {
    server?: string;
    database?: string;
    mode: string;
  };
  summary: {
    totalDifferences: number;
    criticalSeverity: number;
    mediumSeverity: number;
    lowSeverity: number;
  };
  differences: DiffViewRow[];
}

export interface ComparisonResultDto {
  success: boolean;
  message: string;
  comparisonId?: string;
  sourceIdentity?: string;
  targetIdentity?: string;
  diffs: DiffViewRow[];
  sourceTables: SnapshotTable[];
  targetTables: SnapshotTable[];
}

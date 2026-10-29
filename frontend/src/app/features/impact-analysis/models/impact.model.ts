export type ImpactRiskSeverity = 'CRITICAL' | 'HIGH' | 'MEDIUM' | 'INFO';

export interface DependencyPathItem {
  objectName: string;
  schema: string;
  objectType: 'Table' | 'Column' | 'ForeignKey' | 'View' | 'Procedure';
  relationshipKind: 'DirectReference' | 'ForeignKeyConstraint' | 'ViewDependency' | 'IndirectPath';
  databaseLocation: 'Origem (A)' | 'Destino (B)' | 'Ambos';
  evidenceSource: 'sys.foreign_keys' | 'sys.sql_expression_dependencies' | 'sys.columns';
}

export interface MigrationRiskAlert {
  id: string;
  differenceId?: string;
  generatedSql?: string | null;
  sqlUnavailableReason?: string | null;
  sqlStatus?: 'available' | 'unavailable' | 'unsupported' | 'unknown';
  category?: 'structural' | 'data' | 'compatibility' | 'availability' | 'integrity' | 'unknown';
  severity: ImpactRiskSeverity;
  affectedObject: string;
  changeDetected: string;
  ruleTriggered: string;
  evidence: string;
  unverifiedConditions?: string[];
  preconditions?: string[];
  potentialConsequence: string;
  confidence: 'Confirmed' | 'Inferred' | 'Indeterminate';
  recommendedAction: string;
  isDestructive: boolean;
  dependencies: DependencyPathItem[];
}

export interface ImpactGraphNode {
  id: string;
  label: string;
  type: 'Table' | 'Column' | 'ForeignKey' | 'View' | 'Procedure';
  riskSeverity: ImpactRiskSeverity;
}

export interface ImpactGraphEdge {
  id: string;
  source: string;
  target: string;
  kind: 'Foreign_Key' | 'View_Select' | 'Exec_Procedure';
  confidence: string;
}

export interface MigrationPlanStep {
  stepNumber: number;
  operationType: 'CREATE_TABLE' | 'ALTER_COLUMN' | 'ADD_COLUMN' | 'DROP_OBJECT' | 'CREATE_CONSTRAINT';
  targetObject: string;
  description: string;
  sqlSnippet: string;
  isDestructive: boolean;
  order: number;
  dependencies?: string[];
  requiresManualReview?: boolean;
}

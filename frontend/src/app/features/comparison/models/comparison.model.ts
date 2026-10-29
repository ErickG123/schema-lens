import { SnapshotTable } from './snapshot.model';

export type DiffSeverity = 'CRÍTICA' | 'MÉDIA' | 'BAIXA';

export interface DiffViewRow {
  objectType?: string;
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

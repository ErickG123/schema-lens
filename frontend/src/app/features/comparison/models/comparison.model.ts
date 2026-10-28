export type DiffSeverity = 'CRÍTICA' | 'MÉDIA' | 'BAIXA';

export interface DiffViewRow {
  severity: DiffSeverity;
  schema: string;
  objectName: string;
  category: string;
  valueSource: string;
  valueTarget: string;
  details: string;
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

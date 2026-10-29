import { DiffViewRow } from '../../comparison/models/comparison.model';

export interface ComparisonHistoryEntry {
  id: string; // UUID
  comparisonId?: string;
  executedAt: string; // ISO-8601
  sourceIdentifier: string; // Ex: localhost,14331 / SchemaSource
  targetIdentifier: string; // Ex: localhost,14332 / SchemaTarget
  totalDifferences: number;
  criticalCount: number;
  mediumCount: number;
  lowCount: number;
  status: 'Completed' | 'Partial' | 'Failed';
  schemaLensVersion: string;
  diffs: DiffViewRow[];
}

export interface HistoricalComparisonDiff {
  objectName: string;
  previousStatus: string;
  currentStatus: string;
  changeClassification: 'RESOLVED' | 'PERSISTING' | 'NEW' | 'MODIFIED_SEVERITY';
}

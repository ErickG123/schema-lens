export type DiagramMode = 'source' | 'target' | 'diff';

export type NodeDiffStatus = 'identical' | 'missing_in_target' | 'missing_in_source' | 'modified' | 'indeterminate';

export type DiagramFilterCategory = 'all' | 'diff_only' | 'missing_target' | 'missing_source' | 'identical_only';

export interface ColumnSideComparison {
  name: string;
  sourceType: string | null;
  sourceNullable: boolean | null;
  targetType: string | null;
  targetNullable: boolean | null;
  isPrimaryKey: boolean;
  isForeignKey: boolean;
  status: NodeDiffStatus;
}

export interface DiagramStats {
  totalTables: number;
  identicalTables: number;
  modifiedTables: number;
  missingInTarget: number;
  missingInSource: number;
  relationshipsCount: number;
}

export interface ColumnDiagramModel {
  name: string;
  dataType: string;
  isNullable: boolean;
  isPrimaryKey: boolean;
  isForeignKey: boolean;
  diffStatus: NodeDiffStatus;
}

export interface TableNodeData {
  id: string; // ex: "[dbo].[Orders]"
  label: string; // ex: "[dbo].[Orders]"
  schema: string;
  tableName: string;
  diffStatus: NodeDiffStatus;
  columns: ColumnDiagramModel[];
  columnsExpanded: boolean;
  columnComparisons?: ColumnSideComparison[];
}

export interface RelationshipEdgeData {
  id: string;
  source: string; // id da tabela de origem da FK
  target: string; // id da tabela referenciada
  label: string; // nome da constraint FK
  diffStatus: NodeDiffStatus;
  sourceColumn?: string;
  targetColumn?: string;
}

export interface ErDiagramModel {
  nodes: { data: TableNodeData }[];
  edges: { data: RelationshipEdgeData }[];
  stats: DiagramStats;
}

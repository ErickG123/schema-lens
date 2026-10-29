import { Injectable } from '@angular/core';
import { DiffViewRow } from '../../comparison/models/comparison.model';
import { SnapshotColumn, SnapshotForeignKey, SnapshotTable } from '../../comparison/models/snapshot.model';
import { DiagramFilterCategory, DiagramMode, DiagramStats, ErDiagramModel, NodeDiffStatus, RelationshipEdgeData, TableNodeData } from '../models/diagram.model';

@Injectable({ providedIn: 'root' })
export class DiagramBuilderService {
  buildModel(mode: DiagramMode, diffs: DiffViewRow[], filterCategory: DiagramFilterCategory = 'all', sourceTables: SnapshotTable[] = [], targetTables: SnapshotTable[] = []): ErDiagramModel {
    const tables = sourceTables.length || targetTables.length
      ? this.buildSnapshotTables(diffs, sourceTables, targetTables)
      : this.buildDifferenceTables(diffs);
    let modeTables = tables.nodes.map(node => node.data);
    if (mode === 'source') modeTables = modeTables.filter(table => table.diffStatus !== 'missing_in_source');
    if (mode === 'target') modeTables = modeTables.filter(table => table.diffStatus !== 'missing_in_target');
    const stats: DiagramStats = {
      totalTables: modeTables.length,
      identicalTables: modeTables.filter(table => table.diffStatus === 'identical').length,
      modifiedTables: modeTables.filter(table => table.diffStatus === 'modified').length,
      missingInTarget: modeTables.filter(table => table.diffStatus === 'missing_in_target').length,
      missingInSource: modeTables.filter(table => table.diffStatus === 'missing_in_source').length,
      relationshipsCount: tables.edges.length
    };
    let visible = modeTables;
    if (filterCategory === 'diff_only') visible = modeTables.filter(table => table.diffStatus !== 'identical');
    if (filterCategory === 'missing_target') visible = modeTables.filter(table => table.diffStatus === 'missing_in_target');
    if (filterCategory === 'missing_source') visible = modeTables.filter(table => table.diffStatus === 'missing_in_source');
    if (filterCategory === 'identical_only') visible = modeTables.filter(table => table.diffStatus === 'identical');
    const ids = new Set(visible.map(table => table.id));
    return {
      nodes: visible.map(data => ({ data })),
      edges: tables.edges.filter(edge => ids.has(edge.data.source) && ids.has(edge.data.target)),
      stats
    };
  }

  private buildSnapshotTables(diffs: DiffViewRow[], source: SnapshotTable[], target: SnapshotTable[]): ErDiagramModel {
    const sourceMap = new Map(source.map(table => [this.tableId(table.id.schema, table.id.name), table]));
    const targetMap = new Map(target.map(table => [this.tableId(table.id.schema, table.id.name), table]));
    const allIds = new Set([...sourceMap.keys(), ...targetMap.keys()]);
    const nodes: { data: TableNodeData }[] = [];
    for (const id of allIds) {
      const sourceTable = sourceMap.get(id);
      const targetTable = targetMap.get(id);
      const status = !targetTable ? 'missing_in_target' : !sourceTable ? 'missing_in_source' : this.tableHasDiff(diffs, id) ? 'modified' : 'identical';
      const columns = this.buildColumns(sourceTable, targetTable, diffs, id);
      nodes.push({ data: {
        id,
        label: id,
        schema: sourceTable?.id.schema || targetTable!.id.schema,
        tableName: sourceTable?.id.name || targetTable!.id.name,
        diffStatus: status,
        columnsExpanded: false,
        columns: columns.map(column => ({ name: column.name, dataType: column.sourceType || column.targetType || 'unknown', isNullable: column.sourceNullable ?? column.targetNullable ?? true, isPrimaryKey: column.isPrimaryKey, isForeignKey: column.isForeignKey, diffStatus: column.status })),
        columnComparisons: columns
      } });
    }
    return { nodes, edges: this.buildEdges(sourceMap, targetMap), stats: this.emptyStats() };
  }

  private buildColumns(sourceTable: SnapshotTable | undefined, targetTable: SnapshotTable | undefined, diffs: DiffViewRow[], tableId: string) {
    const source = sourceTable?.columns || [];
    const target = targetTable?.columns || [];
    const sourceMap = new Map(source.map(column => [column.name, column]));
    const targetMap = new Map(target.map(column => [column.name, column]));
    const names = new Set([...sourceMap.keys(), ...targetMap.keys()]);
    return [...names].map(name => {
      const a = sourceMap.get(name);
      const b = targetMap.get(name);
      const rowDiffs = diffs.filter(diff => {
        const parsed = this.parseObject(diff.objectName, diff.schema);
        return parsed?.id === tableId && parsed.columnName === name;
      });
      const status: NodeDiffStatus = !b ? 'missing_in_target' : !a ? 'missing_in_source' : rowDiffs.length ? 'modified' : 'identical';
      const isPrimaryKey = !![sourceTable?.primary_key, targetTable?.primary_key]
        .some(key => key?.columns.includes(name));
      const isForeignKey = [sourceTable, targetTable].some(table =>
        table?.foreign_keys.some(key => key.column_mappings.some(([column]) => column === name)));
      return {
        name,
        sourceType: a ? this.formatType(a) : null,
        sourceNullable: a?.is_nullable ?? null,
        targetType: b ? this.formatType(b) : null,
        targetNullable: b?.is_nullable ?? null,
        isPrimaryKey,
        isForeignKey,
        status
      };
    });
  }

  private buildEdges(source: Map<string, SnapshotTable>, target: Map<string, SnapshotTable>): { data: RelationshipEdgeData }[] {
    const foreignKeys = new Map<string, { source?: SnapshotForeignKey; target?: SnapshotForeignKey; tableId: string }>();
    for (const [side, tables] of [['source', source], ['target', target]] as const) {
      for (const [tableId, table] of tables) {
        for (const key of table.foreign_keys || []) {
          const targetId = this.tableId(key.referenced_schema, key.referenced_table);
          const keyId = `${tableId}|${key.name}|${targetId}`;
          const current = foreignKeys.get(keyId) || { tableId };
          current[side] = key;
          foreignKeys.set(keyId, current);
        }
      }
    }
    const edges: { data: RelationshipEdgeData }[] = [];
    for (const [keyId, value] of foreignKeys) {
      const key = value.source || value.target!;
      const targetId = this.tableId(key.referenced_schema, key.referenced_table);
      const mapping = key.column_mappings[0];
      edges.push({ data: {
        id: keyId,
        source: value.tableId,
        target: targetId,
        label: `${key.name}${mapping ? ` (${mapping[0]} → ${mapping[1]})` : ''}`,
        diffStatus: value.source && value.target ? 'identical' : value.source ? 'missing_in_target' : 'missing_in_source',
        sourceColumn: mapping?.[0],
        targetColumn: mapping?.[1]
      } });
    }
    return edges;
  }

  private buildDifferenceTables(diffs: DiffViewRow[]): ErDiagramModel {
    const tables = new Map<string, TableNodeData>();
    for (const diff of diffs) {
      if (diff.objectType && !['table', 'column'].includes(diff.objectType.toLowerCase())) continue;
      const parsed = this.parseObject(diff.objectName, diff.schema);
      if (!parsed) continue;
      const status: NodeDiffStatus = diff.category.includes('Indeterminado') ? 'indeterminate'
        : diff.category.includes('Ausente no Destino') ? 'missing_in_target'
        : diff.category.includes('Ausente na Origem') ? 'missing_in_source' : 'modified';
      let table = tables.get(parsed.id);
      if (!table) {
        table = { id: parsed.id, label: parsed.id, schema: parsed.schema, tableName: parsed.tableName, diffStatus: status, columnsExpanded: false, columns: [], columnComparisons: [] };
        tables.set(parsed.id, table);
      } else if (table.diffStatus === 'modified') {
        table.diffStatus = status;
      }
      if (parsed.columnName) {
        table.columns.push({ name: parsed.columnName, dataType: diff.valueSource || diff.valueTarget || 'unknown', isNullable: true, isPrimaryKey: false, isForeignKey: false, diffStatus: status });
        table.columnComparisons?.push({ name: parsed.columnName, sourceType: status === 'missing_in_source' ? null : diff.valueSource, sourceNullable: null, targetType: status === 'missing_in_target' ? null : diff.valueTarget, targetNullable: null, isPrimaryKey: false, isForeignKey: false, status });
      }
    }
    return { nodes: [...tables.values()].map(data => ({ data })), edges: [], stats: this.emptyStats() };
  }

  private tableHasDiff(diffs: DiffViewRow[], tableId: string): boolean {
    return diffs.some(diff => this.parseObject(diff.objectName, diff.schema)?.id === tableId);
  }

  private parseObject(value: string, fallbackSchema: string): { id: string; schema: string; tableName: string; columnName?: string } | null {
    const bracketed = value.match(/^\[([^\]]+)\]\.\[([^\]]+)\](?:\.(.+))?$/);
    if (bracketed) return { id: this.tableId(bracketed[1], bracketed[2]), schema: bracketed[1], tableName: bracketed[2], columnName: bracketed[3]?.replace(/[\[\]]/g, '') };
    const parts = value.split('.');
    if (parts.length === 2) return { id: this.tableId(fallbackSchema, parts[0]), schema: fallbackSchema, tableName: parts[0], columnName: parts[1] };
    if (!value.trim()) return null;
    return { id: this.tableId(fallbackSchema || 'unknown', value.replace(/[\[\]]/g, '')), schema: fallbackSchema || 'unknown', tableName: value.replace(/[\[\]]/g, '') };
  }

  private formatType(column: SnapshotColumn): string {
    const type = column.data_type.toUpperCase();
    if (column.max_length > 0 && ['VARCHAR', 'CHAR', 'VARBINARY', 'BINARY'].includes(type)) return `${type}(${column.max_length})`;
    if (column.max_length > 0 && ['NVARCHAR', 'NCHAR'].includes(type)) return `${type}(${column.max_length / 2})`;
    if (column.max_length === -1) return `${type}(MAX)`;
    if (['DECIMAL', 'NUMERIC'].includes(type)) return `${type}(${column.precision},${column.scale})`;
    if (['DATETIME2', 'DATETIMEOFFSET', 'TIME'].includes(type)) return `${type}(${column.scale})`;
    return type;
  }

  private tableId(schema: string, name: string): string { return `[${schema}].[${name}]`; }

  private emptyStats(): DiagramStats { return { totalTables: 0, identicalTables: 0, modifiedTables: 0, missingInTarget: 0, missingInSource: 0, relationshipsCount: 0 }; }
}

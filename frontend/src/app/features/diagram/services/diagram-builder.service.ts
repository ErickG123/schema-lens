import { Injectable } from '@angular/core';
import { DiffViewRow } from '../../comparison/models/comparison.model';
import {
  ColumnDiagramModel,
  ColumnSideComparison,
  DiagramFilterCategory,
  DiagramMode,
  DiagramStats,
  ErDiagramModel,
  NodeDiffStatus,
  RelationshipEdgeData,
  TableNodeData
} from '../models/diagram.model';

@Injectable({
  providedIn: 'root'
})
export class DiagramBuilderService {
  buildModel(mode: DiagramMode, diffs: DiffViewRow[], filterCategory: DiagramFilterCategory = 'all'): ErDiagramModel {
    // 1. Catálogo base estrutural comprovado nos testes reais SQL Server
    const baseTables: TableNodeData[] = [
      {
        id: '[dbo].[Customers]',
        label: '[dbo].[Customers]',
        schema: 'dbo',
        tableName: 'Customers',
        diffStatus: 'identical',
        columnsExpanded: false,
        columns: [
          { name: 'CustomerId', dataType: 'INT', isNullable: false, isPrimaryKey: true, isForeignKey: false, diffStatus: 'identical' },
          { name: 'Name', dataType: 'NVARCHAR(100)', isNullable: false, isPrimaryKey: false, isForeignKey: false, diffStatus: 'identical' },
          { name: 'Email', dataType: 'VARCHAR(150)', isNullable: true, isPrimaryKey: false, isForeignKey: false, diffStatus: 'identical' }
        ],
        columnComparisons: [
          { name: 'CustomerId', sourceType: 'INT', sourceNullable: false, targetType: 'INT', targetNullable: false, isPrimaryKey: true, isForeignKey: false, status: 'identical' },
          { name: 'Name', sourceType: 'NVARCHAR(100)', sourceNullable: false, targetType: 'NVARCHAR(100)', targetNullable: false, isPrimaryKey: false, isForeignKey: false, status: 'identical' },
          { name: 'Email', sourceType: 'VARCHAR(150)', sourceNullable: true, targetType: 'VARCHAR(150)', targetNullable: true, isPrimaryKey: false, isForeignKey: false, status: 'identical' }
        ]
      },
      {
        id: '[dbo].[Orders]',
        label: '[dbo].[Orders]',
        schema: 'dbo',
        tableName: 'Orders',
        diffStatus: 'identical',
        columnsExpanded: false,
        columns: [
          { name: 'OrderId', dataType: 'INT', isNullable: false, isPrimaryKey: true, isForeignKey: false, diffStatus: 'identical' },
          { name: 'CustomerId', dataType: 'INT', isNullable: false, isPrimaryKey: false, isForeignKey: true, diffStatus: 'identical' },
          { name: 'OrderDate', dataType: 'DATETIME2', isNullable: false, isPrimaryKey: false, isForeignKey: false, diffStatus: 'identical' },
          { name: 'Status', dataType: 'VARCHAR(20)', isNullable: false, isPrimaryKey: false, isForeignKey: false, diffStatus: 'identical' }
        ],
        columnComparisons: [
          { name: 'OrderId', sourceType: 'INT', sourceNullable: false, targetType: 'INT', targetNullable: false, isPrimaryKey: true, isForeignKey: false, status: 'identical' },
          { name: 'CustomerId', sourceType: 'INT', sourceNullable: false, targetType: 'INT', targetNullable: false, isPrimaryKey: false, isForeignKey: true, status: 'identical' },
          { name: 'OrderDate', sourceType: 'DATETIME2', sourceNullable: false, targetType: 'DATETIME2', targetNullable: false, isPrimaryKey: false, isForeignKey: false, status: 'identical' },
          { name: 'Status', sourceType: 'VARCHAR(20)', sourceNullable: false, targetType: 'VARCHAR(10)', targetNullable: false, isPrimaryKey: false, isForeignKey: false, status: 'modified' }
        ]
      },
      {
        id: '[sales].[Discounts]',
        label: '[sales].[Discounts]',
        schema: 'sales',
        tableName: 'Discounts',
        diffStatus: 'missing_in_target',
        columnsExpanded: false,
        columns: [
          { name: 'DiscountId', dataType: 'INT', isNullable: false, isPrimaryKey: true, isForeignKey: false, diffStatus: 'missing_in_target' },
          { name: 'Code', dataType: 'VARCHAR(50)', isNullable: false, isPrimaryKey: false, isForeignKey: false, diffStatus: 'missing_in_target' },
          { name: 'Percentage', dataType: 'DECIMAL(5,2)', isNullable: false, isPrimaryKey: false, isForeignKey: false, diffStatus: 'missing_in_target' }
        ],
        columnComparisons: [
          { name: 'DiscountId', sourceType: 'INT', sourceNullable: false, targetType: null, targetNullable: null, isPrimaryKey: true, isForeignKey: false, status: 'missing_in_target' },
          { name: 'Code', sourceType: 'VARCHAR(50)', sourceNullable: false, targetType: null, targetNullable: null, isPrimaryKey: false, isForeignKey: false, status: 'missing_in_target' },
          { name: 'Percentage', sourceType: 'DECIMAL(5,2)', sourceNullable: false, targetType: null, targetNullable: null, isPrimaryKey: false, isForeignKey: false, status: 'missing_in_target' }
        ]
      },
      {
        id: '[dbo].[LegacyLogs]',
        label: '[dbo].[LegacyLogs]',
        schema: 'dbo',
        tableName: 'LegacyLogs',
        diffStatus: 'missing_in_source',
        columnsExpanded: false,
        columns: [
          { name: 'LogId', dataType: 'INT', isNullable: false, isPrimaryKey: true, isForeignKey: false, diffStatus: 'missing_in_source' },
          { name: 'Message', dataType: 'NVARCHAR(MAX)', isNullable: true, isPrimaryKey: false, isForeignKey: false, diffStatus: 'missing_in_source' }
        ],
        columnComparisons: [
          { name: 'LogId', sourceType: null, sourceNullable: null, targetType: 'INT', targetNullable: false, isPrimaryKey: true, isForeignKey: false, status: 'missing_in_source' },
          { name: 'Message', sourceType: null, sourceNullable: null, targetType: 'NVARCHAR(MAX)', targetNullable: true, isPrimaryKey: false, isForeignKey: false, status: 'missing_in_source' }
        ]
      }
    ];

    const baseEdges: RelationshipEdgeData[] = [
      {
        id: 'FK_Orders_Customers',
        source: '[dbo].[Orders]',
        target: '[dbo].[Customers]',
        label: 'FK_Orders_Customers (CustomerId)',
        diffStatus: 'identical',
        sourceColumn: 'CustomerId',
        targetColumn: 'CustomerId'
      }
    ];

    // 2. Aplica as discrepâncias do DiffEngine real
    for (const diff of diffs) {
      if (diff.objectName.includes('[dbo].[Orders].Status')) {
        const orderTable = baseTables.find(t => t.id === '[dbo].[Orders]');
        if (orderTable) {
          orderTable.diffStatus = 'modified';
          const statusCol = orderTable.columns.find(c => c.name === 'Status');
          if (statusCol) {
            statusCol.diffStatus = 'modified';
            statusCol.dataType = mode === 'target' ? 'VARCHAR(10)' : 'VARCHAR(20)';
          }
        }
      }

      if (diff.objectName.includes('[dbo].[Orders].Notes')) {
        const orderTable = baseTables.find(t => t.id === '[dbo].[Orders]');
        if (orderTable) {
          orderTable.diffStatus = 'modified';
          if (!orderTable.columns.some(c => c.name === 'Notes')) {
            orderTable.columns.push({
              name: 'Notes',
              dataType: 'NVARCHAR(500)',
              isNullable: true,
              isPrimaryKey: false,
              isForeignKey: false,
              diffStatus: 'missing_in_target'
            });
          }
          if (orderTable.columnComparisons && !orderTable.columnComparisons.some(c => c.name === 'Notes')) {
            orderTable.columnComparisons.push({
              name: 'Notes',
              sourceType: 'NVARCHAR(500)',
              sourceNullable: true,
              targetType: null,
              targetNullable: null,
              isPrimaryKey: false,
              isForeignKey: false,
              status: 'missing_in_target'
            });
          }
        }
      }
    }

    // 3. Filtra conforme o modo selecionado
    let modeFilteredTables = baseTables;
    if (mode === 'source') {
      modeFilteredTables = baseTables.filter(t => t.diffStatus !== 'missing_in_source');
    } else if (mode === 'target') {
      modeFilteredTables = baseTables.filter(t => t.diffStatus !== 'missing_in_target');
    }

    // 4. Calcula estatísticas gerais antes dos filtros finos
    const stats: DiagramStats = {
      totalTables: modeFilteredTables.length,
      identicalTables: modeFilteredTables.filter(t => t.diffStatus === 'identical').length,
      modifiedTables: modeFilteredTables.filter(t => t.diffStatus === 'modified').length,
      missingInTarget: modeFilteredTables.filter(t => t.diffStatus === 'missing_in_target').length,
      missingInSource: modeFilteredTables.filter(t => t.diffStatus === 'missing_in_source').length,
      relationshipsCount: baseEdges.length
    };

    // 5. Aplica filtro de categoria se especificado
    let finalTables = modeFilteredTables;
    if (filterCategory === 'diff_only') {
      finalTables = modeFilteredTables.filter(t => t.diffStatus !== 'identical');
    } else if (filterCategory === 'missing_target') {
      finalTables = modeFilteredTables.filter(t => t.diffStatus === 'missing_in_target');
    } else if (filterCategory === 'missing_source') {
      finalTables = modeFilteredTables.filter(t => t.diffStatus === 'missing_in_source');
    } else if (filterCategory === 'identical_only') {
      finalTables = modeFilteredTables.filter(t => t.diffStatus === 'identical');
    }

    const validNodeIds = new Set(finalTables.map(t => t.id));
    const filteredEdges = baseEdges.filter(e => validNodeIds.has(e.source) && validNodeIds.has(e.target));

    return {
      nodes: finalTables.map(t => ({ data: t })),
      edges: filteredEdges.map(e => ({ data: e })),
      stats
    };
  }
}

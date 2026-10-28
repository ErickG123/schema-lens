import { describe, it } from 'node:test';
import assert from 'node:assert';
import { DiagramBuilderService } from './diagram-builder.service';
import { DiffViewRow } from '../../comparison/models/comparison.model';

describe('DiagramBuilderService Unit Tests', () => {
  const service = new DiagramBuilderService();

  const mockDiffs: DiffViewRow[] = [
    {
      id: 'diff-1',
      severity: 'CRÍTICA',
      schema: 'dbo',
      objectName: '[dbo].[Orders].Status',
      category: 'Divergência de Propriedade',
      valueSource: 'VARCHAR(20)',
      valueTarget: 'VARCHAR(10)',
      details: 'Tamanho menor no destino.'
    },
    {
      id: 'diff-2',
      severity: 'MÉDIA',
      schema: 'sales',
      objectName: '[sales].[Discounts]',
      category: 'Ausente no Destino',
      valueSource: 'Table (3 cols)',
      valueTarget: 'Inexistente',
      details: 'Tabela ausente no alvo.'
    }
  ];

  it('should build model for comparison mode with correct table nodes', () => {
    const model = service.buildModel('diff', mockDiffs, 'all');
    assert.ok(model);
    assert.ok(model.nodes.length >= 3); // Customers, Orders, Discounts, LegacyLogs

    const ordersNode = model.nodes.find(n => n.data.id === '[dbo].[Orders]');
    assert.ok(ordersNode, 'Orders node must exist');
    assert.strictEqual(ordersNode.data.diffStatus, 'modified');

    const discountsNode = model.nodes.find(n => n.data.id === '[sales].[Discounts]');
    assert.ok(discountsNode, 'Discounts node must exist');
    assert.strictEqual(discountsNode.data.diffStatus, 'missing_in_target');
  });

  it('should filter tables by category: diff_only', () => {
    const model = service.buildModel('diff', mockDiffs, 'diff_only');
    assert.ok(model);
    const tableIds = model.nodes.map(n => n.data.id);
    assert.ok(tableIds.includes('[dbo].[Orders]'));
    assert.ok(!tableIds.includes('[dbo].[Customers]')); // Customers is identical
  });

  it('should filter tables by category: missing_target', () => {
    const model = service.buildModel('diff', mockDiffs, 'missing_target');
    assert.ok(model);
    const tableIds = model.nodes.map(n => n.data.id);
    assert.ok(tableIds.includes('[sales].[Discounts]'));
    assert.ok(!tableIds.includes('[dbo].[Orders]'));
  });

  it('should filter tables for source mode (exclude missing_in_source)', () => {
    const model = service.buildModel('source', mockDiffs, 'all');
    const tableIds = model.nodes.map(n => n.data.id);
    assert.ok(!tableIds.includes('[dbo].[LegacyLogs]'));
    assert.ok(tableIds.includes('[sales].[Discounts]'));
  });

  it('should filter tables for target mode (exclude missing_in_target)', () => {
    const model = service.buildModel('target', mockDiffs, 'all');
    const tableIds = model.nodes.map(n => n.data.id);
    assert.ok(!tableIds.includes('[sales].[Discounts]'));
    assert.ok(tableIds.includes('[dbo].[LegacyLogs]'));
  });

  it('should include correct relationships and stats', () => {
    const model = service.buildModel('diff', mockDiffs, 'all');
    assert.ok(model.edges.length > 0);
    const rel = model.edges.find(e => e.data.source === '[dbo].[Orders]' && e.data.target === '[dbo].[Customers]');
    assert.ok(rel, 'FK Orders -> Customers must exist');
    assert.ok(model.stats.totalTables > 0);
  });
});

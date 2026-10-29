import { describe, it } from 'node:test';
import assert from 'node:assert';
import { DiagramBuilderService } from './diagram-builder.service';
import { DiffViewRow } from '../../comparison/models/comparison.model';
import { SnapshotTable } from '../../comparison/models/snapshot.model';

describe('DiagramBuilderService', () => {
  const service = new DiagramBuilderService();
  const distinctSchemaDiffs: DiffViewRow[] = [
    { severity: 'MÉDIA', schema: 'alpha', objectName: 'Products', category: 'Ausente no Destino', valueSource: 'Table with 3 columns', valueTarget: 'Inexistente', details: 'Table exists only in source.' },
    { severity: 'BAIXA', schema: 'beta', objectName: 'Suppliers', category: 'Ausente na Origem', valueSource: 'Inexistente', valueTarget: 'Table with 2 columns', details: 'Table exists only in target.' }
  ];

  it('builds only nodes represented by the current comparison', () => {
    const model = service.buildModel('diff', distinctSchemaDiffs);
    assert.deepStrictEqual(model.nodes.map(node => node.data.id).sort(), ['[alpha].[Products]', '[beta].[Suppliers]']);
    assert.strictEqual(model.stats.totalTables, 2);
    assert.strictEqual(model.edges.length, 0);
  });

  it('filters according to which side contains the table', () => {
    const source = service.buildModel('source', distinctSchemaDiffs);
    const target = service.buildModel('target', distinctSchemaDiffs);
    assert.deepStrictEqual(source.nodes.map(node => node.data.id), ['[alpha].[Products]']);
    assert.deepStrictEqual(target.nodes.map(node => node.data.id), ['[beta].[Suppliers]']);
  });

  it('does not invent tables for an empty comparison', () => {
    const model = service.buildModel('diff', []);
    assert.deepStrictEqual(model.nodes, []);
    assert.strictEqual(model.stats.totalTables, 0);
  });

  it('uses the current snapshots for identical tables, columns, keys and relationships', () => {
    const products: SnapshotTable = {
      id: { schema: 'alpha', name: 'Products' },
      columns: [{ name: 'ProductId', ordinal_position: 1, data_type: 'INT', max_length: 4, precision: 10, scale: 0, is_nullable: false, is_identity: true, is_computed: false }],
      primary_key: { name: 'PK_Products', is_clustered: true, columns: ['ProductId'] },
      foreign_keys: [{ name: 'FK_ProductSupplier', referenced_schema: 'alpha', referenced_table: 'Suppliers', column_mappings: [['SupplierId', 'SupplierId']], delete_rule: 'NO_ACTION', update_rule: 'NO_ACTION', is_disabled: false }],
      check_constraints: [], default_constraints: [], unique_constraints: [], indexes: []
    };
    const suppliers: SnapshotTable = {
      id: { schema: 'alpha', name: 'Suppliers' },
      columns: [{ name: 'SupplierId', ordinal_position: 1, data_type: 'INT', max_length: 4, precision: 10, scale: 0, is_nullable: false, is_identity: true, is_computed: false }],
      primary_key: { name: 'PK_Suppliers', is_clustered: true, columns: ['SupplierId'] },
      foreign_keys: [], check_constraints: [], default_constraints: [], unique_constraints: [], indexes: []
    };

    const model = service.buildModel('source', [], 'all', [products, suppliers], [products, suppliers]);

    assert.deepStrictEqual(model.nodes.map(node => node.data.id).sort(), ['[alpha].[Products]', '[alpha].[Suppliers]']);
    assert.strictEqual(model.stats.identicalTables, 2);
    assert.strictEqual(model.stats.relationshipsCount, 1);
    assert.strictEqual(model.nodes.find(node => node.data.tableName === 'Products')?.data.columns[0].isPrimaryKey, true);
    assert.strictEqual(model.edges[0].data.source, '[alpha].[Products]');
    assert.strictEqual(model.edges[0].data.target, '[alpha].[Suppliers]');
  });
});

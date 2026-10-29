import { describe, it } from 'node:test';
import assert from 'node:assert/strict';
import { DifferenceFilters, DiffViewRow } from '../models/comparison.model';
import { DifferenceFilterService, emptyDifferenceFilters } from './difference-filter.service';

const row = (values: Partial<DiffViewRow> & Pick<DiffViewRow, 'id' | 'schema' | 'objectName'>): DiffViewRow => ({
  id: values.id,
  schema: values.schema,
  objectName: values.objectName,
  category: 'Divergência de Propriedade',
  severity: 'CRÍTICA',
  objectType: 'Column',
  objectTypeCode: 'column',
  operationType: 'alter_column_type',
  differenceKind: 'property_mismatch',
  riskLevel: 'critical',
  propertyName: 'data_type',
  valueSource: 'nvarchar(100)',
  valueTarget: 'varchar(20)',
  details: 'type mismatch',
  generatedSql: 'ALTER TABLE ...',
  sqlStatus: 'available',
  ...values
});

describe('DifferenceFilterService', () => {
  const service = new DifferenceFilterService();
  const rows = [
    row({ id: 'a1', schema: 'SMARTZ', objectName: 'Orders.Notes', operationType: 'alter_column_type', riskLevel: 'critical' }),
    row({ id: 'a2', schema: 'SMARTZ', objectName: 'Orders.Status', operationType: 'alter_column_nullability', riskLevel: 'medium', severity: 'MÉDIA', sqlStatus: 'unavailable', generatedSql: null }),
    row({ id: 'b1', schema: 'audit', objectName: 'Orders.Notes', operationType: 'drop_column', differenceKind: 'missing_in_source', riskLevel: 'low', severity: 'BAIXA', objectTypeCode: 'column', sqlStatus: 'unsupported' }),
    row({ id: 'unknown', schema: 'misc', objectName: 'Legacy.Item', operationType: undefined, objectTypeCode: undefined, riskLevel: undefined, sqlStatus: undefined }),
    row({ id: 'future-op', schema: 'future', objectName: 'Legacy.NewThing', operationType: 'future_operation' as any })
  ];

  it('combines multiple values with OR and separate filters with AND', () => {
    const filters: DifferenceFilters = {
      ...emptyDifferenceFilters(),
      operationTypes: ['alter_column_type', 'alter_column_nullability'],
      schemas: ['SMARTZ'],
      riskLevels: ['critical', 'medium']
    };
    assert.deepEqual(service.filter(rows, filters).map(item => item.id), ['a1', 'a2']);
  });

  it('searches schema and object names partially without case sensitivity', () => {
    const filters = { ...emptyDifferenceFilters(), query: 'oRdErS.stAt' };
    assert.deepEqual(service.filter(rows, filters).map(item => item.id), ['a2']);
  });

  it('filters object type, direction, severity, and script status independently', () => {
    const filters: DifferenceFilters = {
      ...emptyDifferenceFilters(), objectTypes: ['column'], directions: ['missing_in_source'],
      riskLevels: ['low'], scriptStatuses: ['unsupported']
    };
    assert.deepEqual(service.filter(rows, filters).map(item => item.id), ['b1']);
  });

  it('keeps unknown legacy metadata visible and offers only values present in the results', () => {
    const options = service.getOptions(rows);
    assert.ok(options.operations.some(option => option.value === 'unclassified'));
    assert.match(options.operations.find(option => option.value === 'future_operation')?.label || '', /Operação desconhecida/);
    assert.ok(options.scriptStatuses.some(option => option.value === 'unknown'));
    assert.equal(options.operations.some(option => option.value === 'create_view'), false);
    assert.deepEqual(service.filter(rows, { ...emptyDifferenceFilters(), operationTypes: ['unclassified'] }).map(item => item.id), ['unknown']);
  });

  it('does not mutate the comparison rows or generated SQL when filters change', () => {
    const sqlBefore = rows.map(item => item.generatedSql);
    const filtered = service.filter(rows, { ...emptyDifferenceFilters(), query: 'notes' });
    assert.equal(filtered.length, 2);
    assert.deepEqual(rows.map(item => item.generatedSql), sqlBefore);
    assert.equal(rows.length, 5);
  });

  it('uses schema-qualified identity for same-named objects in different schemas', () => {
    const sameName = [rows[0], rows[2]];
    const filtered = service.filter(sameName, { ...emptyDifferenceFilters(), schemas: ['audit'] });
    assert.deepEqual(filtered.map(item => item.id), ['b1']);
  });
});

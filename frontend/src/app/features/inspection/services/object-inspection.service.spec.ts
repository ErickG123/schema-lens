import { describe, it } from 'node:test';
import assert from 'node:assert';
import { ObjectInspectionService } from './object-inspection.service';

describe('ObjectInspectionService Unit Tests', () => {
  const service = new ObjectInspectionService();

  it('should return detailed inspection for absent table (Discounts)', () => {
    const details = service.inspectObject('[sales].[Discounts]');
    assert.ok(details);
    assert.strictEqual(details.objectId, '[sales].[Discounts]');
    assert.strictEqual(details.schema, 'sales');
    assert.strictEqual(details.overallStatus, 'missing_in_target');
    assert.ok(details.sourceSqlDefinition);
    assert.strictEqual(details.targetSqlDefinition, undefined);
  });

  it('should return detailed inspection for divergent table (Orders) with risk flags', () => {
    const details = service.inspectObject('[dbo].[Orders]');
    assert.ok(details);
    assert.strictEqual(details.overallStatus, 'modified');

    const statusProp = details.properties.find(p => p.name.includes('Status'));
    assert.ok(statusProp, 'Status property must exist');
    assert.strictEqual(statusProp.isDestructiveRisk, true);
    assert.strictEqual(statusProp.status, 'different');

    assert.ok(details.relatedObjects.length > 0);
    const fkRel = details.relatedObjects.find(r => r.objectName === '[dbo].[Customers]');
    assert.ok(fkRel, 'Relationship to Customers must exist');
  });

  it('should return fallback inspection for unknown object', () => {
    const details = service.inspectObject('[dbo].[UnknownTable]');
    assert.ok(details);
    assert.strictEqual(details.objectId, '[dbo].[UnknownTable]');
    assert.strictEqual(details.overallStatus, 'identical');
  });
});

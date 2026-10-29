import { describe, it } from 'node:test';
import assert from 'node:assert';
import { ObjectInspectionService } from './object-inspection.service';

describe('ObjectInspectionService', () => {
  const service = new ObjectInspectionService();
  it('does not synthesize an inspection when snapshot detail is unavailable', () => {
    const details = service.inspectObject('[alpha].[Products]');
    assert.strictEqual(details.objectId, '[alpha].[Products]');
    assert.strictEqual(details.schema, 'alpha');
    assert.strictEqual(details.overallStatus, 'indeterminate');
    assert.deepStrictEqual(details.properties, []);
    assert.ok(details.notes);
  });
});

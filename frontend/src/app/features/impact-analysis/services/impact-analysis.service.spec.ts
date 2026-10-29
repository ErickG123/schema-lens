import { describe, it } from 'node:test';
import assert from 'node:assert';
import { ImpactAnalysisService } from './impact-analysis.service';
import { DiffViewRow } from '../../comparison/models/comparison.model';

describe('ImpactAnalysisService', () => {
  const service = new ImpactAnalysisService();
  const diff = (overrides: Partial<DiffViewRow>): DiffViewRow => ({ severity: 'MÉDIA', schema: 'alpha', objectName: 'Products.Description', category: 'Divergência de Propriedade', propertyName: 'is_nullable', valueSource: 'false', valueTarget: 'true', details: 'Nullability differs.', objectType: 'Column', confidence: 'high', ...overrides });

  it('describes tightening nullability as a conditional target data risk', () => {
    const alert = service.analyzeRisks([diff({})])[0];
    assert.strictEqual(alert.category, 'compatibility');
    assert.match(alert.potentialConsequence, /NULLs/);
    assert.ok((alert.unverifiedConditions || []).length > 0);
    assert.strictEqual(alert.confidence, 'Inferred');
  });

  it('describes a stricter destination as compatibility risk without claiming data loss', () => {
    const alert = service.analyzeRisks([diff({ valueSource: 'true', valueTarget: 'false' })])[0];
    assert.match(alert.potentialConsequence, /rejeitar/);
    assert.doesNotMatch(alert.potentialConsequence, /data loss/i);
  });

  it('flags a new required column conditionally and does not invent a fill value', () => {
    const alert = service.analyzeRisks([diff({ objectName: 'Products.Code', objectType: 'Column', category: 'Ausente no Destino', propertyName: undefined, valueSource: 'nvarchar NOT NULL', valueTarget: 'Inexistente' })])[0];
    assert.ok(alert.potentialConsequence.length > 0);
    assert.ok(alert.recommendedAction.length > 0);
    assert.doesNotMatch(alert.recommendedAction, /SET .* =/i);
    assert.ok((alert.unverifiedConditions || []).length > 0);
  });

  it('treats a required column default as unvalidated evidence, not as a safe fill automatically', () => {
    const alert = service.analyzeRisks([diff({ objectType: 'Column', category: 'Ausente no Destino', propertyName: undefined, valueSource: 'int NOT NULL DEFAULT (0)', valueTarget: 'Inexistente' })])[0];
    assert.match(alert.potentialConsequence, /default/i);
    assert.match(alert.recommendedAction, /Revise o default/);
    assert.ok((alert.unverifiedConditions || []).length > 0);
  });

  it('marks constraint risks as conditional and never claims current violations', () => {
    const alert = service.analyzeRisks([diff({ objectType: 'ForeignKey', propertyName: undefined, category: 'Ausente no Destino' })])[0];
    assert.match(alert.potentialConsequence, /pode rejeitar/);
    assert.ok((alert.unverifiedConditions || []).length > 0);
  });

  it('returns stable classifications and no static alert for empty diffs', () => {
    const row = diff({ objectName: 'alpha.Products.Description' });
    assert.deepStrictEqual(service.analyzeRisks([]), []);
    assert.deepStrictEqual(service.analyzeRisks([row]), service.analyzeRisks([row]));
  });

  it('keeps the generated operation, availability reason and destructive state linked to the same diff', () => {
    const row = diff({
      id: 'diff-42',
      generatedSql: 'USE [TargetDb];\nGO\nALTER TABLE [alpha].[Products] DROP COLUMN [OldCode];',
      sqlStatus: 'available',
      isDestructive: true
    });
    const alert = service.analyzeRisks([row])[0];
    assert.strictEqual(alert.differenceId, 'diff-42');
    assert.strictEqual(alert.generatedSql, row.generatedSql);
    assert.strictEqual(alert.sqlStatus, 'available');
    assert.strictEqual(alert.isDestructive, true);

    const unavailable = service.analyzeRisks([diff({ id: 'diff-43', sqlUnavailableReason: 'Snapshot missing column metadata.', sqlStatus: 'unavailable' })])[0];
    assert.strictEqual(unavailable.differenceId, 'diff-43');
    assert.strictEqual(unavailable.generatedSql, undefined);
    assert.strictEqual(unavailable.sqlUnavailableReason, 'Snapshot missing column metadata.');
  });

  it('does not expose an executable plan without snapshot-validated prerequisites', () => {
    assert.deepStrictEqual(service.generateMigrationPlan([diff({})]), []);
  });
});

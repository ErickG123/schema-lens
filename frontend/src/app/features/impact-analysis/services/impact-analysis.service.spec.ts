import { describe, it } from 'node:test';
import assert from 'node:assert';
import { ImpactAnalysisService } from './impact-analysis.service';
import { DiffViewRow } from '../../comparison/models/comparison.model';

describe('ImpactAnalysisService Unit Tests', () => {
  const service = new ImpactAnalysisService();

  const mockDiffs: DiffViewRow[] = [
    {
      id: 'diff-1',
      severity: 'CRÍTICA',
      schema: 'dbo',
      objectName: '[dbo].[Orders].Status',
      category: 'Divergência de Propriedade',
      valueSource: 'VARCHAR(20)',
      valueTarget: 'VARCHAR(10)',
      details: 'Tamanho menor no destino. Risco de truncamento.'
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
    },
    {
      id: 'diff-3',
      severity: 'BAIXA',
      schema: 'dbo',
      objectName: '[dbo].[LegacyLogs]',
      category: 'Ausente na Origem',
      valueSource: 'Inexistente',
      valueTarget: 'Table (3 cols)',
      details: 'Tabela legada existente somente no destino.'
    }
  ];

  it('should identify critical data truncation hazard when column is reduced in size', () => {
    const alerts = service.analyzeRisks(mockDiffs);
    assert.ok(alerts.length >= 1);

    const truncAlert = alerts.find(a => a.ruleTriggered === 'DATA_TRUNCATION_HAZARD');
    assert.ok(truncAlert, 'Data truncation hazard alert should be generated');
    assert.strictEqual(truncAlert.severity, 'CRITICAL');
    assert.strictEqual(truncAlert.isDestructive, true);
    assert.strictEqual(truncAlert.affectedObject, '[dbo].[Orders].Status');
    assert.ok(truncAlert.dependencies.length > 0);
  });

  it('should identify non-destructive schema additions', () => {
    const alerts = service.analyzeRisks(mockDiffs);
    const additionAlert = alerts.find(a => a.ruleTriggered === 'SAFE_OBJECT_CREATION');
    assert.ok(additionAlert, 'Safe addition alert should be present');
    assert.strictEqual(additionAlert.severity, 'INFO');
    assert.strictEqual(additionAlert.isDestructive, false);
  });

  it('should generate ordered migration plan steps', () => {
    const steps = service.generateMigrationPlan(mockDiffs);
    assert.ok(steps.length > 0);

    // Verify ordering: Schema creation comes before Table creation, which comes before column alterations
    for (let i = 0; i < steps.length - 1; i++) {
      assert.ok(steps[i].order <= steps[i + 1].order, `Step ${i} order must be <= Step ${i+1} order`);
    }

    const colStep = steps.find(s => s.operationType === 'ALTER_COLUMN');
    assert.ok(colStep, 'ALTER_COLUMN step should be generated for Status column');
    assert.strictEqual(colStep.isDestructive, false, 'Altering column expansion to VARCHAR(20) is defensive and non-destructive');
  });
});

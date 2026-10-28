import { describe, it, beforeEach } from 'node:test';
import assert from 'node:assert';
import { ComparisonHistoryService } from './comparison-history.service';
import { DiffViewRow } from '../../comparison/models/comparison.model';

// In-memory mock for localStorage in Node test runner
const memoryStore = new Map<string, string>();
(globalThis as any).localStorage = {
  getItem: (key: string) => memoryStore.get(key) || null,
  setItem: (key: string, value: string) => memoryStore.set(key, value),
  removeItem: (key: string) => memoryStore.delete(key),
  clear: () => memoryStore.clear()
};

describe('ComparisonHistoryService Unit Tests', () => {
  beforeEach(() => {
    localStorage.clear();
  });

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

  it('should initialize with default entry when storage is empty', () => {
    const service = new ComparisonHistoryService();
    const items = service.history();
    assert.ok(items.length >= 1);
    assert.strictEqual(items[0].id, 'hist-e2e-001');
  });

  it('should save new comparison run and persist to localStorage', () => {
    const service = new ComparisonHistoryService();
    const initialCount = service.history().length;

    const saved = service.saveComparison('ServerA/Db1', 'ServerB/Db2', mockDiffs);
    assert.ok(saved.id.startsWith('hist-'));

    const items = service.history();
    assert.strictEqual(items.length, initialCount + 1);

    const latest = items[0];
    assert.strictEqual(latest.sourceIdentifier, 'ServerA/Db1');
    assert.strictEqual(latest.targetIdentifier, 'ServerB/Db2');
    assert.strictEqual(latest.totalDifferences, 2);
    assert.strictEqual(latest.criticalCount, 1);
    assert.strictEqual(latest.mediumCount, 1);

    // Verify localStorage persistence
    const savedRaw = localStorage.getItem('schemalens_comparison_history_v1');
    assert.ok(savedRaw);
    assert.ok(savedRaw.includes(saved.id));
  });

  it('should delete entry by id when requested', () => {
    const service = new ComparisonHistoryService();
    assert.ok(service.history().length > 0);

    const targetId = service.history()[0].id;
    service.deleteEntry(targetId);

    const found = service.history().some(h => h.id === targetId);
    assert.strictEqual(found, false);
  });
});

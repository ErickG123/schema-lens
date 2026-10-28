import { describe, it } from 'node:test';
import assert from 'node:assert';
import { TauriIpcService } from './ipc.service';

describe('TauriIpcService Unit & Fallback Tests', () => {
  const service = new TauriIpcService();

  it('should instantiate cleanly without error', () => {
    assert.ok(service);
  });

  it('should gracefully handle invoke fallback when Tauri runtime is not present', async () => {
    // In node environment, window.__TAURI_INTERNALS__ is undefined
    const result = await service.invoke('health_check');
    assert.deepStrictEqual(result, {});
  });

  it('should provide safe helper wrappers without throwing uncaught errors', async () => {
    const connResult = await service.testConnection({ mode: 'file', filePath: 'dummy.sql' });
    assert.deepStrictEqual(connResult, {});

    const compResult = await service.startComparison({}, {});
    assert.deepStrictEqual(compResult, {});

    const sqlResult = await service.generateSql('TargetDb', []);
    assert.deepStrictEqual(sqlResult, {});
  });
});

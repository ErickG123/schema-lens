import { describe, it } from 'node:test';
import assert from 'node:assert';
import { TauriIpcService } from './ipc.service';

describe('TauriIpcService', () => {
  const service = new TauriIpcService();
  it('constructs without error', () => assert.ok(service));
  it('fails explicitly outside the Tauri runtime instead of manufacturing a result', async () => {
    await assert.rejects(() => service.startComparison({}, {}), /unavailable outside the desktop runtime/);
  });
});

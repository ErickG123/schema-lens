import { describe, it } from 'node:test';
import assert from 'node:assert/strict';
import { buildConnectionTestPayload, mapSavedConnection } from './connection-settings.utils';
import { SavedConnection } from './connection.model';

const saved: SavedConnection = {
  id: 'profile-a',
  name: 'Source database',
  server: 'db.example',
  port: 1444,
  database: 'ReferenceDb',
  authType: 'sql',
  username: 'reader',
  trustServerCertificate: false,
  connectTimeoutSeconds: 15,
  createdAtUtc: '2026-10-29T00:00:00Z',
  hasStoredCredential: true
};

describe('connection settings mapping', () => {
  it('restores saved SQL connection metadata without retrieving or populating its password', () => {
    const mapped = mapSavedConnection(saved);
    assert.equal(mapped?.profileId, 'profile-a');
    assert.equal(mapped?.database, 'ReferenceDb');
    assert.equal(mapped?.hasStoredCredential, true);
    assert.equal(mapped?.password, '');
    assert.equal(mapped?.testStatus, 'idle');
  });

  it('restores SQL-file profiles without turning them into online connections', () => {
    const mapped = mapSavedConnection({ ...saved, authType: 'sql_file', sqlFilePath: 'C:/schemas/ref.sql' });
    assert.equal(mapped?.mode, 'file');
    assert.equal(mapped?.filePath, 'C:/schemas/ref.sql');
  });

  it('sends only the connection-test fields, with no profile identity or saved credential reference in payload', () => {
    const mapped = mapSavedConnection(saved)!;
    const payload = buildConnectionTestPayload(mapped);
    assert.equal(payload['server'], 'db.example');
    assert.equal(payload['password'], '');
    assert.equal('profileId' in payload, false);
    assert.equal('hasStoredCredential' in payload, false);
    assert.equal('credentialRef' in payload, false);
  });
});

import { ConnectionFormState, SavedConnection } from './connection.model';

export function mapSavedConnection(connection?: SavedConnection | null): ConnectionFormState | null {
  if (!connection) return null;
  return {
    profileId: connection.id,
    name: connection.name,
    hasStoredCredential: connection.hasStoredCredential,
    mode: connection.authType === 'sql_file' ? 'file' : 'online',
    connectionString: '',
    server: connection.server,
    port: connection.port || 1433,
    database: connection.database || '',
    authType: connection.authType === 'windows' ? 'windows' : 'sql',
    username: connection.username || '',
    password: '',
    trustServerCertificate: connection.trustServerCertificate,
    filePath: connection.sqlFilePath || '',
    testStatus: 'idle'
  };
}

export function buildConnectionTestPayload(config: ConnectionFormState): Record<string, unknown> {
  return {
    mode: config.mode,
    server: config.server,
    port: config.port,
    database: config.database,
    authType: config.authType,
    username: config.username,
    password: config.password || '',
    trustServerCertificate: config.trustServerCertificate,
    filePath: config.filePath
  };
}

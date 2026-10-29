export interface ConnectionFormState {
  profileId?: string;
  name?: string;
  hasStoredCredential?: boolean;
  mode: 'online' | 'file';
  connectionString: string;
  server: string;
  port: number;
  database: string;
  authType: 'sql' | 'windows';
  username: string;
  password?: string;
  trustServerCertificate: boolean;
  filePath?: string;
  testStatus?: 'idle' | 'testing' | 'success' | 'error';
  errorMessage?: string;
  confirmedIdentity?: string;
  confirmedDatabaseName?: string;
}

export interface SavedConnection {
  id: string;
  name: string;
  server: string;
  port?: number | null;
  instance?: string | null;
  database?: string | null;
  authType: 'sql' | 'windows' | 'sql_file';
  username?: string | null;
  trustServerCertificate: boolean;
  connectTimeoutSeconds: number;
  sqlFilePath?: string | null;
  createdAtUtc: string;
  hasStoredCredential: boolean;
}

export interface ConnectionSelection {
  sourceConnectionId?: string | null;
  targetConnectionId?: string | null;
}

export interface SavedConnectionSettings {
  connections: SavedConnection[];
  selection: ConnectionSelection;
}

export interface ConnectionSettingsApplied {
  source: ConnectionFormState;
  target: ConnectionFormState;
  connections: SavedConnection[];
}

export interface ComparisonDashboardState {
  source: ConnectionFormState;
  target: ConnectionFormState;
  status: 'idle' | 'running' | 'completed' | 'error';
  progressPercent: number;
  progressMessage: string;
  result?: any;
}

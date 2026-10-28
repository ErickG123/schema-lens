export interface ConnectionFormState {
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
}

export interface ComparisonDashboardState {
  source: ConnectionFormState;
  target: ConnectionFormState;
  status: 'idle' | 'running' | 'completed' | 'error';
  progressPercent: number;
  progressMessage: string;
  result?: any;
}

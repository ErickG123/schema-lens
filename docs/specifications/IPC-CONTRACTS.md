# SchemaLens — Contratos de Comunicação IPC

## 1. Padrão de Envelope IPC
Todas as invocações IPC retornam a seguinte estrutura unificada:

```typescript
export interface IpcResponse<T> {
  success: boolean;
  data?: T;
  error?: IpcError;
}

export interface IpcError {
  code: string;
  message: string;
  details?: string;
  sanitized: boolean;
}
```

## 2. Comandos do Backend Tauri (`invoke`)

### 2.1 `test_connection`
- **Payload**:
  ```typescript
  export interface TestConnectionRequest {
    connection_string?: string;
    profile_id?: string;
    auth_type: 'sql' | 'windows';
    host?: string;
    port?: number;
    database?: string;
    username?: string;
    password?: string;
    trust_server_certificate: boolean;
    connect_timeout_seconds: number;
  }
  ```
- **Retorno**: `IpcResponse<ConnectionTestResult>`
  ```typescript
  export interface ConnectionTestResult {
    connected: boolean;
    server_version: string;
    databases: string[];
    current_database: string;
    collation: string;
    latency_ms: number;
    requires_trust_warning: boolean;
  }
  ```

### 2.2 `run_comparison`
- **Payload**:
  ```typescript
  export interface RunComparisonRequest {
    source: TestConnectionRequest;
    target: TestConnectionRequest;
    options: ComparisonOptions;
  }
  ```
- **Retorno**: `IpcResponse<ComparisonRunResult>`

### 2.3 `cancel_comparison`
- **Payload**: `{ run_id: string }`
- **Retorno**: `IpcResponse<boolean>`

### 2.4 `generate_migration_sql`
- **Payload**: `{ run_id: string, selected_difference_ids: string[] }`
- **Retorno**: `IpcResponse<MigrationPlanResult>`

### 2.5 `export_report`
- **Payload**: `{ run_id: string, format: 'json' | 'html' | 'csv', output_path: string }`
- **Retorno**: `IpcResponse<{ file_path: string, bytes_written: number }>`

## 3. Eventos Emitidos pelo Backend (`listen`)
- `schema-lens://progress`:
  ```typescript
  export interface ProgressEvent {
    run_id: string;
    stage: 'connecting' | 'reading_source' | 'reading_target' | 'diffing' | 'completed' | 'failed';
    current_category: string;
    percent: number;
    message: string;
  }
  ```

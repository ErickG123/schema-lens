# SchemaLens — Arquitetura do Sistema

## 1. Visão Arquitetural Geral
O SchemaLens é construído como um aplicativo desktop modular utilizando **Tauri 2** como camada de runtime do SO Windows, um frontend reativo em **Angular 19/20** e um backend de alta performance estruturado como um monorepo multi-crates em **Rust**.

```mermaid
flowchart TD
    subgraph UI ["Frontend (Angular + TypeScript)"]
        Components["Componentes UI (Signals & Tailwind)"]
        Services["Serviços de Domínio & IPC Client"]
        State["Gerenciamento de Estado Reativo"]
    end

    subgraph DesktopShell ["Desktop Shell (Tauri 2)"]
        IPCBridge["Tauri IPC Commands & Event Emitter"]
        CapSecurity["Capabilities & Menor Privilégio"]
    end

    subgraph RustBackend ["Backend Monorepo (Rust)"]
        AppCore["application (Casos de Uso & Orquestração)"]
        DomainCore["domain (Entidades de Schema & Regras de Negócio Puras)"]
        ConnParser["dotnet_conn_parser (Parser DFA de Connection Strings)"]
        ReaderEngine["schema-reader (Introspecção via sys.*)"]
        DiffEngine["schema-diff (Motor de Comparação Determinístico)"]
        SqlGen["sql-generator (Planejador de Migração e Geração DDL)"]
        StorageEngine["local-storage (SQLite de Metadados & Histórico)"]
        SecVault["Windows Credential Manager Adapter"]
    end

    subgraph DatabaseServers ["Instâncias SQL Server"]
        SourceDB[("Banco A - Origem")]
        TargetDB[("Banco B - Destino")]
    end

    UI <-->|Tauri IPC / Json Envelopes & Events| DesktopShell
    DesktopShell <--> RustBackend
    ReaderEngine -->|Somente Leitura via Tiberius| SourceDB
    ReaderEngine -->|Somente Leitura via Tiberius| TargetDB
    StorageEngine --> SecVault
```

## 2. Divisão de Crates do Rust
1. **`domain`**:
   - Totalmente independente de frameworks, bibliotecas de banco de dados ou UI.
   - Contém structs puras: `SchemaSnapshot`, `TableMetadata`, `ColumnMetadata`, `IndexMetadata`, `SchemaDifference`, etc.
   - Enums para severidade, tipos de diferença e status de confiança.
2. **`dotnet_conn_parser`**:
   - Parser seguro para connection strings de formato .NET.
   - Normaliza aliases e valida opções de segurança.
3. **`schema-reader`**:
   - Executa consultas estruturadas às views `sys.*` do SQL Server via driver `tiberius`.
   - Constrói o `SchemaSnapshot` normalizado.
4. **`schema-diff`**:
   - Algoritmo de pareamento e comparação determinística de snapshots.
   - Ordenação canônica das diferenças e cálculo de severidade.
5. **`sql-generator`**:
   - Constrói o `MigrationPlan` a partir das diferenças elegíveis.
   - Emite scripts DDL comentados e com identificadores envolvidos com `QUOTENAME` (`[Schema].[Nome]`).
6. **`local-storage`**:
   - Persistência SQLite local para histórico de comparações e perfis.
   - Integração com a API do Windows Credential Manager.
7. **`application`**:
   - Casos de uso e orquestração entre leitor, comparador, gerador e armazenamento.
8. **`tauri-app`**:
   - Binário final Tauri 2, registro de comandos IPC e gerenciamento de ciclo de vida.

## 3. Diagrama de Sequência de Comparação

```mermaid
sequenceDiagram
    autonumber
    actor User as Usuário
    participant UI as Angular Frontend
    participant IPC as Tauri IPC Command
    participant App as Application Layer
    participant Reader as Schema Reader
    participant DB as SQL Server (A e B)
    participant Diff as Schema Diff Engine

    User->>UI: Clicar em "Iniciar Comparação"
    UI->>IPC: invoke("start_comparison", { connA, connB, options })
    IPC->>App: execute(connA, connB, options)
    
    par Introspecção Banco A
        App->>Reader: read_snapshot(connA)
        Reader->>DB: Query sys.tables, sys.columns, sys.indexes...
        DB-->>Reader: Metadados A
        Reader-->>App: Snapshot A
    and Introspecção Banco B
        App->>Reader: read_snapshot(connB)
        Reader->>DB: Query sys.tables, sys.columns, sys.indexes...
        DB-->>Reader: Metadados B
        Reader-->>App: Snapshot B
    end

    App-->>UI: Event("progress", "Iniciando motor de diff...")
    App->>Diff: compare(Snapshot A, Snapshot B, options)
    Diff-->>App: ComparisonResult (Diferenças ordenadas)
    App-->>UI: IpcResponse(ComparisonResult)
    UI-->>User: Renderizar Dashboard e Árvore de Diferenças
```

## 4. Política de Tratamento de Erros e Sanitização
- Toda exceção SQL interna ou de rede tem sua mensagem filtrada por uma camada de redaction antes de alcançar a UI.
- Nenhuma connection string com valor de `Password` ou `PWD` é exposta em logs de console ou respostas IPC.

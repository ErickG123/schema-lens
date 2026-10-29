import { Component, ElementRef, inject, OnInit, ViewChild } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { ConnectionSettingsModalComponent } from './features/connections/components/connection-settings-modal.component';
import { ConnectionFormState, ConnectionSettingsApplied, SavedConnection } from './features/connections/models/connection.model';
import { mapSavedConnection } from './features/connections/models/connection-settings.utils';
import { ComparisonToolbarComponent } from './features/comparison/components/comparison-toolbar.component';
import { DiffResultsComponent } from './features/comparison/components/diff-results.component';
import { ComparisonReport, ComparisonResultDto, DiffViewRow } from './features/comparison/models/comparison.model';
import { SnapshotTable } from './features/comparison/models/snapshot.model';
import { ErDiagramComponent } from './features/diagram/components/er-diagram.component';
import { ObjectDetailPanelComponent } from './features/inspection/components/object-detail-panel.component';
import { DetailedObjectInspection } from './features/inspection/models/inspection.model';
import { ObjectInspectionService } from './features/inspection/services/object-inspection.service';
import { ImpactDashboardComponent } from './features/impact-analysis/components/impact-dashboard.component';
import { MigrationRiskAlert } from './features/impact-analysis/models/impact.model';
import { ComparisonHistoryComponent } from './features/history/components/comparison-history.component';
import { ComparisonHistoryService } from './features/history/services/comparison-history.service';
import { ToastContainerComponent } from './features/notifications/components/toast-container.component';
import { ConfirmDialogComponent } from './features/notifications/components/confirm-dialog.component';
import { NotificationService } from './features/notifications/services/notification.service';
import { TauriIpcService } from './core/services/ipc.service';

@Component({
  selector: 'app-root',
  standalone: true,
  imports: [
    CommonModule,
    FormsModule,
    ConnectionSettingsModalComponent,
    ComparisonToolbarComponent,
    DiffResultsComponent,
    ErDiagramComponent,
    ObjectDetailPanelComponent,
    ImpactDashboardComponent,
    ComparisonHistoryComponent,
    ToastContainerComponent,
    ConfirmDialogComponent
  ],
  templateUrl: './app.component.html',
  styleUrl: './app.component.css'
})
export class AppComponent implements OnInit {
  private ipc = inject(TauriIpcService);
  private notify = inject(NotificationService);
  private inspectionService = inject(ObjectInspectionService);
  private historyService = inject(ComparisonHistoryService);

  @ViewChild('erDiagram') erDiagramComponent?: ErDiagramComponent;
  @ViewChild('settingsButton') private settingsButton?: ElementRef<HTMLButtonElement>;

  activeView: 'table' | 'diagram' | 'impact' | 'history' = 'table';
  connectionSettingsOpen = false;
  activeInspection: DetailedObjectInspection | null = null;

  source: ConnectionFormState = {
    mode: 'online',
    connectionString: '',
    server: '',
    port: 1433,
    database: '',
    authType: 'sql',
    username: '',
    password: '',
    trustServerCertificate: false,
    testStatus: 'idle'
  };

  target: ConnectionFormState = {
    mode: 'online',
    connectionString: '',
    server: '',
    port: 1433,
    database: '',
    authType: 'sql',
    username: '',
    password: '',
    trustServerCertificate: false,
    testStatus: 'idle'
  };

  savedConnections: SavedConnection[] = [];
  comparing = false;
  exporting = false;
  hasRun = false;

  filterQuery = '';
  selectedCategory = 'all';

  diffs: DiffViewRow[] = [];
  sourceTables: SnapshotTable[] = [];
  targetTables: SnapshotTable[] = [];
  private comparisonSequence = 0;

  async ngOnInit(): Promise<void> {
    try {
      const settings = await this.ipc.loadConnectionSettings();
      this.savedConnections = settings.connections || [];
      this.source = this.connectionFromSaved(settings.selection.sourceConnectionId);
      this.target = this.connectionFromSaved(settings.selection.targetConnectionId);
    } catch (error: any) {
      this.notify.error('Falha ao carregar conexões', error?.message || 'As conexões salvas não puderam ser recuperadas.');
    }
  }

  openConnectionSettings(): void {
    this.connectionSettingsOpen = true;
  }

  closeConnectionSettings(): void {
    this.connectionSettingsOpen = false;
    setTimeout(() => this.settingsButton?.nativeElement.focus(), 0);
  }

  applyConnectionSettings(settings: ConnectionSettingsApplied): void {
    const previous = JSON.stringify({ source: this.source, target: this.target });
    this.source = { ...settings.source, password: '' };
    this.target = { ...settings.target, password: '' };
    this.savedConnections = settings.connections;
    const current = JSON.stringify({ source: this.source, target: this.target });
    if (previous !== current && this.hasRun) {
      this.comparisonSequence++;
      this.diffs = [];
      this.sourceTables = [];
      this.targetTables = [];
      this.activeInspection = null;
      this.hasRun = false;
      this.filterQuery = '';
      this.selectedCategory = 'all';
      this.notify.info('Comparação invalidada', 'As conexões mudaram. Execute uma nova comparação para atualizar os resultados.');
    }
  }

  setActiveView(view: 'table' | 'diagram' | 'impact' | 'history'): void {
    this.activeView = view;
    if (view === 'diagram') {
      setTimeout(() => {
        this.erDiagramComponent?.refreshViewport();
      }, 50);
    }
  }

  get filteredDiffs(): DiffViewRow[] {
    return this.diffs.filter(d => {
      const matchText = !this.filterQuery || 
        d.objectName.toLowerCase().includes(this.filterQuery.toLowerCase()) ||
        d.schema.toLowerCase().includes(this.filterQuery.toLowerCase()) ||
        d.details.toLowerCase().includes(this.filterQuery.toLowerCase());
      
      const matchCat = this.selectedCategory === 'all' || 
        (this.selectedCategory === 'missing_in_target' && d.category.includes('Destino')) ||
        (this.selectedCategory === 'missing_in_source' && d.category.includes('Origem')) ||
        (this.selectedCategory === 'property_mismatch' && (d.category.includes('Divergência') || d.category.includes('Propriedade')));

      return matchText && matchCat;
    });
  }

  async runComparison() {
    if (this.comparing) return;
    if (!this.source.profileId || !this.target.profileId) {
      this.notify.warning('Conexões necessárias', 'Selecione uma conexão salva para Origem (A) e outra para Destino (B).');
      this.openConnectionSettings();
      return;
    }
    const sequence = ++this.comparisonSequence;
    this.comparing = true;
    this.hasRun = false;
    this.diffs = [];
    this.sourceTables = [];
    this.targetTables = [];
    this.activeInspection = null;
    this.filterQuery = '';
    this.selectedCategory = 'all';
    try {
      const identitiesConfirmed = await this.ensureConnectionIdentities();
      if (!identitiesConfirmed) return;
      const res = await this.ipc.startComparison(this.source.profileId, this.target.profileId) as ComparisonResultDto;
      if (sequence !== this.comparisonSequence) return;
      if (!res?.success || !Array.isArray(res.diffs)) throw new Error('Backend did not return a valid comparison result.');
      this.diffs = res.diffs;
      this.sourceTables = res.sourceTables || [];
      this.targetTables = res.targetTables || [];
      this.hasRun = true;
      this.historyService.saveComparison(
        res.sourceIdentity || `${this.source.server} / ${this.source.database}`,
        res.targetIdentity || `${this.target.server} / ${this.target.database}`,
        this.diffs,
        res.comparisonId
      );
      this.notify.success('Comparison complete', res.message);
    } catch (e: any) {
      if (sequence !== this.comparisonSequence) return;
      this.diffs = [];
      this.hasRun = false;
      this.notify.error('Comparison failed', e?.message || String(e));
    } finally {
      if (sequence === this.comparisonSequence) this.comparing = false;
    }
  }

  private async ensureConnectionIdentities(): Promise<boolean> {
    for (const side of ['source', 'target'] as const) {
      const config = side === 'source' ? this.source : this.target;
      if (config.testStatus !== 'success') {
        try {
          const result = await this.ipc.testSavedConnection(config.profileId!);
          config.testStatus = 'success';
          config.confirmedIdentity = result?.databaseName
            ? `${result.serverName || 'Servidor desconhecido'} / ${result.databaseName}`
            : undefined;
          config.confirmedDatabaseName = result?.databaseName;
        } catch (error: any) {
          config.testStatus = 'error';
          config.confirmedIdentity = undefined;
          config.confirmedDatabaseName = undefined;
          this.notify.error(`Falha ao testar ${side === 'source' ? 'Origem (A)' : 'Destino (B)'}`, error?.message || 'Conexão não confirmada.');
          return false;
        }
      }
      if (config.confirmedDatabaseName && config.database &&
          config.confirmedDatabaseName.toLocaleLowerCase() !== config.database.toLocaleLowerCase()) {
        const accepted = await this.notify.confirm({
          title: 'Banco confirmado diferente',
          message: `A configuração solicita “${config.database}”, mas o SQL Server confirmou “${config.confirmedDatabaseName}” (${config.confirmedIdentity}). Deseja continuar?`,
          confirmLabel: 'Continuar',
          cancelLabel: 'Cancelar'
        });
        if (!accepted) return false;
      }
    }
    return true;
  }

  private connectionFromSaved(connectionId?: string | null): ConnectionFormState {
    const saved = this.savedConnections.find(connection => connection.id === connectionId);
    return mapSavedConnection(saved) || this.emptyConnection();
  }

  private emptyConnection(): ConnectionFormState {
    return {
      mode: 'online', connectionString: '', server: '', port: 1433, database: '',
      authType: 'sql', username: '', password: '', trustServerCertificate: false, testStatus: 'idle'
    };
  }
  onInspectObject(objectName: string): void {
    this.activeInspection = this.inspectionService.inspectObject(objectName);
  }

  onImpactNavigate(objectIdentifier: string): void {
    this.activeView = 'diagram';
    setTimeout(() => this.erDiagramComponent?.selectObject(objectIdentifier), 50);
  }
  onDiagramTableSelected(tableId: string): void {
    this.filterQuery = tableId;
    this.activeInspection = this.inspectionService.inspectObject(tableId);
    this.notify.info('Objeto Selecionado', `Inspeção detalhada aberta para ${tableId}`);
  }

  onDiagramOpenDiffs(tableId: string): void {
    this.filterQuery = tableId;
    this.activeView = 'table';
    this.notify.info('Navegação para Diferenças', `Filtro aplicado para exibir discrepâncias de ${tableId}`);
  }

  onRestoreSnapshot(diffs: DiffViewRow[]): void {
    this.diffs = diffs;
    this.sourceTables = [];
    this.targetTables = [];
    this.hasRun = true;
    this.activeView = 'table';
    this.notify.info('Snapshot Restaurado', `Carregadas ${diffs.length} diferenças históricas.`);
  }

  private generateTimestamp(): string {
    const now = new Date();
    const pad = (n: number) => n.toString().padStart(2, '0');
    return `${now.getFullYear()}${pad(now.getMonth() + 1)}${pad(now.getDate())}-${pad(now.getHours())}${pad(now.getMinutes())}${pad(now.getSeconds())}`;
  }

  async exportSql() {
    if (!this.hasRun || this.diffs.length === 0) {
      this.notify.warning('Exportação SQL', 'Execute uma comparação antes de gerar o script de migração.');
      return;
    }

    if (this.exporting) return;
    if (this.diffs.some(diff => diff.category.includes('Ausente na Origem'))) {
      const accepted = await this.notify.confirm({
        title: 'Review destination-only objects',
        message: 'This comparison contains objects present only in the destination. Review each object and its dependencies before exporting a script.',
        confirmLabel: 'Review and continue',
        cancelLabel: 'Cancel'
      });
      if (!accepted) return;
    }
    this.exporting = true;

    try {
      const sqlContent = await this.ipc.generateSql();
      const defaultFileName = `SchemaLens-Migration-${this.generateTimestamp()}.sql`;
      const filePath = await this.ipc.promptSaveFilePath(defaultFileName, 'Arquivo SQL', 'sql');

      if (!filePath) {
        this.notify.info('Exportação Cancelada', 'Operação cancelada pelo usuário.');
        return;
      }

      const saveResult = await this.ipc.saveFile(filePath, sqlContent);

      if (saveResult.saved) {
        this.notify.success('Script SQL Exportado', saveResult.message);
      } else {
        this.notify.warning('Exportação não Concluída', saveResult.message);
      }
    } catch (e: any) {
      this.notify.error('Falha na geração/exportação SQL', e?.message || String(e));
    } finally {
      this.exporting = false;
    }
  }

  async exportIndividualSql(row: DiffViewRow): Promise<void> {
    if (!row.generatedSql) {
      this.notify.warning('SQL indisponível', row.sqlUnavailableReason || 'Não há metadados suficientes para gerar esta operação.');
      return;
    }
    if (row.isDestructive) {
      const accepted = await this.notify.confirm({
        title: 'Revisar operação destrutiva',
        message: `A operação para ${row.objectName} pode remover ou alterar dados/estrutura. Confirme a revisão antes de exportar.`,
        confirmLabel: 'Exportar para revisão',
        cancelLabel: 'Cancelar'
      });
      if (!accepted) return;
    }
    const safeObjectName = row.objectName.replace(/[^a-z0-9_-]+/gi, '_').slice(0, 80);
    await this.exportSqlFile(row.generatedSql, `SchemaLens-${safeObjectName}-${this.generateTimestamp()}.sql`);
  }

  async exportRiskSql(alert: MigrationRiskAlert): Promise<void> {
    if (!alert.generatedSql) {
      this.notify.warning('SQL indisponível', alert.sqlUnavailableReason || 'Não há metadados suficientes para gerar esta operação.');
      return;
    }
    if (alert.isDestructive) {
      const accepted = await this.notify.confirm({
        title: 'Revisar operação destrutiva',
        message: `A operação para ${alert.affectedObject} é destrutiva. Revise a perda potencial e as dependências antes de exportar.`,
        confirmLabel: 'Exportar para revisão',
        cancelLabel: 'Cancelar'
      });
      if (!accepted) return;
    }
    await this.exportSqlFile(
      alert.generatedSql,
      `SchemaLens-${alert.affectedObject.replace(/[^a-z0-9_-]+/gi, '_').slice(0, 80)}-${this.generateTimestamp()}.sql`
    );
  }

  private async exportSqlFile(sqlContent: string, defaultFileName: string): Promise<void> {
    if (this.exporting) return;
    this.exporting = true;
    try {
      const filePath = await this.ipc.promptSaveFilePath(defaultFileName, 'Arquivo SQL', 'sql');
      if (!filePath) {
        this.notify.info('Exportação cancelada', 'Nenhum arquivo foi salvo.');
        return;
      }
      const saveResult = await this.ipc.saveFile(filePath, sqlContent);
      if (saveResult.saved) this.notify.success('Script SQL exportado', saveResult.message);
      else this.notify.warning('Exportação não concluída', saveResult.message);
    } catch (e: any) {
      this.notify.error('Falha na exportação SQL', e?.message || String(e));
    } finally {
      this.exporting = false;
    }
  }

  async exportJson() {
    if (!this.hasRun || this.diffs.length === 0) {
      this.notify.warning('Exportação JSON', 'Execute uma comparação antes de exportar o relatório.');
      return;
    }

    if (this.exporting) return;
    this.exporting = true;

    try {
      const defaultFileName = `SchemaLens-Comparison-${this.generateTimestamp()}.json`;
      const filePath = await this.ipc.promptSaveFilePath(defaultFileName, 'Arquivo JSON', 'json');

      if (!filePath) {
        this.notify.info('Exportação Cancelada', 'Operação cancelada pelo usuário.');
        return;
      }

      const reportData: ComparisonReport = {
        formatVersion: '1.0.0',
        tool: 'SchemaLens',
        version: '0.1.0',
        generatedAt: new Date().toISOString(),
        source: {
          server: this.source.server,
          database: this.source.database,
          mode: this.source.mode
        },
        target: {
          server: this.target.server,
          database: this.target.database,
          mode: this.target.mode
        },
        summary: {
          totalDifferences: this.diffs.length,
          criticalSeverity: this.diffs.filter(d => d.severity === 'CR\u00CDTICA').length,
          mediumSeverity: this.diffs.filter(d => d.severity === 'M\u00C9DIA').length,
          lowSeverity: this.diffs.filter(d => d.severity === 'BAIXA').length
        },
        differences: this.diffs
      };

      const jsonContent = JSON.stringify(reportData, null, 2);
      const saveResult = await this.ipc.saveFile(filePath, jsonContent);

      if (saveResult.saved) {
        this.notify.success('Relatório JSON Exportado', saveResult.message);
      } else {
        this.notify.warning('Exportação não Concluída', saveResult.message);
      }
    } catch (e: any) {
      this.notify.error('Falha na Gravação JSON', e?.message || String(e));
    } finally {
      this.exporting = false;
    }
  }
}

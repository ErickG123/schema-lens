import { CommonModule } from '@angular/common';
import { Component, ElementRef, EventEmitter, HostListener, Input, OnChanges, Output, SimpleChanges, ViewChild, inject } from '@angular/core';
import { FormsModule } from '@angular/forms';
import { TauriIpcService } from '../../../core/services/ipc.service';
import { NotificationService } from '../../notifications/services/notification.service';
import { ConnectionPanelComponent } from './connection-panel.component';
import {
  ConnectionFormState,
  ConnectionSettingsApplied,
  SavedConnection,
  SavedConnectionSettings
} from '../models/connection.model';
import { buildConnectionTestPayload, mapSavedConnection } from '../models/connection-settings.utils';

type Side = 'source' | 'target';

@Component({
  selector: 'app-connection-settings-modal',
  standalone: true,
  imports: [CommonModule, FormsModule, ConnectionPanelComponent],
  templateUrl: './connection-settings-modal.component.html',
  styleUrl: './connection-settings-modal.component.css'
})
export class ConnectionSettingsModalComponent implements OnChanges {
  @Input() isOpen = false;
  @Input() source!: ConnectionFormState;
  @Input() target!: ConnectionFormState;
  @Input() comparisonRunning = false;
  @Output() onClose = new EventEmitter<void>();
  @Output() onApplied = new EventEmitter<ConnectionSettingsApplied>();

  @ViewChild('dialog') private dialog?: ElementRef<HTMLElement>;

  private readonly ipc = inject(TauriIpcService);
  private readonly notify = inject(NotificationService);

  connections: SavedConnection[] = [];
  sourceDraft = this.emptyConnection();
  targetDraft = this.emptyConnection();
  loading = true;
  saving = false;
  deletingId: string | null = null;
  testingSource = false;
  testingTarget = false;
  loadError = '';
  private initialFingerprint = '';

  ngOnChanges(changes: SimpleChanges): void {
    if (changes['isOpen']?.currentValue === true) {
      void this.loadSettings();
    }
  }

  get hasUnsavedChanges(): boolean {
    return this.fingerprint() !== this.initialFingerprint;
  }

  get busy(): boolean {
    return this.loading || this.saving || this.testingSource || this.testingTarget || !!this.deletingId;
  }

  async loadSettings(): Promise<void> {
    this.loading = true;
    this.loadError = '';
    this.sourceDraft = this.copyConfig(this.source);
    this.targetDraft = this.copyConfig(this.target);
    try {
      const settings: SavedConnectionSettings = await this.ipc.loadConnectionSettings();
      this.connections = settings.connections || [];
      if (!this.sourceDraft.profileId && settings.selection.sourceConnectionId) {
        this.sourceDraft = this.configFromSaved(settings.selection.sourceConnectionId);
      }
      if (!this.targetDraft.profileId && settings.selection.targetConnectionId) {
        this.targetDraft = this.configFromSaved(settings.selection.targetConnectionId);
      }
      this.initialFingerprint = this.fingerprint();
      setTimeout(() => this.focusFirstControl(), 0);
    } catch (error: any) {
      this.loadError = error?.message || 'Não foi possível carregar as conexões salvas.';
      this.initialFingerprint = this.fingerprint();
    } finally {
      this.loading = false;
    }
  }

  draftFor(side: Side): ConnectionFormState {
    return side === 'source' ? this.sourceDraft : this.targetDraft;
  }

  selectConnection(side: Side, connectionId: string): void {
    if (!connectionId) {
      this.setDraft(side, this.emptyConnection());
      return;
    }
    this.setDraft(side, this.configFromSaved(connectionId));
  }

  addConnection(side: Side): void {
    this.setDraft(side, {
      ...this.emptyConnection(),
      profileId: crypto.randomUUID(),
      name: ''
    });
  }

  async testDraft(side: Side): Promise<void> {
    const config = this.draftFor(side);
    if (this.isTesting(side)) return;
    this.setTesting(side, true);
    config.testStatus = 'testing';
    try {
      const result = await this.ipc.testConnection(buildConnectionTestPayload(config), config.profileId || undefined);
      config.testStatus = 'success';
      config.confirmedIdentity = result?.databaseName
        ? `${result.serverName || 'Servidor desconhecido'} / ${result.databaseName}`
        : undefined;
      config.confirmedDatabaseName = result?.databaseName;
      this.notify.success('Conexão validada', result?.message || 'O servidor confirmou a conexão.');
    } catch (error: any) {
      config.testStatus = 'error';
      config.confirmedIdentity = undefined;
      config.confirmedDatabaseName = undefined;
      this.notify.error('Falha ao testar conexão', error?.message || 'Não foi possível validar esta conexão.');
    } finally {
      this.setTesting(side, false);
    }
  }

  async save(): Promise<void> {
    if (this.busy || this.comparisonRunning) return;
    this.saving = true;
    try {
      const savedSource = await this.saveDraft('source');
      const savedTarget = await this.saveDraft('target');
      const selection = {
        sourceConnectionId: savedSource?.profileId || null,
        targetConnectionId: savedTarget?.profileId || null
      };
      await this.ipc.saveConnectionSelection(selection);
      this.sourceDraft = savedSource || this.emptyConnection();
      this.targetDraft = savedTarget || this.emptyConnection();
      this.initialFingerprint = this.fingerprint();
      this.notify.success('Configurações salvas', 'As conexões foram persistidas localmente.');
      this.onApplied.emit({
        source: this.copyConfig(this.sourceDraft),
        target: this.copyConfig(this.targetDraft),
        connections: [...this.connections]
      });
      this.clearTemporarySecrets();
      this.onClose.emit();
    } catch (error: any) {
      this.notify.error('Falha ao salvar configurações', error?.message || 'O backend não confirmou o salvamento.');
    } finally {
      this.saving = false;
    }
  }

  async deleteConnection(connection: SavedConnection): Promise<void> {
    if (this.busy || this.comparisonRunning) return;
    const accepted = await this.notify.confirm({
      title: 'Excluir conexão salva',
      message: `A conexão “${connection.name}” e a credencial associada serão removidas. Seleções que apontam para ela serão limpas.`,
      confirmLabel: 'Excluir conexão',
      cancelLabel: 'Cancelar'
    });
    if (!accepted) return;
    this.deletingId = connection.id;
    try {
      await this.ipc.deleteConnection(connection.id);
      this.connections = this.connections.filter(item => item.id !== connection.id);
      if (this.sourceDraft.profileId === connection.id) this.sourceDraft = this.emptyConnection();
      if (this.targetDraft.profileId === connection.id) this.targetDraft = this.emptyConnection();
      this.initialFingerprint = this.fingerprint();
      this.notify.success('Conexão excluída', 'A conexão e a seleção associada foram atualizadas.');
      this.onApplied.emit({
        source: this.copyConfig(this.sourceDraft),
        target: this.copyConfig(this.targetDraft),
        connections: [...this.connections]
      });
    } catch (error: any) {
      this.notify.error('Falha ao excluir conexão', error?.message || 'O backend não confirmou a exclusão.');
    } finally {
      this.deletingId = null;
    }
  }

  deleteDraftConnection(side: Side): void {
    const connectionId = this.draftFor(side).profileId;
    const connection = this.connections.find(item => item.id === connectionId);
    if (connection) void this.deleteConnection(connection);
  }

  async requestClose(): Promise<void> {
    if (this.busy) return;
    if (this.hasUnsavedChanges) {
      const accepted = await this.notify.confirm({
        title: 'Descartar alterações?',
        message: 'As alterações não salvas serão descartadas.',
        confirmLabel: 'Descartar alterações',
        cancelLabel: 'Continuar editando'
      });
      if (!accepted) return;
    }
    this.clearTemporarySecrets();
    this.onClose.emit();
  }

  onOverlayClick(event: MouseEvent): void {
    if (event.target === event.currentTarget) void this.requestClose();
  }

  onDialogKeydown(event: KeyboardEvent): void {
    if (event.key === 'Escape') {
      event.preventDefault();
      void this.requestClose();
      return;
    }
    if (event.key !== 'Tab' || !this.dialog) return;
    const focusable = Array.from(this.dialog.nativeElement.querySelectorAll<HTMLElement>(
      'button:not([disabled]), input:not([disabled]), select:not([disabled]), [tabindex]:not([tabindex="-1"])'
    )).filter(element => element.offsetParent !== null);
    if (!focusable.length) return;
    const first = focusable[0];
    const last = focusable[focusable.length - 1];
    if (event.shiftKey && document.activeElement === first) {
      event.preventDefault();
      last.focus();
    } else if (!event.shiftKey && document.activeElement === last) {
      event.preventDefault();
      first.focus();
    }
  }

  private async saveDraft(side: Side): Promise<ConnectionFormState | null> {
    const config = this.draftFor(side);
    if (!config.profileId) return null;
    if (!config.name?.trim()) throw new Error(`Informe um nome para a conexão de ${side === 'source' ? 'Origem' : 'Destino'}.`);
    const profile: SavedConnection = {
      id: config.profileId,
      name: config.name.trim(),
      server: config.server,
      port: config.mode === 'online' ? config.port : null,
      instance: null,
      database: config.mode === 'online' ? config.database : null,
      authType: config.mode === 'file' ? 'sql_file' : config.authType,
      username: config.authType === 'windows' || config.mode === 'file' ? null : config.username,
      trustServerCertificate: config.trustServerCertificate,
      connectTimeoutSeconds: 15,
      sqlFilePath: config.mode === 'file' ? config.filePath || null : null,
      createdAtUtc: this.connections.find(item => item.id === config.profileId)?.createdAtUtc || new Date().toISOString(),
      hasStoredCredential: config.hasStoredCredential || false
    };
    const saved = await this.ipc.saveConnection(profile, config);
    this.connections = [...this.connections.filter(item => item.id !== saved.id), saved];
    const clean = this.configFromSaved(saved.id);
    clean.testStatus = config.testStatus;
    clean.confirmedIdentity = config.confirmedIdentity;
    clean.confirmedDatabaseName = config.confirmedDatabaseName;
    return clean;
  }

  private configFromSaved(connectionId: string): ConnectionFormState {
    const saved = this.connections.find(item => item.id === connectionId);
    return mapSavedConnection(saved) || this.emptyConnection();
  }

  private emptyConnection(): ConnectionFormState {
    return {
      mode: 'online', connectionString: '', server: '', port: 1433, database: '',
      authType: 'sql', username: '', password: '', trustServerCertificate: false, testStatus: 'idle'
    };
  }

  private copyConfig(config?: ConnectionFormState): ConnectionFormState {
    return { ...(config || this.emptyConnection()), password: '' };
  }

  private setDraft(side: Side, config: ConnectionFormState): void {
    if (side === 'source') this.sourceDraft = config;
    else this.targetDraft = config;
  }

  private setTesting(side: Side, value: boolean): void {
    if (side === 'source') this.testingSource = value;
    else this.testingTarget = value;
  }

  private isTesting(side: Side): boolean {
    return side === 'source' ? this.testingSource : this.testingTarget;
  }

  private fingerprint(): string {
    const normalize = (config: ConnectionFormState) => ({
      profileId: config.profileId || '', name: config.name || '', mode: config.mode,
      server: config.server, port: config.port, database: config.database, authType: config.authType,
      username: config.username, password: config.password || '', trustServerCertificate: config.trustServerCertificate,
      filePath: config.filePath || '', hasStoredCredential: !!config.hasStoredCredential
    });
    return JSON.stringify({ source: normalize(this.sourceDraft), target: normalize(this.targetDraft) });
  }

  private focusFirstControl(): void {
    this.dialog?.nativeElement.querySelector<HTMLElement>('select, input, button')?.focus();
  }

  private clearTemporarySecrets(): void {
    this.sourceDraft.password = '';
    this.targetDraft.password = '';
  }
}

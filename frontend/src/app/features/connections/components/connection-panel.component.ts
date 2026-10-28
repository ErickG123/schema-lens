import { Component, EventEmitter, Input, Output } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { ConnectionFormState } from '../models/connection.model';

@Component({
  selector: 'app-connection-panel',
  standalone: true,
  imports: [CommonModule, FormsModule],
  templateUrl: './connection-panel.component.html',
  styleUrl: './connection-panel.component.css'
})
export class ConnectionPanelComponent {
  @Input() side: 'source' | 'target' = 'source';
  @Input() config!: ConnectionFormState;
  @Input() loading = false;
  @Output() onTest = new EventEmitter<void>();

  get statusClass(): string {
    if (this.loading) return 'status-testing';
    if (this.config.testStatus === 'success') return 'status-success';
    if (this.config.testStatus === 'error') return 'status-error';
    return 'status-idle';
  }

  get statusText(): string {
    if (this.loading) return '● Testando...';
    if (this.config.testStatus === 'success') return '● Conectado';
    if (this.config.testStatus === 'error') return '● Erro';
    return '○ Pronto';
  }
}

import { Component, EventEmitter, inject, Output } from '@angular/core';
import { CommonModule } from '@angular/common';
import { DiffViewRow } from '../../comparison/models/comparison.model';
import { ComparisonHistoryService } from '../services/comparison-history.service';
import { TauriIpcService } from '../../../core/services/ipc.service';
import { NotificationService } from '../../notifications/services/notification.service';

@Component({
  selector: 'app-comparison-history',
  standalone: true,
  imports: [CommonModule],
  templateUrl: './comparison-history.component.html',
  styleUrl: './comparison-history.component.css'
})
export class ComparisonHistoryComponent {
  historyService = inject(ComparisonHistoryService);
  private ipc = inject(TauriIpcService);
  private notify = inject(NotificationService);

  @Output() onRestore = new EventEmitter<DiffViewRow[]>();

  async exportHtmlReport(): Promise<void> {
    const list = this.historyService.history();
    if (list.length === 0) {
      this.notify.warning('Exportação de Relatório', 'Nenhum histórico disponível para exportação.');
      return;
    }

    try {
      const filePath = await this.ipc.promptSaveFilePath('SchemaLens-Audit-Report.html', 'Relatório HTML', 'html');
      if (!filePath) {
        this.notify.info('Exportação Cancelada', 'Operação cancelada pelo usuário.');
        return;
      }

      const htmlContent = `<!DOCTYPE html>
<html lang="pt-BR">
<head>
  <meta charset="UTF-8">
  <title>SchemaLens — Relatório Técnico de Auditoria</title>
  <style>
    body { font-family: sans-serif; background: #0f172a; color: #f8fafc; padding: 30px; }
    h1 { color: #60a5fa; }
    table { width: 100%; border-collapse: collapse; margin-top: 20px; }
    th, td { border: 1px solid #334155; padding: 10px; text-align: left; }
    th { background: #1e293b; color: #94a3b8; }
    .crit { color: #ef4444; font-weight: bold; }
  </style>
</head>
<body>
  <h1>SchemaLens — Relatório Técnico de Comparação</h1>
  <p>Gerado em: ${new Date().toLocaleString()}</p>
  <p>Versão da Ferramenta: 0.1.0 • Operação Somente Leitura</p>
  <table>
    <thead>
      <tr>
        <th>Data</th>
        <th>Origem (A)</th>
        <th>Destino (B)</th>
        <th>Diferenças</th>
        <th>Críticas</th>
      </tr>
    </thead>
    <tbody>
      ${list.map(i => `
        <tr>
          <td>${i.executedAt}</td>
          <td>${i.sourceIdentifier}</td>
          <td>${i.targetIdentifier}</td>
          <td>${i.totalDifferences}</td>
          <td class="crit">${i.criticalCount}</td>
        </tr>
      `).join('')}
    </tbody>
  </table>
</body>
</html>`;

      const res = await this.ipc.saveFile(filePath, htmlContent);
      if (res.saved) {
        this.notify.success('Relatório HTML Gerado', res.message);
      } else {
        this.notify.warning('Exportação não Concluída', res.message);
      }
    } catch (e: any) {
      this.notify.error('Erro na Exportação', e?.message || String(e));
    }
  }
}

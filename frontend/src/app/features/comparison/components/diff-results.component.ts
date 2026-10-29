import { Component, EventEmitter, Input, OnChanges, Output, SimpleChanges } from '@angular/core';
import { CommonModule } from '@angular/common';
import { DiffSeverity, DiffViewRow } from '../models/comparison.model';

@Component({
  selector: 'app-diff-results',
  standalone: true,
  imports: [CommonModule],
  templateUrl: './diff-results.component.html',
  styleUrl: './diff-results.component.css'
})
export class DiffResultsComponent implements OnChanges {
  @Input() diffs: DiffViewRow[] = [];
  @Input() hasRun = false;
  @Output() onInspect = new EventEmitter<string>();
  @Output() onExportSql = new EventEmitter<DiffViewRow>();
  selectedSql: DiffViewRow | null = null;
  copiedSql = false;
  private diffStateKey = '';

  ngOnChanges(changes: SimpleChanges): void {
    if (!changes['diffs']) return;
    const nextKey = this.diffs.map(row => `${row.id || row.objectName}:${row.generatedSql || ''}`).join('\u0000');
    if (nextKey !== this.diffStateKey) {
      this.diffStateKey = nextKey;
      this.closeSqlPreview();
    }
  }

  openSqlPreview(row: DiffViewRow): void {
    this.selectedSql = row;
    this.copiedSql = false;
  }

  closeSqlPreview(): void {
    this.selectedSql = null;
  }

  async copySql(): Promise<void> {
    if (!this.selectedSql?.generatedSql || !navigator.clipboard) return;
    try {
      await navigator.clipboard.writeText(this.selectedSql.generatedSql);
      this.copiedSql = true;
    } catch {
      this.copiedSql = false;
    }
  }

  getSeverityClass(sev: DiffSeverity): string {
    if (sev === 'CRÍTICA') return 'sev-critical';
    if (sev === 'MÉDIA') return 'sev-medium';
    return 'sev-low';
  }

  countSeverity(sev: DiffSeverity): number {
    return this.diffs.filter(d => d.severity === sev).length;
  }

  countCategory(keyword: string): number {
    return this.diffs.filter(d => d.category.includes(keyword)).length;
  }
}

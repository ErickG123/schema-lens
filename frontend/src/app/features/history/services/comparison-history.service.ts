import { Injectable, signal } from '@angular/core';
import { DiffViewRow } from '../../comparison/models/comparison.model';
import { ComparisonHistoryEntry } from '../models/history.model';

@Injectable({
  providedIn: 'root'
})
export class ComparisonHistoryService {
  private readonly STORAGE_KEY = 'schemalens_comparison_history_v1';

  history = signal<ComparisonHistoryEntry[]>([]);

  constructor() {
    this.loadHistory();
  }

  loadHistory(): void {
    try {
      const data = localStorage.getItem(this.STORAGE_KEY);
      const stored = data ? JSON.parse(data) : [];
      if (!Array.isArray(stored)) {
        this.history.set([]);
        return;
      }
      const entries = stored.filter((entry: ComparisonHistoryEntry) => entry?.id !== 'hist-e2e-001');
      this.history.set(entries);
      if (entries.length !== stored.length) this.saveToStorage(entries);
    } catch (e) {
      console.warn('Could not load local comparison history:', e);
      this.history.set([]);
    }
  }
  saveComparison(source: string, target: string, diffs: DiffViewRow[], comparisonId?: string): ComparisonHistoryEntry {
    const entry: ComparisonHistoryEntry = {
      id: 'hist-' + Math.random().toString(36).substring(2, 9),
      comparisonId,
      executedAt: new Date().toISOString(),
      sourceIdentifier: source,
      targetIdentifier: target,
      totalDifferences: diffs.length,
      criticalCount: diffs.filter(d => d.severity === 'CR\u00CDTICA').length,
      mediumCount: diffs.filter(d => d.severity === 'M\u00C9DIA').length,
      lowCount: diffs.filter(d => d.severity === 'BAIXA').length,
      status: 'Completed',
      schemaLensVersion: '0.1.0',
      diffs: [...diffs]
    };

    const updated = [entry, ...this.history()];
    this.history.set(updated);
    this.saveToStorage(updated);
    return entry;
  }

  deleteEntry(id: string): void {
    const updated = this.history().filter(h => h.id !== id);
    this.history.set(updated);
    this.saveToStorage(updated);
  }

  private saveToStorage(entries: ComparisonHistoryEntry[]): void {
    try {
      localStorage.setItem(this.STORAGE_KEY, JSON.stringify(entries));
    } catch (e) {
      console.warn('Erro ao persistir histÃ³rico:', e);
    }
  }
}

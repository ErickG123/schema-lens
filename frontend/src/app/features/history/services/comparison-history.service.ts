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
      if (data) {
        this.history.set(JSON.parse(data));
      } else {
        // Mock inicial representativo persistido
        const initialEntry: ComparisonHistoryEntry = {
          id: 'hist-e2e-001',
          executedAt: new Date(Date.now() - 3600000).toISOString(),
          sourceIdentifier: '127.0.0.1,14331 / SchemaSource',
          targetIdentifier: '127.0.0.1,14332 / SchemaTarget',
          totalDifferences: 4,
          criticalCount: 1,
          mediumCount: 2,
          lowCount: 1,
          status: 'Completed',
          schemaLensVersion: '0.1.0',
          diffs: [
            {
              severity: 'MÉDIA',
              schema: 'sales',
              objectName: '[sales].[Discounts]',
              category: 'Ausente no Destino',
              valueSource: 'Table (3 cols)',
              valueTarget: 'Inexistente',
              details: 'Tabela ausente no banco alvo. Necessária criação.'
            },
            {
              severity: 'CRÍTICA',
              schema: 'dbo',
              objectName: '[dbo].[Orders].Status',
              category: 'Divergência Estrutural',
              valueSource: 'VARCHAR(20)',
              valueTarget: 'VARCHAR(10)',
              details: 'Tamanho menor no destino. Risco de truncamento de dados.'
            }
          ]
        };
        this.history.set([initialEntry]);
        this.saveToStorage([initialEntry]);
      }
    } catch (e) {
      console.warn('Erro ao carregar histórico local:', e);
    }
  }

  saveComparison(source: string, target: string, diffs: DiffViewRow[]): ComparisonHistoryEntry {
    const entry: ComparisonHistoryEntry = {
      id: 'hist-' + Math.random().toString(36).substring(2, 9),
      executedAt: new Date().toISOString(),
      sourceIdentifier: source,
      targetIdentifier: target,
      totalDifferences: diffs.length,
      criticalCount: diffs.filter(d => d.severity === 'CRÍTICA').length,
      mediumCount: diffs.filter(d => d.severity === 'MÉDIA').length,
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
      console.warn('Erro ao persistir histórico:', e);
    }
  }
}

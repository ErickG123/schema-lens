import {
  AfterViewInit,
  Component,
  ElementRef,
  EventEmitter,
  inject,
  Input,
  OnChanges,
  OnDestroy,
  Output,
  SimpleChanges,
  ViewChild
} from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import cytoscape, { Core } from 'cytoscape';
import { DiffViewRow } from '../../comparison/models/comparison.model';
import { SnapshotTable } from '../../comparison/models/snapshot.model';
import { DiagramFilterCategory, DiagramMode, DiagramStats, NodeDiffStatus, TableNodeData } from '../models/diagram.model';
import { DiagramBuilderService } from '../services/diagram-builder.service';

@Component({
  selector: 'app-er-diagram',
  standalone: true,
  imports: [CommonModule, FormsModule],
  templateUrl: './er-diagram.component.html',
  styleUrl: './er-diagram.component.css'
})
export class ErDiagramComponent implements AfterViewInit, OnChanges, OnDestroy {
  @Input() diffs: DiffViewRow[] = [];
  @Input() sourceTables: SnapshotTable[] = [];
  @Input() targetTables: SnapshotTable[] = [];
  @Input() currentMode: DiagramMode = 'diff';
  @Output() onTableSelect = new EventEmitter<string>();
  @Output() onOpenDiffs = new EventEmitter<string>();

  @ViewChild('cyContainer') cyContainer!: ElementRef<HTMLDivElement>;

  private builder = inject(DiagramBuilderService);
  private cy: Core | null = null;
  private resizeObserver: ResizeObserver | null = null;

  selectedTable: TableNodeData | null = null;
  searchQuery = '';
  filterCategory: DiagramFilterCategory = 'all';
  activeDetailsTab: 'columns' | 'comparison' | 'fks' = 'comparison';
  
  stats: DiagramStats = {
    totalTables: 0,
    identicalTables: 0,
    modifiedTables: 0,
    missingInTarget: 0,
    missingInSource: 0,
    relationshipsCount: 0
  };

  ngAfterViewInit(): void {
    // Delay microtask to ensure DOM dimension computation
    setTimeout(() => {
      this.initCytoscape();
      this.setupResizeObserver();
    }, 0);
  }

  ngOnChanges(changes: SimpleChanges): void {
    if (this.cy && (changes['diffs'] || changes['currentMode'] || changes['sourceTables'] || changes['targetTables'])) {
      this.refreshGraph();
    }
  }

  ngOnDestroy(): void {
    if (this.resizeObserver) {
      this.resizeObserver.disconnect();
      this.resizeObserver = null;
    }
    if (this.cy) {
      this.cy.destroy();
      this.cy = null;
    }
  }

  public refreshViewport(): void {
    if (!this.cy) return;
    try {
      this.cy.resize();
      if (this.cy.nodes().length > 0) {
        this.cy.fit(undefined, 30);
      }
    } catch (e) {
      console.warn('Erro ao atualizar viewport do Cytoscape:', e);
    }
  }

  private setupResizeObserver(): void {
    if (!this.cyContainer?.nativeElement || typeof ResizeObserver === 'undefined') return;
    this.resizeObserver = new ResizeObserver(() => {
      if (this.cy) {
        this.refreshViewport();
      }
    });
    this.resizeObserver.observe(this.cyContainer.nativeElement);
  }

  setMode(mode: DiagramMode): void {
    this.currentMode = mode;
    this.refreshGraph();
  }

  setFilterCategory(cat: DiagramFilterCategory): void {
    this.filterCategory = cat;
    this.refreshGraph();
  }

  private initCytoscape(): void {
    if (!this.cyContainer) return;

    const graphData = this.builder.buildModel(this.currentMode, this.diffs, this.filterCategory, this.sourceTables, this.targetTables);
    this.stats = graphData.stats;

    this.cy = cytoscape({
      container: this.cyContainer.nativeElement,
      elements: [...graphData.nodes, ...graphData.edges],
      style: [
        {
          selector: 'node',
          style: {
            'label': 'data(label)',
            'color': '#f8fafc',
            'font-size': '12px',
            'font-weight': 'bold',
            'font-family': 'ui-monospace, monospace',
            'text-valign': 'center',
            'text-halign': 'center',
            'background-color': '#1e293b',
            'border-width': 2,
            'border-color': '#3b82f6',
            'shape': 'round-rectangle',
            'width': 180,
            'height': 60,
            'text-wrap': 'wrap'
          }
        },
        {
          selector: 'node[diffStatus = "missing_in_target"]',
          style: {
            'border-color': '#f59e0b',
            'background-color': '#2a2111',
            'color': '#fcd34d'
          }
        },
        {
          selector: 'node[diffStatus = "missing_in_source"]',
          style: {
            'border-color': '#a855f7',
            'background-color': '#251533',
            'color': '#d8b4fe'
          }
        },
        {
          selector: 'node[diffStatus = "modified"]',
          style: {
            'border-color': '#ef4444',
            'background-color': '#2d1518',
            'color': '#fca5a5'
          }
        },
        {
          selector: 'node:selected',
          style: {
            'border-width': 4,
            'border-color': '#ffffff'
          }
        },
        {
          selector: 'edge',
          style: {
            'width': 2,
            'line-color': '#64748b',
            'target-arrow-color': '#38bdf8',
            'target-arrow-shape': 'triangle',
            'arrow-scale': 1.2,
            'curve-style': 'bezier',
            'label': 'data(label)',
            'color': '#f1f5f9',
            'font-size': '11px',
            'font-weight': 600,
            'font-family': 'ui-monospace, monospace',
            'text-background-color': '#0f172a',
            'text-background-opacity': 0.9,
            'text-background-padding': '3px',
            'text-background-shape': 'roundrectangle',
            'text-border-color': '#334155',
            'text-border-width': 1,
            'text-border-opacity': 0.8,
            'text-rotation': 'none',
            'text-margin-y': -2
          }
        }
      ],
      layout: {
        name: 'breadthfirst',
        directed: true,
        padding: 50,
        spacingFactor: 1.8,
        animate: false
      } as any
    });

    this.cy.on('tap', 'node', (evt) => {
      const nodeData = evt.target.data() as TableNodeData;
      this.selectedTable = nodeData;
      this.activeDetailsTab = nodeData.columnComparisons ? 'comparison' : 'columns';
      this.onTableSelect.emit(nodeData.id);
    });

    this.cy.on('tap', (evt) => {
      if (evt.target === this.cy) {
        this.selectedTable = null;
      }
    });

    // Ensure layout & viewport are properly fitted on init
    this.refreshViewport();
  }

  refreshGraph(): void {
    if (!this.cy) return;

    const graphData = this.builder.buildModel(this.currentMode, this.diffs, this.filterCategory, this.sourceTables, this.targetTables);
    this.stats = graphData.stats;
    this.selectedTable = null;
    this.cy.elements().remove();
    this.cy.add([...graphData.nodes, ...graphData.edges]);
    this.resetLayout();
  }

  fitGraph(): void {
    this.cy?.fit(undefined, 30);
  }

  resetLayout(): void {
    if (!this.cy) return;
    this.cy.layout({
      name: 'breadthfirst',
      directed: true,
      padding: 50,
      spacingFactor: 1.8,
      animate: false
    } as any).run();
  }

  onSearch(): void {
    if (!this.cy) return;
    if (!this.searchQuery.trim()) {
      this.cy.nodes().removeClass('highlighted');
      return;
    }

    const query = this.searchQuery.toLowerCase();
    const matched = this.cy.nodes().filter((n) => {
      const label = (n.data('label') || '').toLowerCase();
      return label.includes(query);
    });

    if (matched.length > 0) {
      this.cy.center(matched);
      matched.select();
      this.selectedTable = matched.first().data() as TableNodeData;
      this.activeDetailsTab = this.selectedTable.columnComparisons ? 'comparison' : 'columns';
    }
  }

  selectObject(identifier: string): void {
    if (!this.cy) return;
    const cleanId = identifier.split('.')[0] + '.' + identifier.split('.')[1];
    const node = this.cy.nodes().filter(n => n.data('id') === cleanId || identifier.includes(n.data('tableName')));
    if (node.length > 0) {
      this.cy.center(node);
      node.select();
      this.selectedTable = node.first().data() as TableNodeData;
      this.activeDetailsTab = this.selectedTable.columnComparisons ? 'comparison' : 'columns';
    }
  }

  getStatusLabel(status: NodeDiffStatus): string {
    switch (status) {
      case 'identical': return 'Idêntico em A e B';
      case 'missing_in_target': return 'Ausente no Destino (Apenas Origem A)';
      case 'missing_in_source': return 'Ausente na Origem (Exclusivo Destino B)';
      case 'modified': return 'Estrutura Divergente';
      default: return 'Indeterminado';
    }
  }

  navigateToDiffs(tableName?: string): void {
    this.onOpenDiffs.emit(tableName || this.selectedTable?.id || '');
  }
}

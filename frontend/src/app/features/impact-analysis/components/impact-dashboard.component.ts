import { Component, EventEmitter, inject, Input, OnChanges, Output } from '@angular/core';
import { CommonModule } from '@angular/common';
import { DiffViewRow } from '../../comparison/models/comparison.model';
import { MigrationPlanStep, MigrationRiskAlert } from '../models/impact.model';
import { ImpactAnalysisService } from '../services/impact-analysis.service';

@Component({
  selector: 'app-impact-dashboard',
  standalone: true,
  imports: [CommonModule],
  templateUrl: './impact-dashboard.component.html',
  styleUrl: './impact-dashboard.component.css'
})
export class ImpactDashboardComponent implements OnChanges {
  @Input() diffs: DiffViewRow[] = [];
  @Output() onNavigate = new EventEmitter<string>();

  private impactService = inject(ImpactAnalysisService);

  alerts: MigrationRiskAlert[] = [];
  planSteps: MigrationPlanStep[] = [];
  readonly planNotice = 'Nenhum plano SQL validado está disponível para estes snapshots. Revise as pré-condições dos alertas antes de preparar uma migração.';

  ngOnChanges(): void {
    this.alerts = this.impactService.analyzeRisks(this.diffs);
    this.planSteps = this.impactService.generateMigrationPlan(this.diffs);
  }

  getRiskCount(severity: string): number {
    return this.alerts.filter(a => a.severity === severity).length;
  }
}

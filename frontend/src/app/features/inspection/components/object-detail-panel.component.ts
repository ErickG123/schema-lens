import { Component, EventEmitter, Input, Output } from '@angular/core';
import { CommonModule } from '@angular/common';
import { DetailedObjectInspection } from '../models/inspection.model';

@Component({
  selector: 'app-object-detail-panel',
  standalone: true,
  imports: [CommonModule],
  templateUrl: './object-detail-panel.component.html',
  styleUrl: './object-detail-panel.component.css'
})
export class ObjectDetailPanelComponent {
  @Input() inspection: DetailedObjectInspection | null = null;
  @Output() onClose = new EventEmitter<void>();

  activeTab: 'properties' | 'sql' | 'relations' = 'properties';
}

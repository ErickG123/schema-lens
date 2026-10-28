import { Component, EventEmitter, Input, Output } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';

@Component({
  selector: 'app-comparison-toolbar',
  standalone: true,
  imports: [CommonModule, FormsModule],
  templateUrl: './comparison-toolbar.component.html',
  styleUrl: './comparison-toolbar.component.css'
})
export class ComparisonToolbarComponent {
  @Input() searchQuery = '';
  @Output() searchQueryChange = new EventEmitter<string>();

  @Input() selectedCategory = 'all';
  @Output() selectedCategoryChange = new EventEmitter<string>();

  @Input() diffCount = 0;

  @Output() onExportSql = new EventEmitter<void>();
  @Output() onExportJson = new EventEmitter<void>();
}

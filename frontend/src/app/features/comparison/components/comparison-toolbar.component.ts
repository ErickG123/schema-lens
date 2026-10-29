import { CommonModule } from '@angular/common';
import { Component, EventEmitter, Input, Output } from '@angular/core';
import { FormsModule } from '@angular/forms';
import { DifferenceFilters } from '../models/comparison.model';
import { DifferenceFilterOption, DifferenceFilterOptions } from '../services/difference-filter.service';

type FilterKey = Exclude<keyof DifferenceFilters, 'query'>;
interface FilterGroup {
  key: FilterKey;
  title: string;
  options: DifferenceFilterOption[];
}
interface FilterChip {
  key: FilterKey | 'query';
  value: string;
  label: string;
}

@Component({
  selector: 'app-comparison-toolbar',
  standalone: true,
  imports: [CommonModule, FormsModule],
  templateUrl: './comparison-toolbar.component.html',
  styleUrl: './comparison-toolbar.component.css'
})
export class ComparisonToolbarComponent {
  @Input() filters: DifferenceFilters = {
    query: '', operationTypes: [], objectTypes: [], schemas: [], riskLevels: [], scriptStatuses: [], directions: []
  };
  @Output() filtersChange = new EventEmitter<DifferenceFilters>();
  @Input() filterOptions: DifferenceFilterOptions = {
    operations: [], objectTypes: [], schemas: [], risks: [], scriptStatuses: [], directions: []
  };
  @Input() filteredCount = 0;
  @Input() totalCount = 0;
  @Input() hasRun = false;

  @Output() onExportSql = new EventEmitter<void>();
  @Output() onExportJson = new EventEmitter<void>();

  get groups(): FilterGroup[] {
    const groups: FilterGroup[] = [
      { key: 'operationTypes', title: 'Tipo de operação', options: this.filterOptions.operations },
      { key: 'objectTypes', title: 'Tipo de objeto', options: this.filterOptions.objectTypes },
      { key: 'schemas', title: 'Schema', options: this.filterOptions.schemas },
      { key: 'riskLevels', title: 'Risco', options: this.filterOptions.risks },
      { key: 'scriptStatuses', title: 'Status do SQL', options: this.filterOptions.scriptStatuses },
      { key: 'directions', title: 'Direção', options: this.filterOptions.directions }
    ];
    return groups.filter(group => group.options.length > 0);
  }

  get activeChips(): FilterChip[] {
    const chips: FilterChip[] = [];
    if (this.filters.query.trim()) chips.push({ key: 'query', value: this.filters.query, label: `Busca: ${this.filters.query.trim()}` });
    for (const group of this.groups) {
      for (const value of this.filters[group.key]) {
        const option = group.options.find(item => item.value === value);
        chips.push({ key: group.key, value, label: `${group.title}: ${option?.label || value}` });
      }
    }
    return chips;
  }

  updateQuery(query: string): void {
    this.filtersChange.emit({ ...this.filters, query });
  }

  isSelected(key: FilterKey, value: string): boolean {
    return this.filters[key].includes(value);
  }

  toggle(key: FilterKey, value: string, checked: boolean): void {
    const values = this.filters[key];
    const next = checked
      ? [...new Set([...values, value])]
      : values.filter(item => item !== value);
    this.filtersChange.emit({ ...this.filters, [key]: next });
  }

  selectAllOperations(checked: boolean): void {
    this.filtersChange.emit({
      ...this.filters,
      operationTypes: checked ? this.filterOptions.operations.map(option => option.value) : []
    });
  }

  removeChip(chip: FilterChip): void {
    if (chip.key === 'query') {
      this.updateQuery('');
      return;
    }
    this.toggle(chip.key, chip.value, false);
  }

  clearFilters(): void {
    this.filtersChange.emit({
      query: '', operationTypes: [], objectTypes: [], schemas: [], riskLevels: [], scriptStatuses: [], directions: []
    });
  }

  trackByValue(_index: number, item: DifferenceFilterOption): string {
    return item.value;
  }
}

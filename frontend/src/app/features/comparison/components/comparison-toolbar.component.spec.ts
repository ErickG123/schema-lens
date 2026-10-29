import '@angular/compiler';
import { describe, it } from 'node:test';
import assert from 'node:assert/strict';
import { ComparisonToolbarComponent } from './comparison-toolbar.component';
import { emptyDifferenceFilters } from '../services/difference-filter.service';

describe('ComparisonToolbarComponent filter controls', () => {
  it('selects all current operations and clears all filter categories', () => {
    const toolbar = new ComparisonToolbarComponent();
    toolbar.filterOptions.operations = [
      { value: 'create_table', label: 'Criar tabela', count: 2 },
      { value: 'drop_column', label: 'Remover coluna', count: 1 }
    ];
    const emissions: unknown[] = [];
    toolbar.filtersChange.subscribe(filters => emissions.push(filters));

    toolbar.selectAllOperations(true);
    assert.deepEqual((emissions[0] as ReturnType<typeof emptyDifferenceFilters>).operationTypes, ['create_table', 'drop_column']);

    toolbar.clearFilters();
    assert.deepEqual(emissions[1], emptyDifferenceFilters());
  });

  it('removes one active filter chip without clearing other criteria', () => {
    const toolbar = new ComparisonToolbarComponent();
    toolbar.filters = {
      ...emptyDifferenceFilters(), operationTypes: ['drop_column'], schemas: ['sales']
    };
    toolbar.filterOptions.operations = [{ value: 'drop_column', label: 'Remover coluna', count: 1 }];
    const emissions: unknown[] = [];
    toolbar.filtersChange.subscribe(filters => emissions.push(filters));

    const operationChip = toolbar.activeChips.find(chip => chip.key === 'operationTypes')!;
    toolbar.removeChip(operationChip);
    assert.deepEqual(emissions[0], { ...emptyDifferenceFilters(), schemas: ['sales'] });
  });
});

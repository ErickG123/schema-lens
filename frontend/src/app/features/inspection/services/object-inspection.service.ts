import { Injectable } from '@angular/core';
import { DetailedObjectInspection } from '../models/inspection.model';

@Injectable({ providedIn: 'root' })
export class ObjectInspectionService {
  inspectObject(objectIdentifier: string): DetailedObjectInspection {
    const cleaned = objectIdentifier.replace(/[\[\]]/g, '');
    const parts = cleaned.split('.');
    return {
      objectId: objectIdentifier,
      schema: parts.length > 1 ? parts[0] : 'unknown',
      name: parts[parts.length - 1] || objectIdentifier,
      objectType: 'Table',
      overallStatus: 'indeterminate',
      properties: [],
      relatedObjects: [],
      notes: 'Detalhes completos de inspeção ainda não são fornecidos pelo snapshot desta comparação.'
    };
  }
}

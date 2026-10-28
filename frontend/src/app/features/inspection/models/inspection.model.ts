export interface ObjectDetailProperty {
  name: string;
  sourceValue: string | null;
  targetValue: string | null;
  status: 'identical' | 'different' | 'missing_in_target' | 'missing_in_source';
  isDestructiveRisk?: boolean;
}

export interface RelatedObjectRef {
  objectName: string;
  schema: string;
  type: 'Table' | 'Constraint' | 'Index' | 'ForeignKey' | 'View';
  relationshipKind: 'References' | 'ReferencedBy' | 'BelongsTo' | 'DependsOn';
}

export interface DetailedObjectInspection {
  objectId: string; // [schema].[name]
  schema: string;
  name: string;
  objectType: string;
  overallStatus: 'identical' | 'missing_in_target' | 'missing_in_source' | 'modified';
  properties: ObjectDetailProperty[];
  sourceSqlDefinition?: string;
  targetSqlDefinition?: string;
  relatedObjects: RelatedObjectRef[];
  notes?: string;
}

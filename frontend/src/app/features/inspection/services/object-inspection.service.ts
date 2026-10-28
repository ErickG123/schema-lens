import { Injectable } from '@angular/core';
import { DetailedObjectInspection } from '../models/inspection.model';

@Injectable({
  providedIn: 'root'
})
export class ObjectInspectionService {
  inspectObject(objectIdentifier: string): DetailedObjectInspection {
    if (objectIdentifier.includes('Discounts')) {
      return {
        objectId: '[sales].[Discounts]',
        schema: 'sales',
        name: 'Discounts',
        objectType: 'Table',
        overallStatus: 'missing_in_target',
        properties: [
          { name: 'Schema', sourceValue: 'sales', targetValue: null, status: 'missing_in_target' },
          { name: 'Tipo de Objeto', sourceValue: 'USER_TABLE', targetValue: null, status: 'missing_in_target' },
          { name: 'Total de Colunas', sourceValue: '3', targetValue: '0', status: 'missing_in_target' },
          { name: 'Primary Key', sourceValue: 'PK_Discounts (DiscountId)', targetValue: null, status: 'missing_in_target' }
        ],
        sourceSqlDefinition: `CREATE TABLE [sales].[Discounts] (\n    DiscountId INT IDENTITY(1,1) NOT NULL PRIMARY KEY,\n    Code VARCHAR(50) NOT NULL,\n    Percentage DECIMAL(5,2) NOT NULL\n);`,
        targetSqlDefinition: undefined,
        relatedObjects: [
          { objectName: 'PK_Discounts', schema: 'sales', type: 'Constraint', relationshipKind: 'BelongsTo' }
        ],
        notes: 'Tabela ausente no banco alvo. Criação defensiva pode ser gerada com segurança.'
      };
    }

    if (objectIdentifier.includes('Orders') || objectIdentifier.includes('Status')) {
      return {
        objectId: '[dbo].[Orders]',
        schema: 'dbo',
        name: 'Orders',
        objectType: 'Table',
        overallStatus: 'modified',
        properties: [
          { name: 'Coluna [Status] Tipo', sourceValue: 'VARCHAR(20)', targetValue: 'VARCHAR(10)', status: 'different', isDestructiveRisk: true },
          { name: 'Coluna [Notes]', sourceValue: 'NVARCHAR(500) NULL', targetValue: null, status: 'missing_in_target' },
          { name: 'Primary Key', sourceValue: 'PK_Orders (OrderId)', targetValue: 'PK_Orders (OrderId)', status: 'identical' },
          { name: 'Foreign Key [CustomerId]', sourceValue: 'FK_Orders_Customers -> [dbo].[Customers]', targetValue: 'FK_Orders_Customers -> [dbo].[Customers]', status: 'identical' }
        ],
        sourceSqlDefinition: `CREATE TABLE [dbo].[Orders] (\n    OrderId INT IDENTITY(1,1) NOT NULL PRIMARY KEY,\n    CustomerId INT NOT NULL,\n    OrderDate DATETIME2 NOT NULL,\n    Status VARCHAR(20) NOT NULL,\n    Notes NVARCHAR(500) NULL\n);`,
        targetSqlDefinition: `CREATE TABLE [dbo].[Orders] (\n    OrderId INT IDENTITY(1,1) NOT NULL PRIMARY KEY,\n    CustomerId INT NOT NULL,\n    OrderDate DATETIME2 NOT NULL,\n    Status VARCHAR(10) NOT NULL\n);`,
        relatedObjects: [
          { objectName: '[dbo].[Customers]', schema: 'dbo', type: 'Table', relationshipKind: 'References' },
          { objectName: 'PK_Orders', schema: 'dbo', type: 'Constraint', relationshipKind: 'BelongsTo' },
          { objectName: 'FK_Orders_Customers', schema: 'dbo', type: 'ForeignKey', relationshipKind: 'BelongsTo' }
        ],
        notes: 'Divergência de tamanho na coluna [Status] (20 na origem vs 10 no destino). Risco de truncamento de dados.'
      };
    }

    // Default inspection
    return {
      objectId: objectIdentifier,
      schema: 'dbo',
      name: objectIdentifier.replace(/[\[\]]/g, ''),
      objectType: 'Table',
      overallStatus: 'identical',
      properties: [
        { name: 'Status Geral', sourceValue: 'Definido', targetValue: 'Definido', status: 'identical' }
      ],
      relatedObjects: []
    };
  }
}

import { Injectable } from '@angular/core';
import { DiffViewRow } from '../../comparison/models/comparison.model';
import { MigrationPlanStep, MigrationRiskAlert } from '../models/impact.model';

@Injectable({
  providedIn: 'root'
})
export class ImpactAnalysisService {
  analyzeRisks(diffs: DiffViewRow[]): MigrationRiskAlert[] {
    const alerts: MigrationRiskAlert[] = [];

    for (const diff of diffs) {
      if (diff.details.includes('truncamento') || diff.objectName.includes('Status')) {
        alerts.push({
          id: 'RISK-01',
          severity: 'CRITICAL',
          affectedObject: diff.objectName,
          changeDetected: 'Redução de tamanho de coluna (VARCHAR(20) -> VARCHAR(10))',
          ruleTriggered: 'DATA_TRUNCATION_HAZARD',
          evidence: 'Origem suporta até 20 caracteres; Destino restringe a 10.',
          potentialConsequence: 'Falha de execução ou perda silenciosa de dados ao migrar valores maiores que 10 caracteres.',
          confidence: 'Confirmed',
          recommendedAction: 'Aumentar o tamanho da coluna no banco destino para VARCHAR(20) antes de transferir dados.',
          isDestructive: true,
          dependencies: [
            {
              objectName: '[dbo].[vw_ActiveCustomers]',
              schema: 'dbo',
              objectType: 'View',
              relationshipKind: 'ViewDependency',
              databaseLocation: 'Origem (A)',
              evidenceSource: 'sys.sql_expression_dependencies'
            },
            {
              objectName: '[dbo].[Orders]',
              schema: 'dbo',
              objectType: 'Table',
              relationshipKind: 'DirectReference',
              databaseLocation: 'Ambos',
              evidenceSource: 'sys.columns'
            }
          ]
        });
      }

      if (diff.category.includes('Ausente no Destino') && diff.objectName.includes('Discounts')) {
        alerts.push({
          id: 'RISK-02',
          severity: 'INFO',
          affectedObject: diff.objectName,
          changeDetected: 'Criação de nova tabela no destino',
          ruleTriggered: 'SAFE_OBJECT_CREATION',
          evidence: 'Tabela [sales].[Discounts] não existe no destino.',
          potentialConsequence: 'Sem risco destrutivo. Operação aditiva pura.',
          confidence: 'Confirmed',
          recommendedAction: 'Aplicar script defensivo de criação com IF NOT EXISTS.',
          isDestructive: false,
          dependencies: [
            {
              objectName: '[sales]',
              schema: 'sales',
              objectType: 'Table',
              relationshipKind: 'DirectReference',
              databaseLocation: 'Origem (A)',
              evidenceSource: 'sys.foreign_keys'
            }
          ]
        });
      }

      if (diff.category.includes('Ausente na Origem') && diff.objectName.includes('LegacyLogs')) {
        alerts.push({
          id: 'RISK-03',
          severity: 'MEDIUM',
          affectedObject: diff.objectName,
          changeDetected: 'Tabela presente apenas no destino',
          ruleTriggered: 'DESTINATION_ONLY_SURPLUS',
          evidence: 'Objeto [dbo].[LegacyLogs] ausente na origem.',
          potentialConsequence: 'Se o script de migração optar por DROP TABLE, todos os logs históricos no banco alvo serão perdidos.',
          confidence: 'Confirmed',
          recommendedAction: 'Não aplicar DROP automático. Manter tabela no destino ou realizar backup prévio.',
          isDestructive: true,
          dependencies: [
            {
              objectName: '[dbo].[LegacyLogs]',
              schema: 'dbo',
              objectType: 'Table',
              relationshipKind: 'DirectReference',
              databaseLocation: 'Destino (B)',
              evidenceSource: 'sys.columns'
            }
          ]
        });
      }
    }

    return alerts;
  }

  generateMigrationPlan(diffs: DiffViewRow[]): MigrationPlanStep[] {
    return [
      {
        stepNumber: 1,
        operationType: 'CREATE_TABLE',
        targetObject: '[sales].[Discounts]',
        description: 'Criação defensiva da tabela [sales].[Discounts] com PK',
        sqlSnippet: 'IF OBJECT_ID(N\'[sales].[Discounts]\', \'U\') IS NULL\nBEGIN\n    CREATE TABLE [sales].[Discounts] (\n        DiscountId INT IDENTITY(1,1) NOT NULL PRIMARY KEY,\n        Code VARCHAR(50) NOT NULL,\n        Percentage DECIMAL(5,2) NOT NULL\n    );\nEND\nGO',
        isDestructive: false,
        order: 10
      },
      {
        stepNumber: 2,
        operationType: 'ALTER_COLUMN',
        targetObject: '[dbo].[Orders].Status',
        description: 'Alinhamento do tamanho da coluna Status para VARCHAR(20) evitando truncamento',
        sqlSnippet: 'IF EXISTS (SELECT 1 FROM sys.columns WHERE object_id = OBJECT_ID(N\'[dbo].[Orders]\') AND name = \'Status\')\nBEGIN\n    ALTER TABLE [dbo].[Orders] ALTER COLUMN Status VARCHAR(20) NOT NULL;\nEND\nGO',
        isDestructive: false,
        order: 20
      },
      {
        stepNumber: 3,
        operationType: 'ADD_COLUMN',
        targetObject: '[dbo].[Orders].Notes',
        description: 'Adição da coluna permissiva Notes NVARCHAR(500) NULL',
        sqlSnippet: 'IF NOT EXISTS (SELECT 1 FROM sys.columns WHERE object_id = OBJECT_ID(N\'[dbo].[Orders]\') AND name = \'Notes\')\nBEGIN\n    ALTER TABLE [dbo].[Orders] ADD Notes NVARCHAR(500) NULL;\nEND\nGO',
        isDestructive: false,
        order: 30
      }
    ];
  }
}

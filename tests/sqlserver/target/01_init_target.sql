-- SchemaLens Test Fixture: Banco B (Destino / Target)
IF NOT EXISTS (SELECT name FROM sys.databases WHERE name = 'SchemaTarget')
BEGIN
    CREATE DATABASE SchemaTarget;
END
GO

USE SchemaTarget;
GO

-- 1. Schema sales ausente inicialmente no Destino (diferença de schema)

-- 2. Tabela comum e sincronizada
IF OBJECT_ID('dbo.Customers', 'U') IS NOT NULL DROP TABLE dbo.Customers;
CREATE TABLE dbo.Customers (
    CustomerId INT IDENTITY(1,1) NOT NULL,
    CustomerCode VARCHAR(20) NOT NULL,
    FullName NVARCHAR(150) NOT NULL,
    Email VARCHAR(100) NULL,
    CreatedAt DATETIME2 NOT NULL DEFAULT SYSUTCDATETIME(),
    CONSTRAINT PK_Customers PRIMARY KEY CLUSTERED (CustomerId),
    CONSTRAINT UQ_Customers_Email UNIQUE (Email)
);
GO

-- 3. Tabela exclusiva do Destino (Ausente na Origem A)
IF OBJECT_ID('dbo.LegacyLogs', 'U') IS NOT NULL DROP TABLE dbo.LegacyLogs;
CREATE TABLE dbo.LegacyLogs (
    LogId INT IDENTITY(1,1) NOT NULL,
    LogMessage NVARCHAR(MAX) NOT NULL,
    LoggedAt DATETIME2 NOT NULL,
    CONSTRAINT PK_LegacyLogs PRIMARY KEY CLUSTERED (LogId)
);
GO

-- 4. Tabela com divergências intencionais com a Origem A
-- Divergências:
-- - Coluna 'Notes' ausente
-- - Coluna 'Status' é VARCHAR(10) em vez de VARCHAR(20)
-- - Índice IX_Orders_OrderDate ausente
-- - Check constraint CK_Orders_TotalAmount ausente
IF OBJECT_ID('dbo.Orders', 'U') IS NOT NULL DROP TABLE dbo.Orders;
CREATE TABLE dbo.Orders (
    OrderId INT IDENTITY(1,1) NOT NULL,
    CustomerId INT NOT NULL,
    OrderDate DATETIME2 NOT NULL DEFAULT SYSUTCDATETIME(),
    TotalAmount DECIMAL(12,2) NOT NULL,
    Status VARCHAR(10) NOT NULL DEFAULT 'PENDING',
    CONSTRAINT PK_Orders PRIMARY KEY CLUSTERED (OrderId),
    CONSTRAINT FK_Orders_Customers FOREIGN KEY (CustomerId) REFERENCES dbo.Customers(CustomerId)
);
GO

-- 5. View programável divergente
IF OBJECT_ID('dbo.vw_ActiveCustomers', 'V') IS NOT NULL DROP VIEW dbo.vw_ActiveCustomers;
GO
CREATE VIEW dbo.vw_ActiveCustomers AS
SELECT CustomerId, FullName
FROM dbo.Customers;
GO

-- 6. Stored Procedure ausente no Destino B

-- 7. Sequence ausente no Destino B

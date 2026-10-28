-- SchemaLens Test Fixture: Banco A (Origem / Source)
IF NOT EXISTS (SELECT name FROM sys.databases WHERE name = 'SchemaSource')
BEGIN
    CREATE DATABASE SchemaSource;
END
GO

USE SchemaSource;
GO

-- 1. Schema adicional
IF NOT EXISTS (SELECT * FROM sys.schemas WHERE name = 'sales')
BEGIN
    EXEC('CREATE SCHEMA sales');
END
GO

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

-- 3. Tabela exclusiva da Origem (Ausente no Destino B)
IF OBJECT_ID('sales.Discounts', 'U') IS NOT NULL DROP TABLE sales.Discounts;
CREATE TABLE sales.Discounts (
    DiscountId INT IDENTITY(1,1) NOT NULL,
    Code VARCHAR(15) NOT NULL,
    Rate DECIMAL(5,2) NOT NULL DEFAULT 0.05,
    CONSTRAINT PK_Discounts PRIMARY KEY CLUSTERED (DiscountId)
);
GO

-- 4. Tabela com divergências intencionais (Colunas, Tipos, Índices e Constraints)
IF OBJECT_ID('dbo.Orders', 'U') IS NOT NULL DROP TABLE dbo.Orders;
CREATE TABLE dbo.Orders (
    OrderId INT IDENTITY(1,1) NOT NULL,
    CustomerId INT NOT NULL,
    OrderDate DATETIME2 NOT NULL DEFAULT SYSUTCDATETIME(),
    TotalAmount DECIMAL(12,2) NOT NULL,
    Status VARCHAR(20) NOT NULL DEFAULT 'PENDING',
    Notes NVARCHAR(500) NULL, -- Presente em A, ausente em B
    CONSTRAINT PK_Orders PRIMARY KEY CLUSTERED (OrderId),
    CONSTRAINT FK_Orders_Customers FOREIGN KEY (CustomerId) REFERENCES dbo.Customers(CustomerId),
    CONSTRAINT CK_Orders_TotalAmount CHECK (TotalAmount >= 0)
);
GO

CREATE NONCLUSTERED INDEX IX_Orders_OrderDate ON dbo.Orders(OrderDate) INCLUDE (TotalAmount);
GO

-- 5. View programável
IF OBJECT_ID('dbo.vw_ActiveCustomers', 'V') IS NOT NULL DROP VIEW dbo.vw_ActiveCustomers;
GO
CREATE VIEW dbo.vw_ActiveCustomers AS
SELECT CustomerId, CustomerCode, FullName, Email
FROM dbo.Customers;
GO

-- 6. Stored Procedure programável
IF OBJECT_ID('dbo.sp_GetCustomerOrders', 'P') IS NOT NULL DROP PROCEDURE dbo.sp_GetCustomerOrders;
GO
CREATE PROCEDURE dbo.sp_GetCustomerOrders
    @CustomerId INT
AS
BEGIN
    SET NOCOUNT ON;
    SELECT OrderId, OrderDate, TotalAmount, Status
    FROM dbo.Orders
    WHERE CustomerId = @CustomerId;
END
GO

-- 7. Sequence
IF OBJECT_ID('dbo.InvoiceSequence', 'SO') IS NOT NULL DROP SEQUENCE dbo.InvoiceSequence;
CREATE SEQUENCE dbo.InvoiceSequence
    AS BIGINT
    START WITH 1000
    INCREMENT BY 1;
GO

use schema_diff::DiffEngine;
use schema_lens_domain::*;
use schema_reader::SqlFileReader;

#[test]
fn test_parser_and_diff_complex_indexes_and_constraints() {
    let sql_source = r#"
CREATE SCHEMA sales;
GO

CREATE TABLE [sales].[Orders] (
    [OrderId] INT NOT NULL,
    [CustomerId] INT NOT NULL,
    [TotalAmount] DECIMAL(18,2) NOT NULL CONSTRAINT [DF_Orders_Amount] DEFAULT (0),
    CONSTRAINT [PK_Orders] PRIMARY KEY CLUSTERED ([OrderId]),
    CONSTRAINT [UQ_Orders_Customer] UNIQUE ([CustomerId])
);
GO

CREATE NONCLUSTERED INDEX [IX_Orders_Date]
ON [sales].[Orders] ([CustomerId] ASC)
INCLUDE ([TotalAmount])
WHERE [TotalAmount] > 0;
GO
"#;

    let sql_target = r#"
CREATE SCHEMA sales;
GO

CREATE TABLE [sales].[Orders] (
    [OrderId] INT NOT NULL,
    [CustomerId] INT NOT NULL,
    [TotalAmount] DECIMAL(18,2) NOT NULL,
    CONSTRAINT [PK_Orders] PRIMARY KEY CLUSTERED ([OrderId])
);
GO
"#;

    let snap_src =
        SqlFileReader::parse_sql_content(sql_source, "source.sql").expect("Source parse failed");
    let snap_tgt =
        SqlFileReader::parse_sql_content(sql_target, "target.sql").expect("Target parse failed");

    println!("SRC TABLES: {:?}", snap_src.tables);
    println!("TGT TABLES: {:?}", snap_tgt.tables);

    let diff_result = DiffEngine::compare(&snap_src, &snap_tgt);
    println!("DIFFS (count={}):", diff_result.differences.len());
    for d in &diff_result.differences {
        println!(" - {:?} {}: {:?}", d.object_type, d.object_name, d.category);
    }

    // Esperado identificar:
    // 1. Default constraint DF_Orders_Amount ausente no Target
    // 2. Unique constraint UQ_Orders_Customer ausente no Target
    // 3. Índice IX_Orders_Date ausente no Target
    assert!(diff_result.summary.total_differences >= 3);

    let has_index_diff = diff_result.differences.iter().any(|d| {
        d.object_type == SchemaObjectType::Index
            && d.object_name.contains("IX_Orders_Date")
            && d.category == DifferenceCategory::MissingInTarget
    });
    assert!(
        has_index_diff,
        "Difference for index IX_Orders_Date not found"
    );

    let has_unique_diff = diff_result.differences.iter().any(|d| {
        d.object_type == SchemaObjectType::UniqueConstraint
            && d.category == DifferenceCategory::MissingInTarget
    });
    assert!(
        has_unique_diff,
        "Difference for unique constraint not found"
    );

    let has_default_diff = diff_result.differences.iter().any(|d| {
        d.object_type == SchemaObjectType::DefaultConstraint
            && d.category == DifferenceCategory::MissingInTarget
    });
    assert!(
        has_default_diff,
        "Difference for default constraint not found"
    );
}

#[test]
fn test_parser_indexes_attribute_divergence() {
    let sql_source = r#"
CREATE TABLE dbo.Items (
    ItemId INT NOT NULL,
    ItemName VARCHAR(100) NOT NULL
);
GO
CREATE NONCLUSTERED INDEX IX_Items_Name ON dbo.Items (ItemName) INCLUDE (ItemId);
GO
"#;

    let sql_target = r#"
CREATE TABLE dbo.Items (
    ItemId INT NOT NULL,
    ItemName VARCHAR(100) NOT NULL
);
GO
CREATE NONCLUSTERED INDEX IX_Items_Name ON dbo.Items (ItemName); -- Sem INCLUDE ItemId
GO
"#;

    let snap_src = SqlFileReader::parse_sql_content(sql_source, "src.sql").unwrap();
    let snap_tgt = SqlFileReader::parse_sql_content(sql_target, "tgt.sql").unwrap();

    let diff = DiffEngine::compare(&snap_src, &snap_tgt);
    let index_mismatch = diff
        .differences
        .iter()
        .find(|d| d.object_type == SchemaObjectType::Index)
        .unwrap();
    assert_eq!(
        index_mismatch.category,
        DifferenceCategory::PropertyMismatch
    );
    assert_eq!(index_mismatch.property_name.as_deref(), Some("definition"));
}

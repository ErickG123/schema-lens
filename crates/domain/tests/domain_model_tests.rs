use schema_lens_domain::*;

#[test]
fn test_object_id_formatting_and_equality() {
    let id1 = ObjectId::new("sales", "Orders");
    let id2 = ObjectId::new("sales", "Orders");
    let id3 = ObjectId::new("dbo", "Orders");

    assert_eq!(id1, id2);
    assert_ne!(id1, id3);
    assert_eq!(id1.full_name(), "[sales].[Orders]");
}

#[test]
fn test_difference_severity_ordering() {
    assert!(DifferenceSeverity::Critical > DifferenceSeverity::Medium);
    assert!(DifferenceSeverity::Medium > DifferenceSeverity::Low);
    assert!(DifferenceSeverity::Low > DifferenceSeverity::Info);
}

#[test]
fn test_column_metadata_equality_and_cloning() {
    let col = ColumnMetadata {
        name: "OrderId".to_string(),
        ordinal_position: 1,
        data_type: "INT".to_string(),
        max_length: 4,
        precision: 10,
        scale: 0,
        is_nullable: false,
        is_identity: true,
        identity_seed: Some(1),
        identity_increment: Some(1),
        is_computed: false,
        computed_definition: None,
        collation: None,
    };

    let cloned = col.clone();
    assert_eq!(col, cloned);
}

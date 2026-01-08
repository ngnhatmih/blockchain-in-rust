use rustchain::core::Transaction;

#[test]
fn test_create_transaction() {
    let tx = Transaction::new(
        "Alice".to_string(),
        "Bob".to_string(),
        100,
    );
    
    assert_eq!(tx.from, "Alice");
    assert_eq!(tx.to, "Bob");
    assert_eq!(tx.amount, 100);
}

#[test]
fn test_serialize_transaction() {
    let tx = Transaction::new(
        "Alice".to_string(),
        "Bob".to_string(),
        100,
    );
    
    let json = tx.serialize().expect("Failed to serialize");
    assert!(json.contains("Alice"));
    assert!(json.contains("Bob"));
    assert!(json.contains("100"));
}

#[test]
fn test_deserialize_transaction() {
    let tx1 = Transaction::new(
        "Alice".to_string(),
        "Bob".to_string(),
        100,
    );
    
    let json = tx1.serialize().expect("Could not serialize");
    let tx2 = Transaction::deserialize(&json).expect("Could not deserialize");
    
    assert_eq!(tx1, tx2);
    assert_eq!(tx1.from, tx2.from);
    assert_eq!(tx1.to, tx2.to);
    assert_eq!(tx1.amount, tx2.amount);
}

#[test]
fn test_transaction_to_bytes() {
    let tx = Transaction::new(
        "Alice".to_string(),
        "Bob".to_string(),
        100,
    );
    
    let bytes = tx.to_bytes();
    assert!(!bytes.is_empty());

    let json = String::from_utf8(bytes).expect("Invalid UTF-8");
    let tx2 = Transaction::deserialize(&json).expect("Could not deserialize");
    assert_eq!(tx, tx2);
}


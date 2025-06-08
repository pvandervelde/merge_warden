use super::*;

#[tokio::test]
async fn test_init_telemetry_invalid_connection_string() {
    println!("Calling init_telemetry with empty string");
    let result = init_telemetry("").await;
    println!("Result: {:?}", result);
    assert!(result.is_err());
}

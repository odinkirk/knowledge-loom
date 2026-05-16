// Download utilities tests
// Tests for shared download infrastructure

use knowledge_loom::download::utils::{calculate_checksum, validate_checksum};
use sha2::{Digest, Sha256};

#[test]
fn test_calculate_checksum_returns_valid_hex() {
    let data = b"test data for checksum";
    let checksum = calculate_checksum(data);
    
    // Should be 64 character hex string (SHA-256)
    assert_eq!(checksum.len(), 64);
    
    // Should only contain hex characters
    assert!(checksum.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn test_calculate_checksum_deterministic() {
    let data = b"consistent test data";
    let checksum1 = calculate_checksum(data);
    let checksum2 = calculate_checksum(data);
    
    assert_eq!(checksum1, checksum2);
}

#[test]
fn test_calculate_checksum_different_for_different_data() {
    let data1 = b"first test data";
    let data2 = b"second test data";
    
    let checksum1 = calculate_checksum(data1);
    let checksum2 = calculate_checksum(data2);
    
    assert_ne!(checksum1, checksum2);
}

#[test]
fn test_validate_checksum_match() {
    let data = b"test data to validate";
    let checksum = calculate_checksum(data);
    
    let result = validate_checksum(data, &checksum);
    assert!(result.is_ok());
}

#[test]
fn test_validate_checksum_mismatch() {
    let data = b"test data to validate";
    let wrong_checksum = "wrongchecksum12345678901234567890123456789012345678";
    
    let result = validate_checksum(data, wrong_checksum);
    assert!(result.is_err());
    
    if let Err(e) = result {
        let error_msg = e.to_string();
        assert!(error_msg.contains("Checksum mismatch"));
        assert!(error_msg.contains(wrong_checksum));
    }
}

#[test]
fn test_validate_checksum_empty_data() {
    let data = b"";
    let checksum = calculate_checksum(data);
    
    let result = validate_checksum(data, &checksum);
    assert!(result.is_ok());
}

#[test]
fn test_validate_checksum_large_data() {
    // Test with 1MB of data
    let data: Vec<u8> = (0..1_000_000).map(|i| (i % 256) as u8).collect();
    let checksum = calculate_checksum(&data);
    
    let result = validate_checksum(&data, &checksum);
    assert!(result.is_ok());
}

#[test]
fn test_checksum_matches_sha256_standard() {
    let data = b"test data";
    let checksum = calculate_checksum(data);
    
    // Verify against standard SHA-256 implementation
    let expected = Sha256::digest(data);
    let expected_hex = format!("{:x}", expected);
    
    assert_eq!(checksum, expected_hex);
}

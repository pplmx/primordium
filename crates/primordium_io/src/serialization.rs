//! Serialization utilities with robust error handling.
//!
//! Provides safe serialization/deserialization with proper error handling
//! and validation for all supported formats (JSON, rkyv, Hex).

use crate::error::{IoError, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Serializes data to JSON with error handling.
///
/// # Parameters
/// - `data`: The data to serialize
///
/// # Returns
/// JSON string on success, or `IoError::Serialization` on failure.
pub fn to_json<T>(data: &T) -> Result<String>
where
    T: Serialize,
{
    serde_json::to_string(data)
        .map_err(|e| IoError::serialization(format!("JSON serialization failed: {}", e)))
}

/// Serializes data to pretty-printed JSON.
///
/// # Parameters
/// - `data`: The data to serialize
///
/// # Returns
/// Pretty-printed JSON string on success, or error on failure.
pub fn to_json_pretty<T>(data: &T) -> Result<String>
where
    T: Serialize,
{
    serde_json::to_string_pretty(data)
        .map_err(|e| IoError::serialization(format!("JSON serialization failed: {}", e)))
}

/// Deserializes data from JSON string.
///
/// # Parameters
/// - `json`: The JSON string to deserialize
///
/// # Returns
/// Deserialized data on success, or error on failure.
pub fn from_json<T>(json: &str) -> Result<T>
where
    T: for<'de> Deserialize<'de>,
{
    if json.trim().is_empty() {
        return Err(IoError::validation("Empty JSON string"));
    }

    serde_json::from_str(json)
        .map_err(|e| IoError::serialization(format!("JSON deserialization failed: {}", e)))
}

/// Validates that a JSON string can be deserialized.
///
/// # Parameters
/// - `json`: The JSON string to validate
///
/// # Returns
/// `Ok(())` if valid, or error with details if invalid.
pub fn validate_json<T>(json: &str) -> Result<()>
where
    T: for<'de> Deserialize<'de>,
{
    let _: T = from_json(json)?;
    Ok(())
}

/// Serializes data to HexDNA format (Base16-encoded JSON).
///
/// This is the standard format for exporting/importing genotypes.
///
/// # Parameters
/// - `data`: The data to serialize
///
/// # Returns
/// HexDNA string on success, or error on failure.
pub fn to_hex_dna<T>(data: &T) -> Result<String>
where
    T: Serialize,
{
    let json = to_json(data)?;
    Ok(hex::encode(json.as_bytes()))
}

/// Deserializes data from HexDNA format.
///
/// # Parameters
/// - `hex`: The HexDNA string to deserialize
///
/// # Returns
/// Deserialized data on success, or error on failure.
pub fn from_hex_dna<T>(hex_str: &str) -> Result<T>
where
    T: for<'de> Deserialize<'de>,
{
    if hex_str.trim().is_empty() {
        return Err(IoError::validation("Empty hex string"));
    }

    let bytes = hex::decode(hex_str)
        .map_err(|e| IoError::validation(format!("Invalid hex encoding: {}", e)))?;

    if bytes.is_empty() {
        return Err(IoError::validation("Decoded hex is empty"));
    }

    let json = String::from_utf8(bytes)
        .map_err(|e| IoError::validation(format!("Invalid UTF-8 in hex: {}", e)))?;

    from_json(&json)
}

/// Checks if a string is valid HexDNA format.
///
/// # Parameters
/// - `hex`: The string to check
///
/// # Returns
/// `true` if the string appears to be valid HexDNA.
pub fn is_valid_hex_dna(hex_str: &str) -> bool {
    if hex_str.trim().is_empty() {
        return false;
    }

    // Check if it's valid hex
    if hex::decode(hex_str).is_err() {
        return false;
    }

    true
}

/// Safely writes JSON to a file with validation.
///
/// # Parameters
/// - `data`: The data to serialize
/// - `path`: The file path to write
///
/// # Returns
/// `Ok(())` on success, or error on failure.
pub fn write_json_file<T, P>(data: &T, path: P) -> Result<()>
where
    T: Serialize,
    P: AsRef<Path>,
{
    let json = to_json_pretty(data)?;
    std::fs::write(&path, json).map_err(|e| {
        IoError::FileSystem(e).with_context(format!("writing JSON to {:?}", path.as_ref()))
    })?;
    Ok(())
}

/// Safely reads JSON from a file with validation.
///
/// # Parameters
/// - `path`: The file path to read
///
/// # Returns
/// Deserialized data on success, or error on failure.
pub fn read_json_file<T, P>(path: P) -> Result<T>
where
    T: for<'de> Deserialize<'de>,
    P: AsRef<Path>,
{
    let json = std::fs::read_to_string(&path).map_err(|e| {
        IoError::FileSystem(e).with_context(format!("reading JSON from {:?}", path.as_ref()))
    })?;
    from_json(&json)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestData {
        name: String,
        value: i32,
    }

    #[test]
    fn test_json_roundtrip() {
        let data = TestData {
            name: "test".to_string(),
            value: 42,
        };

        let json = to_json(&data).unwrap();
        let restored: TestData = from_json(&json).unwrap();
        assert_eq!(data, restored);
    }

    #[test]
    fn test_empty_json_fails() {
        let result: Result<TestData> = from_json("");
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_json_fails() {
        let result: Result<TestData> = from_json("{ invalid json");
        assert!(result.is_err());
    }

    #[test]
    fn test_hex_dna_roundtrip() {
        let data = TestData {
            name: "hex_test".to_string(),
            value: 123,
        };

        let hex = to_hex_dna(&data).unwrap();
        let restored: TestData = from_hex_dna(&hex).unwrap();
        assert_eq!(data, restored);
    }

    #[test]
    fn test_empty_hex_fails() {
        let result: Result<TestData> = from_hex_dna("");
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_hex_fails() {
        let result: Result<TestData> = from_hex_dna("not_valid_hex!");
        assert!(result.is_err());
    }

    #[test]
    fn test_is_valid_hex_dna() {
        assert!(!is_valid_hex_dna(""));
        assert!(!is_valid_hex_dna("invalid!"));
        assert!(is_valid_hex_dna("7b7d")); // valid hex for "{}"
    }

    #[test]
    fn test_validate_json() {
        let valid = r#"{"name":"test","value":42}"#;
        assert!(validate_json::<TestData>(valid).is_ok());

        let invalid = r#"{"name":"test"}"#; // missing required field
        assert!(validate_json::<TestData>(invalid).is_err());
    }
}

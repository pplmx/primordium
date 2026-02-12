use primordium_core::blockchain::{BlockchainProvider, OpenTimestampsProvider};

#[tokio::test]
async fn test_opentimestamps_timeout() {
    let provider = OpenTimestampsProvider;

    let valid_hash = "deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef";

    let result = provider.anchor_hash(valid_hash).await;

    match result {
        Ok(tx_id) => {
            assert!(
                tx_id.starts_with("OTS_SUCCESS_"),
                "Should return OTS_SUCCESS prefix"
            );
            assert!(tx_id.len() > 8, "Should include transaction ID");
        }
        Err(e) => {
            let err_str = e.to_string().to_lowercase();
            assert!(
                err_str.contains("timeout")
                    || err_str.contains("error")
                    || err_str.contains("failed"),
                "Error should be timeout-related or connection failure, got: {}",
                err_str
            );
        }
    }
}

#[tokio::test]
async fn test_opentimestamps_invalid_hex() {
    let provider = OpenTimestampsProvider;

    let invalid_hash = "not-a-valid-hex-string-!!!";

    let result = provider.anchor_hash(invalid_hash).await;

    assert!(result.is_err());
    if let Err(e) = result {
        assert!(
            e.to_string().to_lowercase().contains("hex")
                || e.to_string().to_lowercase().contains("hex decoding")
                || e.to_string().to_lowercase().contains("invalid"),
            "Error should mention hex validation: {}",
            e
        );
    }
}

#[tokio::test]
async fn test_opentimestamps_empty_hash() {
    let provider = OpenTimestampsProvider;

    let empty_hash = "";

    let result = provider.anchor_hash(empty_hash).await;

    assert!(result.is_err());
    if let Err(e) = result {
        assert!(
            e.to_string().to_lowercase().contains("hex")
                || e.to_string().to_lowercase().contains("empty")
                || e.to_string().to_lowercase().contains("invalid"),
            "Error should mention validation failure: {}",
            e
        );
    }
}

#[tokio::test]
async fn test_opentimestamps_odd_length_hash() {
    let provider = OpenTimestampsProvider;

    let odd_hash = "deadbeef1";

    let result = provider.anchor_hash(odd_hash).await;

    assert!(result.is_err());
    if let Err(e) = result {
        assert!(
            e.to_string().to_lowercase().contains("hex")
                || e.to_string().to_lowercase().contains("odd"),
            "Error should mention hex validation: {}",
            e
        );
    }
}

#[tokio::test]
async fn test_placeholder_provider_no_timeout_error() {
    let provider = primordium_core::blockchain::PlaceholderProvider {
        network: "ethereum".to_string(),
    };

    let valid_hash = "0000111122223333";

    let result = provider.anchor_hash(valid_hash).await;

    assert!(result.is_ok());
    assert!(result.unwrap().starts_with("0xethereum_tx_"));
}

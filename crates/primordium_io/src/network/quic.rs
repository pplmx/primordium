use anyhow::{Context, Result};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use quinn::crypto::rustls::QuicClientConfig;
use quinn::rustls::pki_types::{CertificateDer, PrivateKeyDer, UnixTime};
use quinn::Endpoint;
use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
use rustls::pki_types::CertificateDer as RustlsCertificateDer;
use rustls::pki_types::PrivatePkcs8KeyDer;
use rustls::DigitallySignedStruct;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;

/// Data structure for secure transfer of entity authority between worlds.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AuthorityTransfer {
    /// Unique identifier of the entity being transferred.
    pub entity_id: uuid::Uuid,
    /// Hex-encoded genotype data.
    pub dna: String,
    /// Cryptographic signature ensuring data integrity and origin.
    pub signature: Vec<u8>,
    /// Creation timestamp of the transfer request.
    pub timestamp: u64,
    /// Anti-replay nonce.
    pub nonce: u64,
}

impl AuthorityTransfer {
    /// Signs the transfer data using the provided Ed25519 secret key.
    pub fn sign(&mut self, secret_key_bytes: &[u8]) -> Result<()> {
        let key_array: [u8; 32] = secret_key_bytes
            .try_into()
            .map_err(|_| anyhow::anyhow!("Invalid secret key length: expected 32 bytes"))?;
        let signing_key = SigningKey::from_bytes(&key_array);
        let message = self.get_message();
        let signature = signing_key.sign(&message);
        self.signature = signature.to_bytes().to_vec();
        Ok(())
    }

    /// Verifies the signature using the provided Ed25519 public key.
    pub fn verify(&self, public_key_bytes: &[u8]) -> bool {
        let Ok(key_array) = public_key_bytes.try_into() else {
            return false;
        };
        let Ok(verifying_key) = VerifyingKey::from_bytes(key_array) else {
            return false;
        };
        let Ok(signature) = Signature::from_slice(&self.signature) else {
            return false;
        };
        let message = self.get_message();
        verifying_key.verify(&message, &signature).is_ok()
    }

    fn get_message(&self) -> Vec<u8> {
        let mut msg = Vec::new();
        msg.extend_from_slice(self.entity_id.as_bytes());
        msg.extend_from_slice(self.dna.as_bytes());
        msg.extend_from_slice(&self.timestamp.to_le_bytes());
        msg.extend_from_slice(&self.nonce.to_le_bytes());
        msg
    }
}

/// Message types for the QUIC-based P2P protocol.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum NetMessage {
    /// Initial connection handshake.
    Handshake { version: String, magic: u64 },
    /// Discovery broadcast to announce peer existence.
    PeerAnnounce { id: uuid::Uuid, address: String },
    /// Secure entity migration.
    MigrateEntity(AuthorityTransfer),
}

/// Persistent QUIC server for handling incoming P2P connections.
pub struct QuicServer {
    endpoint: Endpoint,
}

impl QuicServer {
    /// Initialises a new QUIC server bound to the specified address.
    pub fn new(addr: SocketAddr) -> Result<Self> {
        let (endpoint, _server_cert) = make_server_endpoint(addr)?;
        Ok(Self { endpoint })
    }

    /// Asynchronously accepts a new incoming connection.
    pub async fn accept(&self) -> Option<quinn::Connection> {
        self.endpoint.accept().await?.await.ok()
    }
}

/// QUIC client for connecting to remote world instances.
pub struct QuicClient {
    endpoint: Endpoint,
}

impl QuicClient {
    /// Initialises a new QUIC client with relaxed certificate verification (self-signed).
    pub fn new() -> Result<Self> {
        let mut endpoint = Endpoint::client("0.0.0.0:0".parse()?)?;

        // Create rustls client config with custom verifier
        let crypto = rustls::ClientConfig::builder()
            .dangerous()
            .with_custom_certificate_verifier(Arc::new(SkipServerVerification))
            .with_no_client_auth();

        // Wrap in QuicClientConfig for quinn 0.11
        let client_config = QuicClientConfig::try_from(crypto)?;
        let mut client_config = quinn::ClientConfig::new(Arc::new(client_config));
        let mut transport_config = quinn::TransportConfig::default();
        transport_config.keep_alive_interval(Some(std::time::Duration::from_secs(5)));
        client_config.transport_config(Arc::new(transport_config));

        endpoint.set_default_client_config(client_config);
        Ok(Self { endpoint })
    }

    /// Attempts to connect to a remote peer.
    pub async fn connect(&self, addr: SocketAddr, server_name: &str) -> Result<quinn::Connection> {
        let connection = self
            .endpoint
            .connect(addr, server_name)?
            .await
            .context("Failed to connect")?;
        Ok(connection)
    }
}

fn make_server_endpoint(bind_addr: SocketAddr) -> Result<(Endpoint, Vec<u8>)> {
    let cert = rcgen::generate_simple_self_signed(vec!["localhost".into()])?;
    let cert_der = cert.cert.der().to_vec();

    let cert_chain = vec![CertificateDer::from(cert_der.clone())];
    let priv_key: PrivateKeyDer =
        PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(cert.signing_key.serialize_der()));

    let mut transport_config = quinn::TransportConfig::default();
    let mut server_config = quinn::ServerConfig::with_single_cert(cert_chain, priv_key)?;
    transport_config.keep_alive_interval(Some(std::time::Duration::from_secs(5)));
    server_config.transport_config(Arc::new(transport_config));

    let endpoint = Endpoint::server(server_config, bind_addr)?;
    Ok((endpoint, cert_der))
}
#[derive(Debug)]
struct SkipServerVerification;
impl ServerCertVerifier for SkipServerVerification {
    fn verify_server_cert(
        &self,
        _end_entity: &RustlsCertificateDer,
        _intermediates: &[RustlsCertificateDer],
        _server_name: &quinn::rustls::pki_types::ServerName,
        _ocsp_response: &[u8],
        _now: UnixTime,
    ) -> Result<ServerCertVerified, rustls::Error> {
        Ok(ServerCertVerified::assertion())
    }
    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &RustlsCertificateDer,
        _dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        Ok(HandshakeSignatureValid::assertion())
    }
    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &RustlsCertificateDer,
        _dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        Ok(HandshakeSignatureValid::assertion())
    }
    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        rustls::crypto::ring::default_provider()
            .signature_verification_algorithms
            .supported_schemes()
    }
}

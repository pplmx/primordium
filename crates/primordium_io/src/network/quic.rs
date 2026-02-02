use anyhow::{Context, Result};
use quinn::Endpoint;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AuthorityTransfer {
    pub entity_id: uuid::Uuid,
    pub dna: String,
    pub signature: Vec<u8>,
    pub timestamp: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum NetMessage {
    Handshake { version: String, magic: u64 },
    PeerAnnounce { id: uuid::Uuid, address: String },
    MigrateEntity(AuthorityTransfer),
}

pub struct QuicServer {
    endpoint: Endpoint,
}

impl QuicServer {
    pub fn new(addr: SocketAddr) -> Result<Self> {
        let (endpoint, _server_cert) = make_server_endpoint(addr)?;
        Ok(Self { endpoint })
    }

    pub async fn accept(&self) -> Option<quinn::Connection> {
        self.endpoint.accept().await?.await.ok()
    }
}

pub struct QuicClient {
    endpoint: Endpoint,
}

impl QuicClient {
    pub fn new() -> Result<Self> {
        let mut endpoint = Endpoint::client("0.0.0.0:0".parse()?)?;
        let crypto = rustls::ClientConfig::builder()
            .with_safe_defaults()
            .with_custom_certificate_verifier(Arc::new(SkipServerVerification))
            .with_no_client_auth();

        let mut client_config = quinn::ClientConfig::new(Arc::new(crypto));
        let mut transport_config = quinn::TransportConfig::default();
        transport_config.keep_alive_interval(Some(std::time::Duration::from_secs(5)));
        client_config.transport_config(Arc::new(transport_config));

        endpoint.set_default_client_config(client_config);
        Ok(Self { endpoint })
    }

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
    let cert_der = cert.serialize_der()?;
    let priv_key = cert.serialize_private_key_der();
    let cert_chain = vec![rustls::Certificate(cert_der.clone())];
    let priv_key = rustls::PrivateKey(priv_key);

    let mut server_config = quinn::ServerConfig::with_single_cert(cert_chain, priv_key)?;
    let mut transport_config = quinn::TransportConfig::default();
    transport_config.keep_alive_interval(Some(std::time::Duration::from_secs(5)));
    server_config.transport_config(Arc::new(transport_config));

    let endpoint = Endpoint::server(server_config, bind_addr)?;
    Ok((endpoint, cert_der))
}

struct SkipServerVerification;

impl rustls::client::ServerCertVerifier for SkipServerVerification {
    fn verify_server_cert(
        &self,
        _end_entity: &rustls::Certificate,
        _intermediates: &[rustls::Certificate],
        _server_name: &rustls::ServerName,
        _scts: &mut dyn Iterator<Item = &[u8]>,
        _ocsp_response: &[u8],
        _now: std::time::SystemTime,
    ) -> Result<rustls::client::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::ServerCertVerified::assertion())
    }
}

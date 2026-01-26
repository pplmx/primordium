use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Information about a connected peer
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct PeerInfo {
    /// Unique identifier for this peer
    pub peer_id: Uuid,
    /// Number of entities in this peer's world
    pub entity_count: usize,
    /// Total migrations sent by this peer
    pub migrations_sent: usize,
    /// Total migrations received by this peer
    pub migrations_received: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum TradeResource {
    Energy,
    Oxygen,
    SoilFertility,
    Biomass,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TradeProposal {
    pub id: Uuid,
    pub sender_id: Uuid,
    pub offer_resource: TradeResource,
    pub offer_amount: f32,
    pub request_resource: TradeResource,
    pub request_amount: f32,
}

/// Messages exchanged between Client and Server
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", content = "payload")]
pub enum NetMessage {
    /// Client -> Server: Handshake on connect
    Handshake { client_id: Uuid },
    /// Server -> Client: Acknowledge handshake
    Welcome {
        server_message: String,
        online_count: usize,
    },
    /// Bidirectional: Migrate an entity to another world
    MigrateEntity {
        dna: String,
        energy: f32,
        generation: u32,
        species_name: String, // Basic metadata
        /// NEW: Phase 45 - Validation fields
        fingerprint: String, // Hash of sender's world config
        checksum: String,     // Hash of (dna + energy + gen)
    },
    /// Server -> All Clients: Broadcast stats
    StatsUpdate {
        online_count: usize,
        total_migrations: usize,
    },
    /// Client -> Server: Announce peer presence and stats
    PeerAnnounce {
        entity_count: usize,
        migrations_sent: usize,
        migrations_received: usize,
    },
    /// Server -> All Clients: Broadcast list of connected peers
    PeerList { peers: Vec<PeerInfo> },
    /// Bidirectional: Propose a trade (broadcasted by server)
    TradeOffer(TradeProposal),
    /// Bidirectional: Accept a trade
    TradeAccept {
        proposal_id: Uuid,
        acceptor_id: Uuid,
    },
}

/// Network state visible to the simulation
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct NetworkState {
    /// List of connected peers
    pub peers: Vec<PeerInfo>,
    /// This client's ID (assigned by server)
    pub client_id: Option<Uuid>,
    /// Total migrations sent
    pub migrations_sent: usize,
    /// Total migrations received
    pub migrations_received: usize,
    /// NEW: Active trade offers in the multiverse
    pub trade_offers: Vec<TradeProposal>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_peer_info_serialization_roundtrip() {
        let peer = PeerInfo {
            peer_id: Uuid::new_v4(),
            entity_count: 42,
            migrations_sent: 10,
            migrations_received: 5,
        };

        let json = serde_json::to_string(&peer).expect("Failed to serialize PeerInfo");
        let parsed: PeerInfo = serde_json::from_str(&json).expect("Failed to deserialize PeerInfo");

        assert_eq!(peer, parsed);
    }

    #[test]
    fn test_peer_announce_serialization() {
        let msg = NetMessage::PeerAnnounce {
            entity_count: 100,
            migrations_sent: 25,
            migrations_received: 12,
        };

        let json = serde_json::to_string(&msg).expect("Failed to serialize");
        assert!(json.contains("\"type\":\"PeerAnnounce\""));
        assert!(json.contains("\"entity_count\":100"));

        let parsed: NetMessage = serde_json::from_str(&json).expect("Failed to deserialize");
        if let NetMessage::PeerAnnounce {
            entity_count,
            migrations_sent,
            migrations_received,
        } = parsed
        {
            assert_eq!(entity_count, 100);
            assert_eq!(migrations_sent, 25);
            assert_eq!(migrations_received, 12);
        } else {
            panic!("Expected PeerAnnounce message");
        }
    }

    #[test]
    fn test_peer_list_serialization() {
        let peers = vec![
            PeerInfo {
                peer_id: Uuid::new_v4(),
                entity_count: 50,
                migrations_sent: 5,
                migrations_received: 3,
            },
            PeerInfo {
                peer_id: Uuid::new_v4(),
                entity_count: 75,
                migrations_sent: 8,
                migrations_received: 2,
            },
        ];

        let msg = NetMessage::PeerList {
            peers: peers.clone(),
        };

        let json = serde_json::to_string(&msg).expect("Failed to serialize");
        assert!(json.contains("\"type\":\"PeerList\""));

        let parsed: NetMessage = serde_json::from_str(&json).expect("Failed to deserialize");
        if let NetMessage::PeerList {
            peers: parsed_peers,
        } = parsed
        {
            assert_eq!(parsed_peers.len(), 2);
            assert_eq!(parsed_peers[0].entity_count, 50);
            assert_eq!(parsed_peers[1].entity_count, 75);
        } else {
            panic!("Expected PeerList message");
        }
    }

    #[test]
    fn test_migrate_entity_serialization() {
        let msg = NetMessage::MigrateEntity {
            dna: "ABCD1234".to_string(),
            energy: 150.5,
            generation: 7,
            species_name: "TestOrganism".to_string(),
            fingerprint: "hash".to_string(),
            checksum: "sum".to_string(),
        };

        let json = serde_json::to_string(&msg).expect("Failed to serialize");
        let parsed: NetMessage = serde_json::from_str(&json).expect("Failed to deserialize");

        if let NetMessage::MigrateEntity {
            dna,
            energy,
            generation,
            species_name,
            fingerprint,
            checksum,
        } = parsed
        {
            assert_eq!(dna, "ABCD1234");
            assert!((energy - 150.5).abs() < 0.01);
            assert_eq!(generation, 7);
            assert_eq!(species_name, "TestOrganism");
            assert_eq!(fingerprint, "hash");
            assert_eq!(checksum, "sum");
        } else {
            panic!("Expected MigrateEntity message");
        }
    }

    #[test]
    fn test_handshake_serialization() {
        let client_id = Uuid::new_v4();
        let msg = NetMessage::Handshake { client_id };

        let json = serde_json::to_string(&msg).expect("Failed to serialize");
        assert!(json.contains("\"type\":\"Handshake\""));

        let parsed: NetMessage = serde_json::from_str(&json).expect("Failed to deserialize");
        if let NetMessage::Handshake {
            client_id: parsed_id,
        } = parsed
        {
            assert_eq!(parsed_id, client_id);
        } else {
            panic!("Expected Handshake message");
        }
    }

    #[test]
    fn test_empty_peer_list() {
        let msg = NetMessage::PeerList { peers: vec![] };

        let json = serde_json::to_string(&msg).expect("Failed to serialize");
        let parsed: NetMessage = serde_json::from_str(&json).expect("Failed to deserialize");

        if let NetMessage::PeerList { peers } = parsed {
            assert!(peers.is_empty());
        } else {
            panic!("Expected PeerList message");
        }
    }

    #[test]
    fn test_trade_offer_serialization() {
        let proposal = TradeProposal {
            id: Uuid::new_v4(),
            sender_id: Uuid::new_v4(),
            offer_resource: TradeResource::Energy,
            offer_amount: 500.0,
            request_resource: TradeResource::Oxygen,
            request_amount: 10.0,
        };
        let msg = NetMessage::TradeOffer(proposal.clone());

        let json = serde_json::to_string(&msg).expect("Failed to serialize");
        assert!(json.contains("\"type\":\"TradeOffer\""));
        assert!(json.contains("Energy"));

        let parsed: NetMessage = serde_json::from_str(&json).expect("Failed to deserialize");
        if let NetMessage::TradeOffer(parsed_p) = parsed {
            assert_eq!(parsed_p.id, proposal.id);
            assert_eq!(parsed_p.offer_resource, TradeResource::Energy);
        } else {
            panic!("Expected TradeOffer message");
        }
    }
}

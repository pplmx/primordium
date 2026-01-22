use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
                              // We don't send position as it spawns randomly on edge
    },
    /// Server -> All Clients: Broadcast stats
    StatsUpdate {
        online_count: usize,
        total_migrations: usize,
    },
}

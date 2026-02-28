//! Registry Client - HTTP client for Phase 70 Galactic Federation Registry API
//!
//! Provides connectivity to the central server for:
//! - Hall of Fame (top lineages by civilization level)
//! - Genome marketplace (browse/submit genomes)
//! - Seed marketplace (browse/submit simulation configs)

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use uuid::Uuid;

/// Server URL for the Registry API
const DEFAULT_REGISTRY_URL: &str = "http://localhost:3000";

/// HTTP client timeout
const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

/// A genome record from the marketplace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenomeRecord {
    pub id: String,
    pub lineage_id: Option<String>,
    pub genotype: String,
    pub author: String,
    pub name: String,
    pub description: String,
    pub tags: String,
    pub fitness_score: f64,
    pub offspring_count: u32,
    pub tick: u64,
    pub downloads: u32,
    pub created_at: String,
}

/// A seed (simulation config) record from the marketplace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeedRecord {
    pub id: String,
    pub author: String,
    pub name: String,
    pub description: String,
    pub tags: String,
    pub config_json: String,
    pub avg_tick_time: f64,
    pub max_pop: u32,
    pub performance_summary: String,
    pub downloads: u32,
    pub created_at: String,
}

/// Hall of Fame entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HallOfFameEntry {
    pub id: String,
    pub civilization_level: u32,
    pub is_extinct: bool,
}

/// Response wrapper for Hall of Fame.
#[derive(Debug, Deserialize)]
pub struct HallOfFameResponse {
    pub hall_of_fame: Vec<HallOfFameEntry>,
}

/// Response wrapper for genomes list.
#[derive(Debug, Deserialize)]
pub struct GenomesResponse {
    pub genomes: Vec<GenomeRecord>,
    #[serde(default)]
    pub error: Option<String>,
}

/// Response wrapper for seeds list.
#[derive(Debug, Deserialize)]
pub struct SeedsResponse {
    pub seeds: Vec<SeedRecord>,
    #[serde(default)]
    pub error: Option<String>,
}

/// Submit genome request payload.
#[derive(Serialize)]
struct SubmitGenomeRequest<'a> {
    genotype: &'a str,
    author: &'a str,
    name: &'a str,
    description: &'a str,
    tags: &'a str,
    lineage_id: Option<&'a str>,
    fitness_score: f64,
    offspring_count: u32,
    tick: u64,
}

/// Submit seed request payload.
#[derive(Serialize)]
struct SubmitSeedRequest<'a> {
    author: &'a str,
    name: &'a str,
    description: &'a str,
    tags: &'a str,
    config_json: &'a str,
    avg_tick_time: f64,
    max_pop: u32,
    performance_summary: &'a str,
}

/// Submit response.
#[derive(Debug, Deserialize)]
pub struct SubmitResponse {
    pub success: bool,
    pub id: Option<String>,
    #[serde(default)]
    pub error: Option<String>,
}

/// Submit genome context - bundles all submission parameters.
#[derive(Debug, Clone)]
pub struct SubmitGenomeContext<'a> {
    pub genotype: &'a str,
    pub author: &'a str,
    pub name: &'a str,
    pub description: &'a str,
    pub tags: &'a str,
    pub lineage_id: Option<&'a str>,
    pub fitness_score: f64,
    pub offspring_count: u32,
    pub tick: u64,
}

/// Submit seed context - bundles all submission parameters.
#[derive(Debug, Clone)]
pub struct SubmitSeedContext<'a> {
    pub author: &'a str,
    pub name: &'a str,
    pub description: &'a str,
    pub tags: &'a str,
    pub config_json: &'a str,
    pub avg_tick_time: f64,
    pub max_pop: u32,
    pub performance_summary: &'a str,
}

/// Connection status to the Registry server.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegistryStatus {
    /// Not connected / not configured
    Disconnected,
    /// Attempting to connect
    Connecting,
    /// Successfully connected
    Connected,
    /// Connection error
    Error(String),
}

/// Registry Client for interacting with the Phase 70 central server.
pub struct RegistryClient {
    client: Client,
    server_url: String,
    api_key: Option<String>,
    status: RegistryStatus,
}

impl RegistryClient {
    /// Create a new Registry client.
    pub fn new(server_url: Option<String>, api_key: Option<String>) -> Self {
        let client = Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            server_url: server_url.unwrap_or_else(|| DEFAULT_REGISTRY_URL.to_string()),
            api_key,
            status: RegistryStatus::Disconnected,
        }
    }

    /// Get current connection status.
    pub fn status(&self) -> &RegistryStatus {
        &self.status
    }

    /// Get the server URL.
    pub fn server_url(&self) -> &str {
        &self.server_url
    }

    /// Update the API key.
    pub fn set_api_key(&mut self, key: Option<String>) {
        self.api_key = key;
    }

    /// Query Hall of Fame from the server.
    pub async fn get_hall_of_fame(&mut self) -> Result<Vec<HallOfFameEntry>, String> {
        self.status = RegistryStatus::Connecting;

        let url = format!("{}/api/registry/hall_of_fame", self.server_url);
        let response = self.client.get(&url).send().await.map_err(|e| {
            self.status = RegistryStatus::Error(e.to_string());
            e.to_string()
        })?;

        if !response.status().is_success() {
            let error = format!("HTTP error: {}", response.status());
            self.status = RegistryStatus::Error(error.clone());
            return Err(error);
        }

        let hall_of_fame: HallOfFameResponse = response.json().await.map_err(|e| {
            let error = format!("Failed to parse response: {}", e);
            self.status = RegistryStatus::Error(error.clone());
            error
        })?;

        self.status = RegistryStatus::Connected;
        Ok(hall_of_fame.hall_of_fame)
    }

    /// Query genomes from the marketplace.
    pub async fn get_genomes(
        &mut self,
        limit: Option<u32>,
        sort_by: Option<&str>,
    ) -> Result<Vec<GenomeRecord>, String> {
        self.status = RegistryStatus::Connecting;

        let mut url = format!("{}/api/registry/genomes", self.server_url);
        let mut params = Vec::new();
        if let Some(l) = limit {
            params.push(format!("limit={}", l));
        }
        if let Some(s) = sort_by {
            params.push(format!("sort_by={}", s));
        }
        if !params.is_empty() {
            url.push('?');
            url.push_str(&params.join("&"));
        }

        let response = self.client.get(&url).send().await.map_err(|e| {
            self.status = RegistryStatus::Error(e.to_string());
            e.to_string()
        })?;

        if !response.status().is_success() {
            let error = format!("HTTP error: {}", response.status());
            self.status = RegistryStatus::Error(error.clone());
            return Err(error);
        }

        let genomes: GenomesResponse = response.json().await.map_err(|e| {
            let error = format!("Failed to parse response: {}", e);
            self.status = RegistryStatus::Error(error.clone());
            error
        })?;

        if let Some(err) = genomes.error {
            self.status = RegistryStatus::Error(err.clone());
            return Err(err);
        }

        self.status = RegistryStatus::Connected;
        Ok(genomes.genomes)
    }

    /// Submit a genome to the marketplace.
    pub async fn submit_genome(&self, ctx: SubmitGenomeContext<'_>) -> Result<String, String> {
        let url = format!("{}/api/registry/genomes", self.server_url);

        let request = SubmitGenomeRequest {
            genotype: ctx.genotype,
            author: ctx.author,
            name: ctx.name,
            description: ctx.description,
            tags: ctx.tags,
            lineage_id: ctx.lineage_id,
            fitness_score: ctx.fitness_score,
            offspring_count: ctx.offspring_count,
            tick: ctx.tick,
        };

        let mut req = self.client.post(&url).json(&request);

        if let Some(ref key) = self.api_key {
            req = req.header("Authorization", format!("Bearer {}", key));
        }

        let response = req.send().await.map_err(|e| e.to_string())?;

        if !response.status().is_success() {
            return Err(format!("HTTP error: {}", response.status()));
        }

        let result: SubmitResponse = response.json().await.map_err(|e| e.to_string())?;

        if result.success {
            Ok(result.id.unwrap_or_else(|| Uuid::new_v4().to_string()))
        } else {
            Err(result.error.unwrap_or_else(|| "Unknown error".to_string()))
        }
    }

    /// Query seeds from the marketplace.
    pub async fn get_seeds(
        &mut self,
        limit: Option<u32>,
        sort_by: Option<&str>,
    ) -> Result<Vec<SeedRecord>, String> {
        self.status = RegistryStatus::Connecting;

        let mut url = format!("{}/api/registry/seeds", self.server_url);
        let mut params = Vec::new();
        if let Some(l) = limit {
            params.push(format!("limit={}", l));
        }
        if let Some(s) = sort_by {
            params.push(format!("sort_by={}", s));
        }
        if !params.is_empty() {
            url.push('?');
            url.push_str(&params.join("&"));
        }

        let response = self.client.get(&url).send().await.map_err(|e| {
            self.status = RegistryStatus::Error(e.to_string());
            e.to_string()
        })?;

        if !response.status().is_success() {
            let error = format!("HTTP error: {}", response.status());
            self.status = RegistryStatus::Error(error.clone());
            return Err(error);
        }

        let seeds: SeedsResponse = response.json().await.map_err(|e| {
            let error = format!("Failed to parse response: {}", e);
            self.status = RegistryStatus::Error(error.clone());
            error
        })?;

        if let Some(err) = seeds.error {
            self.status = RegistryStatus::Error(err.clone());
            return Err(err);
        }

        self.status = RegistryStatus::Connected;
        Ok(seeds.seeds)
    }

    /// Submit a seed (simulation config) to the marketplace.
    pub async fn submit_seed(&self, ctx: SubmitSeedContext<'_>) -> Result<String, String> {
        let url = format!("{}/api/registry/seeds", self.server_url);

        let request = SubmitSeedRequest {
            author: ctx.author,
            name: ctx.name,
            description: ctx.description,
            tags: ctx.tags,
            config_json: ctx.config_json,
            avg_tick_time: ctx.avg_tick_time,
            max_pop: ctx.max_pop,
            performance_summary: ctx.performance_summary,
        };

        let mut req = self.client.post(&url).json(&request);

        if let Some(ref key) = self.api_key {
            req = req.header("Authorization", format!("Bearer {}", key));
        }

        let response = req.send().await.map_err(|e| e.to_string())?;

        if !response.status().is_success() {
            return Err(format!("HTTP error: {}", response.status()));
        }

        let result: SubmitResponse = response.json().await.map_err(|e| e.to_string())?;

        if result.success {
            Ok(result.id.unwrap_or_else(|| Uuid::new_v4().to_string()))
        } else {
            Err(result.error.unwrap_or_else(|| "Unknown error".to_string()))
        }
    }
}

impl Default for RegistryClient {
    fn default() -> Self {
        Self::new(None, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = RegistryClient::new(
            Some("http://test:8080".to_string()),
            Some("key".to_string()),
        );
        assert_eq!(client.server_url(), "http://test:8080");
    }

    #[test]
    fn test_default_url() {
        let client = RegistryClient::new(None, None);
        assert_eq!(client.server_url(), "http://localhost:3000");
    }
}

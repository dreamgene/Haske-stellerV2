use std::sync::Arc;

use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::layers::lightning::LightningGateway;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FundingTarget {
    pub network: String,
    pub faucet_name: String,
    pub faucet_url: String,
}

#[derive(Clone)]
pub struct FundingLayer {
    target: FundingTarget,
    gateway: Arc<dyn LightningGateway>,
    client: Client,
    api_key: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FundingResult {
    pub address: String,
    pub sats_requested: u64,
    pub faucet_txid: Option<String>,
    pub target: FundingTarget,
}

#[derive(Debug, thiserror::Error)]
pub enum FundingError {
    #[error("failed to get funding address: {0}")]
    Address(String),
    #[error("faucet request failed: {0}")]
    Http(String),
}

#[derive(Serialize)]
struct FaucetRequest<'a> {
    address: &'a str,
    amount_sats: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    api_key: Option<&'a str>,
}

#[derive(Deserialize)]
struct FaucetResponse {
    txid: Option<String>,
    #[allow(dead_code)]
    sent_sats: Option<u64>,
}

impl FundingLayer {
    /// Default to Bitcoin signet using a configurable faucet.
    /// Endpoint is configurable via `LIGHTNING_PASS_FAUCET_URL`; optional API key via `LIGHTNING_PASS_FAUCET_KEY`.
    pub fn bitcoin_signet_default(gateway: Arc<dyn LightningGateway>) -> Self {
        let faucet_url = std::env::var("LIGHTNING_PASS_FAUCET_URL")
            .unwrap_or_else(|_| "http://localhost:3002/faucet".to_string());
        let api_key = std::env::var("LIGHTNING_PASS_FAUCET_KEY").ok();

        Self {
            target: FundingTarget {
                network: "bitcoin-signet".to_string(),
                faucet_name: "Signet Faucet".to_string(),
                faucet_url,
            },
            gateway,
            client: Client::new(),
            api_key,
        }
    }

    pub fn target(&self) -> &FundingTarget {
        &self.target
    }

    /// Request faucet funds to a fresh on-chain address owned by the node.
    pub async fn request_funds(&self, sats: u64) -> Result<FundingResult, FundingError> {
        let address = self
            .gateway
            .new_onchain_address()
            .await
            .map_err(|e| FundingError::Address(e.to_string()))?;

        let body = FaucetRequest {
            address: &address,
            amount_sats: sats,
            api_key: self.api_key.as_deref(),
        };

        let resp = self
            .client
            .post(&self.target.faucet_url)
            .json(&body)
            .send()
            .await
            .map_err(|e| FundingError::Http(e.to_string()))?;

        let status = resp.status();
        let parsed: FaucetResponse = resp
            .json()
            .await
            .map_err(|e| FundingError::Http(format!("decode: {e}")))?;

        if !status.is_success() {
            return Err(FundingError::Http(format!(
                "faucet HTTP {} txid={:?}",
                status, parsed.txid
            )));
        }

        Ok(FundingResult {
            address,
            sats_requested: sats,
            faucet_txid: parsed.txid,
            target: self.target.clone(),
        })
    }
}

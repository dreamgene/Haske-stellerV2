use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AccessToken {
    pub version: u8,
    pub event_id: String,
    pub payment_hash: String,
    #[serde(default)]
    pub preimage: Option<String>,
    #[serde(default)]
    pub invoice: Option<String>,
    #[serde(default)]
    pub amount_msat: u64,
    #[serde(default)]
    pub settled_at: u64,
    pub expires_at: u64,
    pub nonce: String,
    #[serde(default)]
    pub product: String,
    #[serde(default)]
    pub rail: String,
}

impl AccessToken {
    pub fn new(
        version: u8,
        event_id: impl Into<String>,
        payment_hash: impl Into<String>,
        expires_at: u64,
        nonce: impl Into<String>,
    ) -> Self {
        Self {
            version,
            event_id: event_id.into(),
            payment_hash: payment_hash.into(),
            preimage: None,
            invoice: None,
            amount_msat: 0,
            settled_at: 0,
            expires_at,
            nonce: nonce.into(),
            product: "HASKEpay".to_string(),
            rail: "lightning".to_string(),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new_lightning(
        version: u8,
        event_id: impl Into<String>,
        payment_hash: impl Into<String>,
        preimage: Option<String>,
        invoice: Option<String>,
        amount_msat: u64,
        settled_at: u64,
        expires_at: u64,
        nonce: impl Into<String>,
    ) -> Self {
        Self {
            version,
            event_id: event_id.into(),
            payment_hash: payment_hash.into(),
            preimage,
            invoice,
            amount_msat,
            settled_at,
            expires_at,
            nonce: nonce.into(),
            product: "HASKEpay".to_string(),
            rail: "lightning".to_string(),
        }
    }

    pub fn with_random_nonce(
        version: u8,
        event_id: impl Into<String>,
        payment_hash: impl Into<String>,
        expires_at: u64,
    ) -> Self {
        let nonce: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(16)
            .map(char::from)
            .collect();

        Self::new(version, event_id, payment_hash, expires_at, nonce)
    }
}

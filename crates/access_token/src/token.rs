use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AccessToken {
    pub version: u8,
    pub event_id: String,
    pub payment_hash: String,
    pub expires_at: u64,
    pub nonce: String,
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
            expires_at,
            nonce: nonce.into(),
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

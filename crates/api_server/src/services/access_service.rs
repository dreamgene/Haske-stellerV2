use anyhow::Result;
use ed25519_dalek::SigningKey;
use rand::{distributions::Alphanumeric, Rng};

use access_token::{sign_token, signed_token_to_json, AccessToken};
use qr::{render_ascii_qr, render_png_data_url};
use shared_types::PaymentEvent;

#[derive(Clone)]
pub struct AccessArtifact {
    pub token: String,
    pub qr_png: String,
    pub qr_ascii: String,
    pub settlement_id: String,
    pub payment_hash: Option<String>,
    pub tx_hash: Option<String>,
}

#[derive(Clone)]
pub struct AccessService {
    signing_key: SigningKey,
}

impl AccessService {
    pub fn new(signing_key: SigningKey) -> Self {
        Self { signing_key }
    }

    pub fn issue_token(
        &self,
        event: &PaymentEvent,
        event_id: &str,
        expires_at: u64,
    ) -> Result<AccessArtifact> {
        let nonce: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(16)
            .map(char::from)
            .collect();

        let token = if event.rail == "lightning" {
            AccessToken::new_lightning(
                2,
                event_id,
                event.payment_hash.clone().unwrap_or_default(),
                event.preimage.clone(),
                event.amount_msat.unwrap_or_default(),
                expires_at,
                nonce,
            )
        } else {
            AccessToken::new_stellar(
                1,
                event_id,
                event.tx_hash.clone().unwrap_or_default(),
                event.source_account.clone().unwrap_or_default(),
                &event.amount,
                event
                    .asset
                    .clone()
                    .unwrap_or_else(|| event.currency.clone()),
                event.memo.clone().unwrap_or_default(),
                event.ledger_sequence.unwrap_or_default(),
                expires_at,
                nonce,
            )
        };
        let signed = sign_token(token, &self.signing_key)?;
        let token = signed_token_to_json(&signed)?;
        let qr_png = render_png_data_url(&token, 320)?;
        let qr_ascii = render_ascii_qr(&token)?;

        Ok(AccessArtifact {
            token,
            qr_png,
            qr_ascii,
            settlement_id: event.settlement_id.clone(),
            payment_hash: event.payment_hash.clone(),
            tx_hash: event.tx_hash.clone(),
        })
    }
}

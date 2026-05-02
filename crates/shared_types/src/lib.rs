use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CreatePaymentRequest {
    #[serde(default)]
    pub amount_msat: Option<u64>,
    #[serde(default)]
    pub amount_sats: Option<u64>,
    pub currency: String,
    #[serde(default)]
    pub metadata: Value,
    pub event_id: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PaymentRequest {
    pub session_id: String,
    pub rail: String,
    pub currency: String,
    #[serde(default)]
    pub amount_msat: Option<u64>,
    #[serde(default)]
    pub amount_sats: Option<u64>,
    pub payment_request: String,
    pub qr_payload: String,
    #[serde(default)]
    pub invoice: Option<String>,
    #[serde(default)]
    pub bolt11: Option<String>,
    #[serde(default)]
    pub payment_hash: Option<String>,
    #[serde(default)]
    pub metadata: Value,
    pub expires_at: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PaymentEvent {
    #[serde(default)]
    pub session_id: String,
    pub rail: String,
    pub settlement_id: String,
    #[serde(default)]
    pub payment_hash: Option<String>,
    #[serde(default)]
    pub preimage: Option<String>,
    #[serde(default)]
    pub invoice: Option<String>,
    pub currency: String,
    #[serde(default)]
    pub amount_msat: Option<u64>,
    #[serde(default)]
    pub amount_sats: Option<u64>,
    pub confirmed_at: u64,
    #[serde(default)]
    pub settled_at: Option<u64>,
    #[serde(default)]
    pub provider_metadata: Value,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PaymentStatusResponse {
    pub session_id: String,
    pub status: String,
    pub paid: bool,
    pub request_expires_at: u64,
    pub expires_at: u64,
    pub payment_request: PaymentRequest,
    pub settlement_id: Option<String>,
    pub payment_hash: Option<String>,
    pub invoice: Option<String>,
    pub bolt11: Option<String>,
    pub amount_msat: Option<u64>,
    pub qr_payload: String,
    pub access_token: Option<Value>,
    pub access_qr_png: Option<String>,
    pub access_qr_ascii: Option<String>,
}

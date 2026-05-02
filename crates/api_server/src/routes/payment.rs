use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use qr::render_png_data_url;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use shared_types::CreatePaymentRequest;

use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct CreatePaymentBody {
    pub amount: Option<String>,
    pub amount_msat: Option<u64>,
    pub amount_sats: Option<u64>,
    pub currency: Option<String>,
    pub event_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CreatePaymentResponse {
    pub session_id: String,
    pub rail: String,
    pub currency: String,
    pub amount_msat: Option<u64>,
    pub amount_sats: Option<u64>,
    pub payment_request: String,
    pub invoice: Option<String>,
    pub bolt11: Option<String>,
    pub payment_hash: Option<String>,
    pub metadata: Value,
    pub qr_payload: String,
    pub qr_png: String,
    pub status: String,
    pub request_expires_at: u64,
    pub access_token: Option<Value>,
}

pub async fn create_payment_request(
    State(state): State<AppState>,
    Json(body): Json<CreatePaymentBody>,
) -> Result<Json<CreatePaymentResponse>, StatusCode> {
    let amount_msat = body
        .amount_msat
        .or_else(|| body.amount.and_then(|amount| amount.parse::<u64>().ok()))
        .or_else(|| body.amount_sats.map(|amount| amount * 1_000))
        .unwrap_or(250_000);

    let request = CreatePaymentRequest {
        amount_msat: Some(amount_msat),
        amount_sats: (amount_msat % 1_000 == 0).then_some(amount_msat / 1_000),
        currency: body.currency.unwrap_or_else(|| "msat".to_string()),
        metadata: Value::Null,
        event_id: body.event_id.unwrap_or_else(|| "demo-event".to_string()),
    };

    let payment_request = state
        .payment_provider
        .create_payment_request(request.clone())
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;

    let session = state
        .payment_service
        .insert_session(payment_request, 15 * 60, request.event_id)
        .await;

    let payment_request = session.payment_request;
    let qr_payload = payment_request.qr_payload.clone();

    Ok(Json(CreatePaymentResponse {
        session_id: payment_request.session_id,
        rail: payment_request.rail,
        currency: payment_request.currency,
        amount_msat: payment_request.amount_msat,
        amount_sats: payment_request.amount_sats,
        payment_request: payment_request.payment_request,
        invoice: payment_request.invoice,
        bolt11: payment_request.bolt11,
        payment_hash: payment_request.payment_hash,
        metadata: payment_request.metadata,
        qr_payload: qr_payload.clone(),
        qr_png: render_png_data_url(&qr_payload, 320)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
        status: "waiting".to_string(),
        request_expires_at: payment_request.expires_at,
        access_token: None,
    }))
}

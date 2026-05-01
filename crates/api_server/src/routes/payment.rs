use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use qr::render_png_data_url;
use serde::{Deserialize, Serialize};
use shared_types::CreatePaymentRequest;

use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct CreatePaymentBody {
    pub amount: Option<String>,
    pub amount_msat: Option<u64>,
    pub currency: Option<String>,
    pub event_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CreatePaymentResponse {
    pub session_id: String,
    pub rail: String,
    pub amount: String,
    pub currency: String,
    pub amount_msat: Option<u64>,
    pub payment_request: String,
    pub invoice: Option<String>,
    pub payment_hash: Option<String>,
    pub destination: Option<String>,
    pub asset: Option<String>,
    pub memo: Option<String>,
    pub qr_payload: String,
    pub qr_png: String,
    pub request_expires_at: u64,
}

pub async fn create_payment_request(
    State(state): State<AppState>,
    Json(body): Json<CreatePaymentBody>,
) -> Result<Json<CreatePaymentResponse>, StatusCode> {
    let amount = body
        .amount
        .or_else(|| body.amount_msat.map(|amount| amount.to_string()))
        .unwrap_or_else(|| "250000".to_string());

    let request = CreatePaymentRequest {
        amount,
        currency: body.currency.unwrap_or_else(|| "msat".to_string()),
        asset: String::new(),
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
        amount: payment_request.amount,
        currency: payment_request.currency,
        amount_msat: payment_request.amount_msat,
        payment_request: payment_request.payment_request,
        invoice: payment_request.invoice,
        payment_hash: payment_request.payment_hash,
        destination: payment_request.destination,
        asset: payment_request.asset,
        memo: payment_request.memo,
        qr_payload: qr_payload.clone(),
        qr_png: render_png_data_url(&qr_payload, 320)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
        request_expires_at: payment_request.expires_at,
    }))
}

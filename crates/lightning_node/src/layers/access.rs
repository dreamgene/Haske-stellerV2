use axum::extract::{Path, State};
use axum::http::{Method, StatusCode};
use axum::routing::{get, post};
use axum::Json;
use axum::Router;
use tower_http::cors::{Any, CorsLayer};

use crate::layers::proof::{InvoiceResponse, ProofError, ProofService, StatusResponse};

#[derive(serde::Deserialize)]
struct CreateInvoiceRequest {
    amount_msat: Option<u64>,
    description: Option<String>,
}

#[derive(Clone)]
pub struct AccessState {
    proof: ProofService,
}

impl AccessState {
    pub fn new(proof: ProofService) -> Self {
        Self { proof }
    }
}

pub fn build_router(state: AccessState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST])
        .allow_headers(Any);

    Router::new()
        .route("/api/invoice", post(create_invoice))
        .route("/api/invoice/:invoice_id", get(check_invoice))
        .route("/api/invoice/:invoice_id/access", get(get_access_token))
        .with_state(state)
        .layer(cors)
}

async fn create_invoice(
    State(state): State<AccessState>,
    Json(body): Json<CreateInvoiceRequest>,
) -> Result<Json<InvoiceResponse>, StatusCode> {
    state
        .proof
        .create_invoice(body.amount_msat, body.description)
        .await
        .map(Json)
        .map_err(map_error)
}

async fn check_invoice(
    State(state): State<AccessState>,
    Path(invoice_id): Path<String>,
) -> Result<Json<StatusResponse>, StatusCode> {
    state
        .proof
        .check_invoice(&invoice_id)
        .await
        .map(Json)
        .map_err(map_error)
}

async fn get_access_token(
    State(state): State<AccessState>,
    Path(invoice_id): Path<String>,
) -> Result<Json<StatusResponse>, StatusCode> {
    state
        .proof
        .get_access_token(&invoice_id)
        .await
        .map(Json)
        .map_err(map_error)
}

fn map_error(err: ProofError) -> StatusCode {
    match err {
        ProofError::NotFound => StatusCode::NOT_FOUND,
        ProofError::Internal => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

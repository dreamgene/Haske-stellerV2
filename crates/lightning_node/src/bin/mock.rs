use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use axum::extract::{Path, State};
use axum::http::{Method, StatusCode};
use axum::routing::{get, post};
use axum::Json;
use axum::Router;
use rand::{distributions::Alphanumeric, Rng};
use serde::Serialize;
use tokio::sync::RwLock;
use tower_http::cors::{Any, CorsLayer};

const DEFAULT_AMOUNT_MSAT: u64 = 250_000;
const SETTLE_AFTER_SECS: u64 = 6;
const EXPIRY_SECS: u64 = 15 * 60;

#[derive(Clone)]
struct AppState {
    store: Arc<RwLock<HashMap<String, Session>>>,
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
struct Session {
    invoice_id: String,
    invoice: String,
    amount_msat: u64,
    created_at: u64,
    expires_at: u64,
    access_token: Option<String>,
}

#[derive(Serialize)]
struct InvoiceResponse {
    invoice_id: String,
    amount_msat: u64,
    invoice: String,
    invoice_qr_png: String,
    invoice_qr_ascii: String,
    expires_at: u64,
}

#[derive(Serialize)]
struct StatusResponse {
    status: String,
    expires_at: u64,
    access_token: Option<String>,
    access_qr_png: Option<String>,
    access_qr_ascii: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let bind_addr =
        std::env::var("LIGHTNING_NODE_BIND").unwrap_or_else(|_| "0.0.0.0:3000".to_string());
    let state = AppState {
        store: Arc::new(RwLock::new(HashMap::new())),
    };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST])
        .allow_headers(Any);

    let app = Router::new()
        .route("/api/invoice", post(create_invoice))
        .route("/api/invoice/:invoice_id", get(check_invoice))
        .route("/api/invoice/:invoice_id/access", get(get_access_token))
        .route("/health", get(health))
        .with_state(state)
        .layer(cors);

    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    println!("mock backend listening on http://{}", bind_addr);
    axum::serve(listener, app.into_make_service()).await?;
    Ok(())
}

async fn health() -> &'static str {
    "ok"
}

async fn create_invoice(
    State(state): State<AppState>,
) -> Result<Json<InvoiceResponse>, StatusCode> {
    let now = now_secs();
    let invoice_id = random_id(12);
    let invoice = format!("lnbc{}{}", DEFAULT_AMOUNT_MSAT, random_id(18));
    let expires_at = now + EXPIRY_SECS;

    let session = Session {
        invoice_id: invoice_id.clone(),
        invoice: invoice.clone(),
        amount_msat: DEFAULT_AMOUNT_MSAT,
        created_at: now,
        expires_at,
        access_token: None,
    };

    let mut store = state.store.write().await;
    store.insert(invoice_id.clone(), session);

    Ok(Json(InvoiceResponse {
        invoice_id,
        amount_msat: DEFAULT_AMOUNT_MSAT,
        invoice,
        invoice_qr_png: String::new(),
        invoice_qr_ascii: String::new(),
        expires_at,
    }))
}

async fn check_invoice(
    State(state): State<AppState>,
    Path(invoice_id): Path<String>,
) -> Result<Json<StatusResponse>, StatusCode> {
    let mut store = state.store.write().await;
    let session = store.get_mut(&invoice_id).ok_or(StatusCode::NOT_FOUND)?;

    let settled = now_secs().saturating_sub(session.created_at) >= SETTLE_AFTER_SECS;
    if settled && session.access_token.is_none() {
        session.access_token = Some(format!("access_{}", random_id(24)));
    }

    Ok(Json(StatusResponse {
        status: if settled { "settled" } else { "pending" }.to_string(),
        expires_at: session.expires_at,
        access_token: session.access_token.clone(),
        access_qr_png: None,
        access_qr_ascii: None,
    }))
}

async fn get_access_token(
    State(state): State<AppState>,
    Path(invoice_id): Path<String>,
) -> Result<Json<StatusResponse>, StatusCode> {
    let mut store = state.store.write().await;
    let session = store.get_mut(&invoice_id).ok_or(StatusCode::NOT_FOUND)?;

    let settled = now_secs().saturating_sub(session.created_at) >= SETTLE_AFTER_SECS;
    if settled && session.access_token.is_none() {
        session.access_token = Some(format!("access_{}", random_id(24)));
    }

    Ok(Json(StatusResponse {
        status: if settled { "settled" } else { "pending" }.to_string(),
        expires_at: session.expires_at,
        access_token: session.access_token.clone(),
        access_qr_png: None,
        access_qr_ascii: None,
    }))
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::from_secs(0))
        .as_secs()
}

fn random_id(len: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(len)
        .map(char::from)
        .collect::<String>()
        .to_lowercase()
}

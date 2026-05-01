use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use ed25519_dalek::SigningKey;
use rand::{distributions::Alphanumeric, Rng};
use serde::Serialize;
use tokio::sync::RwLock;

use access_token::{sign_compact_payload, CompactAccessPayload};

use crate::layers::lightning::{LightningGateway, PaymentEvent, PaymentEventSink};
use crate::qr::{render_ascii_qr, render_png_data_url};

const DEFAULT_EXPIRY_SECS: u64 = 15 * 60;
const DEMO_EXPIRY_SECS: u64 = 2 * 60;
const DEFAULT_AMOUNT_MSAT: u64 = 250_000;

#[derive(Clone)]
pub struct AppConfig {
    pub demo_mode: bool,
    pub event_id: String,
    pub invoice_expiry_secs: u64,
    pub amount_msat: u64,
}

#[derive(Clone)]
pub struct ProofService {
    inner: Arc<ProofServiceInner>,
}

struct ProofServiceInner {
    gateway: Arc<dyn LightningGateway>,
    keypair: Arc<SigningKey>,
    config: AppConfig,
    store: AccessStore,
}

#[derive(Clone)]
struct AccessStore {
    sessions: Arc<RwLock<HashMap<String, AccessSession>>>,
    payment_index: Arc<RwLock<HashMap<String, String>>>,
}

impl AccessStore {
    fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            payment_index: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[derive(Clone, Debug)]
enum InvoiceStatus {
    Pending,
    Settled,
}

#[derive(Clone, Debug)]
struct AccessSession {
    invoice_id: String,
    payment_hash: String,
    amount_msat: u64,
    expires_at: u64,
    #[allow(dead_code)]
    invoice_qr_png: String,
    access_qr_ascii: Option<String>,
    status: InvoiceStatus,
    access_token: Option<String>,
    access_qr_png: Option<String>,
}

#[derive(Serialize)]
pub struct InvoiceResponse {
    pub invoice_id: String,
    pub amount_msat: u64,
    pub invoice: String,
    pub invoice_qr_png: String,
    pub invoice_qr_ascii: String,
    pub expires_at: u64,
}

#[derive(Serialize)]
pub struct StatusResponse {
    pub status: String,
    pub expires_at: u64,
    pub access_token: Option<String>,
    pub access_qr_png: Option<String>,
    pub access_qr_ascii: Option<String>,
}

#[derive(Debug)]
pub enum ProofError {
    NotFound,
    Internal,
}

impl std::fmt::Display for ProofError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProofError::NotFound => write!(f, "invoice not found"),
            ProofError::Internal => write!(f, "internal error"),
        }
    }
}

impl std::error::Error for ProofError {}

impl ProofService {
    pub fn new(gateway: Arc<dyn LightningGateway>, keypair: SigningKey, config: AppConfig) -> Self {
        Self {
            inner: Arc::new(ProofServiceInner {
                gateway,
                keypair: Arc::new(keypair),
                config,
                store: AccessStore::new(),
            }),
        }
    }

    pub fn config(&self) -> &AppConfig {
        &self.inner.config
    }

    pub async fn create_invoice(
        &self,
        amount_override_msat: Option<u64>,
        description: Option<String>,
    ) -> Result<InvoiceResponse, ProofError> {
        let expiry = self.inner.config.invoice_expiry_secs;
        let expiry_secs = u32::try_from(expiry).map_err(|_| ProofError::Internal)?;
        let amount_msat = amount_override_msat.unwrap_or(self.inner.config.amount_msat);
        let memo = description.unwrap_or_else(|| "HASKEpay access".to_string());

        let invoice = self
            .inner
            .gateway
            .create_invoice(amount_msat, &memo, expiry_secs)
            .await
            .map_err(|_| ProofError::Internal)?;

        let invoice_id = random_id();
        let payment_hash = invoice.payment_hash;
        let expires_at = now_secs() + expiry;
        let invoice_qr_png =
            render_png_data_url(&invoice.invoice, 320).map_err(|_| ProofError::Internal)?;
        let invoice_qr_ascii =
            render_ascii_qr(&invoice.invoice).map_err(|_| ProofError::Internal)?;

        let session = AccessSession {
            invoice_id: invoice_id.clone(),
            payment_hash: payment_hash.clone(),
            amount_msat,
            expires_at,
            invoice_qr_png: invoice_qr_png.clone(),
            access_qr_ascii: None,
            status: InvoiceStatus::Pending,
            access_token: None,
            access_qr_png: None,
        };

        {
            let mut store = self.inner.store.sessions.write().await;
            store.insert(invoice_id.clone(), session);
        }
        {
            let mut index = self.inner.store.payment_index.write().await;
            index.insert(payment_hash, invoice_id.clone());
        }

        Ok(InvoiceResponse {
            invoice_id,
            amount_msat,
            invoice: invoice.invoice,
            invoice_qr_png,
            invoice_qr_ascii,
            expires_at,
        })
    }

    pub async fn check_invoice(&self, invoice_id: &str) -> Result<StatusResponse, ProofError> {
        let store = self.inner.store.sessions.read().await;
        let session = store.get(invoice_id).ok_or(ProofError::NotFound)?;

        let status = match session.status {
            InvoiceStatus::Pending => "pending",
            InvoiceStatus::Settled => "settled",
        };

        Ok(StatusResponse {
            status: status.to_string(),
            expires_at: session.expires_at,
            access_token: session.access_token.clone(),
            access_qr_png: session.access_qr_png.clone(),
            access_qr_ascii: session.access_qr_ascii.clone(),
        })
    }

    pub async fn get_access_token(&self, invoice_id: &str) -> Result<StatusResponse, ProofError> {
        self.check_invoice(invoice_id).await
    }

    pub async fn handle_payment(&self, event: PaymentEvent) -> Result<(), ProofError> {
        let invoice_id = {
            let index = self.inner.store.payment_index.read().await;
            index.get(&event.payment_hash).cloned()
        };

        let Some(invoice_id) = invoice_id else {
            return Ok(());
        };

        let mut store = self.inner.store.sessions.write().await;
        let Some(session) = store.get_mut(&invoice_id) else {
            return Ok(());
        };

        if matches!(session.status, InvoiceStatus::Settled) {
            return Ok(());
        }

        let payload = CompactAccessPayload {
            payment_hash: event.payment_hash.clone(),
            expires_at: session.expires_at,
            event_id: self.inner.config.event_id.clone(),
        };

        let token = sign_compact_payload(&payload, &self.inner.keypair)
            .map_err(|_| ProofError::Internal)?;
        let qr_png = render_png_data_url(&token, 320).map_err(|_| ProofError::Internal)?;
        let qr_ascii = render_ascii_qr(&token).map_err(|_| ProofError::Internal)?;

        session.status = InvoiceStatus::Settled;
        session.access_token = Some(token);
        session.access_qr_png = Some(qr_png);
        session.access_qr_ascii = Some(qr_ascii);

        if self.inner.config.demo_mode {
            println!(
                "[demo] access issued invoice_id={} hash={} amount_msat={} expires_at={}",
                session.invoice_id, session.payment_hash, session.amount_msat, session.expires_at
            );
        }

        Ok(())
    }
}

#[derive(Clone)]
pub struct PaymentEventHandler {
    proof: ProofService,
}

impl PaymentEventHandler {
    pub fn new(proof: ProofService) -> Self {
        Self { proof }
    }
}

impl PaymentEventSink for PaymentEventHandler {
    fn handle(&self, event: PaymentEvent) -> anyhow::Result<()> {
        let proof = self.proof.clone();
        tokio::spawn(async move {
            if let Err(err) = proof.handle_payment(event).await {
                eprintln!("payment handling failed: {}", err);
            }
        });
        Ok(())
    }
}

pub fn demo_config() -> AppConfig {
    AppConfig {
        demo_mode: true,
        event_id: "demo-event".to_string(),
        invoice_expiry_secs: DEMO_EXPIRY_SECS,
        amount_msat: DEFAULT_AMOUNT_MSAT,
    }
}

pub fn default_config(event_id: String) -> AppConfig {
    AppConfig {
        demo_mode: false,
        event_id,
        invoice_expiry_secs: DEFAULT_EXPIRY_SECS,
        amount_msat: DEFAULT_AMOUNT_MSAT,
    }
}

fn random_id() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(12)
        .map(char::from)
        .collect()
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use payment_core::PaymentProvider;
use rand::{distributions::Alphanumeric, Rng};
use serde_json::{json, Value};
use shared_types::{CreatePaymentRequest, PaymentEvent, PaymentRequest};
use tokio::sync::RwLock;

const DEFAULT_REQUEST_EXPIRY_SECS: u64 = 15 * 60;

#[derive(Clone, Debug)]
pub struct LightningAdapterConfig {
    pub request_expiry_secs: u64,
}

impl Default for LightningAdapterConfig {
    fn default() -> Self {
        Self {
            request_expiry_secs: DEFAULT_REQUEST_EXPIRY_SECS,
        }
    }
}

#[derive(Clone, Debug)]
pub struct CreateInvoiceRequest {
    pub session_id: String,
    pub amount_msat: u64,
    pub description: String,
    pub expires_at: u64,
}

#[derive(Clone, Debug)]
pub struct CreatedInvoice {
    pub invoice: String,
    pub payment_hash: String,
    pub amount_msat: u64,
    pub provider_metadata: Value,
}

#[derive(Clone, Debug)]
pub struct InvoiceSettlement {
    pub settled: bool,
    pub preimage: Option<String>,
    pub settled_at: Option<u64>,
    pub provider_metadata: Value,
}

#[async_trait]
pub trait LightningInvoiceProvider: Send + Sync {
    async fn create_invoice(&self, request: CreateInvoiceRequest) -> Result<CreatedInvoice>;

    async fn check_invoice_settlement(
        &self,
        session_id: &str,
        invoice: &str,
        payment_hash: &str,
    ) -> Result<InvoiceSettlement>;
}

#[derive(Clone)]
pub struct LightningAdapter {
    provider: Arc<dyn LightningInvoiceProvider>,
    config: LightningAdapterConfig,
}

impl LightningAdapter {
    pub fn new(
        provider: Arc<dyn LightningInvoiceProvider>,
        config: LightningAdapterConfig,
    ) -> Self {
        Self { provider, config }
    }

    pub fn mock(settle_after_secs: Option<u64>) -> Self {
        Self::new(
            Arc::new(MockLightningProvider::new(MockLightningConfig {
                settle_after_secs,
            })),
            LightningAdapterConfig::default(),
        )
    }
}

#[async_trait]
impl PaymentProvider for LightningAdapter {
    async fn create_payment_request(
        &self,
        request: CreatePaymentRequest,
    ) -> Result<PaymentRequest> {
        let amount_msat = request
            .amount
            .parse::<u64>()
            .map_err(|_| anyhow!("amount must be millisatoshis for Lightning requests"))?;
        let session_id = random_id(16);
        let expires_at = now_secs() + self.config.request_expiry_secs;
        let invoice = self
            .provider
            .create_invoice(CreateInvoiceRequest {
                session_id: session_id.clone(),
                amount_msat,
                description: request.event_id,
                expires_at,
            })
            .await?;

        Ok(PaymentRequest {
            session_id,
            rail: "lightning".to_string(),
            amount: invoice.amount_msat.to_string(),
            currency: "msat".to_string(),
            amount_msat: Some(invoice.amount_msat),
            payment_request: invoice.invoice.clone(),
            qr_payload: invoice.invoice.clone(),
            invoice: Some(invoice.invoice),
            payment_hash: Some(invoice.payment_hash),
            destination: None,
            asset: None,
            memo: None,
            expires_at,
        })
    }

    async fn find_confirmed_payment(
        &self,
        request: &PaymentRequest,
    ) -> Result<Option<PaymentEvent>> {
        let Some(invoice) = request.invoice.as_deref() else {
            return Ok(None);
        };
        let Some(payment_hash) = request.payment_hash.as_deref() else {
            return Ok(None);
        };

        let settlement = self
            .provider
            .check_invoice_settlement(&request.session_id, invoice, payment_hash)
            .await?;

        if !settlement.settled {
            return Ok(None);
        }

        let settled_at = settlement.settled_at.unwrap_or_else(now_secs);

        Ok(Some(PaymentEvent {
            session_id: request.session_id.clone(),
            rail: "lightning".to_string(),
            settlement_id: payment_hash.to_string(),
            payment_hash: Some(payment_hash.to_string()),
            preimage: settlement.preimage,
            invoice: Some(invoice.to_string()),
            tx_hash: None,
            source_account: None,
            destination_account: None,
            amount: request.amount.clone(),
            currency: "msat".to_string(),
            amount_msat: request.amount_msat,
            asset: None,
            memo: None,
            ledger_sequence: None,
            confirmed_at: settled_at,
            settled_at: Some(settled_at),
            provider_metadata: settlement.provider_metadata,
        }))
    }
}

#[derive(Clone, Debug)]
pub struct MockLightningConfig {
    pub settle_after_secs: Option<u64>,
}

#[derive(Clone)]
pub struct MockLightningProvider {
    config: MockLightningConfig,
    invoices: Arc<RwLock<HashMap<String, MockInvoiceRecord>>>,
}

#[derive(Clone, Debug)]
struct MockInvoiceRecord {
    session_id: String,
    invoice: String,
    payment_hash: String,
    amount_msat: u64,
    created_at: u64,
}

impl MockLightningProvider {
    pub fn new(config: MockLightningConfig) -> Self {
        Self {
            config,
            invoices: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl LightningInvoiceProvider for MockLightningProvider {
    async fn create_invoice(&self, request: CreateInvoiceRequest) -> Result<CreatedInvoice> {
        let payment_hash = random_hex(32);
        let invoice = format!("lnbc{}n1{}", request.amount_msat, random_id(96));
        let record = MockInvoiceRecord {
            session_id: request.session_id,
            invoice: invoice.clone(),
            payment_hash: payment_hash.clone(),
            amount_msat: request.amount_msat,
            created_at: now_secs(),
        };

        self.invoices
            .write()
            .await
            .insert(payment_hash.clone(), record);

        Ok(CreatedInvoice {
            invoice,
            payment_hash,
            amount_msat: request.amount_msat,
            provider_metadata: json!({ "provider": "mock" }),
        })
    }

    async fn check_invoice_settlement(
        &self,
        session_id: &str,
        invoice: &str,
        payment_hash: &str,
    ) -> Result<InvoiceSettlement> {
        let Some(settle_after_secs) = self.config.settle_after_secs else {
            return Ok(InvoiceSettlement {
                settled: false,
                preimage: None,
                settled_at: None,
                provider_metadata: json!({ "provider": "mock" }),
            });
        };

        let record = {
            let invoices = self.invoices.read().await;
            invoices.get(payment_hash).cloned()
        };

        let Some(record) = record else {
            return Ok(InvoiceSettlement {
                settled: false,
                preimage: None,
                settled_at: None,
                provider_metadata: json!({ "provider": "mock", "reason": "unknown_invoice" }),
            });
        };

        if record.session_id != session_id || record.invoice != invoice {
            return Ok(InvoiceSettlement {
                settled: false,
                preimage: None,
                settled_at: None,
                provider_metadata: json!({ "provider": "mock", "reason": "invoice_mismatch" }),
            });
        }

        let elapsed = now_secs().saturating_sub(record.created_at);
        let settled = elapsed >= settle_after_secs;
        let settled_at = settled.then_some(record.created_at + settle_after_secs);

        Ok(InvoiceSettlement {
            settled,
            preimage: settled.then(|| format!("mock_preimage_{}", record.payment_hash)),
            settled_at,
            provider_metadata: json!({
                "provider": "mock",
                "amount_msat": record.amount_msat,
                "settle_after_secs": settle_after_secs
            }),
        })
    }
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
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

fn random_hex(bytes: usize) -> String {
    let mut rng = rand::thread_rng();
    (0..bytes)
        .map(|_| format!("{:02x}", rng.gen::<u8>()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn mock_provider_creates_lightning_payment_request() {
        let adapter = LightningAdapter::mock(None);
        let request = adapter
            .create_payment_request(CreatePaymentRequest {
                amount: "250000".to_string(),
                currency: "msat".to_string(),
                asset: String::new(),
                event_id: "test-event".to_string(),
            })
            .await
            .unwrap();

        assert_eq!(request.rail, "lightning");
        assert_eq!(request.amount_msat, Some(250_000));
        assert!(request.invoice.unwrap().starts_with("lnbc250000n1"));
        assert_eq!(request.payment_hash.unwrap().len(), 64);
    }

    #[tokio::test]
    async fn mock_provider_returns_no_settlement_when_disabled() {
        let adapter = LightningAdapter::mock(None);
        let request = adapter
            .create_payment_request(CreatePaymentRequest {
                amount: "250000".to_string(),
                currency: "msat".to_string(),
                asset: String::new(),
                event_id: "test-event".to_string(),
            })
            .await
            .unwrap();

        let event = adapter.find_confirmed_payment(&request).await.unwrap();
        assert!(event.is_none());
    }

    #[tokio::test]
    async fn mock_provider_returns_normalized_settlement_event() {
        let adapter = LightningAdapter::mock(Some(0));
        let request = adapter
            .create_payment_request(CreatePaymentRequest {
                amount: "250000".to_string(),
                currency: "msat".to_string(),
                asset: String::new(),
                event_id: "test-event".to_string(),
            })
            .await
            .unwrap();

        let event = adapter
            .find_confirmed_payment(&request)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(event.session_id, request.session_id);
        assert_eq!(event.payment_hash, request.payment_hash);
        assert_eq!(event.invoice, request.invoice);
        assert_eq!(event.amount_msat, Some(250_000));
        assert!(event.preimage.is_some());
        assert!(event.settled_at.is_some());
        assert_eq!(event.provider_metadata["provider"], "mock");
    }
}

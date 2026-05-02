use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use async_trait::async_trait;
use breez_sdk_core::{
    BreezEvent, BreezServices, EventListener, PaymentStatus, ReceiveOnchainRequest,
    ReceivePaymentRequest, SendPaymentRequest,
};

#[derive(Debug, Clone)]
pub struct LightningInvoice {
    pub invoice: String,
    pub payment_hash: String,
}

#[async_trait]
pub trait LightningGateway: Send + Sync {
    async fn create_invoice(
        &self,
        amount_msat: u64,
        description: &str,
        expiry_secs: u32,
    ) -> Result<LightningInvoice>;
    async fn new_onchain_address(&self) -> Result<String>;
    async fn pay_invoice(&self, bolt11: &str, amount_msat: Option<u64>) -> Result<PaymentStatus>;
}

#[derive(Clone)]
pub struct BreezGateway {
    services: Arc<BreezServices>,
}

impl BreezGateway {
    pub fn new(services: Arc<BreezServices>) -> Self {
        Self { services }
    }
}

#[async_trait]
impl LightningGateway for BreezGateway {
    async fn create_invoice(
        &self,
        amount_msat: u64,
        description: &str,
        expiry_secs: u32,
    ) -> Result<LightningInvoice> {
        let req = ReceivePaymentRequest {
            amount_msat,
            description: description.to_string(),
            expiry: Some(expiry_secs.into()),
            ..Default::default()
        };
        let invoice = self.services.receive_payment(req).await?;

        Ok(LightningInvoice {
            invoice: invoice.ln_invoice.bolt11.clone(),
            payment_hash: invoice.ln_invoice.payment_hash.clone(),
        })
    }

    async fn new_onchain_address(&self) -> Result<String> {
        let swap = self
            .services
            .receive_onchain(ReceiveOnchainRequest {
                opening_fee_params: None,
            })
            .await?;
        Ok(swap.bitcoin_address)
    }

    async fn pay_invoice(&self, bolt11: &str, amount_msat: Option<u64>) -> Result<PaymentStatus> {
        let resp = self
            .services
            .send_payment(SendPaymentRequest {
                bolt11: bolt11.to_string(),
                use_trampoline: true,
                amount_msat,
                label: None,
            })
            .await?;
        Ok(resp.payment.status)
    }
}

#[derive(Debug, Clone)]
pub struct PaymentEvent {
    pub payment_hash: String,
    #[allow(dead_code)]
    pub amount_msat: u64,
    #[allow(dead_code)]
    pub timestamp: u64,
}

pub trait PaymentEventSink: Send + Sync {
    fn handle(&self, event: PaymentEvent) -> Result<()>;
}

#[derive(Clone)]
pub struct BreezEventForwarder {
    handler: Arc<Mutex<Option<Arc<dyn PaymentEventSink>>>>,
}

impl BreezEventForwarder {
    pub fn new() -> Self {
        Self {
            handler: Arc::new(Mutex::new(None)),
        }
    }

    pub fn set_handler(&self, handler: Arc<dyn PaymentEventSink>) {
        let mut guard = self.handler.lock().unwrap();
        *guard = Some(handler);
    }
}

impl EventListener for BreezEventForwarder {
    fn on_event(&self, event: BreezEvent) {
        if let BreezEvent::InvoicePaid { details } = event {
            let timestamp = now_secs();
            let payload = PaymentEvent {
                payment_hash: details.payment_hash.clone(),
                amount_msat: details
                    .payment
                    .as_ref()
                    .map(|p| p.amount_msat)
                    .unwrap_or_default(),
                timestamp,
            };
            if let Some(handler) = self.handler.lock().unwrap().clone() {
                if let Err(err) = handler.handle(payload) {
                    eprintln!("failed handling payment event: {}", err);
                }
            }
        }
    }
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

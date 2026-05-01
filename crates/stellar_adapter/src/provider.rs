use anyhow::Result;
use async_trait::async_trait;
use payment_core::PaymentProvider;
use shared_types::{CreatePaymentRequest, PaymentEvent, PaymentRequest};

use crate::client::HorizonClient;
use crate::memo::generate_memo;
use crate::parser::parse_payment;

#[derive(Clone, Debug)]
pub struct StellarConfig {
    pub horizon_url: String,
    pub destination_address: String,
}

#[derive(Clone)]
pub struct StellarProvider {
    config: StellarConfig,
    client: HorizonClient,
}

impl StellarProvider {
    pub fn new(config: StellarConfig) -> Self {
        let client = HorizonClient::new(config.horizon_url.clone());
        Self { config, client }
    }
}

#[async_trait]
impl PaymentProvider for StellarProvider {
    async fn create_payment_request(
        &self,
        request: CreatePaymentRequest,
    ) -> Result<PaymentRequest> {
        let memo = generate_memo();
        let session_id = memo.clone();
        let qr_payload = format!(
            "web+stellar:pay?destination={}&amount={}&memo={}",
            self.config.destination_address, request.amount, memo
        );
        let asset = if request.asset.is_empty() {
            request.currency.clone()
        } else {
            request.asset.clone()
        };

        Ok(PaymentRequest {
            session_id,
            rail: "stellar".to_string(),
            amount: request.amount,
            currency: asset.clone(),
            amount_msat: None,
            payment_request: qr_payload.clone(),
            qr_payload,
            invoice: None,
            payment_hash: None,
            destination: Some(self.config.destination_address.clone()),
            asset: Some(asset),
            memo: Some(memo),
            expires_at: 0,
        })
    }

    async fn find_confirmed_payment(
        &self,
        request: &PaymentRequest,
    ) -> Result<Option<PaymentEvent>> {
        let Some(destination) = request.destination.as_deref() else {
            return Ok(None);
        };
        let Some(memo) = request.memo.as_deref() else {
            return Ok(None);
        };
        let asset = request.asset.as_deref().unwrap_or(&request.currency);
        let payments = self.client.payments_for_account(destination).await?;
        for record in payments.embedded.records {
            if !record.transaction_successful || record.record_type != "payment" {
                continue;
            }

            let event = parse_payment(&record)?;
            if event.destination_account.as_deref() == Some(destination)
                && event.memo.as_deref() == Some(memo)
                && event.amount == request.amount
                && event
                    .asset
                    .as_deref()
                    .unwrap_or_default()
                    .eq_ignore_ascii_case(asset)
            {
                return Ok(Some(event));
            }
        }

        Ok(None)
    }
}

use anyhow::Result;
use async_trait::async_trait;
use shared_types::{CreatePaymentRequest, PaymentEvent, PaymentRequest};

/// Payment provider boundary for HASKEpay checkout flows.
///
/// The primary settlement model is Lightning-native: providers should return
/// invoices/payment requests keyed by `payment_hash`, with amounts expressed in
/// millisatoshis or sats. The normalized event should use `payment_hash` and
/// `settlement_id` as the settlement handles exposed to the rest of the
/// application.
#[async_trait]
pub trait PaymentProvider: Send + Sync {
    async fn create_payment_request(&self, request: CreatePaymentRequest)
        -> Result<PaymentRequest>;

    async fn find_confirmed_payment(
        &self,
        request: &PaymentRequest,
    ) -> Result<Option<PaymentEvent>>;
}

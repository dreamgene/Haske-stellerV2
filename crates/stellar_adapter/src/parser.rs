use anyhow::Result;
use shared_types::PaymentEvent;

use crate::types::PaymentRecord;

pub fn parse_payment(record: &PaymentRecord) -> Result<PaymentEvent> {
    let asset = match record.asset_type.as_str() {
        "native" => "XLM".to_string(),
        _ => record
            .asset_code
            .clone()
            .unwrap_or_else(|| record.asset_type.clone()),
    };

    Ok(PaymentEvent {
        session_id: String::new(),
        rail: "stellar".to_string(),
        settlement_id: if record.transaction_hash.is_empty() {
            record.id.clone()
        } else {
            record.transaction_hash.clone()
        },
        payment_hash: None,
        preimage: None,
        invoice: None,
        tx_hash: Some(if record.transaction_hash.is_empty() {
            record.id.clone()
        } else {
            record.transaction_hash.clone()
        }),
        source_account: Some(record.from.clone()),
        destination_account: Some(record.to.clone()),
        amount: record.amount.clone(),
        currency: asset.clone(),
        amount_msat: None,
        asset: Some(asset),
        memo: record.memo.clone(),
        ledger_sequence: record.ledger,
        confirmed_at: 0,
        settled_at: None,
        provider_metadata: serde_json::Value::Null,
    })
}

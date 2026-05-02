# HASKEpay Architecture

HASKEpay is a Bitcoin Lightning-native access-pass system.

Core promise:

**A confirmed payment becomes a signed access pass that verifies offline.**

## End-to-End Flow

Canonical flow:

```text
Buyer Phone
-> Bitcoin Lightning invoice payment
-> Lightning node / Lightning provider
-> Rust API
-> signed access token
-> QR encoder
-> offline verifier CLI
```

```text
Buyer Phone
  |
  | Bitcoin Lightning invoice payment
  v
Lightning node / Lightning provider
  |
  | invoice status check or invoice subscription
  v
Rust API
  |
  | signed access token
  v
QR encoder
  |
  v
Offline verifier CLI
```

## Runtime Responsibilities

Buyer phone:
- Requests a payment session.
- Receives a BOLT11 invoice and invoice QR.
- Pays from a Lightning wallet.
- Receives a signed access pass after settlement.

Lightning node / Lightning provider:
- Creates BOLT11 invoices.
- Exposes invoice status checking or settlement subscription.
- Reports settlement using Lightning-native identifiers.

Rust API:
- Creates payment sessions.
- Stores invoice, `payment_hash`, amount, and expiry.
- Checks invoice status or subscribes for invoice settlement.
- Issues signed access tokens only after settlement.

QR encoder:
- Encodes the Lightning invoice for payment.
- Encodes the signed access token for gate verification.

Offline verifier CLI:
- Verifies the access token signature.
- Checks expiry.
- Does not require a network call, user account, gate-side database, or online
  gate check.

## Lightning Settlement Fields

HASKEpay stores and exposes Lightning-native settlement fields:
- `payment_hash`
- preimage, when available from the provider
- `amount_msat`
- invoice
- `settled_at`

## Non-Negotiable Product Constraints

- Preserve offline verifier behavior.
- Do not introduce user accounts.
- Do not introduce online gate checks.
- Do not introduce a gate-side database.
- Keep access issuance tied to confirmed payment settlement.

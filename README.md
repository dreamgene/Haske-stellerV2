# HASKEpay

**Bitcoin Lightning-settled access that works offline.**

HASKEpay turns a settled Lightning payment into a signed access pass that can be
verified without an internet connection at the gate.

There are:
- No user accounts
- No ticket database required at entry
- No online gate verification dependency

Access is issued only after payment settlement, then verified offline with the
issuer public key.

## Scope

Current build targets:
- BOLT11 Lightning invoice generation
- Payment settlement tracking through a payment provider abstraction
- Access token issuance after settlement
- QR encoding for invoices and access passes
- Offline verifier CLI for signed tokens
- Minimal Axum API for checkout flows
- Vite checkout UI for buyers

The active API server is Lightning-native and uses Lightning settlement as the
runtime payment path.

## Architecture

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
  - creates payment sessions
  - stores invoice + payment_hash
  - checks/subscribes for invoice settlement
  - issues a signed access token after confirmed settlement
  v
Signed access token
  |
  v
QR encoder
  |
  v
Offline verifier CLI
```

Core promise:

**A confirmed payment becomes a signed access pass that verifies offline.**

Payment flow:
- Buyer opens checkout on a phone.
- Rust API asks a Lightning node/provider for a BOLT11 invoice.
- Buyer pays the invoice from a Lightning wallet.
- Lightning node/provider reports settlement by invoice status check or invoice
  subscription.
- Rust API matches settlement using the Lightning invoice and `payment_hash`.
- Rust API signs an access token containing Lightning settlement data.
- QR encoder turns the signed token into an access-pass QR.
- Offline verifier CLI validates the signature and expiry without a network
  call, user account, gate-side database, or online gate check.

Online components:
- `crates/api_server`: Axum routes, session state, watcher loop, access issuance
- `crates/payment_core`: rail-neutral payment provider trait
- `crates/qr`: QR rendering helpers

Offline components:
- `crates/access_token`: signed access token formats and verification helpers
- `crates/verifier_cli`: offline QR/token verifier

Reference components:
- `crates/lightning_node`: older Breez-based Lightning server path, excluded
  from the default workspace build because it requires extra native tooling such
  as `protoc`
- `vendor/ldk-node`: vendored LDK reference code

## API

`POST /api/payment-request`

Creates a Lightning payment request.

Default request:

```json
{
  "amount": "250000",
  "currency": "msat",
  "event_id": "haske-demo-event"
}
```

Response shape:

```json
{
  "session_id": "abc123",
  "rail": "lightning",
  "amount": "250000",
  "currency": "msat",
  "amount_msat": 250000,
  "payment_request": "lnbc...",
  "invoice": "lnbc...",
  "payment_hash": "...",
  "qr_payload": "lnbc...",
  "qr_png": "data:image/png;base64,...",
  "request_expires_at": 1735689750
}
```

`GET /api/payment-status/:session_id`

Returns `waiting`, `confirmed`, or `expired`. When confirmed, the response
includes the signed access token and QR data.

`GET /api/access-token/:session_id`

Returns the same token-bearing status response for retrieval flows.

## Access Tokens

HASKEpay tokens are Ed25519-signed payloads. Lightning tokens use token version
`2` and include:

- `product`
- `rail`
- `event_id`
- `payment_hash`
- `preimage`, when available from the provider
- `amount_msat`
- `invoice`
- `settled_at`
- `expires_at`
- `nonce`

Offline verification validates the issuer signature and expiry. If future
providers include a payment preimage in the token, the verifier can additionally
check that the preimage hashes to the `payment_hash`.

## Local Development

Run the API:

```bash
HASKEPAY_MOCK_SETTLE_AFTER_SECS=6 cargo run -p api_server
```

Without `HASKEPAY_MOCK_SETTLE_AFTER_SECS`, invoices remain pending. That is
intentional for provider integration work.

Run the UI:

```bash
cd haske-ui
npm run dev
```

Useful checks:

```bash
cargo check --workspace
cargo test --workspace
cd haske-ui && npm run lint && npm run build
```

## Lightning Backend Notes

HASKEpay treats Bitcoin Lightning as the payment rail and tracks settlement
through Lightning invoice identity, primarily `payment_hash`.

The default workspace excludes `crates/lightning_node` for now because that
reference crate pulls dependencies that require `protoc`. Bring it back only
after deciding whether the production provider should be Breez, LDK node, CLN,
LND, or a trait-backed adapter supporting several backends.

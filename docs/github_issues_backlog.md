# HASKEpay Backlog

HASKEpay is a Lightning-first project.

## 1. Choose the production Lightning backend

Acceptance: document whether production settlement uses LDK node, CLN, LND,
Breez, or multiple provider adapters behind `payment_core`. The chosen backend
must support Lightning invoice status checking or settlement subscription.

## 2. Replace the mock Lightning provider

Acceptance: `api_server` can create a real BOLT11 invoice and detect settlement
without `HASKEPAY_MOCK_SETTLE_AFTER_SECS`. Settlement records should expose
`payment_hash`, preimage when available, `amount_msat`, invoice, and
`settled_at`.

## 3. Add payment provider integration tests

Acceptance: tests cover invoice creation, pending status, settlement, duplicate
settlement handling, expired sessions, and matching by invoice + `payment_hash`.

## 4. Add token v2 verification tests

Acceptance: `access_token` and `verifier_cli` verify HASKEpay Lightning tokens
with valid signatures and reject expired or tampered payloads.

## 5. Decide fate of `crates/lightning_node`

Acceptance: either reintegrate it as a maintained provider crate or move it to
an archive path outside the default workspace.

## 6. Add durable session storage

Acceptance: pending and settled sessions survive API restarts.

## 7. Add issuer key management docs

Acceptance: operators can generate, store, rotate, and publish Ed25519 issuer
keys safely.

## 8. Add frontend settlement states

Acceptance: checkout distinguishes invoice created, payment pending, settlement
confirmed, pass issued, expired, and provider failure.

## 9. Add end-to-end demo

Acceptance: a documented local demo starts API + UI, creates a Lightning invoice,
simulates or performs settlement, and verifies the resulting pass offline.

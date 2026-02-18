# HASKE ⚡🎟️

**Payment-settled access that works offline.**

HASkE Pass is a minimal platform that turns a Lightning payment into a
cryptographically verifiable access pass (QR code) that can be checked
**without internet access**.

There are:
- No user accounts
- No ticket databases
- No online verification servers at the gate

Access is derived directly from payment settlement.

---

## Scope

**A Lightning payment settles and immediately yields a signed access pass that can be verified offline, with no accounts, dashboards, refunds, subscriptions, or online verification databases.**

Must-build components:
- Lightning payment receiver (Breez SDK) that confirms settlement
- Access token issuer that signs a pass after settlement
- QR encoder for the signed pass
- Offline verifier (CLI) that validates the signature
- Minimal buyer-facing checkout page to display invoice and pass

---

## Demo Screenshots

### 1. Buyer Pays Invoice

![Invoice QR](screenshots/invoice_qr.png)

> The buyer scans this Lightning invoice QR with any Lightning wallet.

---

### 2. Payment Settled, Access Token Generated

![Access QR](screenshots/access_qr.png)

> Once payment is claimed, a cryptographically signed access token is issued as a QR.

---

### 3. Offline Verification

![Verifier CLI](screenshots/verifier_cli.png)

> The gate scanner verifies the token offline. VALID -> access granted.

---

## System Architecture (Breez)

Component diagram (text-based):

Buyer Phone
|  (LN payment + QR display)
v
Rust API (axum)
|  - invoice endpoint
|  - payment status
|  - token signing (Ed25519)
v
Lightning Node (Breez SDK)
|
v
Access Token Generator
|  - creates token payload
|  - signs with Ed25519 private key
v
QR Encoder
|  - encodes signed token as QR
v
Buyer Phone (shows access QR)
|
v
Gate Verifier (offline CLI)
|  - verifies Ed25519 signature
|  - checks token fields (expiry, amount, nonce)

Data flow from payment to gate entry:
- Buyer requests invoice from Rust API.
- API asks Breez SDK to create invoice.
- Buyer pays invoice with a Lightning wallet.
- Breez SDK reports settlement to the API.
- API builds token payload (payment hash, amount, expiry, nonce).
- API signs token with Ed25519 private key.
- API returns signed token and QR payload to buyer.
- Buyer presents QR at gate.
- Offline verifier scans QR, decodes signed token, verifies Ed25519 signature, checks fields, grants access.

What runs online vs offline:
- Online: Rust API (axum), Breez SDK Lightning node, token signing service (Ed25519 private key), QR generation endpoint.
- Offline: Gate verifier CLI, Ed25519 public key for signature verification, token validation logic (expiry/amount/nonce checks).

---

## Lightning Node Setup (Breez SDK)

The node is created with `BreezServices::init` and runs entirely in-process (no external LDK daemon). It creates invoices and listens for `InvoicePaid` events from Breez.

Implementation lives in:
- `crates/lightning_node/src/main.rs` (wires Breez services, HTTP server, and payment listener)
- `crates/lightning_node/src/layers/lightning.rs` (Breez gateway + event forwarder)
- `crates/lightning_node/src/layers/access.rs` and `crates/lightning_node/src/layers/proof.rs` (API + token issuance)

Notes:
- Default network is Signet unless overridden via `LIGHTNING_NODE_NETWORK` (`bitcoin`, `signet`, `regtest` supported).
- `BREEZ_MNEMONIC` is required; `BREEZ_API_KEY` is optional for hosted services.
- `BREEZ_WORKDIR` defaults to `./data/breez` and is created automatically.
- Breez surfaces an on-chain address via `services.onchain_address()`; channels are opened automatically via LSP once funded.
- Demo auto-funding: set `LIGHTNING_PASS_FAUCET_URL` (and optional `LIGHTNING_PASS_FAUCET_KEY`) plus `LIGHTNING_PASS_AUTO_FUND_SATS` to request signet/regtest sats into the Breez wallet using `FundingLayer::bitcoin_signet_default`.
- Payment flow (Breez):
  1. `BreezServices::init` (Signet by default).
  2. Get on-chain address (`services.onchain_address()`), fund via faucet.
  3. Create invoice (`services.receive_payment` via `LightningGateway`).
  4. Listen for `BreezEvent::InvoicePaid` and issue the access token.

---

## Threat Model (Simple)

- Screenshot reuse: short token expiry and a per-token nonce make screenshots time-limited; verifier rejects expired tokens without needing any database.
- Fake QR codes: Ed25519 signatures ensure only issuer-signed tokens validate; unverifiable tokens are rejected offline.
- Double entry: enforce short expiries and include event-specific context in the signed payload (e.g., amount/venue/time); without a database, this reduces replay window and ties access to a specific event.
- Clock drift: allow a small verification grace window (e.g., +/- 60s) and keep verifier devices synced periodically; still offline at verify time.

---

## Token Signing Module (Ed25519)

Minimal Rust module that takes `payment_hash + expiry + event_id`, signs it with Ed25519, and outputs a compact encoded string:

Implementation lives in `crates/access_token/src/compact.rs`.

Key generation + storage (recommended):
- Generate once with `Keypair::generate(&mut rand::rngs::OsRng)`.
- Store the private key offline or in a locked file on the signing host (e.g., base64-encoded), load on startup.
- Distribute only the public key to offline verifiers; rotate keys per event if needed.

---

## QR Generation (Token Issuer)

Requirements covered:
- Terminal QR (ASCII) for dev.
- PNG QR for demos.
- Deterministic output (same payload -> same QR).

Implementation lives in `crates/lightning_node/src/qr.rs`.

---

## Offline Verifier CLI (No Network)

Behavior:
- Accepts a token string, a token file, or a QR image.
- Verifies Ed25519 signature.
- Checks expiry.
- Prints `VALID` or `INVALID`.
- No network calls, no databases.

Public key embedding:
- The verifier embeds the issuer's Ed25519 public key as a base64 string constant.
- This keeps verification fully offline; the key can be replaced per event or rotated by rebuilding.

CLI usage: `verifier --token "<compact_token>"`, `verifier --token-file path/to/token.txt`, `verifier --qr-image path/to/qr.png`.
Implementation lives in `crates/verifier_cli/src/main.rs`.

---

## Minimal HTTP API (Demo)

Endpoints:
- `POST /api/invoice` -> create invoice (returns invoice QR + id)
- `GET /api/invoice/:invoice_id` -> check payment status (returns access token/QR when settled)
- `GET /api/invoice/:invoice_id/access` -> retrieve access token (same response as status)

Implementation:
- `crates/lightning_node/src/layers/access.rs` (axum routes + handlers + in-memory store)
- `crates/lightning_node/src/layers/proof.rs` (token issuance + payment settlement handling)
- `crates/lightning_node/src/main.rs` (server wiring + Breez payment listener)

---

## Demo Mode

Behavior:
- Fixed `event_id` (`demo-event`).
- Short expiry (2 minutes).
- Logs prefixed with `[demo]` and simplified for live walkthroughs.

Toggle (safe):
- Set `LIGHTNING_PASS_DEMO=1` or `LIGHTNING_PASS_DEMO=true` to enable demo mode.
- Omit the env var to run default mode with longer expiry and configurable event id.
- For live demos, set the env var in the process environment only (not in `.env` checked into source).
- Keep production configs free of `LIGHTNING_PASS_DEMO` and confirm logs do not show the `[demo]` prefix.

---

## Demo Failure Scenarios

Network blip (invoice status delay):
- User sees: QR stays in "waiting for payment" state.
- Operator sees: `[demo]` logs show status poll timeouts or missing callbacks.
- Live recovery: switch to a known-working hotspot, re-open the same invoice page, or generate a fresh invoice.

Wallet delay (payment settles late):
- User sees: payment sent but no access QR yet.
- Operator sees: `[demo]` logs show invoice pending, then settled later.
- Live recovery: narrate the async nature of settlement and refresh the status; show the access QR once it appears.

Expired QR (short demo expiry):
- User sees: "expired" or "invalid" on the verifier.
- Operator sees: `[demo]` logs show token expiry check failing.
- Live recovery: generate a new invoice to mint a fresh token, then re-scan to show success.

---

## Chain + Gossip Sync Checks

Look for logs indicating:
- Headers synced.
- Rapid Gossip Sync loaded.

If gossip does not load, payments may fail later.

---

## Access Token Specification (Minimal)

Field list:
- `payment_hash` (string, hex): Lightning payment hash tied to settlement.
- `amount_msat` (number): Amount paid in millisatoshis.
- `expires_at` (number): Unix timestamp in seconds.
- `nonce` (string): Random 12-16 char nonce to prevent reuse.
- `event_id` (string): Event or venue identifier (keeps tokens scoped).

Example payload (before signing): `payment_hash=4c1ff0...1234`, `amount_msat=250000`, `expires_at=1735689750`, `nonce=A9k3Lm2Qx7pR`, `event_id=venue-2025-01-15`.

Encoding format:
- JSON payload, signed with Ed25519, then packed as a JSON object: `{ token: <payload>, signature: <base64> }`.

Why this format:
- JSON is compact enough for QR, easy to debug in a demo, and deterministic for signing/verifying across Rust and CLI tooling.

---

## What This Demonstrates

- Instant Lightning payments using Breez SDK
- Access passes issued only after payment is settled
- Offline verification using cryptographic signatures
- A simple "pay -> enter" user experience

This repository is built as an **investor demo**, not a full production system.

---

## Architecture Overview
Buyer Phone
|
| (Lightning payment)
v
Lightning Node (Breez SDK)
|
| (payment settled)
v
Access Token Issuer
|
| (signed QR)
v
Gate Verifier (offline)

---

## Investor Narrative

One sentence:
- This platform turns a payment into a time-limited entry pass that can be checked at the door with no internet.

One paragraph:
- A guest pays on their phone and receives a QR pass within seconds. The venue staff scan that pass at the gate, and the scanner verifies it locally without needing a network connection. That means lines keep moving even if Wi-Fi is down, while the business still gets a clean record of who paid and who entered.

Simple diagram (ASCII):
Phone payment -> QR pass -> Door scanner (offline check) -> Entry

---

## Demo Script (3 Minutes)

Setup context (say):
- "This is a pay-to-enter flow where the gate has no internet, but the pass still verifies."

Show payment (say):
- "I tap pay on my phone, and within a few seconds I get a QR pass."
- "That pass is short-lived and tied to this event only."

Turn off internet at the gate (say):
- "Now I'm turning off the gate's internet to simulate a real outage."

Verify access offline (say):
- "I'm scanning the QR on the gate device."
- "Notice it verifies instantly, even offline, and grants entry."

Wrap-up (say):
- "So the business gets reliable entry, the guest gets a smooth experience, and the venue isn't blocked by network issues."


Lightning Proof-of-Payment Access System

A cryptographic proof-of-payment backend built on the Bitcoin Lightning Network, optimized for unreliable networks and fast physical or digital access.

This system converts a successful Lightning payment into a verifiable, replay-safe access credential that can be validated instantly — even offline.

Core Idea

A Lightning payment is not the end — it is the proof.

This project provides the missing layer between:

“A payment was made”
and

“Access is granted.”

It transforms Lightning settlement events into cryptographic artifacts suitable for:

Event ticketing

Physical gates and doors

API access

Low-connectivity environments

What This Project Is (and Is Not)
✅ This project is

A proof-of-payment system

An access control backend

A Lightning-native verification layer

Optimized for speed and offline validation

❌ This project is not

A Lightning wallet

A faucet or liquidity provider

A routing or payment processor

A custodial fund manager

System Architecture
Layered Design
┌────────────────────────────────────┐
│  Access Layer                      │
│  (Gate, Ticket, API, Verifier)     │
└──────────────▲─────────────────────┘
               │ cryptographic proof
┌──────────────┴─────────────────────┐
│  Proof Layer (THIS PROJECT)        │
│                                   │
│  • Invoice issuance               │
│  • Payment verification           │
│  • Preimage extraction            │
│  • Proof generation               │
│  • Replay protection              │
└──────────────▲─────────────────────┘
               │ Lightning events
┌──────────────┴─────────────────────┐
│  Lightning Layer                   │
│  (LDK Node)                        │
│                                   │
│  • Payment routing                │
│  • HTLC settlement                │
└──────────────▲─────────────────────┘
               │ funding
┌──────────────┴─────────────────────┐
│  Funding Layer                     │
│  (Testnet Faucets / Wallets)       │
└────────────────────────────────────┘

Backend Components
Daemon (Rust)

Runs an LDK Lightning node

Creates BOLT11 invoices

Listens for payment events

Emits PaymentClaimed events

HTTP API (Axum)

Exposes a simple REST interface:

Endpoint	Description
POST /api/invoice	Create a Lightning invoice
GET /api/invoice/{id}	Check invoice payment status
GET /api/invoice/{id}/access	Retrieve access proof
State Management

Tracks:

Invoice ID

Payment hash

Payment preimage

Claim status

Expiration timestamps

Storage can be in-memory or a lightweight database.

Payment → Access Flow

Client requests an invoice

Backend generates a BOLT11 invoice

Client pays using any Lightning wallet

LDK emits a PaymentClaimed event

Backend extracts the payment preimage

A cryptographic access proof is derived

Proof is returned as a token or QR

Access verifier validates proof and grants access

Security Model

Proof source: Lightning payment preimage

Forge resistance: Cryptographically infeasible

Replay protection: One-time or time-bounded proofs

Trust assumptions: No trusted server required at access point

Offline & Low-Connectivity Support

This system is designed for environments with:

Poor internet

Intermittent connectivity

High throughput access points

Mechanisms include:

Preimage-based proofs

Signed tokens or hash commitments

Local verification rules at the access layer

Testnet Development Strategy

Used for development, testing, and demos.

Funding

Bitcoin Testnet faucets

Compatible Testnet Wallets

Phoenix (Testnet)

Mutiny (Testnet)

Alby (Testnet)

Success Criteria
Technical

A Lightning payment deterministically produces a verifiable proof

Product

Access is granted instantly without waiting for confirmations

Architectural

Clean separation between:

Payment

Proof

Access

Positioning Summary

This project sits between Lightning payments and real-world access.

It is best described as:

A Lightning-native proof-of-payment and access control layer.

License

MIT (or specify)

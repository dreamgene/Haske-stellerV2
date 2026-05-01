# HASKEpay UI

Buyer-facing checkout UI for HASKEpay.

HASKEpay is Bitcoin Lightning-native. The UI requests a BOLT11 invoice from the
API, shows the invoice QR, polls for settlement, and then displays the signed
offline-verifiable access pass.

## Development

```bash
npm run dev
```

## Checks

```bash
npm run lint
npm run build
```

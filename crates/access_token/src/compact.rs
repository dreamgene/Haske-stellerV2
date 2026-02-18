use base64::engine::general_purpose::URL_SAFE_NO_PAD as B64;
use base64::Engine;
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CompactAccessPayload {
    pub payment_hash: String,
    pub expires_at: u64,
    pub event_id: String,
}

pub fn sign_compact_payload(
    payload: &CompactAccessPayload,
    signing_key: &SigningKey,
) -> Result<String, serde_json::Error> {
    let payload_json = serde_json::to_vec(payload)?;
    let signature = signing_key.sign(&payload_json);

    let payload_b64 = B64.encode(payload_json);
    let sig_b64 = B64.encode(signature.to_bytes());

    Ok(format!("{}.{}", payload_b64, sig_b64))
}

#[derive(Debug)]
pub enum CompactVerifyError {
    InvalidFormat,
    InvalidPayload(serde_json::Error),
    InvalidSignature,
    InvalidSignatureLength,
    Base64(base64::DecodeError),
}

pub fn verify_compact_token(
    token: &str,
    public_key: &VerifyingKey,
) -> Result<CompactAccessPayload, CompactVerifyError> {
    let mut parts = token.splitn(2, '.');
    let payload_b64 = parts.next().ok_or(CompactVerifyError::InvalidFormat)?;
    let sig_b64 = parts.next().ok_or(CompactVerifyError::InvalidFormat)?;

    let payload_bytes = B64
        .decode(payload_b64.as_bytes())
        .map_err(CompactVerifyError::Base64)?;
    let signature_bytes = B64
        .decode(sig_b64.as_bytes())
        .map_err(CompactVerifyError::Base64)?;
    let signature_bytes: [u8; 64] = signature_bytes
        .as_slice()
        .try_into()
        .map_err(|_| CompactVerifyError::InvalidSignatureLength)?;
    let signature = Signature::from_bytes(&signature_bytes);

    public_key
        .verify(&payload_bytes, &signature)
        .map_err(|_| CompactVerifyError::InvalidSignature)?;

    let payload =
        serde_json::from_slice(&payload_bytes).map_err(CompactVerifyError::InvalidPayload)?;
    Ok(payload)
}

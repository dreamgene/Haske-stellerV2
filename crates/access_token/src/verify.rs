use std::error::Error;
use std::fmt;

use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use base64::Engine;
use ed25519_dalek::{Signature, Verifier, VerifyingKey};

use crate::sign::SignedAccessToken;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VerificationStatus {
    Valid,
    Invalid,
}

#[derive(Debug)]
pub enum VerifyError {
    Decode(base64::DecodeError),
    InvalidSignatureLength,
    InvalidSignature,
    InvalidToken(serde_json::Error),
    Expired { now: u64, expires_at: u64 },
}

impl fmt::Display for VerifyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VerifyError::Decode(err) => write!(f, "invalid base64 signature: {err}"),
            VerifyError::InvalidSignatureLength => write!(f, "signature length is invalid"),
            VerifyError::InvalidSignature => write!(f, "signature did not verify"),
            VerifyError::InvalidToken(err) => write!(f, "invalid token payload: {err}"),
            VerifyError::Expired { now, expires_at } => {
                write!(f, "token expired at {expires_at} (now {now})")
            }
        }
    }
}

impl Error for VerifyError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            VerifyError::Decode(err) => Some(err),
            VerifyError::InvalidToken(err) => Some(err),
            _ => None,
        }
    }
}

pub fn signed_token_from_json(input: &str) -> Result<SignedAccessToken, VerifyError> {
    serde_json::from_str(input).map_err(VerifyError::InvalidToken)
}

pub fn verify_signed_token(
    signed: &SignedAccessToken,
    public_key: &VerifyingKey,
) -> Result<(), VerifyError> {
    verify_signed_token_at(signed, public_key, now_secs())
}

pub fn verify_token_string(input: &str, public_key: &VerifyingKey) -> VerificationStatus {
    let signed = match signed_token_from_json(input) {
        Ok(signed) => signed,
        Err(_) => return VerificationStatus::Invalid,
    };
    if verify_signed_token(&signed, public_key).is_ok() {
        VerificationStatus::Valid
    } else {
        VerificationStatus::Invalid
    }
}

pub fn verify_signed_token_at(
    signed: &SignedAccessToken,
    public_key: &VerifyingKey,
    now_secs: u64,
) -> Result<(), VerifyError> {
    let payload = serde_json::to_vec(&signed.token).map_err(VerifyError::InvalidToken)?;
    let signature_bytes = BASE64_STANDARD
        .decode(&signed.signature)
        .map_err(VerifyError::Decode)?;
    let signature_bytes: [u8; 64] = signature_bytes
        .as_slice()
        .try_into()
        .map_err(|_| VerifyError::InvalidSignatureLength)?;
    let signature = Signature::from_bytes(&signature_bytes);

    public_key
        .verify(&payload, &signature)
        .map_err(|_| VerifyError::InvalidSignature)
        .and_then(|_| {
            if signed.token.expires_at <= now_secs {
                Err(VerifyError::Expired {
                    now: now_secs,
                    expires_at: signed.token.expires_at,
                })
            } else {
                Ok(())
            }
        })
}

fn now_secs() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};

    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

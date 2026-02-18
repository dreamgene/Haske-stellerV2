use std::error::Error;
use std::fmt;

use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use base64::Engine;
use ed25519_dalek::{Signer, SigningKey};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};

use crate::token::AccessToken;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SignedAccessToken {
    pub token: AccessToken,
    pub signature: String,
}

#[derive(Debug)]
pub enum SignError {
    Serialize(serde_json::Error),
}

impl fmt::Display for SignError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SignError::Serialize(err) => write!(f, "failed to serialize token: {err}"),
        }
    }
}

impl Error for SignError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            SignError::Serialize(err) => Some(err),
        }
    }
}

pub fn sign_token(
    token: AccessToken,
    signing_key: &SigningKey,
) -> Result<SignedAccessToken, SignError> {
    let payload = serde_json::to_vec(&token).map_err(SignError::Serialize)?;
    let signature = signing_key.sign(&payload);
    let signature_b64 = BASE64_STANDARD.encode(signature.to_bytes());

    Ok(SignedAccessToken {
        token,
        signature: signature_b64,
    })
}

pub fn signed_token_to_json(signed: &SignedAccessToken) -> Result<String, SignError> {
    serde_json::to_string(signed).map_err(SignError::Serialize)
}

pub fn generate_keypair() -> SigningKey {
    SigningKey::generate(&mut OsRng)
}

pub fn sign_token_to_string(
    token: AccessToken,
    signing_key: &SigningKey,
) -> Result<String, SignError> {
    let signed = sign_token(token, signing_key)?;
    signed_token_to_json(&signed)
}

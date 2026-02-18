use std::error::Error;
use std::fs;
use std::path::PathBuf;
use std::process;
use std::time::{SystemTime, UNIX_EPOCH};

use access_token::{verify_compact_token, verify_token_string, VerificationStatus};
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use base64::Engine;
use clap::Parser;
use ed25519_dalek::VerifyingKey;
use rqrr::PreparedImage;

#[derive(Parser)]
#[command(name = "verifier")]
#[command(about = "Verify a Lightning Pass access token offline")]
struct Args {
    #[arg(long, value_name = "PATH")]
    public_key: Option<PathBuf>,
    #[arg(long, value_name = "TOKEN")]
    token: Option<String>,
    #[arg(long, value_name = "PATH")]
    qr_image: Option<PathBuf>,
}

fn main() {
    match run(Args::parse()) {
        Ok(true) => println!("VALID"),
        Ok(false) => {
            println!("INVALID");
            process::exit(1);
        }
        Err(_err) => {
            println!("INVALID");
            process::exit(1);
        }
    }
}

fn run(args: Args) -> Result<bool, Box<dyn Error>> {
    let public_key = load_public_key(args.public_key)?;

    let token = if let Some(token) = args.token {
        token
    } else if let Some(qr_image) = args.qr_image {
        decode_qr_image(&qr_image)?
    } else {
        return Err("provide --token or --qr-image".into());
    };

    let token = token.trim();
    if verify_token_string(token, &public_key) == VerificationStatus::Valid {
        return Ok(true);
    }

    let payload = match verify_compact_token(token, &public_key) {
        Ok(payload) => payload,
        Err(_) => return Ok(false),
    };

    Ok(!is_expired(payload.expires_at)?)
}

fn load_public_key(path: Option<PathBuf>) -> Result<VerifyingKey, Box<dyn Error>> {
    let public_key_b64 = if let Some(path) = path {
        fs::read_to_string(path)?
    } else {
        // Embedded public key (base64) so verifier can run fully offline.
        // Replace with your actual public key from the issuer.
        "REPLACE_WITH_BASE64_PUBLIC_KEY".to_string()
    };
    let public_key_bytes = BASE64_STANDARD.decode(public_key_b64.trim())?;
    let public_key_bytes: [u8; 32] = public_key_bytes
        .as_slice()
        .try_into()
        .map_err(|_| "invalid public key length")?;
    let public_key = VerifyingKey::from_bytes(&public_key_bytes)?;
    Ok(public_key)
}

fn decode_qr_image(path: &PathBuf) -> Result<String, Box<dyn Error>> {
    let image = image::open(path)?;
    let grayscale = image.to_luma8();
    let mut prepared = PreparedImage::prepare(grayscale);
    let grids = prepared.detect_grids();
    for grid in grids {
        let (_, content) = grid.decode()?;
        return Ok(content);
    }
    Err("no QR code detected".into())
}

fn is_expired(expires_at: u64) -> Result<bool, Box<dyn Error>> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    Ok(now > expires_at)
}

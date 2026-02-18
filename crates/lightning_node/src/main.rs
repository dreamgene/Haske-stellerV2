mod layers;
mod qr;

use anyhow::{anyhow, Result};
use breez_sdk_core::{
    mnemonic_to_seed, BreezServices, ConnectRequest, EnvironmentType, GreenlightNodeConfig,
    Network, NodeConfig,
};
use ed25519_dalek::SigningKey;
use rand::rngs::OsRng;
use std::sync::Arc;

use crate::layers::access::{build_router, AccessState};
use crate::layers::funding::FundingLayer;
use crate::layers::lightning::{BreezEventForwarder, BreezGateway, LightningGateway};
use crate::layers::proof::{default_config, demo_config, PaymentEventHandler, ProofService};

#[tokio::main]
async fn main() -> Result<()> {
    let demo_mode = std::env::var("LIGHTNING_PASS_DEMO")
        .map(|val| val == "1" || val.eq_ignore_ascii_case("true"))
        .unwrap_or(false);

    let event_forwarder = BreezEventForwarder::new();
    let services = init_breez_services(event_forwarder.clone()).await?;

    let keypair = SigningKey::generate(&mut OsRng);
    let config = if demo_mode {
        demo_config()
    } else {
        default_config("live-event".to_string())
    };
    let gateway: Arc<dyn LightningGateway> = Arc::new(BreezGateway::new(Arc::clone(&services)));
    let proof_service = ProofService::new(Arc::clone(&gateway), keypair, config);
    let access_state = AccessState::new(proof_service.clone());
    let funding_layer = FundingLayer::bitcoin_signet_default(Arc::clone(&gateway));

    println!(
        "[funding] network={} faucet={} url={}",
        funding_layer.target().network,
        funding_layer.target().faucet_name,
        funding_layer.target().faucet_url
    );

    if let Some(auto_sats) = std::env::var("LIGHTNING_PASS_AUTO_FUND_SATS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
    {
        match funding_layer.request_funds(auto_sats).await {
            Ok(res) => println!(
                "[funding] requested {} sats -> txid={:?} address={}",
                res.sats_requested, res.faucet_txid, res.address
            ),
            Err(err) => eprintln!("[funding] auto funding failed: {err}"),
        }
    }

    let app = build_router(access_state);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    let server = axum::serve(listener, app.into_make_service());

    if proof_service.config().demo_mode {
        println!(
            "[demo] mode enabled event_id={} expiry_secs={}",
            proof_service.config().event_id,
            proof_service.config().invoice_expiry_secs
        );
    }

    let handler = Arc::new(PaymentEventHandler::new(proof_service));
    event_forwarder.set_handler(handler);

    server.await?;
    Ok(())
}

async fn init_breez_services(event_listener: BreezEventForwarder) -> Result<Arc<BreezServices>> {
    let network = parse_network(std::env::var("LIGHTNING_NODE_NETWORK").ok());
    let api_key = std::env::var("BREEZ_API_KEY").unwrap_or_default();
    let working_dir = std::env::var("BREEZ_WORKDIR").unwrap_or_else(|_| "./data/breez".to_string());
    std::fs::create_dir_all(&working_dir)?;

    let mnemonic =
        std::env::var("BREEZ_MNEMONIC").map_err(|_| anyhow!("missing BREEZ_MNEMONIC env var"))?;
    let seed = mnemonic_to_seed(mnemonic)?;

    let invite_code = std::env::var("BREEZ_INVITE_CODE").ok();
    let node_config = NodeConfig::Greenlight {
        config: GreenlightNodeConfig {
            partner_credentials: None,
            invite_code,
        },
    };

    let mut config = BreezServices::default_config(EnvironmentType::Staging, api_key, node_config);
    config.network = network;
    config.working_dir = working_dir.clone();

    let req = ConnectRequest {
        config,
        seed,
        restore_only: None,
    };

    let services = BreezServices::connect(req, Box::new(event_listener)).await?;
    Ok(services)
}

fn parse_network(value: Option<String>) -> Network {
    match value
        .unwrap_or_else(|| "signet".to_string())
        .to_lowercase()
        .as_str()
    {
        "bitcoin" | "mainnet" => Network::Bitcoin,
        "signet" => Network::Signet,
        "regtest" => Network::Regtest,
        _ => Network::Signet,
    }
}

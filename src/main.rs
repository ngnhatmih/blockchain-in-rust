use rustchain::p2p::{P2PConfig, P2PNetwork};
use rustchain::core::{Blockchain, Transaction};
use rustchain::core::genesis::{create_genesis_block, GenesisConfig};
use std::sync::Arc;
use tokio::sync::Mutex;
use futures::StreamExt;
use std::env;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_ansi(false) // Disable colors for old terminals
        .with_timer(tracing_subscriber::fmt::time::ChronoLocal::new("%Y-%m-%d %H:%M:%S".to_string()))
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"))
        )
        .init();

    tracing::info!("Starting RustChain Node...");

    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    let config_path = args
        .get(1)
        .map(|s| s.as_str())
        .unwrap_or("configs/node.toml");

    // Load configuration
    let config = if std::path::Path::new(config_path).exists() {
        tracing::info!("Loading configuration from: {}", config_path);
        P2PConfig::from_file(config_path)?
    } else {
        tracing::warn!("Config file not found: {}, using default configuration", config_path);
        P2PConfig::default()
    };

    tracing::info!("Listen address: {}", config.listen_addr);
    tracing::info!("Max peers: {}", config.max_peers);
    tracing::info!("Bootstrap peers: {}", config.bootstrap_peers.len());
    if !config.bootstrap_peers.is_empty() {
        for peer in &config.bootstrap_peers {
            tracing::info!("   - {}", peer);
        }
    }

    // Create genesis block
    tracing::info!("Creating genesis block...");
    let genesis_tx = Transaction::new(
        "Genesis".to_string(),
        "Alice".to_string(),
        1000,
    );
    let genesis_config = GenesisConfig::new(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        vec![genesis_tx],
    );
    let genesis_block = create_genesis_block(&genesis_config, None);
    tracing::info!("Genesis block created (hash: {})", &genesis_block.hash[..16]);

    // Initialize blockchain
    let blockchain = Arc::new(Mutex::new(Blockchain::new(genesis_block)));
    {
        let chain = blockchain.lock().await;
        tracing::info!("Blockchain initialized with height: {}", chain.len());
    }

    // Create P2P network
    tracing::info!("Initializing P2P network...");
    let mut network = P2PNetwork::new(config)?
        .with_blockchain(blockchain.clone());
    
    // Start listening
    network.listen()?;
    tracing::info!("Listening on {}", network.config.listen_addr);
    tracing::info!("Peer ID: {}", network.local_peer_id);

    // Bootstrap to other peers if configured
    if !network.config.bootstrap_peers.is_empty() {
        tracing::info!("Connecting to bootstrap peers...");
        if let Err(e) = network.bootstrap() {
            tracing::warn!("Failed to bootstrap: {}", e);
        }
    }

    // Main event loop
    tracing::info!("Starting event loop...");
    tracing::info!("Press Ctrl+C to stop the node");
    
    loop {
        tokio::select! {
            event = network.swarm.next() => {
                match event {
                    Some(e) => {
                        network.handle_event(e).await;
                        
                        // Periodically log blockchain status
                        let height = {
                            let chain = blockchain.lock().await;
                            chain.len()
                        };
                        if height % 10 == 0 && height > 0 {
                            tracing::info!("Current blockchain height: {}", height);
                        }
                    }
                    None => {
                        tracing::warn!("Event stream ended");
                        break;
                    }
                }
            }
        }
    }

    tracing::info!("Node shutting down...");
    Ok(())
}

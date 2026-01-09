use rustchain::p2p::{P2PConfig, P2PNetwork};
use rustchain::core::{Blockchain, Block, Transaction};
use rustchain::core::genesis::{create_genesis_block, GenesisConfig};
use std::sync::Arc;
use tokio::sync::Mutex;
use std::time::Duration;
use futures::StreamExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    tracing::info!("🚀 Starting blockchain sync test...");

    // Create genesis block
    let genesis_tx = Transaction::new("Genesis".to_string(), "Alice".to_string(), 1000);
    let genesis_config = GenesisConfig::new(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        vec![genesis_tx],
    );
    let genesis_block = create_genesis_block(&genesis_config, None);

    // Node 1: Has blockchain with multiple blocks
    tracing::info!("📦 Setting up Node 1 with existing blocks...");
    let blockchain1 = Arc::new(Mutex::new(Blockchain::new(genesis_block.clone())));
    
    // Add some blocks to Node 1
    {
        let mut chain = blockchain1.lock().await;
        for i in 1..=5 {
            let latest = chain.get_latest_block().unwrap();
            let tx = Transaction::new(
                format!("User{}", i),
                format!("User{}", i + 1),
                100 * i as u64,
            );
            let new_block = Block::new(i, latest.hash.clone(), vec![tx]);
            chain.append_block(new_block.clone())
                .expect("Failed to append block");
            tracing::info!("✅ Node 1: Added block {} (hash: {})", i, &new_block.hash[..16]);
        }
        let height = chain.len();
        tracing::info!("📊 Node 1: Blockchain height = {}", height);
    }

    // Node 2: Starts with only genesis block (will sync from Node 1)
    tracing::info!("📦 Setting up Node 2 with only genesis block...");
    let blockchain2 = Arc::new(Mutex::new(Blockchain::new(genesis_block)));
    {
        let chain = blockchain2.lock().await;
        tracing::info!("📊 Node 2: Initial blockchain height = {}", chain.len());
    }

    // Setup Node 1: Listen on port 9000
    let mut config1 = P2PConfig::default();
    config1.listen_addr = "/ip4/127.0.0.1/tcp/9000".parse()?;
    config1.enable_mdns = false;
    let mut node1 = P2PNetwork::new(config1)?
        .with_blockchain(blockchain1.clone());
    node1.listen()?;
    tracing::info!("🌐 Node 1: Peer ID = {}", node1.local_peer_id);
    tracing::info!("🌐 Node 1: Listening on {}", node1.config.listen_addr);

    // Setup Node 2: Listen on port 9001, connect to node 1
    let mut config2 = P2PConfig::default();
    config2.listen_addr = "/ip4/127.0.0.1/tcp/9001".parse()?;
    config2.bootstrap_peers = vec!["/ip4/127.0.0.1/tcp/9000".parse()?];
    config2.enable_mdns = false;
    let mut node2 = P2PNetwork::new(config2)?
        .with_blockchain(blockchain2.clone());
    node2.listen()?;
    tracing::info!("🌐 Node 2: Peer ID = {}", node2.local_peer_id);
    tracing::info!("🌐 Node 2: Listening on {}", node2.config.listen_addr);

    // Start event loops for both nodes
    let node1_handle = tokio::spawn(async move {
        let mut node1 = node1;
        loop {
            tokio::select! {
                event = node1.swarm.next() => {
                    match event {
                        Some(e) => node1.handle_event(e).await,
                        None => break,
                    }
                }
            }
        }
    });

    // Clone blockchain2 for the monitoring loop
    let blockchain2_monitor = blockchain2.clone();
    
    let node2_handle = tokio::spawn(async move {
        let mut node2 = node2;
        let blockchain2 = blockchain2.clone();
        
        // Wait a bit then dial node 1
        tokio::time::sleep(Duration::from_millis(500)).await;
        if let Err(e) = node2.bootstrap() {
            tracing::error!("Node 2 failed to bootstrap: {}", e);
        }
        
        let mut connected = false;
        let mut last_height = 0;
        
        loop {
            tokio::select! {
                event = node2.swarm.next() => {
                    match event {
                        Some(e) => {
                            node2.handle_event(e).await;
                            
                            // Check if connected
                            let peer_count = node2.peer_count();
                            if peer_count > 0 && !connected {
                                connected = true;
                                tracing::info!("✅ Node 2 connected to Node 1! Peer count: {}", peer_count);
                            }
                            
                            // Monitor sync progress
                            let current_height = {
                                let chain = blockchain2.lock().await;
                                chain.len()
                            };
                            
                            if current_height != last_height {
                                tracing::info!("📈 Node 2: Blockchain height updated to {}", current_height);
                                last_height = current_height;
                            }
                        }
                        None => break,
                    }
                }
            }
        }
    });

    // Monitor sync progress
    tracing::info!("⏳ Waiting for sync to complete...");
    
    let sync_timeout = Duration::from_secs(30);
    let start_time = std::time::Instant::now();
    
    loop {
        tokio::time::sleep(Duration::from_millis(500)).await;
        
        let node2_height = {
            let chain = blockchain2_monitor.lock().await;
            chain.len()
        };
        
        let node1_height = {
            let chain = blockchain1.lock().await;
            chain.len()
        };
        
        if node2_height == node1_height {
            tracing::info!("🎉 Sync completed! Node 2 height: {}, Node 1 height: {}", 
                          node2_height, node1_height);
            
            // Verify blocks are synced correctly
            let chain2 = blockchain2_monitor.lock().await;
            let chain1 = blockchain1.lock().await;
            
            tracing::info!("🔍 Verifying synced blocks...");
            for i in 0..node2_height {
                let block2 = chain2.get_block(i).unwrap();
                let block1 = chain1.get_block(i).unwrap();
                
                if block2.hash == block1.hash {
                    tracing::info!("✅ Block {} verified (hash: {})", i, &block2.hash[..16]);
                } else {
                    tracing::error!("❌ Block {} mismatch!", i);
                }
            }
            
            break;
        }
        
        if start_time.elapsed() > sync_timeout {
            tracing::warn!("⏱️  Sync timeout after 30 seconds");
            tracing::warn!("   Node 1 height: {}, Node 2 height: {}", node1_height, node2_height);
            break;
        }
    }

    // Wait a bit more to see final state
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Clean shutdown
    tracing::info!("🛑 Shutting down...");
    node1_handle.abort();
    node2_handle.abort();

    Ok(())
}


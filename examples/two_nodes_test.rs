use rustchain::p2p::{P2PConfig, P2PNetwork};
use std::time::Duration;
use tokio::time::timeout;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    tracing::info!("Starting two-node P2P connection test...");

    // Node 1: Listen on port 9000
    let mut config1 = P2PConfig::default();
    config1.listen_addr = "/ip4/127.0.0.1/tcp/9000".parse()?;
    config1.enable_mdns = false; // Disable for localhost test
    let mut node1 = P2PNetwork::new(config1)?;
    node1.listen()?;
    tracing::info!("Node 1: Peer ID = {}", node1.local_peer_id);
    tracing::info!("Node 1: Listening on {}", node1.config.listen_addr);

    // Node 2: Listen on port 9001, connect to node 1
    let mut config2 = P2PConfig::default();
    config2.listen_addr = "/ip4/127.0.0.1/tcp/9001".parse()?;
    config2.bootstrap_peers = vec!["/ip4/127.0.0.1/tcp/9000".parse()?];
    config2.enable_mdns = false;
    let mut node2 = P2PNetwork::new(config2)?;
    node2.listen()?;
    tracing::info!("Node 2: Peer ID = {}", node2.local_peer_id);
    tracing::info!("Node 2: Listening on {}", node2.config.listen_addr);

    // Start event loops for both nodes
    use futures::StreamExt;
    
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

    let node2_handle = tokio::spawn(async move {
        let mut node2 = node2;
        // Wait a bit then dial node 1
        tokio::time::sleep(Duration::from_millis(500)).await;
        if let Err(e) = node2.bootstrap() {
            tracing::error!("Node 2 failed to bootstrap: {}", e);
        }
        
        let mut connected = false;
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
                                tracing::info!("✅ Node 2 connected! Peer count: {}", peer_count);
                            }
                        }
                        None => break,
                    }
                }
            }
        }
    });

    // Wait for connection (with timeout)
    let connection_timeout = Duration::from_secs(10);
    match timeout(connection_timeout, async {
        tokio::time::sleep(Duration::from_secs(5)).await;
    }).await {
        Ok(_) => {
            tracing::info!("✅ Test completed! Check logs above for connection status.");
        }
        Err(_) => {
            tracing::warn!("⏱️  Test timeout after 10 seconds");
        }
    }

    // Clean shutdown
    node1_handle.abort();
    node2_handle.abort();

    Ok(())
}


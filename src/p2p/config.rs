use libp2p::Multiaddr;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P2PConfig {
    pub listen_addr: Multiaddr,
    pub bootstrap_peers: Vec<Multiaddr>,
    pub max_peers: usize,
    pub enable_mdns: bool,
    pub heartbeat_interval: u64,
}

impl Default for P2PConfig {
    fn default() -> Self {
        Self {
            listen_addr: "/ip4/0.0.0.0/tcp/9000".parse().expect("Invalid default listen_addr"),
            bootstrap_peers: Vec::new(),
            max_peers: 50,
            enable_mdns: false,
            heartbeat_interval: 30,
        }
    }
}

impl P2PConfig {
    pub fn from_file(path: &str) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: P2PConfig = toml::from_str(&content)?;
        config.validate()?;
        Ok(config)
    }
    
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.max_peers == 0 {
            return Err(anyhow::anyhow!("max_peers must be > 0"));
        }
        // Validate that bootstrap peer addresses are valid Multiaddrs
        // (They're already Multiaddr types, so this is just a sanity check)
        for addr in &self.bootstrap_peers {
            if addr.to_string().is_empty() {
                return Err(anyhow::anyhow!("Bootstrap peer address cannot be empty"));
            }
        }
        Ok(())
    }
}

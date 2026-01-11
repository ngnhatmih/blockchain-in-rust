use libp2p::Multiaddr;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P2PConfig {
    #[serde(with = "multiaddr_serde")]
    pub listen_addr: Multiaddr,
    #[serde(with = "multiaddr_vec_serde")]
    pub bootstrap_peers: Vec<Multiaddr>,
    pub max_peers: usize,
    pub enable_mdns: bool,
    pub heartbeat_interval: u64,
}

// Helper modules for serializing Multiaddr as string
mod multiaddr_serde {
    use libp2p::Multiaddr;
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(addr: &Multiaddr, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&addr.to_string())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Multiaddr, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

mod multiaddr_vec_serde {
    use libp2p::Multiaddr;
    use serde::{Deserialize, Deserializer, Serializer, ser::SerializeSeq};

    pub fn serialize<S>(addrs: &Vec<Multiaddr>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(addrs.len()))?;
        for addr in addrs {
            seq.serialize_element(&addr.to_string())?;
        }
        seq.end()
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<Multiaddr>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let strings: Vec<String> = Vec::deserialize(deserializer)?;
        strings
            .into_iter()
            .map(|s| s.parse().map_err(serde::de::Error::custom))
            .collect()
    }
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

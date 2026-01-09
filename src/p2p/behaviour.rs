use libp2p::{
    gossipsub::{self, IdentTopic, MessageId, Message, ValidationMode, MessageAuthenticity},
    identify,
    ping,
    request_response::{self, ProtocolSupport, Config as RequestResponseConfig, Codec},
    mdns,
    PeerId,
};
use libp2p::swarm::NetworkBehaviour;
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::io;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BlockSyncRequest {
    GetBlock { height: u64 },
    GetBlocks { start: u64, end: u64 },
    GetStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BlockSyncResponse {
    Block(Vec<u8>),
    Blocks(Vec<Vec<u8>>),
    Status { height: u64 },
    NotFound,
}

#[derive(Clone)]
pub struct JsonCodec;

#[async_trait::async_trait]
impl Codec for JsonCodec {
    type Protocol = &'static str;
    type Request = BlockSyncRequest;
    type Response = BlockSyncResponse;

    async fn read_request<T>(&mut self, _: &Self::Protocol, io: &mut T) -> io::Result<Self::Request>
    where
        T: futures::AsyncRead + Unpin + Send,
    {
        let mut buf = Vec::new();
        futures::AsyncReadExt::read_to_end(io, &mut buf).await?;
        serde_json::from_slice(&buf)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    async fn read_response<T>(&mut self, _: &Self::Protocol, io: &mut T) -> io::Result<Self::Response>
    where
        T: futures::AsyncRead + Unpin + Send,
    {
        let mut buf = Vec::new();
        futures::AsyncReadExt::read_to_end(io, &mut buf).await?;
        serde_json::from_slice(&buf)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    async fn write_request<T>(&mut self, _: &Self::Protocol, io: &mut T, req: Self::Request) -> io::Result<()>
    where
        T: futures::AsyncWrite + Unpin + Send,
    {
        let buf = serde_json::to_vec(&req)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        futures::AsyncWriteExt::write_all(io, &buf).await?;
        Ok(())
    }

    async fn write_response<T>(&mut self, _: &Self::Protocol, io: &mut T, resp: Self::Response) -> io::Result<()>
    where
        T: futures::AsyncWrite + Unpin + Send,
    {
        let buf = serde_json::to_vec(&resp)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        futures::AsyncWriteExt::write_all(io, &buf).await?;
        Ok(())
    }
}

#[derive(NetworkBehaviour)]
#[behaviour(out_event = "BlockchainBehaviourEvent")]
pub struct BlockchainBehaviour {
    pub block_sync: request_response::Behaviour<JsonCodec>,
    pub gossipsub: gossipsub::Behaviour,
    pub ping: ping::Behaviour,
    pub identify: identify::Behaviour,
    pub mdns: mdns::tokio::Behaviour,
}

#[derive(Debug)]
pub enum BlockchainBehaviourEvent {
    BlockSync(request_response::Event<BlockSyncRequest, BlockSyncResponse>),
    Gossipsub(gossipsub::Event),
    Ping(ping::Event),
    Identify(identify::Event),
    Mdns(mdns::Event),
}

impl From<request_response::Event<BlockSyncRequest, BlockSyncResponse>> for BlockchainBehaviourEvent {
    fn from(event: request_response::Event<BlockSyncRequest, BlockSyncResponse>) -> Self {
        BlockchainBehaviourEvent::BlockSync(event)
    }
}

impl From<gossipsub::Event> for BlockchainBehaviourEvent {
    fn from(event: gossipsub::Event) -> Self {
        BlockchainBehaviourEvent::Gossipsub(event)
    }
}

impl From<ping::Event> for BlockchainBehaviourEvent {
    fn from(event: ping::Event) -> Self {
        BlockchainBehaviourEvent::Ping(event)
    }
}

impl From<identify::Event> for BlockchainBehaviourEvent {
    fn from(event: identify::Event) -> Self {
        BlockchainBehaviourEvent::Identify(event)
    }
}

impl From<mdns::Event> for BlockchainBehaviourEvent {
    fn from(event: mdns::Event) -> Self {
        BlockchainBehaviourEvent::Mdns(event)
    }
}

impl BlockchainBehaviour {
    pub fn new(
        local_peer_id: PeerId,
        local_key: &libp2p::identity::Keypair,
        _enable_mdns: bool,
        heartbeat_interval: u64,
    ) -> anyhow::Result<Self> {
        // 1. Block Sync Protocol (Request-Response)
        let block_sync = request_response::Behaviour::with_codec(
            JsonCodec,
            [("/blockchain/sync/1.0.0", ProtocolSupport::Full)],
            RequestResponseConfig::default(),
        );

        // 2. Gossipsub Protocol (Pub/Sub for block announcements)
        let message_id_fn = |message: &Message| {
            let mut hasher = Sha256::new();
            hasher.update(&message.data);
            MessageId::from(hasher.finalize().to_vec())
        };

        let gossipsub_config = gossipsub::ConfigBuilder::default()
            .heartbeat_interval(Duration::from_secs(heartbeat_interval))
            .validation_mode(ValidationMode::Strict)
            .message_id_fn(message_id_fn)
            .build()
            .map_err(|e| anyhow::anyhow!("Failed to build gossipsub config: {}", e))?;

        let mut gossipsub = gossipsub::Behaviour::new(
            MessageAuthenticity::Signed(local_key.clone()),
            gossipsub_config,
        )
        .map_err(|e| anyhow::anyhow!("Failed to create gossipsub: {}", e))?;

        let block_topic = IdentTopic::new("blocks");
        gossipsub.subscribe(&block_topic)
            .map_err(|e| anyhow::anyhow!("Failed to subscribe to blocks topic: {}", e))?;

        // 3. Ping Protocol (Keep-alive and latency measurement)
        let ping = ping::Behaviour::new(
            ping::Config::new()
                .with_interval(Duration::from_secs(heartbeat_interval))
        );

        // 4. Identify Protocol (Peer discovery and info exchange)
        let identify_config = identify::Config::new(
            "/blockchain/1.0.0".to_string(),
            local_key.public()
        );
        let identify = identify::Behaviour::new(identify_config);

        // mDNS for local network discovery
        // Note: Always created, but events can be ignored if enable_mdns is false
        let mdns = mdns::tokio::Behaviour::new(
            mdns::Config::default(),
            local_peer_id,
        )?;

        Ok(Self {
            block_sync,
            gossipsub,
            ping,
            identify,
            mdns,
        })
    }
}


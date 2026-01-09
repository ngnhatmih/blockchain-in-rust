# Network và P2P trong Blockchain Rust

## Tổng quan về P2P Network

P2P (Peer-to-Peer) là mô hình mạng phân tán, trong đó các node (peer) kết nối trực tiếp với nhau mà không cần server trung tâm. Trong blockchain, mỗi node vừa là client vừa là server, cho phép:

- **Phân tán**: Không có điểm thất bại đơn lẻ (single point of failure)
- **Khả năng mở rộng**: Dễ dàng thêm node mới vào mạng
- **Kháng kiểm duyệt**: Không có cơ quan trung tâm kiểm soát
- **Truyền tin hiệu quả**: Block mới được lan truyền nhanh chóng qua mạng

## Tại sao sử dụng libp2p?

**libp2p** là một framework mạng modular và extensible được thiết kế cho các ứng dụng P2P. Lý do chọn libp2p:

1. **Modular Architecture**: Cho phép chọn lựa các thành phần cần thiết (transport, security, discovery, routing)
2. **Cross-platform**: Hoạt động trên nhiều nền tảng (Linux, Windows, macOS, mobile)
3. **Production-ready**: Được sử dụng bởi nhiều dự án lớn (IPFS, Ethereum 2.0, Polkadot)
4. **Rich Protocol Suite**: Cung cấp sẵn nhiều protocol phổ biến (gossipsub, request-response, ping, identify)
5. **Type-safe**: Được viết bằng Rust, đảm bảo an toàn bộ nhớ và hiệu suất cao
6. **NAT Traversal**: Hỗ trợ các kỹ thuật vượt qua NAT/firewall

## Cấu trúc Folder

```
src/
├── p2p/                    # Core P2P networking layer
│   ├── mod.rs             # Module exports
│   ├── config.rs          # P2P configuration (listen address, bootstrap peers, etc.)
│   ├── swarm.rs           # Swarm management và event handling
│   ├── behaviour.rs       # libp2p behaviours (gossipsub, request-response, ping, identify, mdns)
│   └── peer_manager.rs    # Peer reputation và blacklist management
│
└── network/                # High-level network protocols
    ├── mod.rs             # Module exports
    ├── sync.rs            # Block synchronization manager
    └── protocols/         # Network protocol handlers
        ├── mod.rs
        ├── block_announce.rs  # Block announcement via gossipsub
        └── block_sync.rs      # Block sync via request-response
```

## Các tính năng được hỗ trợ

1. **Peer Discovery**
   - Bootstrap peers (kết nối đến các node đã biết)
   - mDNS discovery (tự động tìm peer trong mạng local)
   - Identify protocol (trao đổi thông tin peer)

2. **Block Propagation**
   - Gossipsub protocol để lan truyền block mới
   - Topic-based pub/sub messaging
   - Message validation và deduplication

3. **Block Synchronization**
   - Request-Response protocol để đồng bộ block
   - Batch block requests (yêu cầu nhiều block cùng lúc)
   - Status checking (kiểm tra chiều cao blockchain)

4. **Connection Management**
   - TCP transport với Noise encryption
   - Yamux multiplexing (nhiều stream trên một connection)
   - Keep-alive với Ping protocol
   - Connection timeout và retry logic

5. **Peer Management**
   - Reputation system (đánh giá độ tin cậy peer)
   - Blacklist/ban mechanism
   - Validator tracking

## Các khái niệm căn bản của libp2p

### 1. PeerId - Định danh duy nhất của mỗi peer

**PeerId** là định danh duy nhất cho mỗi peer trong mạng, được tạo từ public key của peer. Mỗi peer có một PeerId không đổi.

```rust
// src/p2p/swarm.rs
let local_key = identity::Keypair::generate_ed25519();
let local_peer_id = PeerId::from(local_key.public());
tracing::info!("Local peer id: {}", local_peer_id);
```

**Ví dụ sử dụng trong project:**
- Lưu trữ thông tin peer trong `PeerManager`
- Theo dõi peer đang sync trong `SyncManager`
- Gửi request đến peer cụ thể

```rust
// src/p2p/peer_manager.rs
pub struct PeerInfo {
    pub peer_id: PeerId,  // Định danh peer
    pub reputation: i32,
    pub misbehavior_count: u32,
    pub last_seen: Instant,
    pub is_validator: bool,
}
```

### 2. Multiaddr - Địa chỉ mạng đa giao thức

**Multiaddr** là định dạng địa chỉ mạng có thể mô tả nhiều lớp giao thức (protocol stack). Ví dụ: `/ip4/127.0.0.1/tcp/9000` hoặc `/ip6/::1/tcp/9000/ws`.

```rust
// src/p2p/config.rs
pub struct P2PConfig {
    pub listen_addr: Multiaddr,           // Địa chỉ lắng nghe
    pub bootstrap_peers: Vec<Multiaddr>,  // Danh sách peer để kết nối ban đầu
    // ...
}

// Ví dụ sử dụng
let config = P2PConfig {
    listen_addr: "/ip4/0.0.0.0/tcp/9000".parse()?,
    bootstrap_peers: vec!["/ip4/127.0.0.1/tcp/9000".parse()?],
    // ...
};
```

**Ví dụ trong test:**
```rust
// examples/two_nodes_test.rs
let mut config1 = P2PConfig::default();
config1.listen_addr = "/ip4/127.0.0.1/tcp/9000".parse()?;

let mut config2 = P2PConfig::default();
config2.listen_addr = "/ip4/127.0.0.1/tcp/9001".parse()?;
config2.bootstrap_peers = vec!["/ip4/127.0.0.1/tcp/9000".parse()?];
```

### 3. Transport - Lớp vận chuyển dữ liệu

**Transport** xác định cách dữ liệu được truyền qua mạng. Project sử dụng **TCP + Noise (encryption) + Yamux (multiplexing)**.

```rust
// src/p2p/swarm.rs
// Build transport: TCP + Noise + Yamux
let transport = tcp::tokio::Transport::default()
    .upgrade(upgrade::Version::V1Lazy)           // Protocol upgrade
    .authenticate(noise::Config::new(&local_key)?) // Noise encryption
    .multiplex(yamux::Config::default())          // Yamux multiplexing
    .boxed();
```

**Giải thích:**
- **TCP**: Giao thức truyền tải cơ bản
- **Noise**: Mã hóa end-to-end, đảm bảo bảo mật
- **Yamux**: Multiplexing, cho phép nhiều stream trên một connection

### 4. Swarm - Quản lý kết nối và events

**Swarm** là trái tim của libp2p, quản lý tất cả kết nối, events, và routing. Swarm kết hợp Transport và Behaviour.

```rust
// src/p2p/swarm.rs
pub struct P2PNetwork {
    pub swarm: Swarm<BlockchainBehaviour>,  // Swarm với custom behaviour
    pub local_peer_id: PeerId,
    // ...
}

// Khởi tạo Swarm
let swarm = Swarm::new(
    transport,           // Transport layer
    behaviour,           // Network behaviour
    local_peer_id,       // Peer ID
    swarm_config,        // Configuration
);

// Lắng nghe trên địa chỉ
swarm.listen_on(config.listen_addr.clone())?;

// Kết nối đến peer khác
swarm.dial(bootstrap_addr)?;
```

**Event loop - xử lý events từ Swarm:**
```rust
// examples/two_nodes_test.rs
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
```

### 5. Behaviour - Định nghĩa logic mạng

**Behaviour** xác định cách peer phản ứng với các sự kiện mạng. Project sử dụng `NetworkBehaviour` macro để kết hợp nhiều behaviours.

```rust
// src/p2p/behaviour.rs
#[derive(NetworkBehaviour)]
#[behaviour(out_event = "BlockchainBehaviourEvent")]
pub struct BlockchainBehaviour {
    pub block_sync: request_response::Behaviour<JsonCodec>,  // Request-Response
    pub gossipsub: gossipsub::Behaviour,                      // Pub/Sub
    pub ping: ping::Behaviour,                                 // Keep-alive
    pub identify: identify::Behaviour,                        // Peer info
    pub mdns: mdns::tokio::Behaviour,                         // Local discovery
}
```

**Khởi tạo Behaviour:**
```rust
// src/p2p/behaviour.rs
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
        let gossipsub_config = gossipsub::ConfigBuilder::default()
            .heartbeat_interval(Duration::from_secs(heartbeat_interval))
            .validation_mode(ValidationMode::Strict)
            .message_id_fn(message_id_fn)
            .build()?;

        let mut gossipsub = gossipsub::Behaviour::new(
            MessageAuthenticity::Signed(local_key.clone()),
            gossipsub_config,
        )?;

        let block_topic = IdentTopic::new("blocks");
        gossipsub.subscribe(&block_topic)?;

        // 3. Ping Protocol
        let ping = ping::Behaviour::new(
            ping::Config::new()
                .with_interval(Duration::from_secs(heartbeat_interval))
        );

        // 4. Identify Protocol
        let identify = identify::Behaviour::new(
            identify::Config::new(
                "/blockchain/1.0.0".to_string(),
                local_key.public()
            )
        );

        // 5. mDNS
        let mdns = mdns::tokio::Behaviour::new(
            mdns::Config::default(),
            local_peer_id,
        )?;

        Ok(Self { block_sync, gossipsub, ping, identify, mdns })
    }
}
```

### 6. Gossipsub - Pub/Sub cho Block Announcement

**Gossipsub** là giao thức pub/sub dựa trên gossip, cho phép lan truyền message hiệu quả trong mạng P2P. Sử dụng để thông báo block mới.

```rust
// src/network/protocols/block_announce.rs
pub struct BlockAnnounceHandler {
    block_topic: IdentTopic,  // Topic "blocks"
}

impl BlockAnnounceHandler {
    pub fn announce_block(
        &self,
        block: &Block,
        swarm: &mut Swarm<BlockchainBehaviour>,
    ) -> anyhow::Result<()> {
        let data = serde_json::to_vec(block)?;
        
        // Publish block lên topic "blocks"
        swarm.behaviour_mut()
            .gossipsub
            .publish(self.block_topic.clone(), data)?;
        
        tracing::debug!("Announced block {}", block.index);
        Ok(())
    }
    
    pub async fn handle_message(
        &self,
        message: Message,
        blockchain: Arc<Mutex<Blockchain>>,
    ) -> anyhow::Result<()> {
        let block: Block = serde_json::from_slice(&message.data)?;
        let mut chain = blockchain.lock().await;
        
        // Validate và append block
        if chain.validate_new_block(&block) {
            chain.append_block(block.clone())?;
            tracing::info!("Received and added block {} via gossip", block.index);
        }
        Ok(())
    }
}
```

**Xử lý Gossipsub events:**
```rust
// src/p2p/swarm.rs
super::behaviour::BlockchainBehaviourEvent::Gossipsub(gossipsub_event) => {
    match gossipsub_event {
        libp2p::gossipsub::Event::Message { message, .. } => {
            // Xử lý message nhận được
            if let Some(ref handler) = self.block_announce_handler {
                if let Some(ref blockchain) = self.blockchain {
                    handler.handle_message(message, blockchain.clone()).await?;
                }
            }
        }
        libp2p::gossipsub::Event::Subscribed { peer_id, topic } => {
            tracing::info!("Peer {} subscribed to topic {}", peer_id, topic);
        }
        // ...
    }
}
```

### 7. Request-Response - Đồng bộ Block

**Request-Response** là giao thức cho phép peer gửi request và nhận response. Sử dụng để đồng bộ block từ peer khác.

```rust
// src/p2p/behaviour.rs
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
```

**Xử lý Request (server side):**
```rust
// src/network/protocols/block_sync.rs
pub async fn handle_request(
    &self,
    request: BlockSyncRequest,
    channel: ResponseChannel<BlockSyncResponse>,
    swarm: &mut Swarm<BlockchainBehaviour>,
) -> anyhow::Result<()> {
    let response = match request {
        BlockSyncRequest::GetBlock { height } => {
            let chain = self.blockchain.lock().await;
            if let Some(block) = chain.get_block(height as usize) {
                let serialized = serde_json::to_vec(block)?;
                BlockSyncResponse::Block(serialized)
            } else {
                BlockSyncResponse::NotFound
            }
        }
        BlockSyncRequest::GetBlocks { start, end } => {
            let chain = self.blockchain.lock().await;
            let mut blocks = Vec::new();
            for height in start..=end {
                if let Some(block) = chain.get_block(height as usize) {
                    blocks.push(serde_json::to_vec(block)?);
                }
            }
            BlockSyncResponse::Blocks(blocks)
        }
        BlockSyncRequest::GetStatus => {
            let chain = self.blockchain.lock().await;
            BlockSyncResponse::Status { height: chain.len() as u64 }
        }
    };

    swarm.behaviour_mut()
        .block_sync
        .send_response(channel, response)?;
    Ok(())
}
```

**Gửi Request (client side):**
```rust
// src/p2p/swarm.rs
pub fn send_block_sync_request(
    &mut self,
    peer: PeerId,
    request: BlockSyncRequest,
) -> anyhow::Result<()> {
    self.swarm.behaviour_mut()
        .block_sync
        .send_request(&peer, request);
    Ok(())
}
```

**Xử lý Response:**
```rust
// src/p2p/swarm.rs
super::behaviour::BlockchainBehaviourEvent::BlockSync(sync_event) => {
    match sync_event {
        libp2p::request_response::Event::Message { message, .. } => {
            match message {
                libp2p::request_response::Message::Response { response, .. } => {
                    match response {
                        BlockSyncResponse::Status { height } => {
                            // Xử lý status response
                            let ranges = sync.process_status_response(height).await;
                            // Request blocks dựa trên status
                        }
                        BlockSyncResponse::Blocks(blocks_data) => {
                            // Xử lý blocks nhận được
                            sync.process_blocks_response(blocks_data).await?;
                        }
                        // ...
                    }
                }
                // ...
            }
        }
    }
}
```

### 8. Ping - Keep-alive và đo độ trễ

**Ping** protocol dùng để kiểm tra kết nối còn sống và đo độ trễ (latency) giữa các peer.

```rust
// src/p2p/behaviour.rs
let ping = ping::Behaviour::new(
    ping::Config::new()
        .with_interval(Duration::from_secs(heartbeat_interval))
);
```

**Xử lý Ping events:**
```rust
// src/p2p/swarm.rs
super::behaviour::BlockchainBehaviourEvent::Ping(ping_event) => {
    match ping_event {
        libp2p::ping::Event { peer, result, .. } => {
            match result {
                Ok(duration) => {
                    tracing::debug!("Ping to {}: {:?}", peer, duration);
                    // Tăng reputation nếu ping thành công
                    let mut pm = self.peer_manager.lock().await;
                    pm.increase_reputation(&peer, 1);
                }
                Err(e) => {
                    tracing::warn!("Ping failed to {}: {:?}", peer, e);
                    // Giảm reputation nếu ping thất bại
                    let mut pm = self.peer_manager.lock().await;
                    pm.decrease_reputation(&peer, 5);
                }
            }
        }
    }
}
```

### 9. Identify - Trao đổi thông tin Peer

**Identify** protocol cho phép peer trao đổi thông tin về nhau (protocol version, public key, listen addresses).

```rust
// src/p2p/behaviour.rs
let identify_config = identify::Config::new(
    "/blockchain/1.0.0".to_string(),  // Protocol version
    local_key.public()                 // Public key
);
let identify = identify::Behaviour::new(identify_config);
```

**Xử lý Identify events:**
```rust
// src/p2p/swarm.rs
super::behaviour::BlockchainBehaviourEvent::Identify(identify_event) => {
    match identify_event {
        libp2p::identify::Event::Received { peer_id, info } => {
            tracing::info!(
                "Received identify info from {}: protocol={}", 
                peer_id, 
                info.protocol_version
            );
        }
        libp2p::identify::Event::Sent { .. } => {
            tracing::debug!("Sent identify info");
        }
        // ...
    }
}
```

### 10. mDNS - Peer Discovery trong mạng local

**mDNS** (multicast DNS) cho phép tự động phát hiện peer trong cùng mạng local mà không cần bootstrap.

```rust
// src/p2p/behaviour.rs
let mdns = mdns::tokio::Behaviour::new(
    mdns::Config::default(),
    local_peer_id,
)?;
```

**Xử lý mDNS events:**
```rust
// src/p2p/swarm.rs
super::behaviour::BlockchainBehaviourEvent::Mdns(mdns_event) => {
    if !self.config.enable_mdns {
        return; // Ignore nếu mDNS bị tắt
    }
    match mdns_event {
        libp2p::mdns::Event::Discovered(list) => {
            for (peer_id, multiaddr) in list {
                tracing::info!("mDNS discovered peer: {} at {}", peer_id, multiaddr);
                // Có thể tự động kết nối đến peer được phát hiện
                if self.peer_count() < self.config.max_peers {
                    // Dial peer
                }
            }
        }
        libp2p::mdns::Event::Expired(list) => {
            for (peer_id, _) in list {
                tracing::debug!("mDNS peer expired: {}", peer_id);
            }
        }
    }
}
```

### 11. Peer Manager - Quản lý Reputation và Blacklist

**PeerManager** quản lý thông tin peer, reputation system, và blacklist để đảm bảo chất lượng kết nối.

```rust
// src/p2p/peer_manager.rs
pub struct PeerManager {
    peers: HashMap<PeerId, PeerInfo>,
    blacklist: HashSet<PeerId>,
    validators: HashSet<PeerId>,
    ban_threshold: i32,
    unban_threshold: i32,
}

impl PeerManager {
    pub fn add_peer(&mut self, peer_id: PeerId, is_validator: bool) {
        let peer_info = self.peers.entry(peer_id).or_insert_with(|| PeerInfo {
            peer_id,
            reputation: 0,
            misbehavior_count: 0,
            last_seen: Instant::now(),
            is_validator,
        });
        peer_info.last_seen = Instant::now();
    }
    
    pub fn increase_reputation(&mut self, peer_id: &PeerId, delta: i32) {
        if let Some(peer_info) = self.peers.get_mut(peer_id) {
            peer_info.reputation += delta;
            // Tự động unban nếu reputation cải thiện
            if peer_info.reputation > self.unban_threshold && self.blacklist.contains(peer_id) {
                self.unban_peer(peer_id);
            }
        }
    }
    
    pub fn decrease_reputation(&mut self, peer_id: &PeerId, delta: i32) {
        if let Some(peer_info) = self.peers.get_mut(peer_id) {
            peer_info.reputation -= delta;
            peer_info.misbehavior_count += 1;
            // Tự động ban nếu reputation quá thấp
            if peer_info.reputation < self.ban_threshold {
                self.ban_peer(peer_id);
            }
        }
    }
}
```

**Sử dụng trong event handling:**
```rust
// src/p2p/swarm.rs
SwarmEvent::ConnectionEstablished { peer_id, .. } => {
    tracing::info!("Connection established with peer: {}", peer_id);
    let mut pm = self.peer_manager.lock().await;
    pm.add_peer(peer_id, false);
}
```

### 12. Sync Manager - Đồng bộ Blockchain

**SyncManager** quản lý quá trình đồng bộ block từ peer khác, bao gồm status checking, batch requests, và state management.

```rust
// src/network/sync.rs
pub struct SyncManager {
    blockchain: Arc<Mutex<Blockchain>>,
    syncing_peer: Option<PeerId>,
    state: SyncState,
    target_height: u64,
    pending_blocks: HashMap<u64, bool>,
    batch_size: u64,
}

impl SyncManager {
    pub async fn process_status_response(&mut self, peer_height: u64) -> Vec<(u64, u64)> {
        let local_height = {
            let chain = self.blockchain.lock().await;
            chain.len() as u64
        };
        
        if peer_height <= local_height {
            self.state = SyncState::Completed;
            return Vec::new();
        }
        
        self.target_height = peer_height;
        self.state = SyncState::SyncingBlocks;
        
        // Tính toán các block cần request
        self.calculate_missing_blocks(local_height, peer_height)
    }
    
    pub async fn process_blocks_response(&mut self, blocks_data: Vec<Vec<u8>>) -> anyhow::Result<bool> {
        for block_data in blocks_data {
            let block: Block = serde_json::from_slice(&block_data)?;
            let mut chain = self.blockchain.lock().await;
            
            if chain.validate_new_block(&block) {
                chain.append_block(block.clone())?;
                tracing::info!("Synced block {}", block.index);
            }
        }
        
        // Kiểm tra xem đã sync xong chưa
        let current_height = {
            let chain = self.blockchain.lock().await;
            chain.len() as u64
        };
        
        if current_height >= self.target_height {
            self.state = SyncState::Completed;
            return Ok(true);
        }
        
        Ok(false)
    }
}
```

**Tự động bắt đầu sync khi kết nối:**
```rust
// src/p2p/swarm.rs
SwarmEvent::ConnectionEstablished { peer_id, .. } => {
    // Auto-start sync nếu chưa đang sync
    if let Some(ref sync_mgr) = self.sync_manager {
        let mut sync = sync_mgr.lock().await;
        if !sync.is_syncing() {
            sync.start_sync(peer_id);
            // Request status để bắt đầu sync
            self.send_block_sync_request(peer_id, BlockSyncRequest::GetStatus)?;
        }
    }
}
```

## Luồng hoạt động tổng quan

1. **Khởi tạo Node:**
   - Tạo keypair và PeerId
   - Khởi tạo Transport (TCP + Noise + Yamux)
   - Tạo Behaviour với các protocol
   - Tạo Swarm và lắng nghe trên địa chỉ

2. **Peer Discovery:**
   - Kết nối đến bootstrap peers
   - mDNS discovery (nếu enabled)
   - Identify protocol trao đổi thông tin

3. **Block Propagation:**
   - Khi có block mới, publish lên Gossipsub topic "blocks"
   - Các peer subscribe topic sẽ nhận được block
   - Validate và append block vào chain

4. **Block Synchronization:**
   - Khi kết nối peer mới, tự động request status
   - So sánh chiều cao, tính toán block cần sync
   - Request blocks theo batch
   - Validate và append blocks

5. **Connection Maintenance:**
   - Ping protocol giữ kết nối sống
   - PeerManager theo dõi reputation
   - Ban/unban peer dựa trên behavior

## Ví dụ sử dụng

Xem file `examples/two_nodes_test.rs` để xem ví dụ đầy đủ về cách khởi tạo và kết nối hai node.

```rust
// Khởi tạo node 1
let mut config1 = P2PConfig::default();
config1.listen_addr = "/ip4/127.0.0.1/tcp/9000".parse()?;
let mut node1 = P2PNetwork::new(config1)?;
node1.listen()?;

// Khởi tạo node 2 và kết nối đến node 1
let mut config2 = P2PConfig::default();
config2.listen_addr = "/ip4/127.0.0.1/tcp/9001".parse()?;
config2.bootstrap_peers = vec!["/ip4/127.0.0.1/tcp/9000".parse()?];
let mut node2 = P2PNetwork::new(config2)?;
node2.listen()?;
node2.bootstrap()?;

// Event loop
loop {
    tokio::select! {
        event = node1.swarm.next() => {
            node1.handle_event(event).await;
        }
        event = node2.swarm.next() => {
            node2.handle_event(event).await;
        }
    }
}
```

## Tài liệu tham khảo

- [libp2p Documentation](https://docs.rs/libp2p/)
- [libp2p Specification](https://github.com/libp2p/specs)
- [Gossipsub Specification](https://github.com/libp2p/specs/blob/master/pubsub/gossipsub/gossipsub-v1.1.md)
- [Noise Protocol Framework](http://noiseprotocol.org/)


# Configuration Files

This directory contains configuration files for RustChain nodes.

## Available Configs

### `node.toml`
Default configuration for Node 1 (listens on port 9000, no bootstrap peers).

### `local_node1.toml`
Configuration for Node 2 (listens on port 9001, connects to Node 1 on port 9000).

### `local_node2.toml`
Configuration for Node 3 (listens on port 9002, connects to Node 1 on port 9000).

## Usage

### Run with default config
```bash
cargo run
# or
cargo run -- configs/node.toml
```

### Run with custom config
```bash
cargo run -- configs/node_with_bootstrap.toml
```

### Run with custom path
```bash
cargo run -- /path/to/your/config.toml
```

## Configuration Options

- **listen_addr**: Address to listen on for incoming connections
  - Format: `/ip4/<ip>/tcp/<port>` or `/ip6/<ip>/tcp/<port>`
  - Example: `/ip4/0.0.0.0/tcp/9000`

- **bootstrap_peers**: List of peer addresses to connect to on startup
  - Format: Array of multiaddr strings
  - Example: `["/ip4/127.0.0.1/tcp/9000"]`

- **max_peers**: Maximum number of peers to connect to
  - Default: 50

- **enable_mdns**: Enable mDNS for local network discovery
  - Useful for local development
  - Default: false

- **heartbeat_interval**: Heartbeat interval in seconds
  - Used for ping protocol and gossipsub heartbeat
  - Default: 30

## Example: Running Multiple Nodes Locally

### Chạy 2 Node

#### Terminal 1 - Node 1 (Standalone, listens on port 9000)
```bash
cargo run -- configs/node.toml
```

#### Terminal 2 - Node 2 (Connects to Node 1, listens on port 9001)
```bash
cargo run -- configs/local_node1.toml
```

### Chạy 3 Node

#### Terminal 1 - Node 1 (Standalone, listens on port 9000)
```bash
cargo run -- configs/node.toml
```

#### Terminal 2 - Node 2 (Connects to Node 1, listens on port 9001)
```bash
cargo run -- configs/local_node1.toml
```

#### Terminal 3 - Node 3 (Connects to Node 1, listens on port 9002)
```bash
cargo run -- configs/local_node2.toml
```

**Lưu ý:**
- Chạy Node 1 trước, sau đó mới chạy Node 2 và Node 3
- Node 2 và Node 3 sẽ tự động kết nối với Node 1 thông qua bootstrap peer
- Các node sẽ tự động phát hiện và kết nối với nhau thông qua P2P network
- Tất cả node chạy trên localhost (127.0.0.1)
- Node 1: port 9000
- Node 2: port 9001
- Node 3: port 9002


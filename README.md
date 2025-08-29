# Mini Starknet Indexer

A high-performance Rust-based Starknet event indexer with unified GraphQL API, real-time WebSocket subscriptions, and advanced ABI-aware event decoding. Built for production use with configurable filtering, multi-contract support, and TheGraph-like clean data structures.

## 🚀 Key Features

### 🎯 **Unified API Design**
- **Single Events Query**: One powerful query handles all use cases (single/multiple contracts, advanced filtering, pagination)
- **Single Events Subscription**: One real-time subscription for all scenarios
- **TheGraph-like Clean Data**: Events return clean `{ field_name: decoded_value }` structures
- **No API Complexity**: No more `events`, `eventsAdvanced`, `eventsByContract` - just one `events` query for everything

### 🔧 **Advanced Event Decoding**
- **Full ABI Awareness**: Automatically fetches and parses contract ABIs for intelligent event decoding
- **Unlimited Nested Structs**: Complete support for complex nested data structures
- **Smart Type Conversion**: Automatic conversion of felt252, uint types, booleans, addresses, and strings
- **Readable Output**: Raw hex values converted to human-readable strings and numbers
- **Backward Compatible**: Handles events indexed with older formats

### ⚡ **Real-time Capabilities**
- **True WebSocket Events**: Instant event broadcasting when indexed (no polling delays)
- **Filtered Subscriptions**: Subscribe to specific contracts, event types, and keys
- **Multiple Subscribers**: Concurrent real-time connections supported
- **Automatic Management**: Self-managing subscription lifecycle

### 🏗️ **Production-Ready Architecture**
- **Multi-Contract Indexing**: Index multiple contracts simultaneously with independent configurations
- **Configurable Start Blocks**: Each contract can start from different block heights
- **Rate Limiting**: Built-in RPC throttling and retry logic to prevent 429 errors
- **Database Optimization**: Advanced filtering and indexing for fast queries
- **Address Normalization**: Automatic Starknet address validation and padding

## 📋 Quick Start

### Prerequisites
- Rust 1.70+ and Cargo
- Access to a Starknet RPC endpoint

### Basic Usage
```bash
# Start with single contract (auto-detects from environment or uses defaults)
cargo run

# Index specific contract from recent block
cargo run -- --contract-config "0x02cf12918a78bb09bb553590cc05d1ee8edd6bbb829c84464c0374fa620c983e:1900000"

# Index multiple contracts with different start blocks
cargo run -- --contract-config "0x02cf12918a78bb09bb553590cc05d1ee8edd6bbb829c84464c0374fa620c983e:1900000,0x04718f5a0fc34cc1af16a1cdee98ffb20c31f5cd61d6ab07201858f4287c938d:1901000"

# Add filtering for specific event types
cargo run -- --contract-config "0x02cf12918a78bb09bb553590cc05d1ee8edd6bbb829c84464c0374fa620c983e:1900000" --event-types "Transfer,Approval"
```

### Instant GraphQL Playground
Once running, access the GraphQL playground at:
- **Playground**: http://localhost:3000/graphql
- **API Endpoint**: POST http://localhost:3000/graphql
- **WebSocket**: ws://localhost:3000/ws

## ⚙️ Configuration

### Environment Variables
```bash
# Core Configuration
RPC_URL=https://starknet-mainnet.public.blastapi.io
CONTRACT_CONFIG=0x02cf12918a78bb09bb553590cc05d1ee8edd6bbb829c84464c0374fa620c983e:1900000
DATABASE_URL=sqlite:events.db

# Multiple contracts (comma-separated)
CONTRACT_CONFIG=0x02cf12918a78bb09bb553590cc05d1ee8edd6bbb829c84464c0374fa620c983e:1900000,0x04718f5a0fc34cc1af16a1cdee98ffb20c31f5cd61d6ab07201858f4287c938d:1901000
```

### Command Line Options
```bash
# View all options
cargo run -- --help

# Key options:
--rpc-url <URL>                 # Starknet RPC endpoint
--contract-config <CONFIG>      # address:start_block,address:start_block
--start-block <BLOCK>          # Global fallback start block
--chunk-size <SIZE>            # Blocks per chunk (default: 2000)
--sync-interval <SECONDS>      # Sync check interval (default: 2)
--event-types <TYPES>          # Filter specific event types
--event-keys <KEYS>            # Filter specific event keys
--batch-mode                   # Enable batch processing
--max-retries <RETRIES>        # RPC retry attempts (default: 3)
```

### Address Validation & Normalization
All contract addresses are automatically:
- **Validated**: Must start with `0x` and contain valid hex
- **Normalized**: Padded to 64 characters (32 bytes)
- **Error-Checked**: Invalid addresses cause clear error messages

```bash
# These are automatically normalized:
0x123 → 0x0000000000000000000000000000000000000000000000000000000000000123
0x1   → 0x0000000000000000000000000000000000000000000000000000000000000001

# Invalid addresses cause errors:
❌ invalid_address → "contract address must start with 0x"
❌ 0xinvalid      → "contract address must be hexadecimal"
```

## 🎯 Unified GraphQL API

### Universal Events Query
**One query handles everything** - single contracts, multiple contracts, advanced filtering, pagination, and ordering:

```graphql
query UniversalEvents {
  events(
    # Single contract
    contractAddress: "0x02cf12918a78bb09bb553590cc05d1ee8edd6bbb829c84464c0374fa620c983e"
    
    # OR multiple contracts
    contractAddresses: [
      "0x02cf12918a78bb09bb553590cc05d1ee8edd6bbb829c84464c0374fa620c983e",
      "0x04718f5a0fc34cc1af16a1cdee98ffb20c31f5cd61d6ab07201858f4287c938d"
    ]
    
    # Advanced filtering
    eventTypes: ["Transfer", "Approval"]
    eventKeys: ["0x1234"]
    fromBlock: "1900000"
    toBlock: "2000000"
    fromTimestamp: "2024-01-01T00:00:00Z"
    toTimestamp: "2024-12-31T23:59:59Z"
    transactionHash: "0xabc..."
    
    # Pagination & ordering
    first: 10
    after: "cursor123"
    orderBy: BLOCK_NUMBER_DESC
  ) {
    totalCount
    pageInfo {
      hasNextPage
      hasPreviousPage
      startCursor
      endCursor
    }
    edges {
      cursor
      node {
        id
        contractAddress
        eventType
        blockNumber
        transactionHash
        timestamp
        data  # Clean decoded data structure
        rawData
        rawKeys
      }
    }
  }
}
```

### Clean Data Structure
Events return clean, TheGraph-like data structures:

```json
{
  "eventType": "Transfer",
  "data": {
    "from": "0x1234...",
    "to": "0x5678...",
    "value": 1000000000000000000
  }
}
```

**Instead of messy nested structures:**
```json
{
  "eventType": "Transfer", 
  "decodedData": {
    "fields": [
      {"name": "from", "value": "0x1234...", "decodedValue": {"hex": "0x1234...", "type": "address"}},
      {"name": "to", "value": "0x5678...", "decodedValue": {"hex": "0x5678...", "type": "address"}},
      {"name": "value", "value": "0xde0b6b3a7640000", "decodedValue": {"decimal": 1000000000000000000}}
    ]
  }
}
```

### Real-time Subscriptions
**One subscription handles all real-time scenarios:**

```graphql
subscription RealTimeEvents {
  events(
    # Single or multiple contracts
    contractAddress: "0x02cf12918a78bb09bb553590cc05d1ee8edd6bbb829c84464c0374fa620c983e"
    
    # Event filtering
    eventTypes: ["Transfer"]
    eventKeys: ["0x1234"]
  ) {
    id
    contractAddress
    eventType
    blockNumber
    data
  }
}
```

**Real-time Features:**
- ✅ **Instant Delivery**: Events broadcast immediately when indexed
- ✅ **True WebSocket**: No polling delays
- ✅ **Filtered Streams**: Subscribe to specific events only
- ✅ **Multiple Connections**: Concurrent subscribers supported

## 🔧 REST API Endpoints

### Health & Status
```bash
# Health check
GET /test

# Detailed sync status
GET /sync-status
# Returns:
{
  "status": "fully_synced",
  "current_block": 1903179,
  "last_synced_block": 1903179,
  "blocks_behind": 0,
  "sync_percentage": "100.00%",
  "contracts": [
    {
      "address": "0x02cf12918a78bb09bb553590cc05d1ee8edd6bbb829c84464c0374fa620c983e",
      "last_synced_block": 1903179,
      "status": "fully_synced"
    }
  ]
}

# Contract statistics
GET /stats/{contract_address}
# Returns:
{
  "contract_address": "0x02cf12918a78bb09bb553590cc05d1ee8edd6bbb829c84464c0374fa620c983e",
  "total_events": 1500,
  "event_types": {
    "Transfer": 1200,
    "Approval": 300
  },
  "block_range": {
    "min": 1900000,
    "max": 1903179
  },
  "time_range": {
    "min": "2024-01-01T00:00:00Z",
    "max": "2024-01-01T12:00:00Z"
  }
}
```

### Contract Information
```bash
# Get contract ABI
GET /get-abi/{contract_address}
```

## 🏗️ Technical Architecture

### Core Components
```
src/
├── main.rs              # CLI parser, server setup, and configuration
├── indexer.rs           # Multi-contract background indexing with rate limiting
├── database.rs          # SQLite operations with advanced filtering and ordering
├── starknet.rs          # RPC client, ABI parsing, and intelligent event decoding
├── realtime.rs          # Real-time WebSocket event broadcasting
└── graphql/
    ├── schema.rs        # Unified GraphQL schema
    ├── types.rs         # Clean, simplified GraphQL types
    └── resolvers/
        ├── events.rs    # Universal events query with all filtering options
        ├── subscriptions.rs # Universal real-time subscriptions
        └── contracts.rs # Contract ABI and metadata queries
```

### Database Schema
```sql
-- Events with decoded and raw data
CREATE TABLE events (
    id TEXT PRIMARY KEY,
    contract_address TEXT NOT NULL,
    event_type TEXT NOT NULL,
    block_number INTEGER NOT NULL,
    transaction_hash TEXT NOT NULL,
    log_index INTEGER NOT NULL,
    timestamp TEXT NOT NULL,
    decoded_data TEXT,          -- Clean JSON: {"field": "value"}
    raw_data TEXT NOT NULL,     -- Original data array
    raw_keys TEXT NOT NULL      -- Original keys array
);

-- Multi-contract indexer state
CREATE TABLE indexer_state (
    id INTEGER PRIMARY KEY,
    contract_address TEXT UNIQUE NOT NULL,
    last_synced_block INTEGER NOT NULL,
    updated_at TEXT NOT NULL
);

-- Optimized indexes for fast queries
CREATE INDEX idx_events_contract_block ON events(contract_address, block_number);
CREATE INDEX idx_events_type ON events(event_type);
CREATE INDEX idx_events_timestamp ON events(timestamp);
```

### Key Dependencies
```toml
[dependencies]
axum = "0.7"                    # High-performance HTTP server
async-graphql = "7"             # GraphQL server with subscriptions
sqlx = "0.7"                   # Async database operations
tokio = "1.0"                  # Async runtime
serde_json = "1.0"             # JSON processing for decoded data
reqwest = "0.11"               # HTTP client for RPC calls
tokio-tungstenite = "0.21"     # WebSocket support
clap = "4.5"                   # CLI argument parsing
chrono = "0.4"                 # Timestamp handling
hex = "0.4"                    # Hex string processing
```

## 🚀 Advanced Features

### ABI-Aware Event Decoding
The indexer automatically:
1. **Fetches Contract ABIs** from RPC endpoints
2. **Parses Event Definitions** including struct and enum types
3. **Matches Events** using heuristic scoring for best ABI fit
4. **Decodes Values** with smart type conversion:
   - `felt252` → readable strings or decimal numbers
   - `uint` types → appropriate numeric values
   - `bool` → true/false
   - `ContractAddress` → normalized hex strings
   - Complex structs → nested JSON objects

### Multi-Contract Management
- **Independent Configuration**: Each contract has its own start block
- **Staggered Startup**: 2-second delays prevent RPC rate limits
- **Parallel Processing**: Multiple contracts indexed simultaneously
- **Unified Queries**: Query events from multiple contracts in single request

### Rate Limiting & Reliability
- **Exponential Backoff**: Automatic retry with increasing delays
- **RPC Throttling**: Built-in delays between requests
- **Error Recovery**: Graceful handling of network failures
- **Configurable Retries**: Customizable retry attempts

### Performance Optimizations
- **Chunked Processing**: Configurable block batch sizes
- **Smart Filtering**: Event filtering during indexing
- **Database Indexing**: Optimized queries for fast retrieval
- **Memory Management**: Efficient handling of large datasets

## 📊 Usage Examples

### High-Performance Configuration
```bash
# Fast sync with filtering
cargo run -- \
  --contract-config "0x02cf12918a78bb09bb553590cc05d1ee8edd6bbb829c84464c0374fa620c983e:1900000" \
  --chunk-size 5000 \
  --sync-interval 1 \
  --batch-mode \
  --event-types "Transfer,Approval" \
  --max-retries 5
```

### Development Setup
```bash
# Quick development setup
cargo run -- \
  --contract-config "0x02cf12918a78bb09bb553590cc05d1ee8edd6bbb829c84464c0374fa620c983e:0" \
  --chunk-size 1000 \
  --event-types "Transfer"
```

### Multi-Contract Production
```bash
# Production multi-contract setup
export CONTRACT_CONFIG="0x02cf12918a78bb09bb553590cc05d1ee8edd6bbb829c84464c0374fa620c983e:1900000,0x04718f5a0fc34cc1af16a1cdee98ffb20c31f5cd61d6ab07201858f4287c938d:1901000"
export RPC_URL=https://your-production-rpc.com
export DATABASE_URL=sqlite:production_events.db

cargo run -- --batch-mode --max-retries 5
```

## 🔍 Monitoring & Debugging

### Health Monitoring
Monitor these endpoints for production health:
- **`GET /sync-status`** - Real-time sync status for all contracts
- **`GET /stats/{contract}`** - Detailed per-contract statistics
- **Console Logs** - Detailed indexing progress and error information

### Performance Metrics
```bash
# Monitor sync status
curl http://localhost:3000/sync-status | jq .

# Check contract statistics
curl http://localhost:3000/stats/0x02cf12918a78bb09bb553590cc05d1ee8edd6bbb829c84464c0374fa620c983e | jq .

# Test GraphQL endpoint
curl -X POST http://localhost:3000/graphql \
  -H "Content-Type: application/json" \
  -d '{"query": "{ events(contractAddress: \"0x02cf12918a78bb09bb553590cc05d1ee8edd6bbb829c84464c0374fa620c983e\", first: 5) { edges { node { eventType data } } } }"}' | jq .
```

## 🚨 Troubleshooting

### Common Issues & Solutions

**1. Rate Limiting (429 Errors)**
- **Solution**: Use different RPC endpoint or reduce chunk size
- **Command**: `--chunk-size 1000 --sync-interval 3`

**2. Memory Usage High**
- **Solution**: Add event filtering and reduce chunk size
- **Command**: `--event-types "Transfer" --chunk-size 1000`

**3. Slow Initial Sync**
- **Solution**: Start from recent block
- **Command**: `--contract-config "0xADDRESS:1900000"`

**4. Database Growing Large**
- **Solution**: Filter events during indexing
- **Command**: `--event-types "Transfer,Approval"`

**5. RPC Connection Issues**
- **Solution**: Increase retry attempts and check endpoint
- **Command**: `--max-retries 10`

### Performance Tuning

```bash
# Memory-constrained environments
cargo run -- --chunk-size 500 --event-types "Transfer"

# High-throughput requirements  
cargo run -- --chunk-size 10000 --sync-interval 1 --batch-mode

# Network reliability issues
cargo run -- --max-retries 10 --sync-interval 5
```

## 🎯 Production Considerations

### Scaling & Deployment
- **Database**: Consider PostgreSQL for high-volume production
- **Horizontal Scaling**: Run multiple instances for different contracts
- **Connection Pooling**: Implement WebSocket connection management
- **Load Balancing**: Use reverse proxy for multiple indexer instances

### Recommended Production Setup
```bash
# Use environment variables for production
export RPC_URL=https://your-production-rpc.com
export CONTRACT_CONFIG=0xCONTRACT1:1900000,0xCONTRACT2:1901000
export DATABASE_URL=sqlite:production_events.db

# Run with production settings
cargo run --release -- \
  --chunk-size 5000 \
  --sync-interval 2 \
  --batch-mode \
  --max-retries 5
```

### Monitoring & Alerting
Set up monitoring for:
- Sync status lag (blocks behind)
- RPC error rates
- Database size growth
- WebSocket connection counts
- Memory usage patterns

## 🤝 Contributing

### Development Setup
```bash
# Clone and build
git clone <repository>
cd mini-starknet-indexer-rs
cargo build

# Run tests
cargo test

# Run with development settings
cargo run -- --contract-config "0x02cf12918a78bb09bb553590cc05d1ee8edd6bbb829c84464c0374fa620c983e:1900000"

# Access GraphQL playground
open http://localhost:3000/graphql
```

### Code Structure
- **Warning-Free**: All compiler warnings have been eliminated
- **Type Safety**: Comprehensive error handling and type checking
- **Documentation**: Inline documentation for all major functions
- **Testing**: Unit tests for core functionality

---

**🎯 This indexer provides a production-ready foundation for Starknet event monitoring with clean APIs, real-time capabilities, and enterprise-grade reliability. Perfect for DeFi protocols, NFT platforms, and any application requiring reliable Starknet event data.**
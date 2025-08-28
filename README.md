# Mini Starknet Indexer

A Rust-based Starknet event indexer with REST and GraphQL APIs for fetching and decoding contract events. This indexer provides real-time event monitoring with configurable filtering and performance optimization.

## How It Works

### Core Architecture
- **Background Indexer**: Continuously monitors the blockchain for new events from specified contracts
- **Database Storage**: SQLite database for persistent event storage with advanced filtering
- **Real-time APIs**: REST and GraphQL endpoints for querying indexed events
- **Address Normalization**: Automatically normalizes Starknet addresses (e.g., `0x02` and `0x2` are treated as the same)

### Indexing Process
1. **Historical Sync**: Scans blocks from a configurable start point to current block
2. **Continuous Sync**: Polls for new blocks every 2 seconds (configurable)
3. **Event Filtering**: Applies filters during indexing to reduce storage and improve performance
4. **Retry Mechanism**: Handles RPC failures with configurable retry attempts
5. **Rate Limiting**: Built-in delays to avoid hitting RPC rate limits

### Subscription Implementation
**Important**: GraphQL subscriptions use polling, not real-time WebSocket events. The subscription polls the RPC every 3 seconds to check for new events in the latest block. This is not a true real-time subscription but provides near real-time updates.

## Quick Start

### Prerequisites
- Rust and Cargo installed
- Access to a Starknet RPC endpoint

### Basic Usage
```bash
# Start with default settings
cargo run

# Start with specific contract
cargo run -- --contract-address 0x02cf12918a78bb09bb553590cc05d1ee8edd6bbb829c84464c0374fa620c983e

# Start from a specific block (faster sync)
cargo run -- --contract-address 0x02cf12918a78bb09bb553590cc05d1ee8edd6bbb829c84464c0374fa620c983e --start-block 1866762
```

## Configuration

### Environment Variables
```bash
RPC_URL=https://starknet-mainnet.public.blastapi.io  # Starknet RPC endpoint
CONTRACT_ADDRESS=0x02cf12918a78bb09bb553590cc05d1ee8edd6bbb829c84464c0374fa620c983e  # Contract to index
DATABASE_URL=sqlite:events.db  # Database file location
```

### Command Line Options
```bash
cargo run -- --help

# Available options:
--rpc-url <URL>               # RPC URL (overrides RPC_URL env)
--contract-address <ADDRESS>  # Contract address (overrides CONTRACT_ADDRESS env)
--start-block <BLOCK>         # Start indexing from this block number
--chunk-size <SIZE>           # Blocks per chunk (default: 2000)
--sync-interval <SECONDS>     # Continuous sync interval (default: 2)
--event-keys <KEYS>           # Comma-separated event keys to filter
--event-types <TYPES>         # Comma-separated event types to filter
--batch-mode                  # Enable batch processing
--max-retries <RETRIES>       # Max RPC retries (default: 3)
```

### Advanced Usage Examples

#### High-Performance Configuration
```bash
cargo run -- \
  --contract-address 0x02cf12918a78bb09bb553590cc05d1ee8edd6bbb829c84464c0374fa620c983e \
  --start-block 1866762 \
  --chunk-size 5000 \
  --sync-interval 1 \
  --batch-mode \
  --event-types "Transfer,Approval" \
  --max-retries 5
```

#### Filtered Indexing
```bash
# Only index Transfer events
cargo run -- --event-types "Transfer" --contract-address 0x...

# Only index events with specific keys
cargo run -- --event-keys "0x1234,0x5678" --contract-address 0x...

# Start from recent block for faster sync
cargo run -- --start-block 1900000 --contract-address 0x...
```

## API Endpoints

### REST API

#### Health Check
```bash
GET /test
```
Returns basic health status.

#### Contract ABI
```bash
GET /get-abi/{contract_address}
```
Fetches raw contract ABI from RPC.

#### Sync Status
```bash
GET /sync-status
```
Returns detailed indexer sync status:
```json
{
  "status": "fully_synced",
  "current_block": 1903179,
  "last_synced_block": 1903179,
  "blocks_behind": 0,
  "sync_percentage": "100.00%",
  "contract_address": "0x02cf12918a78bb09bb553590cc05d1ee8edd6bbb829c84464c0374fa620c983e",
  "last_updated": "2024-01-01T12:00:00Z"
}
```

#### Indexer Statistics
```bash
GET /stats/{contract_address}
```
Returns comprehensive statistics:
```json
{
  "contract_address": "0x02cf12918a78bb09bb553590cc05d1ee8edd6bbb829c84464c0374fa620c983e",
  "total_events": 1500,
  "event_types": {
    "Transfer": 1200,
    "Approval": 300
  },
  "block_range": {
    "min": 1866762,
    "max": 1903179
  },
  "time_range": {
    "min": "2024-01-01T00:00:00Z",
    "max": "2024-01-01T12:00:00Z"
  }
}
```

#### Fetch Events (POST)
```bash
POST /
Content-Type: application/json

{
  "address": "0x02cf12918a78bb09bb553590cc05d1ee8edd6bbb829c84464c0374fa620c983e",
  "chunk_size": 100
}
```

### GraphQL API

#### Endpoints
- **HTTP**: `POST /graphql` - Queries and mutations
- **WebSocket**: `GET /ws` - Subscriptions (polling-based)
- **Playground**: `GET /graphql` - GraphiQL interface

#### Basic Event Query
```graphql
query GetEvents($contractAddress: String!) {
  events(contractAddress: $contractAddress, first: 10) {
    totalCount
    pageInfo { hasNextPage endCursor }
    edges {
      cursor
      node {
        id
        eventType
        blockNumber
        transactionHash
        decodedData { json }
      }
    }
  }
}
```

#### Advanced Filtering
```graphql
query GetFilteredEvents($addr: String!) {
  events(
    contractAddress: $addr
    fromBlock: "1866762"
    toBlock: "1900000"
    eventTypes: ["Transfer"]
    eventKeys: ["0x1234"]
    fromTimestamp: "2024-01-01T00:00:00Z"
    toTimestamp: "2024-01-01T12:00:00Z"
    transactionHash: "0xabcdef..."
    first: 10
  ) {
    edges {
      node {
        id
        eventType
        blockNumber
        transactionHash
        decodedData { json }
      }
    }
    pageInfo { hasNextPage endCursor }
  }
}
```

#### Structured Advanced Query
```graphql
query GetAdvancedEvents($addr: String!) {
  eventsAdvanced(
    contractAddress: $addr
    filters: {
      blockRange: { fromBlock: "1866762", toBlock: "1900000" }
      timeRange: { fromTimestamp: "2024-01-01T00:00:00Z", toTimestamp: "2024-01-01T12:00:00Z" }
      eventTypes: ["Transfer", "Approval"]
      eventKeys: ["0x1234", "0x5678"]
    }
    pagination: { first: 10 }
  ) {
    edges {
      node {
        id
        eventType
        blockNumber
        transactionHash
        decodedData { json }
      }
    }
    pageInfo { hasNextPage endCursor }
  }
}
```

#### Indexer Statistics
```graphql
query GetStats($addr: String!) {
  indexerStats(contractAddress: $addr)
}
```

#### Subscription (Polling-Based)
```graphql
subscription OnEvents($addr: String!) {
  eventStream(contractAddress: $addr, eventTypes: ["Transfer"]) {
    id
    eventType
    blockNumber
    transactionHash
    decodedData { json }
  }
}
```

**Note**: This subscription polls the RPC every 3 seconds for new events in the latest block. It's not a true real-time WebSocket subscription.

## Technical Details

### Database Schema
```sql
-- Events table
CREATE TABLE events (
    id TEXT PRIMARY KEY,
    contract_address TEXT NOT NULL,
    event_type TEXT NOT NULL,
    block_number INTEGER NOT NULL,
    transaction_hash TEXT NOT NULL,
    log_index INTEGER NOT NULL,
    timestamp TEXT NOT NULL,
    decoded_data TEXT,
    raw_data TEXT NOT NULL,
    raw_keys TEXT NOT NULL
);

-- Indexer state table
CREATE TABLE indexer_state (
    id INTEGER PRIMARY KEY,
    contract_address TEXT UNIQUE NOT NULL,
    last_synced_block INTEGER NOT NULL,
    updated_at TEXT NOT NULL
);
```

### Performance Optimizations
- **Chunk Processing**: Configurable block chunk size (default: 2000 blocks)
- **Event Filtering**: Filter events during indexing to reduce storage
- **Address Normalization**: Consistent address handling
- **Retry Logic**: Configurable retry attempts for RPC failures
- **Rate Limiting**: Built-in delays to avoid RPC rate limits
- **Database Indexes**: Optimized queries with proper indexing

### Rate Limiting & Reliability
- **RPC Retries**: Configurable retry attempts (default: 3)
- **Delay Between Chunks**: 500ms delay between block chunks
- **Continuous Sync**: 2-second polling interval (configurable)
- **Subscription Polling**: 3-second interval for GraphQL subscriptions

### Limitations
- **Polling-Based Subscriptions**: Not true real-time WebSocket events
- **Single Contract**: Indexes one contract at a time
- **SQLite Storage**: Not suitable for high-volume production use
- **Memory Usage**: All events loaded into memory for complex filtering

## Development

### Project Structure
```
src/
├── main.rs              # CLI and server setup
├── indexer.rs           # Background indexing logic
├── database.rs          # Database operations and filtering
├── starknet.rs          # RPC client and event decoding
└── graphql/
    ├── schema.rs        # GraphQL schema setup
    ├── types.rs         # GraphQL types and inputs
    └── resolvers/
        ├── events.rs    # Event queries and filtering
        └── subscriptions.rs  # Polling-based subscriptions
```

### Dependencies
- `axum` - HTTP server framework
- `async-graphql` - GraphQL server
- `sqlx` - Database operations
- `tokio` - Async runtime
- `serde` - Serialization
- `reqwest` - HTTP client for RPC calls

### Building and Testing
```bash
# Build
cargo build

# Run tests
cargo test

# Run with specific configuration
cargo run -- --contract-address 0x... --start-block 1866762
```

## Production Considerations

### Scaling Limitations
- **Single Contract**: Currently indexes one contract per instance
- **SQLite**: Not suitable for high-volume production
- **Memory**: Complex filtering loads events into memory
- **Polling**: Subscriptions use polling, not real-time events

### Recommended Improvements
- **Multi-Contract Support**: Index multiple contracts simultaneously
- **PostgreSQL**: Use proper database for production
- **True WebSocket**: Implement real-time event streaming
- **Horizontal Scaling**: Support multiple indexer instances
- **Event Batching**: Batch database operations for better performance

### Monitoring
- **Sync Status**: Monitor `/sync-status` endpoint
- **Statistics**: Use `/stats/{contract}` for detailed metrics
- **Logs**: Watch console output for indexing progress
- **Database Size**: Monitor SQLite file size growth

## Troubleshooting

### Common Issues
1. **Rate Limiting**: Increase delays or use better RPC endpoint
2. **Memory Usage**: Reduce chunk size or add event filtering
3. **Slow Sync**: Use `--start-block` to skip historical data
4. **RPC Errors**: Increase `--max-retries` or check RPC endpoint

### Performance Tuning
```bash
# For high-throughput contracts
cargo run -- --chunk-size 10000 --sync-interval 1 --batch-mode

# For memory-constrained environments
cargo run -- --chunk-size 1000 --event-types "Transfer"

# For faster initial sync
cargo run -- --start-block 1900000 --chunk-size 5000
```

This indexer provides a solid foundation for Starknet event monitoring with configurable filtering and performance optimization. While it has some limitations for production use, it's well-suited for development, testing, and moderate-scale event indexing needs.

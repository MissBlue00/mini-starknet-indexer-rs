# Mini Starknet Indexer

A Rust-based Starknet event indexer with REST and GraphQL APIs for fetching and decoding contract events. This indexer provides real-time event monitoring with configurable filtering and performance optimization.

## How It Works

### Core Architecture
- **Background Indexer**: Continuously monitors the blockchain for new events from specified contracts
- **Multi-Contract Support**: Index multiple contracts simultaneously using allow lists
- **Database Storage**: SQLite database for persistent event storage with advanced filtering
- **Real-time APIs**: REST and GraphQL endpoints for querying indexed events
- **Real-time Event Broadcasting**: WebSocket-based event streaming for instant updates
- **Address Normalization**: Automatically normalizes Starknet addresses (e.g., `0x02` and `0x2` are treated as the same)

### Indexing Process
1. **Historical Sync**: Scans blocks from a configurable start point to current block
2. **Continuous Sync**: Polls for new blocks every 2 seconds (configurable)
3. **Event Filtering**: Applies filters during indexing to reduce storage and improve performance
4. **Retry Mechanism**: Handles RPC failures with configurable retry attempts
5. **Rate Limiting**: Built-in delays to avoid hitting RPC rate limits

### Subscription Implementation
**Real-time WebSocket Subscriptions**: GraphQL subscriptions now use true real-time WebSocket events instead of polling. When new events are indexed, they are immediately broadcast to all connected subscribers via WebSocket connections. This provides instant real-time updates as events occur on the blockchain.

## Quick Start

### Prerequisites
- Rust and Cargo installed
- Access to a Starknet RPC endpoint

### Basic Usage
```bash
# Start with default settings
cargo run

# Start with single contract
cargo run -- --allow-list "0x02cf12918a78bb09bb553590cc05d1ee8edd6bbb829c84464c0374fa620c983e"

# Start from a specific block (faster sync)
cargo run -- --allow-list "0x02cf12918a78bb09bb553590cc05d1ee8edd6bbb829c84464c0374fa620c983e" --start-block 1866762

# Start with multiple contracts
cargo run -- --allow-list "0x02cf12918a78bb09bb553590cc05d1ee8edd6bbb829c84464c0374fa620c983e,0x04718f5a0fc34cc1af16a1cdee98ffb20c31f5cd61d6ab07201858f4287c938d"
```

## Configuration

### Environment Variables
```bash
RPC_URL=https://starknet-mainnet.public.blastapi.io  # Starknet RPC endpoint
CONTRACT_ALLOW_LIST=0x02cf12918a78bb09bb553590cc05d1ee8edd6bbb829c84464c0374fa620c983e  # Single contract to index
# CONTRACT_ALLOW_LIST=0x02cf12918a78bb09bb553590cc05d1ee8edd6bbb829c84464c0374fa620c983e,0x04718f5a0fc34cc1af16a1cdee98ffb20c31f5cd61d6ab07201858f4287c938d  # Multiple contracts to index
DATABASE_URL=sqlite:events.db  # Database file location
```

### Command Line Options
```bash
cargo run -- --help

# Available options:
--rpc-url <URL>               # RPC URL (overrides RPC_URL env)
--allow-list <ADDRESSES>      # Comma-separated list of contract addresses to index (overrides CONTRACT_ALLOW_LIST env)
--start-block <BLOCK>         # Start indexing from this block number
--chunk-size <SIZE>           # Blocks per chunk (default: 2000)
--sync-interval <SECONDS>     # Continuous sync interval (default: 2)
--event-keys <KEYS>           # Comma-separated event keys to filter
--event-types <TYPES>         # Comma-separated event types to filter
--batch-mode                  # Enable batch processing
--max-retries <RETRIES>       # Max RPC retries (default: 3)
```

### Contract Configuration

The indexer supports indexing single or multiple contracts using a unified configuration approach. Simply provide contracts in the format `address:start_block` in a comma-separated list. **All addresses are automatically validated and normalized** to ensure consistency and prevent errors.

#### Using Contract Configuration

**Command Line:**
```bash
# Single contract (starts from block 0)
cargo run -- --contract-config "0x02cf12918a78bb09bb553590cc05d1ee8edd6bbb829c84464c0374fa620c983e:0"

# Single contract with specific start block
cargo run -- --contract-config "0x02cf12918a78bb09bb553590cc05d1ee8edd6bbb829c84464c0374fa620c983e:1900000"

# Multiple contracts with different start blocks
cargo run -- --contract-config "0x02cf12918a78bb09bb553590cc05d1ee8edd6bbb829c84464c0374fa620c983e:1900000,0x04718f5a0fc34cc1af16a1cdee98ffb20c31f5cd61d6ab07201858f4287c938d:1901000"

# With filtering
cargo run -- --contract-config "0x1234:0,0x5678:0" --event-types "Transfer,Approval"
```

#### Address Validation and Normalization

All contract addresses are automatically validated and normalized:

- **Validation**: Ensures addresses start with `0x` and contain valid hexadecimal characters
- **Normalization**: Pads addresses to 64 characters (32 bytes) with leading zeros
- **Error Handling**: Invalid addresses cause the indexer to exit with a clear error message

**Examples:**
```bash
# These addresses are automatically normalized:
0x123 → 0x0000000000000000000000000000000000000000000000000000000000000123
0x1   → 0x0000000000000000000000000000000000000000000000000000000000000001
0xabc → 0x0000000000000000000000000000000000000000000000000000000000000abc

# Invalid addresses will cause errors:
❌ invalid_address → "contract address must start with 0x"
❌ 0xinvalid      → "contract address must be hexadecimal"
```

#### Start Block Configuration

Each contract can have its own start block specified in the configuration:

**Format:** `address:start_block,address:start_block`

**Examples:**
```bash
# Command line
cargo run -- --contract-config "0x123:1900000,0x456:1901000,0x789:1902000"

# Environment variable
export CONTRACT_CONFIG="0x123:1900000,0x456:1901000,0x789:1902000"
cargo run
```

**Benefits:**
- **Flexible Sync**: Start indexing different contracts from different blocks
- **Performance**: Skip historical data for contracts that don't need it
- **Selective Indexing**: Only index recent events for certain contracts
- **Fallback**: Use `--start-block` for global default, or default to block 0
- **Rate Limiting**: Automatic staggered startup and retry logic to prevent RPC rate limits

**Priority Order:**
1. Contract-specific start block (from `--contract-config`)
2. Global start block (from `--start-block`)
3. Default (block 0)

**Rate Limiting Features:**
- **Staggered Startup**: Multiple contracts start with 2-second delays to avoid overwhelming RPC
- **Exponential Backoff**: Automatic retry with increasing delays on rate limit errors
- **RPC Throttling**: Built-in delays between requests to prevent 429 errors

**Environment Variable:**
```bash
# Single contract (starts from block 0)
export CONTRACT_CONFIG="0x02cf12918a78bb09bb553590cc05d1ee8edd6bbb829c84464c0374fa620c983e:0"

# Single contract with specific start block
export CONTRACT_CONFIG="0x02cf12918a78bb09bb553590cc05d1ee8edd6bbb829c84464c0374fa620c983e:1900000"

# Multiple contracts with different start blocks
export CONTRACT_CONFIG="0x02cf12918a78bb09bb553590cc05d1ee8edd6bbb829c84464c0374fa620c983e:1900000,0x04718f5a0fc34cc1af16a1cdee98ffb20c31f5cd61d6ab07201858f4287c938d:1901000"

# Addresses are automatically normalized (e.g., 0x123 becomes 0x0000000000000000000000000000000000000000000000000000000000000123)
cargo run
```

#### GraphQL Multi-Contract Queries

Query events from multiple contracts with clear separation by contract address:

##### ⭐ **Recommended: Grouped by Contract**
```graphql
query GetEventsByContract($addresses: [String!]!) {
  eventsByContract(
    contractAddresses: $addresses
    first: 5
    eventTypes: ["Transfer", "Approval"]
  ) {
    contracts {
      contractAddress
      events {
        edges {
          node {
            id
            eventType
            blockNumber
            transactionHash
            decodedData { json }
          }
        }
        totalCount
      }
    }
    totalContracts
    totalEvents
  }
}
```

**Example Response:**
```json
{
  "data": {
    "eventsByContract": {
      "contracts": [
        {
          "contractAddress": "0x02cf12918a78bb09bb553590cc05d1ee8edd6bbb829c84464c0374fa620c983e",
          "events": {
            "edges": [
              {
                "node": {
                  "id": "0x40055e1c78f993a6adaa21674832ea28ee3ddc7f235b3c9d280478b721c8ecd:56",
                  "eventType": "U8Event",
                  "blockNumber": "1867957",
                  "transactionHash": "0x40055e1c78f993a6adaa21674832ea28ee3ddc7f235b3c9d280478b721c8ecd",
                  "decodedData": {
                    "json": "{\"_keys\":[\"0x1b3f460470a2db288f8bf618e8a6680d13b76f4aad6ab571a741264e1b0d6c2\"]}"
                  }
                }
              }
            ],
            "totalCount": 57
          }
        },
        {
          "contractAddress": "0x04718f5a0fc34cc1af16a1cdee98ffb20c31f5cd61d6ab07201858f4287c938d",
          "events": {
            "edges": [
              {
                "node": {
                  "id": "0x278c4df4d38e3235fd4d97d1b28126cf20ec793618b344c16ccdca0471366:32",
                  "eventType": "Transfer",
                  "blockNumber": "1904725",
                  "transactionHash": "0x278c4df4d38e3235fd4d97d1b28126cf20ec793618b344c16ccdca0471366",
                  "decodedData": {
                    "json": "{\"from\":\"0x20d7fb2face98b97dbcd35c228adcd3dbb8b89915fa5af740b5b34548a9b5e1\",\"to\":\"0x1176a1bd84444c89232ec27754698e5d2e7e1a7f1539f12027f28b23ec9f3d8\",\"value\":\"0x41a13074db00\"}"
                  }
                }
              }
            ],
            "totalCount": 4675
          }
        }
      ],
      "totalContracts": 2,
      "totalEvents": 4732
    }
  }
}
```

**Benefits:**
- ✅ **Clear Separation**: Each contract has its own section
- ✅ **Easy Processing**: No need to check `contractAddress` field in each event
- ✅ **Per-Contract Stats**: Each contract shows its own `totalCount`
- ✅ **Better Performance**: Events queried per contract with parallel processing

##### **Legacy: Flat List (Still Available)**
```graphql
query GetMultiContractEvents($addresses: [String!]!) {
  eventsMultiContract(
    contractAddresses: $addresses
    first: 10
    eventTypes: ["Transfer"]
  ) {
    totalCount
    edges {
      node {
        id
        contractAddress
        eventType
        blockNumber
        transactionHash
        decodedData { json }
      }
    }
  }
}
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
cargo run -- --event-types "Transfer" --allow-list "0x..."

# Only index events with specific keys
cargo run -- --event-keys "0x1234,0x5678" --allow-list "0x..."

# Start from recent block for faster sync
cargo run -- --start-block 1900000 --allow-list "0x..."

# Index multiple contracts with filtering
cargo run -- --allow-list "0x1234,0x5678" --event-types "Transfer,Approval"
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

#### Multi-Contract Event Query
```graphql
query GetMultiContractEvents($addresses: [String!]!) {
  eventsMultiContract(
    contractAddresses: $addresses
    first: 10
    eventTypes: ["Transfer", "Approval"]
  ) {
    totalCount
    pageInfo { hasNextPage endCursor }
    edges {
      cursor
      node {
        id
        contractAddress
        eventType
        blockNumber
        transactionHash
        decodedData { json }
      }
    }
  }
}
```

**Variables:**
```json
{
  "addresses": [
    "0x02cf12918a78bb09bb553590cc05d1ee8edd6bbb829c84464c0374fa620c983e",
    "0x04718f5a0fc34cc1af16a1cdee98ffb20c31f5cd61d6ab07201858f4287c938d"
  ]
}
```

#### Real-time WebSocket Subscription
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

**Real-time Features**:
- ✅ **True WebSocket Events**: Events are broadcast immediately when indexed
- ✅ **Instant Updates**: No polling delays - events arrive as they happen
- ✅ **Efficient Filtering**: Subscribe to specific event types and keys
- ✅ **Multiple Subscribers**: Support for multiple concurrent subscriptions
- ✅ **Automatic Cleanup**: Subscriptions are managed automatically

**Usage**: Connect to `ws://localhost:3000/ws` and subscribe to events. Events will be pushed in real-time as they are indexed from the blockchain.

## Real-time Features

### WebSocket Subscriptions
The indexer now supports true real-time WebSocket subscriptions that broadcast events immediately when they are indexed from the blockchain. This eliminates the polling delay and provides instant updates.

#### How It Works
1. **Event Indexing**: When the background indexer finds new events, they are immediately broadcast to all connected subscribers
2. **WebSocket Connections**: Clients connect to `ws://localhost:3000/ws` for real-time event streams
3. **Filtered Subscriptions**: Subscribe to specific contracts, event types, and event keys
4. **Automatic Management**: Subscriptions are automatically created and cleaned up

#### Subscription Features
- **Real-time Broadcasting**: Events are pushed instantly when indexed
- **Multiple Subscribers**: Support for concurrent subscriptions
- **Event Filtering**: Filter by contract address, event types, and event keys
- **Connection Management**: Automatic handling of WebSocket connections
- **Error Handling**: Graceful handling of connection errors and timeouts

#### Example Usage
```javascript
// Connect to WebSocket endpoint
const ws = new WebSocket('ws://localhost:3000/ws');

// Subscribe to Transfer events from a specific contract
const subscription = {
  query: `
    subscription OnTransferEvents($addr: String!) {
      eventStream(contractAddress: $addr, eventTypes: ["Transfer"]) {
        id
        eventType
        blockNumber
        transactionHash
        decodedData { json }
      }
    }
  `,
  variables: {
    addr: "0x02cf12918a78bb09bb553590cc05d1ee8edd6bbb829c84464c0374fa620c983e"
  }
};

ws.send(JSON.stringify(subscription));

// Handle incoming events
ws.onmessage = (event) => {
  const data = JSON.parse(event.data);
  console.log('New Transfer event:', data);
};
```

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
- **Real-time Broadcasting**: Efficient event broadcasting to WebSocket subscribers

### Rate Limiting & Reliability
- **RPC Retries**: Configurable retry attempts (default: 3)
- **Delay Between Chunks**: 500ms delay between block chunks
- **Continuous Sync**: 2-second polling interval (configurable)
- **Real-time Events**: Instant WebSocket broadcasting when events are indexed

### Limitations
- **SQLite Storage**: Not suitable for high-volume production use
- **Memory Usage**: All events loaded into memory for complex filtering
- **WebSocket Connections**: Limited by available system resources

## Development

### Project Structure
```
src/
├── main.rs              # CLI and server setup
├── indexer.rs           # Background indexing logic
├── database.rs          # Database operations and filtering
├── starknet.rs          # RPC client and event decoding
├── realtime.rs          # Real-time event streaming and WebSocket management
└── graphql/
    ├── schema.rs        # GraphQL schema setup
    ├── types.rs         # GraphQL types and inputs
    └── resolvers/
        ├── events.rs    # Event queries and filtering
        └── subscriptions.rs  # Real-time WebSocket subscriptions
```

### Dependencies
- `axum` - HTTP server framework
- `async-graphql` - GraphQL server
- `sqlx` - Database operations
- `tokio` - Async runtime
- `serde` - Serialization
- `reqwest` - HTTP client for RPC calls
- `tokio-tungstenite` - WebSocket support
- `uuid` - Unique identifier generation

### Building and Testing
```bash
# Build
cargo build

# Run tests
cargo test

# Run with specific configuration
cargo run -- --contract-address 0x... --start-block 1866762

# Test real-time WebSocket subscriptions
cd examples
npm install
npm test
```

## Production Considerations

### Scaling Limitations
- **Single Contract**: Currently indexes one contract per instance
- **SQLite**: Not suitable for high-volume production
- **Memory**: Complex filtering loads events into memory
- **Polling**: Subscriptions use polling, not real-time events

### Recommended Improvements
- **PostgreSQL**: Use proper database for production
- **Horizontal Scaling**: Support multiple indexer instances
- **Event Batching**: Batch database operations for better performance
- **Connection Management**: Implement WebSocket connection pooling
- **Event Persistence**: Store subscription events for offline clients

### Monitoring
- **Sync Status**: Monitor `/sync-status` endpoint
- **Statistics**: Use `/stats/{contract}` for detailed metrics
- **Logs**: Watch console output for indexing progress
- **Database Size**: Monitor SQLite file size growth

## Troubleshooting

### Common Issues
1. **Rate Limiting**: Multiple contracts are automatically staggered to prevent RPC rate limits. If you still see 429 errors, try using a different RPC endpoint or increasing delays.
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

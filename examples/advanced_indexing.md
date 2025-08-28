# Advanced Indexing Examples

This document demonstrates how to use the new block filtering and advanced indexing features.

## Quick Start with Block Filtering

### 1. Start Indexing from a Recent Block

If you want to focus on recent events and skip historical data:

```bash
# Start indexing from block 1,000,000 (much faster than starting from 0)
cargo run -- --start-block 1000000 --contract-address 0x04718f5a0fc34cc1af16a1cdee98ffb20c31f5cd61d6ab07201858f4287c938d
```

### 2. Filter by Event Types

Only index specific event types to reduce storage and improve performance:

```bash
# Only index Transfer and Approval events
cargo run -- --event-types "Transfer,Approval" --contract-address 0x04718f5a0fc34cc1af16a1cdee98ffb20c31f5cd61d6ab07201858f4287c938d
```

### 3. Filter by Event Keys

Only index events with specific keys (useful for tracking specific addresses or parameters):

```bash
# Only index events with specific keys
cargo run -- --event-keys "0x1234,0x5678" --contract-address 0x04718f5a0fc34cc1af16a1cdee98ffb20c31f5cd61d6ab07201858f4287c938d
```

### 4. Performance Optimization

Optimize for your specific use case:

```bash
# High-performance configuration for recent events only
cargo run -- \
  --start-block 1000000 \
  --chunk-size 5000 \
  --sync-interval 1 \
  --batch-mode \
  --event-types "Transfer" \
  --contract-address 0x04718f5a0fc34cc1af16a1cdee98ffb20c31f5cd61d6ab07201858f4287c938d
```

## API Usage Examples

### 1. Check Sync Status

```bash
curl http://localhost:3000/sync-status
```

### 2. Get Indexer Statistics

```bash
curl http://localhost:3000/stats/0x04718f5a0fc34cc1af16a1cdee98ffb20c31f5cd61d6ab07201858f4287c938d
```

### 3. GraphQL Queries with Advanced Filtering

#### Basic Filtering
```graphql
query GetRecentTransfers($addr: String!) {
  events(
    contractAddress: $addr
    fromBlock: "1000000"
    eventTypes: ["Transfer"]
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
  }
}
```

#### Advanced Filtering with Timestamps
```graphql
query GetFilteredEvents($addr: String!) {
  events(
    contractAddress: $addr
    fromBlock: "1000000"
    toBlock: "1100000"
    eventTypes: ["Transfer"]
    eventKeys: ["0x1234"]
    fromTimestamp: "2024-01-01T00:00:00Z"
    toTimestamp: "2024-01-01T12:00:00Z"
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
  }
}
```

#### Structured Advanced Query
```graphql
query GetAdvancedEvents($addr: String!) {
  eventsAdvanced(
    contractAddress: $addr
    filters: {
      blockRange: { fromBlock: "1000000", toBlock: "1100000" }
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
    pageInfo {
      hasNextPage
      endCursor
    }
  }
}
```

#### Get Indexer Statistics
```graphql
query GetStats($addr: String!) {
  indexerStats(contractAddress: $addr)
}
```

## Use Cases

### 1. New Contract Deployment

For newly deployed contracts, start indexing from the deployment block:

```bash
# Contract deployed at block 1,500,000
cargo run -- --start-block 1500000 --contract-address 0x...
```

### 2. Focus on Recent Activity

For active contracts, focus on recent events:

```bash
# Only index last 100,000 blocks
cargo run -- --start-block 1400000 --contract-address 0x...
```

### 3. Specific Event Tracking

Track only specific events for better performance:

```bash
# Only track Transfer events
cargo run -- --event-types "Transfer" --contract-address 0x...
```

### 4. Address-Specific Tracking

Track events for specific addresses:

```bash
# Only track events with specific keys (addresses)
cargo run -- --event-keys "0x1234,0x5678" --contract-address 0x...
```

### 5. High-Performance Setup

For production environments with high throughput:

```bash
cargo run -- \
  --start-block 1000000 \
  --chunk-size 10000 \
  --sync-interval 1 \
  --batch-mode \
  --max-retries 5 \
  --event-types "Transfer,Approval" \
  --contract-address 0x...
```

## Monitoring

### 1. Sync Status

Monitor the indexer's sync status:

```bash
# Check sync status
curl http://localhost:3000/sync-status

# Expected response:
{
  "status": "fully_synced",
  "current_block": 1234567,
  "last_synced_block": 1234567,
  "blocks_behind": 0,
  "sync_percentage": "100.00%",
  "contract_address": "0x...",
  "last_updated": "2024-01-01T12:00:00Z"
}
```

### 2. Indexer Statistics

Get detailed statistics about indexed events:

```bash
# Get statistics
curl http://localhost:3000/stats/0x...

# Expected response:
{
  "contract_address": "0x...",
  "total_events": 1500,
  "event_types": {
    "Transfer": 1200,
    "Approval": 300
  },
  "block_range": {
    "min": 1000000,
    "max": 1234567
  },
  "time_range": {
    "min": "2024-01-01T00:00:00Z",
    "max": "2024-01-01T12:00:00Z"
  }
}
```

## Performance Tips

1. **Use start-block**: Always use `--start-block` for new contracts or when resuming indexing
2. **Filter events**: Use `--event-types` and `--event-keys` to reduce storage and improve performance
3. **Optimize chunk size**: Use larger chunk sizes (5000-10000) for better throughput
4. **Enable batch mode**: Use `--batch-mode` for better performance
5. **Monitor sync status**: Regularly check sync status to ensure the indexer is keeping up
6. **Use appropriate RPC**: Use a reliable RPC endpoint to avoid rate limiting

## Troubleshooting

### Indexer Falling Behind

If the indexer is falling behind:

1. Increase chunk size: `--chunk-size 10000`
2. Decrease sync interval: `--sync-interval 1`
3. Enable batch mode: `--batch-mode`
4. Use a better RPC endpoint

### High Memory Usage

If memory usage is high:

1. Reduce chunk size: `--chunk-size 1000`
2. Filter events: `--event-types "Transfer"`
3. Use start-block to skip historical data

### Slow Queries

If queries are slow:

1. Add database indexes (already implemented)
2. Use specific filters in GraphQL queries
3. Limit result sets with pagination
4. Use block range filters to reduce data scanned

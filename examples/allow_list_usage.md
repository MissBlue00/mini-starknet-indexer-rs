# Allow List Feature Guide

This guide demonstrates how to use the allow list feature to index multiple contracts simultaneously in the Mini Starknet Indexer.

## Overview

The allow list feature enables you to monitor multiple contracts at once, which is useful for:
- Monitoring related contracts (e.g., token + governance contracts)
- Tracking multiple tokens simultaneously
- Building comprehensive event monitoring systems

## Configuration Methods

### 1. Command Line

```bash
# Basic allow list
cargo run -- --allow-list "0x02cf12918a78bb09bb553590cc05d1ee8edd6bbb829c84464c0374fa620c983e,0x04718f5a0fc34cc1af16a1cdee98ffb20c31f5cd61d6ab07201858f4287c938d"

# With filtering
cargo run -- --allow-list "0x1234,0x5678" --event-types "Transfer,Approval"

# With performance tuning
cargo run -- --allow-list "0x1234,0x5678" --chunk-size 5000 --sync-interval 1 --batch-mode
```

### 2. Environment Variable

```bash
# Set environment variable
export CONTRACT_ALLOW_LIST="0x02cf12918a78bb09bb553590cc05d1ee8edd6bbb829c84464c0374fa620c983e,0x04718f5a0fc34cc1af16a1cdee98ffb20c31f5cd61d6ab07201858f4287c938d"

# Start indexer
cargo run
```

### 3. .env File

```bash
# Create .env file
echo "CONTRACT_ALLOW_LIST=0x02cf12918a78bb09bb553590cc05d1ee8edd6bbb829c84464c0374fa620c983e,0x04718f5a0fc34cc1af16a1cdee98ffb20c31f5cd61d6ab07201858f4287c938d" > .env

# Start indexer
cargo run
```

## GraphQL Queries

### Single Contract Query

```graphql
query GetSingleContractEvents($address: String!) {
  events(contractAddress: $address, first: 10) {
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

**Variables:**
```json
{
  "address": "0x02cf12918a78bb09bb553590cc05d1ee8edd6bbb829c84464c0374fa620c983e"
}
```

### Multi-Contract Query

```graphql
query GetMultiContractEvents($addresses: [String!]!) {
  eventsMultiContract(
    contractAddresses: $addresses
    first: 10
    eventTypes: ["Transfer", "Approval"]
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

**Variables:**
```json
{
  "addresses": [
    "0x02cf12918a78bb09bb553590cc05d1ee8edd6bbb829c84464c0374fa620c983e",
    "0x04718f5a0fc34cc1af16a1cdee98ffb20c31f5cd61d6ab07201858f4287c938d"
  ]
}
```

### Advanced Multi-Contract Query

```graphql
query GetAdvancedMultiContractEvents($addresses: [String!]!) {
  eventsMultiContract(
    contractAddresses: $addresses
    fromBlock: "1900000"
    toBlock: "1901000"
    eventTypes: ["Transfer"]
    eventKeys: ["0x1234"]
    fromTimestamp: "2024-01-01T00:00:00Z"
    toTimestamp: "2024-01-01T12:00:00Z"
    first: 20
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
        logIndex
        timestamp
        decodedData { json }
        rawData
        rawKeys
      }
    }
  }
}
```

## REST API Examples

### Using curl

```bash
# Single contract events
curl -X POST http://localhost:3000/ \
  -H "Content-Type: application/json" \
  -d '{
    "address": "0x02cf12918a78bb09bb553590cc05d1ee8edd6bbb829c84464c0374fa620c983e",
    "chunk_size": 100
  }'

# Sync status
curl http://localhost:3000/sync-status

# Indexer stats
curl http://localhost:3000/stats/0x02cf12918a78bb09bb553590cc05d1ee8edd6bbb829c84464c0374fa620c983e
```

## Real-World Examples

### Example 1: Token + Governance Monitoring

```bash
# Start indexer for token and governance contracts
cargo run -- --allow-list "0x02cf12918a78bb09bb553590cc05d1ee8edd6bbb829c84464c0374fa620c983e,0x04718f5a0fc34cc1af16a1cdee98ffb20c31f5cd61d6ab07201858f4287c938d" --event-types "Transfer,ProposalCreated,VoteCast"
```

**GraphQL Query:**
```graphql
query GetTokenAndGovernanceEvents($addresses: [String!]!) {
  eventsMultiContract(
    contractAddresses: $addresses
    eventTypes: ["Transfer", "ProposalCreated", "VoteCast"]
    first: 50
  ) {
    edges {
      node {
        contractAddress
        eventType
        blockNumber
        decodedData { json }
      }
    }
  }
}
```

### Example 2: Multiple Token Monitoring

```bash
# Monitor multiple ERC20 tokens
cargo run -- --allow-list "0x1234,0x5678,0x9abc" --event-types "Transfer,Approval"
```

### Example 3: DEX Monitoring

```bash
# Monitor DEX contracts
cargo run -- --allow-list "0xrouter,0xfactory,0xpair" --event-types "Swap,Mint,Burn"
```

## Performance Considerations

### For High-Volume Contracts

```bash
# Use larger chunk sizes and faster sync intervals
cargo run -- --allow-list "0x1234,0x5678" --chunk-size 10000 --sync-interval 1 --batch-mode
```

### For Memory-Constrained Environments

```bash
# Use smaller chunk sizes and event filtering
cargo run -- --allow-list "0x1234,0x5678" --chunk-size 1000 --event-types "Transfer"
```

### For Faster Initial Sync

```bash
# Start from recent block
cargo run -- --allow-list "0x1234,0x5678" --start-block 1900000
```

## Monitoring and Debugging

### Check Sync Status

```bash
curl http://localhost:3000/sync-status
```

**Response:**
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

### Check Indexer Statistics

```bash
curl http://localhost:3000/stats/0x02cf12918a78bb09bb553590cc05d1ee8edd6bbb829c84464c0374fa620c983e
```

**Response:**
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

## Best Practices

1. **Start with Recent Blocks**: Use `--start-block` to skip historical data for faster initial sync
2. **Filter Events**: Use `--event-types` to only index relevant events
3. **Monitor Performance**: Check sync status and statistics regularly
4. **Use Appropriate Chunk Sizes**: Larger chunks for high-volume contracts, smaller for memory-constrained environments
5. **Enable Batch Mode**: Use `--batch-mode` for better performance on high-volume contracts

## Troubleshooting

### Common Issues

1. **Rate Limiting**: Increase delays or use better RPC endpoint
2. **Memory Usage**: Reduce chunk size or add event filtering
3. **Slow Sync**: Use `--start-block` to skip historical data
4. **RPC Errors**: Increase `--max-retries` or check RPC endpoint

### Performance Tuning

```bash
# For high-throughput contracts
cargo run -- --allow-list "0x1234,0x5678" --chunk-size 10000 --sync-interval 1 --batch-mode

# For memory-constrained environments
cargo run -- --allow-list "0x1234,0x5678" --chunk-size 1000 --event-types "Transfer"

# For faster initial sync
cargo run -- --allow-list "0x1234,0x5678" --start-block 1900000 --chunk-size 5000
```

This guide covers all aspects of using the allow list feature. The indexer will automatically handle multiple contracts and provide unified access through the GraphQL API.

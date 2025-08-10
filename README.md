## Mini Starknet Indexer

Rust server exposing REST and GraphQL APIs to fetch and decode Starknet contract events via the Starknet RPC.

## Quick start

1. Install Rust and Cargo
2. Set environment (optional):
   - `RPC_URL` (default: `https://starknet-mainnet.public.blastapi.io`)
   - `CONTRACT_ADDRESS` (optional default for REST event fetch)
3. Run the server:

```bash
cargo run
```

Server runs at `http://localhost:3000`

## REST API

- POST `/` — Fetch events (best-effort decoding)
  - Body:
    ```json
    { "address": "0x...", "chunk_size": 100 }
    ```
- GET `/get-abi/:contract_address` — Raw class/ABI from RPC
- GET `/test` — Health check

## GraphQL API

- HTTP endpoint: `POST /graphql`
- Subscriptions (WebSocket): `GET ws://localhost:3000/ws`
- GraphiQL playground: `GET /graphiql`

### Query examples

- Basic events
```graphql
query GetEvents($contractAddress: String!) {
  events(contractAddress: $contractAddress, first: 5) {
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

- Filtered events (by type and block range)
```graphql
query GetFilteredEvents($addr: String!, $from: String!, $to: String!) {
  events(contractAddress: $addr, fromBlock: $from, toBlock: $to, eventTypes: ["Transfer"], first: 10) {
    edges { node { id eventType blockNumber transactionHash decodedData { json } } }
    pageInfo { hasNextPage endCursor }
  }
}
```

- Contract info + ABI
```graphql
query GetContract($address: String!) {
  contract(address: $address) {
    address
    verified
    events { name type inputs { name type indexed } }
    abi
  }
}
```

### Subscription example

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

Notes:
- Subscriptions are implemented via periodic polling of the RPC (3s). For production, integrate a dedicated indexer/broker.
- `decodedData.json` contains best-effort decoded fields based on the contract ABI.

### Pagination

- Cursor-based via `pageInfo.endCursor` and `pageInfo.hasNextPage`
- Request the next page by passing `after: <endCursor>`

## Internals

- `src/starknet.rs` wraps common Starknet RPC calls
- `src/graphql/` contains schema, types, and resolvers
- Decoding is ABI-driven and best-effort; unknown events return raw keys/data

## Dependencies

- `axum`, `tokio` — server/runtime
- `async-graphql`, `async-graphql-axum` — GraphQL server & Axum integration
- `reqwest`, `serde`, `serde_json` — HTTP and JSON
- `tokio-stream`, `futures` — subscription stream helpers

## Troubleshooting

- If GraphiQL subscriptions fail locally, ensure the WS endpoint is `ws://localhost:3000/ws` in the GraphiQL UI.
- Set a project-specific `RPC_URL` if the public endpoint rate-limits.

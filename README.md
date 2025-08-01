# Mini Starknet Indexer

A Rust-based web server that fetches events from Starknet contracts using the Starknet RPC API.

## Features

- ✅ POST request to Starknet RPC endpoint
- ✅ Fetches events from specified contract addresses
- ✅ Graceful error handling and networking issues
- ✅ Full JSON response logging
- ✅ Structured request/response handling with serde

## Setup

1. Install Rust and Cargo
2. Clone this repository
3. Run the server:

```bash
cargo run
```

The server will start on `http://localhost:3000`

## API Endpoints

### POST `/fetch-events`

Fetches events from a Starknet contract.

**Request Body:**
```json
{
  "address": "0x04718f5a0fc34cc1af16a1cdee98ffb20c31f5cd61d6ab07201858f4287c938d",
  "chunk_size": 100
}
```

**Example Request:**
```bash
curl -X POST http://localhost:3000/fetch-events \
  -H "Content-Type: application/json" \
  -d '{
    "address": "0x04718f5a0fc34cc1af16a1cdee98ffb20c31f5cd61d6ab07201858f4287c938d",
    "chunk_size": 100
  }'
```

**Response:**
The endpoint returns the full JSON response from the Starknet RPC API, including:
- Events array
- Continuation token for pagination
- Page information

## RPC Endpoint

The implementation uses the public Starknet RPC endpoint:
- **URL:** `https://starknet-mainnet.public.blastapi.io`
- **Method:** `starknet_getEvents`

## Error Handling

The implementation includes comprehensive error handling for:
- Network connectivity issues
- HTTP status errors
- JSON parsing errors
- Invalid responses

All errors are logged to the console with descriptive messages.

## Dependencies

- `axum` - Web framework
- `tokio` - Async runtime
- `serde` - Serialization/deserialization
- `serde_json` - JSON handling
- `reqwest` - HTTP client
- `tower-http` - HTTP middleware

## Testing

You can test the API using the provided test script:

```bash
./test_starknet_events.sh
```

Or manually with curl as shown in the examples above.

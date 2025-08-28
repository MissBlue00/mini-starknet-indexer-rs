#!/bin/bash

# Demo script for the allow list feature
# This script shows how to start the indexer with multiple contracts

echo "🚀 Mini Starknet Indexer - Allow List Demo"
echo "=========================================="

# Example contract addresses (replace with real ones)
TOKEN_CONTRACT="0x02cf12918a78bb09bb553590cc05d1ee8edd6bbb829c84464c0374fa620c983e"
GOVERNANCE_CONTRACT="0x04718f5a0fc34cc1af16a1cdee98ffb20c31f5cd61d6ab07201858f4287c938d"

echo ""
echo "📋 Available Demo Commands:"
echo ""

echo "1. Start indexer with single contract:"
echo "   cargo run -- --contract-config \"$TOKEN_CONTRACT:0\""
echo ""

echo "2. Start indexer with multiple contracts:"
echo "   cargo run -- --contract-config \"$TOKEN_CONTRACT:0,$GOVERNANCE_CONTRACT:0\""
echo ""

echo "3. Start with filtering:"
echo "   cargo run -- --contract-config \"$TOKEN_CONTRACT:0,$GOVERNANCE_CONTRACT:0\" --event-types \"Transfer,Approval\""
echo ""

echo "4. Start from recent block (faster sync):"
echo "   cargo run -- --contract-config \"$TOKEN_CONTRACT:1900000,$GOVERNANCE_CONTRACT:1900000\""
echo ""

echo "5. Per-contract start blocks:"
echo "   cargo run -- --contract-config \"$TOKEN_CONTRACT:1900000,$GOVERNANCE_CONTRACT:1901000\""
echo ""

echo "6. Using environment variable:"
echo "   export CONTRACT_CONFIG=\"$TOKEN_CONTRACT:1900000,$GOVERNANCE_CONTRACT:1901000\""
echo "   cargo run"
echo ""

echo "🔍 GraphQL Query Examples:"
echo ""

echo "Single contract query:"
echo "curl -X POST http://localhost:3000/graphql \\"
echo "  -H 'Content-Type: application/json' \\"
echo "  -d '{"
echo "    \"query\": \"query { events(contractAddress: \\\"$TOKEN_CONTRACT\\\", first: 5) { edges { node { id eventType blockNumber } } } }\""
echo "  }'"
echo ""

echo "Multi-contract query (Grouped by Contract):"
echo "curl -X POST http://localhost:3000/graphql \\"
echo "  -H 'Content-Type: application/json' \\"
echo "  -d '{"
echo "    \"query\": \"query { eventsByContract(contractAddresses: [\\\"$TOKEN_CONTRACT\\\", \\\"$GOVERNANCE_CONTRACT\\\"], first: 5) { contracts { contractAddress events { edges { node { id eventType blockNumber } } totalCount } } totalContracts totalEvents } }\""
echo "  }'"
echo ""

echo "Legacy multi-contract query (Flat List):"
echo "curl -X POST http://localhost:3000/graphql \\"
echo "  -H 'Content-Type: application/json' \\"
echo "  -d '{"
echo "    \"query\": \"query { eventsMultiContract(contractAddresses: [\\\"$TOKEN_CONTRACT\\\", \\\"$GOVERNANCE_CONTRACT\\\"], first: 5) { edges { node { id contractAddress eventType blockNumber } } } }\""
echo "  }'"
echo ""

echo "📊 Monitor endpoints:"
echo "   Sync status: http://localhost:3000/sync-status"
echo "   GraphQL playground: http://localhost:3000/graphql"
echo "   GraphiQL interface: http://localhost:3000/graphiql"
echo ""

echo "💡 Tips:"
echo "   - Use --start-block to skip historical data for faster sync"
echo "   - Use --event-types to filter specific events"
echo "   - Use --chunk-size to adjust performance"
echo "   - Check /sync-status to monitor indexing progress"
echo ""

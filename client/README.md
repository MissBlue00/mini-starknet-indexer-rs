# Starknet Indexer - Frontend Client

A modern Next.js frontend for monitoring and analyzing Starknet smart contract deployments and events.

## Features

- **Contract Monitoring**: Enter any Starknet contract address to view its deployment information
- **Real-time Event Tracking**: Monitor contract events with advanced filtering capabilities
- **Interactive Dashboard**: View contract statistics, event types distribution, and analytics
- **Advanced Filtering**: Filter events by type, block range, timestamp, and more
- **GraphQL Integration**: Powered by Apollo Client for efficient data fetching
- **Modern UI**: Built with Tailwind CSS and Lucide React icons

## Getting Started

### Prerequisites

- Node.js 18+ 
- pnpm (recommended) or npm
- Running Starknet Indexer backend server

### Installation

```bash
# Install dependencies
pnpm install

# Start development server
pnpm run dev
```

The application will be available at `http://localhost:3000`.

### Backend Connection

Make sure the Starknet Indexer backend is running on port 3000 with GraphQL endpoint at `/graphql`. The frontend expects:

- GraphQL API: `http://localhost:3000/graphql`
- GraphiQL Playground: `http://localhost:3000/graphiql`

## Pages

### Home Page (`/`)
- Landing page with overview of features
- Quick access to deployments page
- Link to GraphQL playground

### Deployments Page (`/deployments`)
- **Contract Search**: Enter contract address to view deployment details
- **Contract Information**: Shows contract verification status, ABI details, and event schemas
- **Statistics Dashboard**: 
  - Total events count
  - Unique event types
  - Block range (oldest to latest)
  - Event type distribution
- **Event Filtering**:
  - Filter by event types
  - Filter by block range
  - Filter by timestamp range
  - Clear all filters option
- **Events List**: Real-time display of contract events with decoded data

## Usage

1. **Navigate to Deployments**: Click "View Deployments" from the home page or go to `/deployments`

2. **Enter Contract Address**: Input a Starknet contract address (e.g., `0x123...abc`)

3. **View Contract Details**: 
   - Contract verification status
   - Available event types
   - ABI information

4. **Monitor Events**:
   - Real-time event updates
   - Decoded event data
   - Transaction details
   - Block information

5. **Apply Filters**:
   - Click "Filters" to show advanced options
   - Select specific event types
   - Set block ranges
   - Clear filters as needed

## GraphQL Queries

The frontend uses these main GraphQL queries:

### Get Events
```graphql
query GetEvents(
  $contractAddress: String
  $eventTypes: [String!]
  $fromBlock: String
  $toBlock: String
  $first: Int
  $orderBy: EventOrderBy
) {
  events(
    contractAddress: $contractAddress
    eventTypes: $eventTypes
    fromBlock: $fromBlock
    toBlock: $toBlock
    first: $first
    orderBy: $orderBy
  ) {
    edges {
      node {
        id
        contractAddress
        eventType
        blockNumber
        transactionHash
        timestamp
        data
      }
    }
    totalCount
  }
}
```

### Get Contract Details
```graphql
query GetContract($address: String!) {
  contract(address: $address) {
    address
    verified
    events {
      name
      type
    }
  }
}
```

## Architecture

- **Framework**: Next.js 15 with App Router
- **Styling**: Tailwind CSS
- **GraphQL Client**: Apollo Client
- **Icons**: Lucide React
- **TypeScript**: Full type safety with custom GraphQL types

## File Structure

```
src/
├── app/
│   ├── deployments/
│   │   └── page.tsx          # Deployments page component
│   ├── apollo-wrapper.tsx    # Apollo Provider wrapper
│   ├── layout.tsx           # Root layout with Apollo setup
│   └── page.tsx             # Home page
├── lib/
│   ├── apollo-client.ts     # Apollo Client configuration
│   └── graphql/
│       └── queries.ts       # GraphQL query definitions
└── types/
    └── graphql.ts          # TypeScript type definitions
```

## Development

### Available Scripts

- `pnpm run dev` - Start development server
- `pnpm run build` - Build for production
- `pnpm run start` - Start production server
- `pnpm run lint` - Run ESLint

### Adding New Features

1. **New GraphQL Queries**: Add to `src/lib/graphql/queries.ts`
2. **Type Definitions**: Update `src/types/graphql.ts`
3. **UI Components**: Create in `src/components/` (create this directory as needed)
4. **New Pages**: Add to `src/app/` following Next.js App Router conventions

## Troubleshooting

### Common Issues

1. **GraphQL Connection Error**: 
   - Ensure backend server is running on port 3000
   - Check that GraphQL endpoint is accessible at `http://localhost:3000/graphql`

2. **No Events Showing**:
   - Verify the contract address is correct and properly formatted
   - Check if the contract has been indexed by the backend
   - Ensure events exist for the specified filters

3. **Styling Issues**:
   - Make sure Tailwind CSS is properly configured
   - Check if all required dependencies are installed

### Backend Requirements

The frontend expects the backend to provide:
- GraphQL schema with `events` and `contract` queries
- Event data with decoded JSON in the `data` field
- Contract ABI information and verification status
- Real-time subscriptions (future enhancement)

## Future Enhancements

- [ ] Real-time event subscriptions via WebSocket
- [ ] Event export functionality
- [ ] Multiple contract monitoring
- [ ] Custom dashboard creation
- [ ] Event analytics and charts
- [ ] Mobile responsive improvements
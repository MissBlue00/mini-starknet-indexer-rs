# Deployment-Specific GraphQL Endpoints

This document describes the implementation of unique GraphQL endpoints for each deployment, where each endpoint only serves data from contracts associated with that specific deployment.

## Overview

After creating a deployment, you can now access a unique GraphQL endpoint that only returns data for contracts associated with that deployment. This provides data isolation and allows multiple deployments to coexist with their own independent data views.

## Architecture

### Backend Components

1. **DeploymentContext** (`src/graphql/deployment_context.rs`)
   - Manages deployment-specific context for GraphQL operations
   - Filters contract addresses to only include those belonging to the deployment
   - Provides deployment-specific database access

2. **Deployment-Specific Resolvers** 
   - `DeploymentEventQueryRoot` - Filters events to deployment contracts only
   - `DeploymentContractQueryRoot` - Only returns contracts belonging to the deployment

3. **Dynamic Schema Generation** (`src/graphql/deployment_schema.rs`)
   - Creates deployment-specific GraphQL schemas
   - Each schema is scoped to a particular deployment's data

4. **Routing & Caching** (`src/deployment_service_handler.rs`)
   - Handles HTTP routing for deployment endpoints
   - Caches schemas for performance
   - Manages deployment endpoint discovery

### Frontend Components

1. **Deployment Client** (`client/src/lib/deployment-client.ts`)
   - Creates Apollo clients for specific deployments
   - Manages multiple deployment connections
   - Provides utilities for endpoint discovery

2. **React Hooks** (`client/src/hooks/useDeploymentClient.ts`)
   - `useDeploymentEndpoints()` - Fetches available deployments
   - `useDeploymentClient()` - Gets client for specific deployment
   - `useDeploymentClientManager()` - Manages multiple clients

3. **UI Components**
   - `DeploymentSelector` - UI for selecting active deployment
   - Demo page showing deployment-specific data

## API Endpoints

### Deployment Discovery
```
GET /deployments/endpoints
```
Returns list of all deployments with their GraphQL endpoints:
```json
{
  "deployments": [
    {
      "id": "deployment-123",
      "name": "My Deployment",
      "description": "Production deployment",
      "network": "mainnet",
      "status": "active",
      "endpoints": {
        "graphql": "/deployment/deployment-123/graphql",
        "graphiql": "/deployment/deployment-123/graphiql", 
        "websocket": "/deployment/deployment-123/ws"
      }
    }
  ],
  "total_count": 1
}
```

### Deployment-Specific GraphQL
```
POST /deployment/{deployment_id}/graphql
```
GraphQL endpoint that only returns data for contracts associated with the specified deployment.

### GraphiQL Interface
```
GET /deployment/{deployment_id}/graphiql
```
Interactive GraphQL explorer scoped to the deployment's data.

## Usage Examples

### Backend: Creating a Deployment
```bash
# Create a deployment via GraphQL mutation
curl -X POST http://localhost:3000/graphql \
  -H "Content-Type: application/json" \
  -d '{
    "query": "mutation { createDeployment(input: { name: \"My App\", network: \"mainnet\", contractAddress: \"0x123...\" }) { id name } }"
  }'
```

### Frontend: Using Deployment Client
```typescript
import { createDeploymentClient, fetchDeploymentEndpoints } from './lib/deployment-client';

// Get available deployments
const deployments = await fetchDeploymentEndpoints();

// Create client for specific deployment
const client = createDeploymentClient('deployment-123');

// Query deployment-specific data
const { data } = await client.query({
  query: GET_EVENTS,
  variables: { contractAddress: '0x123...', first: 10 }
});
```

### Frontend: React Hook Usage
```typescript
import { useDeploymentClientManager } from './hooks/useDeploymentClient';

function MyComponent() {
  const { activeClient, switchDeployment } = useDeploymentClientManager();
  
  // Switch to a deployment
  switchDeployment('deployment-123');
  
  // Use the active client
  return (
    <ApolloProvider client={activeClient}>
      <EventsList />
    </ApolloProvider>
  );
}
```

## Data Isolation

Each deployment endpoint provides complete data isolation:

1. **Contract Filtering**: Only contracts associated with the deployment are queryable
2. **Event Filtering**: Only events from deployment contracts are returned  
3. **Schema Isolation**: Each deployment gets its own GraphQL schema instance
4. **Independent Caching**: Apollo clients cache data separately per deployment

## Performance Considerations

1. **Schema Caching**: Deployment schemas are cached in memory to avoid rebuilding
2. **Client Caching**: Frontend manages multiple Apollo clients with separate caches
3. **Lazy Loading**: Schemas are created on-demand when first accessed
4. **Connection Pooling**: Database connections are shared across deployments

## Security & Access Control

- Deployments are isolated by ID - no cross-deployment data access
- Each endpoint validates the deployment exists before serving data
- GraphQL introspection is available per deployment for development
- Rate limiting can be applied per deployment endpoint

## Development Workflow

1. **Create Deployment**: Use the main GraphQL endpoint to create a deployment
2. **Get Endpoint**: Fetch deployment endpoints to get the unique GraphQL URL
3. **Configure Client**: Point your GraphQL client to the deployment-specific endpoint
4. **Query Data**: All queries now return only data from that deployment's contracts

## Testing

Visit `/deployment-demo` in the client to see a working example of:
- Deployment selection UI
- Dynamic client switching  
- Deployment-specific data queries
- GraphiQL integration

## Future Enhancements

- **WebSocket Support**: Deployment-specific subscriptions
- **Database Isolation**: Separate databases per deployment
- **Access Control**: Authentication/authorization per deployment
- **Metrics**: Per-deployment usage analytics
- **Backup/Restore**: Deployment-specific data management

# Deployment Database Tracking System

This document provides examples of how to use the deployment tracking system through GraphQL queries and mutations.

## GraphQL Schema

The system provides the following GraphQL operations for deployment management:

### Queries

#### Get a single deployment by ID
```graphql
query GetDeployment($id: String!) {
  deployment(id: $id) {
    id
    name
    description
    databaseUrl
    contractAddress
    network
    status
    createdAt
    updatedAt
    metadata
  }
}
```

#### Get a list of deployments with filtering and pagination
```graphql
query GetDeployments($filter: DeploymentFilter, $first: Int, $after: String) {
  deployments(filter: $filter, first: $first, after: $after) {
    edges {
      node {
        id
        name
        description
        databaseUrl
        contractAddress
        network
        status
        createdAt
        updatedAt
        metadata
      }
      cursor
    }
    pageInfo {
      hasNextPage
      hasPreviousPage
      startCursor
      endCursor
    }
    totalCount
  }
}
```

### Mutations

#### Create a new deployment
```graphql
mutation CreateDeployment($input: CreateDeploymentInput!) {
  createDeployment(input: $input) {
    id
    name
    description
    databaseUrl
    contractAddress
    network
    status
    createdAt
    updatedAt
    metadata
  }
}
```

#### Update an existing deployment
```graphql
mutation UpdateDeployment($input: UpdateDeploymentInput!) {
  updateDeployment(input: $input) {
    id
    name
    description
    databaseUrl
    contractAddress
    network
    status
    createdAt
    updatedAt
    metadata
  }
}
```

#### Delete a deployment
```graphql
mutation DeleteDeployment($id: String!) {
  deleteDeployment(id: $id)
}
```

## Example Usage

### 1. Create a new deployment

**Variables:**
```json
{
  "input": {
    "name": "My DeFi Protocol",
    "description": "Tracking events for our DeFi protocol on Starknet",
    "network": "mainnet",
    "contractAddress": "0x04718f5a0fc34cc1af16a1cdee98ffb20c31f5cd61d6ab07201858f4287c938d",
    "metadata": {
      "version": "1.0.0",
      "owner": "defi-team",
      "tags": ["defi", "protocol", "mainnet"]
    }
  }
}
```

**Query:**
```graphql
mutation CreateDeployment($input: CreateDeploymentInput!) {
  createDeployment(input: $input) {
    id
    name
    description
    databaseUrl
    contractAddress
    network
    status
    createdAt
    metadata
  }
}
```

### 2. List all active deployments

**Variables:**
```json
{
  "filter": {
    "status": "ACTIVE"
  },
  "first": 10
}
```

**Query:**
```graphql
query GetActiveDeployments($filter: DeploymentFilter, $first: Int) {
  deployments(filter: $filter, first: $first) {
    edges {
      node {
        id
        name
        network
        status
        contractAddress
        createdAt
      }
    }
    totalCount
  }
}
```

### 3. Filter deployments by network

**Variables:**
```json
{
  "filter": {
    "network": "testnet"
  },
  "first": 20
}
```

**Query:**
```graphql
query GetTestnetDeployments($filter: DeploymentFilter, $first: Int) {
  deployments(filter: $filter, first: $first) {
    edges {
      node {
        id
        name
        databaseUrl
        contractAddress
        status
      }
    }
    pageInfo {
      hasNextPage
      endCursor
    }
  }
}
```

### 4. Update deployment status

**Variables:**
```json
{
  "input": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "status": "INACTIVE",
    "metadata": {
      "lastUpdated": "2024-01-15T10:30:00Z",
      "reason": "Contract deprecated"
    }
  }
}
```

**Query:**
```graphql
mutation UpdateDeploymentStatus($input: UpdateDeploymentInput!) {
  updateDeployment(input: $input) {
    id
    name
    status
    updatedAt
    metadata
  }
}
```

### 5. Get deployment details

**Variables:**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000"
}
```

**Query:**
```graphql
query GetDeploymentDetails($id: String!) {
  deployment(id: $id) {
    id
    name
    description
    databaseUrl
    contractAddress
    network
    status
    createdAt
    updatedAt
    metadata
  }
}
```

## Database Structure

Each deployment creates its own SQLite database file in the `deployments/` directory with the following naming convention:
```
deployments/{deployment_name}_{network}.db
```

For example:
- `deployments/my_defi_protocol_mainnet.db`
- `deployments/test_contract_testnet.db`

## Deployment Status Values

- `ACTIVE`: Deployment is active and indexing
- `INACTIVE`: Deployment is paused or stopped
- `ERROR`: Deployment encountered an error

## Networks Supported

- `mainnet`: Starknet Mainnet
- `testnet`: Starknet Testnet  
- `devnet`: Local development network
- `local`: Local testing environment

## Semi-Mock Implementation

The current implementation is semi-mock, meaning:

1. ✅ **Database tracking**: Full implementation with SQLite storage
2. ✅ **File system management**: Creates actual database files
3. ✅ **GraphQL API**: Complete CRUD operations
4. ⚠️ **Indexer integration**: Basic setup, not fully integrated with background indexing
5. ⚠️ **Contract deployment**: Simulated, not connected to actual Starknet deployment

The system is ready for production use for tracking existing deployments, and can be extended to integrate with actual deployment processes.

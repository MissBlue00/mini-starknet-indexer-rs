import { ApolloClient, InMemoryCache, createHttpLink, from } from '@apollo/client';
import { setContext } from '@apollo/client/link/context';

export interface DeploymentEndpoint {
  id: string;
  name: string;
  description?: string;
  network: string;
  status: string;
  endpoints: {
    graphql: string;
    graphiql: string;
    websocket: string;
  };
}

/**
 * Creates an Apollo Client for a specific deployment with API key authentication
 * @param deploymentId - The deployment ID
 * @param apiKey - The API key for authentication (required for deployment endpoints)
 * @param baseUrl - The base URL of the indexer service (default: http://localhost:3000)
 * @returns Apollo Client instance configured for the deployment
 */
export function createDeploymentClient(deploymentId: string, apiKey: string, baseUrl: string = 'http://localhost:3000') {
  const httpLink = createHttpLink({
    uri: `${baseUrl}/deployment/${deploymentId}/graphql`,
  });

  // Add deployment context and API key authentication to headers
  const contextLink = setContext((_, { headers }) => {
    return {
      headers: {
        ...headers,
        'x-deployment-id': deploymentId,
        'Authorization': `Bearer ${apiKey}`,
      }
    };
  });

  return new ApolloClient({
    link: from([contextLink, httpLink]),
    cache: new InMemoryCache(),
    defaultOptions: {
      watchQuery: {
        errorPolicy: 'all',
      },
      query: {
        errorPolicy: 'all',
      },
    },
  });
}

/**
 * Fetches the list of available deployment endpoints
 * @param baseUrl - The base URL of the indexer service
 * @returns Promise<DeploymentEndpoint[]>
 */
export async function fetchDeploymentEndpoints(baseUrl: string = 'http://localhost:3000'): Promise<DeploymentEndpoint[]> {
  try {
    const response = await fetch(`${baseUrl}/deployments/endpoints`);
    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`);
    }
    const data = await response.json();
    return data.deployments || [];
  } catch (error) {
    console.error('Failed to fetch deployment endpoints:', error);
    return [];
  }
}

/**
 * Creates a deployment-specific GraphiQL URL
 * @param deploymentId - The deployment ID
 * @param baseUrl - The base URL of the indexer service
 * @returns GraphiQL URL for the deployment
 */
export function getDeploymentGraphiQLUrl(deploymentId: string, baseUrl: string = 'http://localhost:3000'): string {
  return `${baseUrl}/deployment/${deploymentId}/graphiql`;
}

/**
 * Creates a deployment-specific WebSocket URL for subscriptions
 * @param deploymentId - The deployment ID
 * @param baseUrl - The base URL of the indexer service
 * @returns WebSocket URL for the deployment
 */
export function getDeploymentWebSocketUrl(deploymentId: string, baseUrl: string = 'http://localhost:3000'): string {
  const wsUrl = baseUrl.replace(/^https?/, 'ws');
  return `${wsUrl}/deployment/${deploymentId}/ws`;
}

/**
 * Multi-deployment client manager
 */
export class DeploymentClientManager {
  private clients: Map<string, ApolloClient<any>> = new Map();
  private apiKeys: Map<string, string> = new Map(); // deploymentId -> apiKey mapping
  private baseUrl: string;

  constructor(baseUrl: string = 'http://localhost:3000') {
    this.baseUrl = baseUrl;
  }

  /**
   * Sets the API key for a deployment
   */
  setApiKey(deploymentId: string, apiKey: string): void {
    this.apiKeys.set(deploymentId, apiKey);
    // Remove existing client so it gets recreated with the new API key
    this.removeClient(deploymentId);
  }

  /**
   * Gets the API key for a deployment
   */
  getApiKey(deploymentId: string): string | undefined {
    return this.apiKeys.get(deploymentId);
  }

  /**
   * Gets or creates a client for a specific deployment
   */
  getClient(deploymentId: string): ApolloClient<any> {
    if (!this.clients.has(deploymentId)) {
      const apiKey = this.apiKeys.get(deploymentId);
      if (!apiKey) {
        throw new Error(`No API key found for deployment ${deploymentId}. Please set an API key first using setApiKey().`);
      }
      const client = createDeploymentClient(deploymentId, apiKey, this.baseUrl);
      this.clients.set(deploymentId, client);
    }
    return this.clients.get(deploymentId)!;
  }

  /**
   * Removes a client from the cache
   */
  removeClient(deploymentId: string): void {
    const client = this.clients.get(deploymentId);
    if (client) {
      client.stop();
      this.clients.delete(deploymentId);
    }
    // Also remove the API key
    this.apiKeys.delete(deploymentId);
  }

  /**
   * Clears all cached clients and API keys
   */
  clearAll(): void {
    for (const client of this.clients.values()) {
      client.stop();
    }
    this.clients.clear();
    this.apiKeys.clear();
  }

  /**
   * Gets all cached deployment IDs
   */
  getCachedDeploymentIds(): string[] {
    return Array.from(this.clients.keys());
  }
}

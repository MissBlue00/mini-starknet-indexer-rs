import { useState, useEffect, useMemo } from 'react';
import { ApolloClient } from '@apollo/client';
import { 
  DeploymentEndpoint, 
  fetchDeploymentEndpoints, 
  createDeploymentClient,
  DeploymentClientManager 
} from '../lib/deployment-client';

// Global client manager instance
const clientManager = new DeploymentClientManager();

/**
 * Hook to fetch and manage deployment endpoints
 */
export function useDeploymentEndpoints(baseUrl?: string) {
  const [endpoints, setEndpoints] = useState<DeploymentEndpoint[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    async function loadEndpoints() {
      try {
        setLoading(true);
        setError(null);
        const deployments = await fetchDeploymentEndpoints(baseUrl);
        setEndpoints(deployments);
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to load deployments');
      } finally {
        setLoading(false);
      }
    }

    loadEndpoints();
  }, [baseUrl]);

  return { endpoints, loading, error, refetch: () => loadEndpoints() };
}

/**
 * Hook to get a deployment-specific Apollo client
 */
export function useDeploymentClient(deploymentId: string | null, apiKey?: string, baseUrl?: string) {
  const client = useMemo(() => {
    if (!deploymentId) return null;
    
    // Set API key if provided
    if (apiKey) {
      clientManager.setApiKey(deploymentId, apiKey);
    }
    
    try {
      return clientManager.getClient(deploymentId);
    } catch (error) {
      console.error(`Failed to get client for deployment ${deploymentId}:`, error);
      return null;
    }
  }, [deploymentId, apiKey]);

  return client;
}

/**
 * Hook to manage multiple deployment clients with API key support
 */
export function useDeploymentClientManager() {
  const [activeDeploymentId, setActiveDeploymentId] = useState<string | null>(null);

  const switchDeployment = (deploymentId: string) => {
    setActiveDeploymentId(deploymentId);
  };

  const clearDeployment = () => {
    setActiveDeploymentId(null);
  };

  const setApiKey = (deploymentId: string, apiKey: string) => {
    clientManager.setApiKey(deploymentId, apiKey);
  };

  const getApiKey = (deploymentId: string) => {
    return clientManager.getApiKey(deploymentId);
  };

  const getClient = (deploymentId: string) => {
    try {
      return clientManager.getClient(deploymentId);
    } catch (error) {
      console.error(`Failed to get client for deployment ${deploymentId}:`, error);
      return null;
    }
  };

  const removeClient = (deploymentId: string) => {
    clientManager.removeClient(deploymentId);
    if (activeDeploymentId === deploymentId) {
      setActiveDeploymentId(null);
    }
  };

  const activeClient = useMemo(() => {
    if (!activeDeploymentId) return null;
    try {
      return clientManager.getClient(activeDeploymentId);
    } catch (error) {
      console.error(`Failed to get active client for deployment ${activeDeploymentId}:`, error);
      return null;
    }
  }, [activeDeploymentId]);

  return {
    activeDeploymentId,
    activeClient,
    switchDeployment,
    clearDeployment,
    setApiKey,
    getApiKey,
    getClient,
    removeClient,
    cachedDeploymentIds: clientManager.getCachedDeploymentIds(),
  };
}

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
export function useDeploymentClient(deploymentId: string | null, baseUrl?: string) {
  const client = useMemo(() => {
    if (!deploymentId) return null;
    return clientManager.getClient(deploymentId);
  }, [deploymentId]);

  return client;
}

/**
 * Hook to manage multiple deployment clients
 */
export function useDeploymentClientManager() {
  const [activeDeploymentId, setActiveDeploymentId] = useState<string | null>(null);

  const switchDeployment = (deploymentId: string) => {
    setActiveDeploymentId(deploymentId);
  };

  const clearDeployment = () => {
    setActiveDeploymentId(null);
  };

  const getClient = (deploymentId: string) => {
    return clientManager.getClient(deploymentId);
  };

  const removeClient = (deploymentId: string) => {
    clientManager.removeClient(deploymentId);
    if (activeDeploymentId === deploymentId) {
      setActiveDeploymentId(null);
    }
  };

  const activeClient = useMemo(() => {
    return activeDeploymentId ? clientManager.getClient(activeDeploymentId) : null;
  }, [activeDeploymentId]);

  return {
    activeDeploymentId,
    activeClient,
    switchDeployment,
    clearDeployment,
    getClient,
    removeClient,
    cachedDeploymentIds: clientManager.getCachedDeploymentIds(),
  };
}

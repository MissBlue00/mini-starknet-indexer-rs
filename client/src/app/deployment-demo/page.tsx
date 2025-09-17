'use client';

import React, { useState } from 'react';
import { ApolloProvider, useQuery } from '@apollo/client';
import { useDeploymentClientManager } from '../../hooks/useDeploymentClient';
import DeploymentSelector from '../../components/DeploymentSelector';
import { GET_EVENTS, GET_CONTRACT } from '../../lib/graphql/queries';

// Component to display events from the selected deployment
function DeploymentEvents() {
  const { activeClient, activeDeploymentId } = useDeploymentClientManager();
  const [contractAddress, setContractAddress] = useState(
    '0x049d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7'
  );

  if (!activeClient || !activeDeploymentId) {
    return (
      <div className="p-4 bg-gray-50 border border-gray-200 rounded-lg">
        <p className="text-gray-600">Select a deployment to view events</p>
      </div>
    );
  }

  return (
    <ApolloProvider client={activeClient}>
      <div className="space-y-4">
        <div className="flex items-center gap-4">
          <label htmlFor="contract-address" className="text-sm font-medium text-gray-700">
            Contract Address:
          </label>
          <input
            id="contract-address"
            type="text"
            value={contractAddress}
            onChange={(e) => setContractAddress(e.target.value)}
            className="flex-1 px-3 py-2 border border-gray-300 rounded-md text-sm"
            placeholder="Enter contract address"
          />
        </div>

        <EventsList contractAddress={contractAddress} />
      </div>
    </ApolloProvider>
  );
}

// Component to fetch and display events
function EventsList({ contractAddress }: { contractAddress: string }) {
  const { data, loading, error } = useQuery(GET_EVENTS, {
    variables: {
      contractAddress,
      first: 20,
      orderBy: 'BLOCK_NUMBER_DESC',
    },
    skip: !contractAddress,
  });

  if (!contractAddress) {
    return (
      <div className="p-4 bg-gray-50 border border-gray-200 rounded-lg">
        <p className="text-gray-600">Enter a contract address to view events</p>
      </div>
    );
  }

  if (loading) {
    return (
      <div className="space-y-3">
        {[...Array(3)].map((_, i) => (
          <div key={i} className="p-4 bg-white border border-gray-200 rounded-lg animate-pulse">
            <div className="h-4 bg-gray-200 rounded w-1/4 mb-2"></div>
            <div className="h-3 bg-gray-200 rounded w-1/2 mb-2"></div>
            <div className="h-3 bg-gray-200 rounded w-3/4"></div>
          </div>
        ))}
      </div>
    );
  }

  if (error) {
    return (
      <div className="p-4 bg-red-50 border border-red-200 rounded-lg">
        <p className="text-red-800 text-sm">Error: {error.message}</p>
      </div>
    );
  }

  const events = data?.events?.edges || [];

  if (events.length === 0) {
    return (
      <div className="p-4 bg-yellow-50 border border-yellow-200 rounded-lg">
        <p className="text-yellow-800 text-sm">
          No events found for this contract in the selected deployment.
        </p>
      </div>
    );
  }

  return (
    <div className="space-y-3">
      <h3 className="text-lg font-medium text-gray-900">
        Events ({data.events.totalCount})
      </h3>
      
      {events.map(({ node: event }: any) => (
        <div key={event.id} className="p-4 bg-white border border-gray-200 rounded-lg">
          <div className="flex items-center justify-between mb-2">
            <div className="flex items-center gap-2">
              <span className="font-medium text-gray-900">{event.eventType}</span>
              <span className="text-sm text-gray-500">#{event.blockNumber}</span>
            </div>
            <span className="text-xs text-gray-500">{event.timestamp}</span>
          </div>
          
          <div className="text-sm text-gray-600 mb-2">
            <span className="font-medium">Contract:</span> {event.contractAddress}
          </div>
          
          <div className="text-sm text-gray-600 mb-2">
            <span className="font-medium">Tx Hash:</span> 
            <span className="font-mono ml-1">{event.transactionHash}</span>
          </div>

          {event.data && (
            <div className="mt-3 p-3 bg-gray-50 rounded">
              <div className="text-sm font-medium text-gray-700 mb-2">Decoded Data:</div>
              <pre className="text-xs text-gray-600 overflow-x-auto">
                {JSON.stringify(event.data, null, 2)}
              </pre>
            </div>
          )}
        </div>
      ))}
    </div>
  );
}

// Component to display contract info
function ContractInfo({ contractAddress }: { contractAddress: string }) {
  const { data, loading, error } = useQuery(GET_CONTRACT, {
    variables: { address: contractAddress },
    skip: !contractAddress,
  });

  if (!contractAddress || loading) return null;
  if (error) return <p className="text-red-600 text-sm">Error loading contract: {error.message}</p>;
  if (!data?.contract) return <p className="text-gray-600 text-sm">Contract not found</p>;

  return (
    <div className="p-4 bg-white border border-gray-200 rounded-lg">
      <h4 className="font-medium text-gray-900 mb-2">Contract Information</h4>
      <div className="text-sm text-gray-600 space-y-1">
        <div><span className="font-medium">Address:</span> {data.contract.address}</div>
        {data.contract.name && (
          <div><span className="font-medium">Name:</span> {data.contract.name}</div>
        )}
        <div><span className="font-medium">Verified:</span> {data.contract.verified ? 'Yes' : 'No'}</div>
        <div><span className="font-medium">Events:</span> {data.contract.events.length}</div>
      </div>
    </div>
  );
}

export default function DeploymentDemo() {
  return (
    <div className="min-h-screen bg-gray-100">
      <div className="max-w-7xl mx-auto py-8 px-4 sm:px-6 lg:px-8">
        <div className="mb-8">
          <h1 className="text-3xl font-bold text-gray-900">Deployment-Specific GraphQL Demo</h1>
          <p className="text-gray-600 mt-2">
            Each deployment provides its own GraphQL endpoint with data specific to that deployment only.
          </p>
        </div>

        <div className="grid grid-cols-1 lg:grid-cols-3 gap-8">
          {/* Deployment Selector */}
          <div className="lg:col-span-1">
            <div className="bg-white p-6 rounded-lg shadow">
              <DeploymentSelector />
            </div>
          </div>

          {/* Events Display */}
          <div className="lg:col-span-2">
            <div className="bg-white p-6 rounded-lg shadow">
              <h2 className="text-xl font-semibold text-gray-900 mb-4">
                Deployment Events
              </h2>
              <DeploymentEvents />
            </div>
          </div>
        </div>

        {/* Instructions */}
        <div className="mt-8 bg-white p-6 rounded-lg shadow">
          <h2 className="text-xl font-semibold text-gray-900 mb-4">How It Works</h2>
          <div className="prose text-gray-600">
            <ol className="list-decimal list-inside space-y-2">
              <li>Each deployment gets its own unique GraphQL endpoint: <code>/deployment/[id]/graphql</code></li>
              <li>Select a deployment from the list to switch the GraphQL client</li>
              <li>All queries will then only return data from contracts associated with that deployment</li>
              <li>Each deployment can have different contracts and different data</li>
              <li>GraphiQL interfaces are also deployment-specific for easy testing</li>
            </ol>
            
            <div className="mt-4 p-4 bg-blue-50 border border-blue-200 rounded">
              <h3 className="font-medium text-blue-900">API Endpoints</h3>
              <ul className="mt-2 space-y-1 text-sm text-blue-800">
                <li><code>GET /deployments/endpoints</code> - List all deployments</li>
                <li><code>POST /deployment/[id]/graphql</code> - GraphQL endpoint</li>
                <li><code>GET /deployment/[id]/graphiql</code> - GraphiQL interface</li>
              </ul>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

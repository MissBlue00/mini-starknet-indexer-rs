import React, { useState } from 'react';
import { useDeploymentEndpoints, useDeploymentClientManager } from '../hooks/useDeploymentClient';
import { ApiKeyManager } from './ApiKeyManager';

export default function DeploymentSelector() {
  const { endpoints, loading, error } = useDeploymentEndpoints();
  const { activeDeploymentId, switchDeployment, clearDeployment, getApiKey } = useDeploymentClientManager();
  const [selectedDeploymentId, setSelectedDeploymentId] = useState<string | null>(null);

  if (loading) {
    return (
      <div className="p-4 bg-gray-50 rounded-lg">
        <div className="animate-pulse">
          <div className="h-4 bg-gray-200 rounded w-1/4 mb-2"></div>
          <div className="h-8 bg-gray-200 rounded"></div>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="p-4 bg-red-50 border border-red-200 rounded-lg">
        <p className="text-red-800 text-sm">Error loading deployments: {error}</p>
      </div>
    );
  }

  if (endpoints.length === 0) {
    return (
      <div className="p-4 bg-yellow-50 border border-yellow-200 rounded-lg">
        <p className="text-yellow-800 text-sm">No deployments found. Create a deployment first.</p>
      </div>
    );
  }

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <h3 className="text-lg font-medium text-gray-900">Select Deployment</h3>
        {activeDeploymentId && (
          <button
            onClick={clearDeployment}
            className="text-sm text-gray-500 hover:text-gray-700"
          >
            Clear Selection
          </button>
        )}
      </div>

      <div className="grid gap-3">
        {endpoints.map((endpoint) => (
          <div
            key={endpoint.id}
            className={`p-4 border rounded-lg cursor-pointer transition-colors ${
              activeDeploymentId === endpoint.id
                ? 'border-blue-500 bg-blue-50'
                : 'border-gray-200 hover:border-gray-300 hover:bg-gray-50'
            }`}
            onClick={() => switchDeployment(endpoint.id)}
          >
            <div className="flex items-center justify-between">
              <div className="flex-1">
                <div className="flex items-center gap-2">
                  <h4 className="font-medium text-gray-900">{endpoint.name}</h4>
                  <span
                    className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${
                      endpoint.status === 'active'
                        ? 'bg-green-100 text-green-800'
                        : endpoint.status === 'error'
                        ? 'bg-red-100 text-red-800'
                        : 'bg-gray-100 text-gray-800'
                    }`}
                  >
                    {endpoint.status}
                  </span>
                </div>
                {endpoint.description && (
                  <p className="text-sm text-gray-600 mt-1">{endpoint.description}</p>
                )}
                <div className="flex items-center gap-4 mt-2 text-xs text-gray-500">
                  <span>Network: {endpoint.network}</span>
                  <span>ID: {endpoint.id}</span>
                </div>
              </div>
              
              {activeDeploymentId === endpoint.id && (
                <div className="flex items-center gap-2">
                  <div className="w-2 h-2 bg-blue-500 rounded-full"></div>
                  <span className="text-sm text-blue-600 font-medium">Active</span>
                </div>
              )}
            </div>

            {/* Quick links */}
            <div className="flex gap-2 mt-3">
              <a
                href={endpoint.endpoints.graphiql}
                target="_blank"
                rel="noopener noreferrer"
                className="text-xs text-blue-600 hover:text-blue-800 underline"
                onClick={(e) => e.stopPropagation()}
              >
                GraphiQL
              </a>
              <span className="text-xs text-gray-400">•</span>
              <button
                onClick={(e) => {
                  e.stopPropagation();
                  navigator.clipboard.writeText(endpoint.endpoints.graphql);
                }}
                className="text-xs text-gray-600 hover:text-gray-800"
              >
                Copy GraphQL URL
              </button>
            </div>
          </div>
        ))}
      </div>

      {activeDeploymentId && (
        <div className="p-3 bg-blue-50 border border-blue-200 rounded-lg">
          <p className="text-sm text-blue-800">
            <strong>Selected:</strong> {endpoints.find(e => e.id === activeDeploymentId)?.name}
          </p>
          <p className="text-xs text-blue-600 mt-1">
            All GraphQL queries will now use this deployment&apos;s data only.
          </p>
          
          {/* API Key Status */}
          {getApiKey(activeDeploymentId) ? (
            <div className="mt-2 p-2 bg-green-50 border border-green-200 rounded">
              <p className="text-xs text-green-800">
                ✅ API key configured - deployment queries will work
              </p>
            </div>
          ) : (
            <div className="mt-2 p-2 bg-yellow-50 border border-yellow-200 rounded">
              <p className="text-xs text-yellow-800">
                ⚠️ No API key configured - deployment queries will fail
              </p>
            </div>
          )}
        </div>
      )}

      {/* API Key Management Section */}
      {selectedDeploymentId && (
        <div className="mt-6">
          <button
            onClick={() => setSelectedDeploymentId(null)}
            className="text-sm text-gray-500 hover:text-gray-700 mb-2"
          >
            ← Back to deployments
          </button>
          <ApiKeyManager 
            deploymentId={selectedDeploymentId}
            deploymentName={endpoints.find(e => e.id === selectedDeploymentId)?.name}
          />
        </div>
      )}

      {/* Show API Key Management Button for Selected Deployment */}
      {activeDeploymentId && !selectedDeploymentId && (
        <div className="mt-4">
          <button
            onClick={() => setSelectedDeploymentId(activeDeploymentId)}
            className="px-4 py-2 bg-gray-100 text-gray-700 rounded hover:bg-gray-200 text-sm"
          >
            Manage API Key
          </button>
        </div>
      )}
    </div>
  );
}

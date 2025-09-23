import React, { useState } from 'react';
import { useQuery, useMutation } from '@apollo/client';
import { gql } from '@apollo/client';

const GET_DEPLOYMENT_CONTRACTS = gql`
  query GetDeploymentContracts($deploymentId: String!) {
    deploymentContracts(deploymentId: $deploymentId) {
      id
      contractAddress
      name
      description
      startBlock
      status
      createdAt
      updatedAt
      metadata
    }
  }
`;

const ADD_DEPLOYMENT_CONTRACT = gql`
  mutation AddDeploymentContract($input: AddDeploymentContractInput!) {
    addDeploymentContract(input: $input) {
      id
      contractAddress
      name
      description
      startBlock
      status
      createdAt
    }
  }
`;

const UPDATE_DEPLOYMENT_CONTRACT = gql`
  mutation UpdateDeploymentContract($input: UpdateDeploymentContractInput!) {
    updateDeploymentContract(input: $input) {
      id
      contractAddress
      name
      description
      startBlock
      status
      updatedAt
    }
  }
`;

const REMOVE_DEPLOYMENT_CONTRACT = gql`
  mutation RemoveDeploymentContract($id: String!) {
    removeDeploymentContract(id: $id)
  }
`;

interface DeploymentContract {
  id: string;
  contractAddress: string;
  name?: string;
  description?: string;
  startBlock?: string;
  status: string;
  createdAt: string;
  updatedAt: string;
  metadata?: Record<string, unknown>;
}

interface DeploymentContractManagerProps {
  deploymentId: string;
  deploymentName: string;
}

export default function DeploymentContractManager({ deploymentId, deploymentName }: DeploymentContractManagerProps) {
  const [showAddForm, setShowAddForm] = useState(false);
  const [editingContract, setEditingContract] = useState<string | null>(null);
  const [formData, setFormData] = useState({
    contractAddress: '',
    name: '',
    description: '',
    startBlock: '',
    metadata: ''
  });

  const { data, loading, error, refetch } = useQuery(GET_DEPLOYMENT_CONTRACTS, {
    variables: { deploymentId },
    skip: !deploymentId
  });

  const [addContract] = useMutation(ADD_DEPLOYMENT_CONTRACT, {
    onCompleted: () => {
      setShowAddForm(false);
      setFormData({ contractAddress: '', name: '', description: '', startBlock: '', metadata: '' });
      refetch();
    }
  });

  const [updateContract] = useMutation(UPDATE_DEPLOYMENT_CONTRACT, {
    onCompleted: () => {
      setEditingContract(null);
      setFormData({ contractAddress: '', name: '', description: '', startBlock: '', metadata: '' });
      refetch();
    }
  });

  const [removeContract] = useMutation(REMOVE_DEPLOYMENT_CONTRACT, {
    onCompleted: () => {
      refetch();
    }
  });

  const contracts: DeploymentContract[] = data?.deploymentContracts || [];

  const handleAddContract = async (e: React.FormEvent) => {
    e.preventDefault();
    try {
      await addContract({
        variables: {
          input: {
            deploymentId,
            contractAddress: formData.contractAddress,
            name: formData.name || null,
            description: formData.description || null,
            startBlock: formData.startBlock || null,
            metadata: formData.metadata ? JSON.parse(formData.metadata) : null
          }
        }
      });
    } catch (error) {
      console.error('Error adding contract:', error);
    }
  };

  const handleUpdateContract = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!editingContract) return;
    
    try {
      await updateContract({
        variables: {
          input: {
            id: editingContract,
            name: formData.name || null,
            description: formData.description || null,
            startBlock: formData.startBlock || null,
            metadata: formData.metadata ? JSON.parse(formData.metadata) : null
          }
        }
      });
    } catch (error) {
      console.error('Error updating contract:', error);
    }
  };

  const handleRemoveContract = async (contractId: string) => {
    if (!confirm('Are you sure you want to remove this contract?')) return;
    
    try {
      await removeContract({
        variables: { id: contractId }
      });
    } catch (error) {
      console.error('Error removing contract:', error);
    }
  };

  const startEditing = (contract: DeploymentContract) => {
    setEditingContract(contract.id);
    setFormData({
      contractAddress: contract.contractAddress,
      name: contract.name || '',
      description: contract.description || '',
      startBlock: contract.startBlock || '',
      metadata: contract.metadata ? JSON.stringify(contract.metadata, null, 2) : ''
    });
  };

  const cancelEditing = () => {
    setEditingContract(null);
    setShowAddForm(false);
    setFormData({ contractAddress: '', name: '', description: '', startBlock: '', metadata: '' });
  };

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
        <p className="text-red-800 text-sm">Error loading contracts: {error.message}</p>
      </div>
    );
  }

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <h3 className="text-lg font-medium text-gray-900">
          Contracts for {deploymentName}
        </h3>
        <button
          onClick={() => setShowAddForm(true)}
          className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
        >
          Add Contract
        </button>
      </div>

      {/* Add/Edit Form */}
      {(showAddForm || editingContract) && (
        <div className="p-4 bg-gray-50 border border-gray-200 rounded-lg">
          <h4 className="text-md font-medium text-gray-900 mb-3">
            {editingContract ? 'Edit Contract' : 'Add New Contract'}
          </h4>
          <form onSubmit={editingContract ? handleUpdateContract : handleAddContract} className="space-y-3">
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Contract Address *
              </label>
              <input
                type="text"
                value={formData.contractAddress}
                onChange={(e) => setFormData({ ...formData, contractAddress: e.target.value })}
                className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                placeholder="0x..."
                required
                disabled={!!editingContract}
              />
            </div>
            
            <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">
                  Name
                </label>
                <input
                  type="text"
                  value={formData.name}
                  onChange={(e) => setFormData({ ...formData, name: e.target.value })}
                  className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                  placeholder="Contract name"
                />
              </div>
              
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">
                  Start Block
                </label>
                <input
                  type="number"
                  value={formData.startBlock}
                  onChange={(e) => setFormData({ ...formData, startBlock: e.target.value })}
                  className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                  placeholder="Block number"
                />
              </div>
            </div>
            
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Description
              </label>
              <textarea
                value={formData.description}
                onChange={(e) => setFormData({ ...formData, description: e.target.value })}
                className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                placeholder="Contract description"
                rows={2}
              />
            </div>
            
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Metadata (JSON)
              </label>
              <textarea
                value={formData.metadata}
                onChange={(e) => setFormData({ ...formData, metadata: e.target.value })}
                className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 font-mono text-sm"
                placeholder='{"key": "value"}'
                rows={3}
              />
            </div>
            
            <div className="flex gap-2">
              <button
                type="submit"
                className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
              >
                {editingContract ? 'Update Contract' : 'Add Contract'}
              </button>
              <button
                type="button"
                onClick={cancelEditing}
                className="px-4 py-2 bg-gray-500 text-white rounded-lg hover:bg-gray-600 transition-colors"
              >
                Cancel
              </button>
            </div>
          </form>
        </div>
      )}

      {/* Contracts List */}
      <div className="space-y-3">
        {contracts.length === 0 ? (
          <div className="p-4 bg-yellow-50 border border-yellow-200 rounded-lg">
            <p className="text-yellow-800 text-sm">No contracts found. Add a contract to start indexing.</p>
          </div>
        ) : (
          contracts.map((contract) => (
            <div key={contract.id} className="p-4 border border-gray-200 rounded-lg">
              <div className="flex items-center justify-between">
                <div className="flex-1">
                  <div className="flex items-center gap-2">
                    <h4 className="font-medium text-gray-900">
                      {contract.name || 'Unnamed Contract'}
                    </h4>
                    <span
                      className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${
                        contract.status === 'active'
                          ? 'bg-green-100 text-green-800'
                          : contract.status === 'error'
                          ? 'bg-red-100 text-red-800'
                          : 'bg-gray-100 text-gray-800'
                      }`}
                    >
                      {contract.status}
                    </span>
                  </div>
                  
                  <p className="text-sm text-gray-600 mt-1 font-mono">
                    {contract.contractAddress}
                  </p>
                  
                  {contract.description && (
                    <p className="text-sm text-gray-600 mt-1">{contract.description}</p>
                  )}
                  
                  <div className="flex items-center gap-4 mt-2 text-xs text-gray-500">
                    {contract.startBlock && (
                      <span>Start Block: {contract.startBlock}</span>
                    )}
                    <span>Added: {new Date(contract.createdAt).toLocaleDateString()}</span>
                  </div>
                </div>
                
                <div className="flex items-center gap-2">
                  <button
                    onClick={() => startEditing(contract)}
                    className="px-3 py-1 text-sm bg-gray-100 text-gray-700 rounded hover:bg-gray-200 transition-colors"
                  >
                    Edit
                  </button>
                  <button
                    onClick={() => handleRemoveContract(contract.id)}
                    className="px-3 py-1 text-sm bg-red-100 text-red-700 rounded hover:bg-red-200 transition-colors"
                  >
                    Remove
                  </button>
                </div>
              </div>
            </div>
          ))
        )}
      </div>
    </div>
  );
}

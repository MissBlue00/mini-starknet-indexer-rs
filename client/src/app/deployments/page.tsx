'use client';

import { useState } from 'react';
import { useQuery } from '@apollo/client';
import { GET_DEPLOYMENTS } from '@/lib/graphql/queries';
import { Search, Filter, Hash, Copy, ExternalLink, BarChart3, Database, Rocket, CheckCircle, XCircle, Code } from 'lucide-react';
import { clsx } from 'clsx';
import Link from 'next/link';

interface Deployment {
  address: string;
  name?: string;
  verified: boolean;
  events: Array<{
    name: string;
    type: string;
    inputs: Array<{
      name: string;
      type: string;
      indexed: boolean;
    }>;
    anonymous: boolean;
  }>;
}

interface DeploymentsQueryResult {
  deployments: Deployment[];
}

export default function DeploymentsPage() {
  const [searchQuery, setSearchQuery] = useState('');
  const [showFilters, setShowFilters] = useState(false);
  const [verifiedFilter, setVerifiedFilter] = useState<'all' | 'verified' | 'unverified'>('all');

  // Query all deployments
  const { data, loading, error, refetch } = useQuery<DeploymentsQueryResult>(GET_DEPLOYMENTS, {
    variables: {
      first: 50,
      after: null,
    },
  });

  const deployments = data?.deployments || [];

  // Filter deployments based on search and filters
  const filteredDeployments = deployments.filter((deployment) => {
    // Search filter
    if (searchQuery) {
      const query = searchQuery.toLowerCase();
      const matchesAddress = deployment.address.toLowerCase().includes(query);
      const matchesName = deployment.name?.toLowerCase().includes(query);
      if (!matchesAddress && !matchesName) {
        return false;
      }
    }

    // Verified filter
    if (verifiedFilter === 'verified' && !deployment.verified) {
      return false;
    }
    if (verifiedFilter === 'unverified' && deployment.verified) {
      return false;
    }

    return true;
  });

  const copyToClipboard = (text: string) => {
    navigator.clipboard.writeText(text);
  };

  const stats = {
    totalDeployments: deployments.length,
    verifiedDeployments: deployments.filter(d => d.verified).length,
    unverifiedDeployments: deployments.filter(d => !d.verified).length,
    totalEvents: deployments.reduce((sum, d) => sum + d.events.length, 0),
  };

  return (
    <div className="min-h-screen bg-slate-950 text-white">
      {/* Header */}
      <div className="border-b border-slate-800 bg-slate-900/50 backdrop-blur-sm sticky top-0 z-50">
        <div className="max-w-7xl mx-auto px-6 py-4">
          <div className="flex items-center justify-between">
            <div>
              <h1 className="text-2xl font-bold text-white flex items-center gap-3">
                <Rocket className="h-8 w-8 text-blue-500" />
                Contract Deployments
              </h1>
              <p className="text-slate-400 text-sm mt-1">Track and monitor all your Starknet contract deployments</p>
            </div>
            <div className="flex items-center gap-3">
              <button 
                onClick={() => refetch()}
                className="px-4 py-2 bg-blue-600 hover:bg-blue-700 rounded-lg text-sm font-medium transition-colors flex items-center gap-2"
              >
                <Database className="h-4 w-4" />
                Refresh
              </button>
              <div className="flex items-center gap-2 text-sm text-slate-400">
                <div className="w-2 h-2 bg-green-500 rounded-full"></div>
                {filteredDeployments.length} Active
              </div>
            </div>
          </div>
        </div>
      </div>

      <div className="max-w-7xl mx-auto px-6 py-8">
        {/* Stats Cards */}
        <div className="grid grid-cols-1 md:grid-cols-4 gap-4 mb-8">
          <div className="bg-slate-900/50 border border-slate-800 rounded-xl p-6">
            <div className="flex items-center justify-between mb-3">
              <div className="text-slate-400 text-sm font-medium">TOTAL DEPLOYMENTS</div>
              <Rocket className="h-5 w-5 text-slate-500" />
            </div>
            <div className="text-2xl font-bold text-white">{stats.totalDeployments}</div>
            <div className="text-slate-400 text-xs mt-1">Contract deployments</div>
          </div>

          <div className="bg-slate-900/50 border border-slate-800 rounded-xl p-6">
            <div className="flex items-center justify-between mb-3">
              <div className="text-slate-400 text-sm font-medium">VERIFIED</div>
              <CheckCircle className="h-5 w-5 text-green-500" />
            </div>
            <div className="text-2xl font-bold text-green-400">{stats.verifiedDeployments}</div>
            <div className="text-slate-400 text-xs mt-1">Verified contracts</div>
          </div>

          <div className="bg-slate-900/50 border border-slate-800 rounded-xl p-6">
            <div className="flex items-center justify-between mb-3">
              <div className="text-slate-400 text-sm font-medium">UNVERIFIED</div>
              <XCircle className="h-5 w-5 text-red-500" />
            </div>
            <div className="text-2xl font-bold text-red-400">{stats.unverifiedDeployments}</div>
            <div className="text-slate-400 text-xs mt-1">Unverified contracts</div>
          </div>

          <div className="bg-slate-900/50 border border-slate-800 rounded-xl p-6">
            <div className="flex items-center justify-between mb-3">
              <div className="text-slate-400 text-sm font-medium">EVENT TYPES</div>
              <BarChart3 className="h-5 w-5 text-slate-500" />
            </div>
            <div className="text-2xl font-bold text-white">{stats.totalEvents}</div>
            <div className="text-slate-400 text-xs mt-1">Total event types</div>
          </div>
        </div>

        {/* Search and Filters */}
        <div className="bg-slate-900/50 border border-slate-800 rounded-xl p-6 mb-8">
          <div className="flex gap-3 mb-4">
            <div className="flex-1 relative">
              <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 text-slate-500 h-5 w-5" />
              <input
                type="text"
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                placeholder="Search by contract address or name..."
                className="w-full pl-10 pr-4 py-3 bg-slate-800 border border-slate-700 rounded-lg text-white placeholder-slate-500 focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all"
              />
            </div>
            <button
              onClick={() => setShowFilters(!showFilters)}
              className={clsx(
                "px-4 py-3 border rounded-lg font-medium transition-colors flex items-center gap-2",
                showFilters 
                  ? "bg-slate-700 border-slate-600 text-white" 
                  : "border-slate-700 text-slate-400 hover:text-white hover:border-slate-600"
              )}
            >
              <Filter className="h-5 w-5" />
              Filters
            </button>
          </div>

          {/* Filters */}
          {showFilters && (
            <div className="border-t border-slate-800 pt-4">
              <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
                <div>
                  <label className="block text-sm font-medium text-slate-300 mb-2">Verification Status</label>
                  <select
                    value={verifiedFilter}
                    onChange={(e) => setVerifiedFilter(e.target.value as 'all' | 'verified' | 'unverified')}
                    className="w-full bg-slate-800 border border-slate-700 rounded-lg px-3 py-2 text-white focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                  >
                    <option value="all">All Contracts</option>
                    <option value="verified">Verified Only</option>
                    <option value="unverified">Unverified Only</option>
                  </select>
                </div>
              </div>
            </div>
          )}
        </div>

        {/* Deployments List */}
        {loading ? (
          <div className="bg-slate-900/50 border border-slate-800 rounded-xl p-8">
            <div className="animate-pulse space-y-4">
              {[...Array(5)].map((_, i) => (
                <div key={i} className="bg-slate-800/50 rounded-lg p-6">
                  <div className="h-6 bg-slate-700 rounded mb-3"></div>
                  <div className="h-4 bg-slate-700 rounded mb-2"></div>
                  <div className="h-4 bg-slate-700 rounded"></div>
                </div>
              ))}
            </div>
          </div>
        ) : error ? (
          <div className="bg-slate-900/50 border border-slate-800 rounded-xl p-8 text-center text-red-400">
            Error loading deployments: {error.message}
          </div>
        ) : filteredDeployments.length === 0 ? (
          <div className="bg-slate-900/50 border border-slate-800 rounded-xl p-12 text-center">
            <Rocket className="h-12 w-12 text-slate-600 mx-auto mb-4" />
            <h3 className="text-slate-400 font-medium mb-2">No deployments found</h3>
            <p className="text-slate-500 text-sm">
              {searchQuery ? 'No deployments match your search criteria.' : 'No contract deployments found in the system.'}
            </p>
          </div>
        ) : (
          <div className="bg-slate-900/50 border border-slate-800 rounded-xl overflow-hidden">
            <div className="p-6 border-b border-slate-800">
              <h3 className="text-lg font-semibold text-white flex items-center gap-2">
                <Database className="h-5 w-5 text-blue-500" />
                Contract Deployments
              </h3>
              <p className="text-slate-400 text-sm mt-1">
                {filteredDeployments.length} of {deployments.length} deployments
              </p>
            </div>

            <div className="divide-y divide-slate-800">
              {filteredDeployments.map((deployment) => (
                <Link 
                  key={deployment.address} 
                  href={`/deployment/${encodeURIComponent(deployment.address)}`}
                  className="block p-6 hover:bg-slate-800/30 transition-colors group"
                >
                  <div className="flex items-start justify-between mb-4">
                    <div className="flex items-center gap-4">
                      <div className="w-12 h-12 bg-gradient-to-br from-blue-500 to-purple-600 rounded-lg flex items-center justify-center">
                        <Code className="h-6 w-6 text-white" />
                      </div>
                      <div>
                        <div className="flex items-center gap-3 mb-2">
                          <h4 className="text-white font-semibold group-hover:text-blue-400 transition-colors">
                            {deployment.name || 'Unnamed Contract'}
                          </h4>
                          <div className={clsx(
                            "flex items-center gap-1 px-2 py-1 rounded-full text-xs font-medium",
                            deployment.verified
                              ? "bg-green-900/50 text-green-400 border border-green-800"
                              : "bg-red-900/50 text-red-400 border border-red-800"
                          )}>
                            {deployment.verified ? (
                              <CheckCircle className="h-3 w-3" />
                            ) : (
                              <XCircle className="h-3 w-3" />
                            )}
                            {deployment.verified ? 'Verified' : 'Unverified'}
                          </div>
                        </div>
                        <div className="flex items-center gap-2 text-sm text-slate-400">
                          <Hash className="h-4 w-4" />
                          <code className="bg-slate-800 px-2 py-1 rounded font-mono text-xs">
                            {deployment.address.slice(0, 10)}...{deployment.address.slice(-8)}
                          </code>
                          <button 
                            onClick={(e) => {
                              e.preventDefault();
                              copyToClipboard(deployment.address);
                            }}
                            className="text-slate-500 hover:text-slate-300 transition-colors"
                          >
                            <Copy className="h-4 w-4" />
                          </button>
                        </div>
                      </div>
                    </div>
                    <div className="flex items-center gap-3">
                      <div className="text-right">
                        <div className="text-white font-semibold">{deployment.events.length}</div>
                        <div className="text-slate-400 text-xs">Event Types</div>
                      </div>
                      <ExternalLink className="h-5 w-5 text-slate-500 group-hover:text-blue-400 transition-colors" />
                    </div>
                  </div>
                  
                  {deployment.events.length > 0 && (
                    <div className="ml-16">
                      <div className="flex flex-wrap gap-2">
                        {deployment.events.slice(0, 4).map((event, index) => (
                          <span
                            key={index}
                            className="px-2 py-1 bg-slate-800 text-slate-300 text-xs rounded-full border border-slate-700"
                          >
                            {event.name}
                          </span>
                        ))}
                        {deployment.events.length > 4 && (
                          <span className="px-2 py-1 bg-slate-800 text-slate-400 text-xs rounded-full border border-slate-700">
                            +{deployment.events.length - 4} more
                          </span>
                        )}
                      </div>
                    </div>
                  )}
                </Link>
              ))}
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

'use client';

import { useState } from 'react';
import { useQuery } from '@apollo/client';
import { GET_EVENTS, GET_CONTRACT } from '@/lib/graphql/queries';
import { Search, Filter, Hash, Activity, Clock, TrendingUp, Copy, ExternalLink, Settings, Plus, BarChart3, Database, Zap } from 'lucide-react';
import { clsx } from 'clsx';

interface Event {
  id: string;
  contractAddress: string;
  eventType: string;
  blockNumber: string;
  transactionHash: string;
  logIndex: number;
  timestamp: string;
  data: Record<string, unknown>;
  rawData: string[];
  rawKeys: string[];
}

interface DeploymentFilters {
  contractAddress: string;
  eventTypes: string[];
  fromBlock: string;
  toBlock: string;
  fromTimestamp: string;
  toTimestamp: string;
}

export default function DeploymentsPage() {
  const [filters, setFilters] = useState<DeploymentFilters>({
    contractAddress: '0x049d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7',
    eventTypes: [],
    fromBlock: '',
    toBlock: '',
    fromTimestamp: '',
    toTimestamp: '',
  });

  const [showFilters, setShowFilters] = useState(false);
  const [searchInput, setSearchInput] = useState('0x049d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7');

  // Query events based on current filters
  const { data: eventsData, loading: eventsLoading, error: eventsError } = useQuery(GET_EVENTS, {
    variables: {
      contractAddress: filters.contractAddress || undefined,
      eventTypes: filters.eventTypes.length > 0 ? filters.eventTypes : undefined,
      fromBlock: filters.fromBlock || undefined,
      toBlock: filters.toBlock || undefined,
      fromTimestamp: filters.fromTimestamp || undefined,
      toTimestamp: filters.toTimestamp || undefined,
      first: 50,
      orderBy: 'BLOCK_NUMBER_DESC',
    },
    skip: !filters.contractAddress,
  });

  // Query contract details when we have an address
  const { data: contractData, loading: contractLoading } = useQuery(GET_CONTRACT, {
    variables: { address: filters.contractAddress },
    skip: !filters.contractAddress,
  });

  const events = eventsData?.events?.edges?.map((edge: { node: Event }) => edge.node) || [];
  const contract = contractData?.contract;

  const handleSearch = () => {
    if (searchInput.trim()) {
      setFilters(prev => ({
        ...prev,
        contractAddress: searchInput.trim(),
      }));
    }
  };

  const handleFilterChange = (key: keyof DeploymentFilters, value: string | string[]) => {
    setFilters(prev => ({
      ...prev,
      [key]: value,
    }));
  };

  const clearFilters = () => {
    setFilters({
      contractAddress: filters.contractAddress,
      eventTypes: [],
      fromBlock: '',
      toBlock: '',
      fromTimestamp: '',
      toTimestamp: '',
    });
  };

  const copyToClipboard = (text: string) => {
    navigator.clipboard.writeText(text);
  };

  // Get unique event types from current events for filter options
  const availableEventTypes = [...new Set(events.map((event: Event) => event.eventType))] as string[];

  // Calculate statistics
  const stats = {
    totalEvents: events.length,
    uniqueEventTypes: availableEventTypes.length,
    latestBlock: events.length > 0 ? Math.max(...events.map((e: Event) => parseInt(e.blockNumber))) : 0,
    oldestBlock: events.length > 0 ? Math.min(...events.map((e: Event) => parseInt(e.blockNumber))) : 0,
  };

  return (
    <div className="min-h-screen bg-slate-950 text-white">
      {/* Header */}
      <div className="border-b border-slate-800 bg-slate-900/50 backdrop-blur-sm sticky top-0 z-50">
        <div className="max-w-7xl mx-auto px-6 py-4">
          <div className="flex items-center justify-between">
            <div>
              <h1 className="text-2xl font-bold text-white">My Dashboard</h1>
              <p className="text-slate-400 text-sm mt-1">Monitor and analyze your Starknet deployments</p>
            </div>
            <div className="flex items-center gap-3">
              <button className="px-4 py-2 bg-blue-600 hover:bg-blue-700 rounded-lg text-sm font-medium transition-colors flex items-center gap-2">
                <Plus className="h-4 w-4" />
                Create Subgraph
              </button>
              <div className="flex items-center gap-2 text-sm text-slate-400">
                <div className="w-2 h-2 bg-green-500 rounded-full"></div>
                Free Plan
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
              <div className="text-slate-400 text-sm font-medium">QUERIES MADE</div>
              <Database className="h-5 w-5 text-slate-500" />
            </div>
            <div className="text-2xl font-bold text-white">{stats.totalEvents.toLocaleString()}</div>
            <div className="text-slate-400 text-xs mt-1">Events indexed</div>
          </div>

          <div className="bg-slate-900/50 border border-slate-800 rounded-xl p-6">
            <div className="flex items-center justify-between mb-3">
              <div className="text-slate-400 text-sm font-medium">RENEWS IN</div>
              <Clock className="h-5 w-5 text-slate-500" />
            </div>
            <div className="text-2xl font-bold text-white">16 <span className="text-sm font-normal text-slate-400">DAYS</span></div>
            <div className="text-slate-400 text-xs mt-1">Next billing cycle</div>
          </div>

          <div className="bg-slate-900/50 border border-slate-800 rounded-xl p-6">
            <div className="flex items-center justify-between mb-3">
              <div className="text-slate-400 text-sm font-medium">API KEYS</div>
              <Zap className="h-5 w-5 text-slate-500" />
            </div>
            <div className="text-2xl font-bold text-white">{stats.uniqueEventTypes}</div>
            <div className="text-slate-400 text-xs mt-1">Event types</div>
          </div>

          <div className="bg-slate-900/50 border border-slate-800 rounded-xl p-6">
            <div className="flex items-center justify-between mb-3">
              <div className="text-slate-400 text-sm font-medium">BLOCK RANGE</div>
              <BarChart3 className="h-5 w-5 text-slate-500" />
            </div>
            <div className="text-2xl font-bold text-white">{stats.latestBlock - stats.oldestBlock}</div>
            <div className="text-slate-400 text-xs mt-1">Blocks covered</div>
          </div>
        </div>

        {/* Search Section */}
        <div className="bg-slate-900/50 border border-slate-800 rounded-xl p-6 mb-8">
          <div className="mb-4">
            <h2 className="text-lg font-semibold text-white mb-2">Contract Explorer</h2>
            <p className="text-slate-400 text-sm">Search and analyze Starknet smart contracts</p>
          </div>
          
          <div className="flex gap-3 mb-4">
            <div className="flex-1 relative">
              <Hash className="absolute left-3 top-1/2 transform -translate-y-1/2 text-slate-500 h-5 w-5" />
              <input
                type="text"
                value={searchInput}
                onChange={(e) => setSearchInput(e.target.value)}
                placeholder="Enter contract address (0x...)"
                className="w-full pl-10 pr-4 py-3 bg-slate-800 border border-slate-700 rounded-lg text-white placeholder-slate-500 focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all"
                onKeyPress={(e) => e.key === 'Enter' && handleSearch()}
              />
            </div>
            <button
              onClick={handleSearch}
              disabled={!searchInput.trim()}
              className="px-6 py-3 bg-blue-600 hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed rounded-lg text-white font-medium transition-colors flex items-center gap-2"
            >
              <Search className="h-5 w-5" />
              Search
            </button>
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

          {/* Advanced Filters */}
          {showFilters && (
            <div className="border-t border-slate-800 pt-4">
              <div className="grid grid-cols-1 md:grid-cols-3 lg:grid-cols-5 gap-4">
                <div>
                  <label className="block text-sm font-medium text-slate-300 mb-2">Event Types</label>
                  <select
                    multiple
                    value={filters.eventTypes}
                    onChange={(e) => handleFilterChange('eventTypes', Array.from(e.target.selectedOptions, option => option.value))}
                    className="w-full bg-slate-800 border border-slate-700 rounded-lg px-3 py-2 text-white focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                  >
                    {availableEventTypes.map((type) => (
                      <option key={type} value={type} className="py-1">{type}</option>
                    ))}
                  </select>
                </div>
                <div>
                  <label className="block text-sm font-medium text-slate-300 mb-2">From Block</label>
                  <input
                    type="number"
                    value={filters.fromBlock}
                    onChange={(e) => handleFilterChange('fromBlock', e.target.value)}
                    placeholder="Start block"
                    className="w-full bg-slate-800 border border-slate-700 rounded-lg px-3 py-2 text-white placeholder-slate-500 focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                  />
                </div>
                <div>
                  <label className="block text-sm font-medium text-slate-300 mb-2">To Block</label>
                  <input
                    type="number"
                    value={filters.toBlock}
                    onChange={(e) => handleFilterChange('toBlock', e.target.value)}
                    placeholder="End block"
                    className="w-full bg-slate-800 border border-slate-700 rounded-lg px-3 py-2 text-white placeholder-slate-500 focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                  />
                </div>
                <div className="md:col-span-2 lg:col-span-1">
                  <label className="block text-sm font-medium text-slate-300 mb-2">&nbsp;</label>
                  <button
                    onClick={clearFilters}
                    className="w-full px-4 py-2 text-slate-400 border border-slate-700 rounded-lg hover:text-white hover:border-slate-600 transition-colors"
                  >
                    Clear Filters
                  </button>
                </div>
              </div>
            </div>
          )}
        </div>

        {/* Contract Details & Events */}
        {filters.contractAddress && (
          <div className="grid grid-cols-1 xl:grid-cols-4 gap-8">
            {/* Sidebar - Contract Info */}
            <div className="xl:col-span-1 space-y-6">
              {/* Contract Card */}
              <div className="bg-slate-900/50 border border-slate-800 rounded-xl p-6">
                <div className="flex items-center gap-3 mb-4">
                  <div className="w-12 h-12 bg-gradient-to-br from-blue-500 to-purple-600 rounded-lg flex items-center justify-center">
                    <Hash className="h-6 w-6 text-white" />
                  </div>
                  <div>
                    <h3 className="font-semibold text-white">{contract?.name || 'Unknown Contract'}</h3>
                    <div className="flex items-center gap-2 mt-1">
                      <div className={clsx(
                        "w-2 h-2 rounded-full",
                        contract?.verified ? "bg-green-500" : "bg-red-500"
                      )}></div>
                      <span className="text-xs text-slate-400">
                        {contract?.verified ? 'Verified' : 'Unverified'}
                      </span>
                    </div>
                  </div>
                </div>
                
                {contractLoading ? (
                  <div className="animate-pulse">
                    <div className="h-4 bg-slate-800 rounded mb-2"></div>
                    <div className="h-4 bg-slate-800 rounded mb-2"></div>
                    <div className="h-4 bg-slate-800 rounded"></div>
                  </div>
                ) : (
                  <div className="space-y-3">
                    <div>
                      <div className="text-xs text-slate-500 uppercase tracking-wider mb-1">Address</div>
                      <div className="flex items-center gap-2">
                        <code className="text-sm text-slate-300 bg-slate-800 px-2 py-1 rounded font-mono">
                          {filters.contractAddress.slice(0, 10)}...{filters.contractAddress.slice(-8)}
                        </code>
                        <button 
                          onClick={() => copyToClipboard(filters.contractAddress)}
                          className="text-slate-500 hover:text-slate-300"
                        >
                          <Copy className="h-4 w-4" />
                        </button>
                        <button className="text-slate-500 hover:text-slate-300">
                          <ExternalLink className="h-4 w-4" />
                        </button>
                      </div>
                    </div>
                    
                    <div>
                      <div className="text-xs text-slate-500 uppercase tracking-wider mb-1">Event Types</div>
                      <div className="text-sm text-slate-300">{contract?.events?.length || 0} defined</div>
                    </div>
                  </div>
                )}
              </div>

              {/* Stats Card */}
              <div className="bg-slate-900/50 border border-slate-800 rounded-xl p-6">
                <h3 className="font-semibold text-white mb-4 flex items-center gap-2">
                  <TrendingUp className="h-5 w-5 text-green-500" />
                  Analytics
                </h3>
                <div className="space-y-4">
                  <div className="flex justify-between items-center">
                    <span className="text-slate-400 text-sm">Total Events</span>
                    <span className="text-white font-semibold">{stats.totalEvents}</span>
                  </div>
                  <div className="flex justify-between items-center">
                    <span className="text-slate-400 text-sm">Event Types</span>
                    <span className="text-white font-semibold">{stats.uniqueEventTypes}</span>
                  </div>
                  <div className="flex justify-between items-center">
                    <span className="text-slate-400 text-sm">Latest Block</span>
                    <span className="text-white font-semibold">{stats.latestBlock.toLocaleString()}</span>
                  </div>
                  <div className="flex justify-between items-center">
                    <span className="text-slate-400 text-sm">Block Range</span>
                    <span className="text-white font-semibold">{(stats.latestBlock - stats.oldestBlock).toLocaleString()}</span>
                  </div>
                </div>
              </div>

              {/* Event Distribution */}
              <div className="bg-slate-900/50 border border-slate-800 rounded-xl p-6">
                <h3 className="font-semibold text-white mb-4 flex items-center gap-2">
                  <Activity className="h-5 w-5 text-purple-500" />
                  Event Distribution
                </h3>
                <div className="space-y-3">
                  {availableEventTypes.slice(0, 4).map((type) => {
                    const count = events.filter((e: Event) => e.eventType === type).length;
                    const percentage = stats.totalEvents > 0 ? (count / stats.totalEvents) * 100 : 0;
                    return (
                      <div key={type} className="space-y-2">
                        <div className="flex justify-between items-center">
                          <span className="text-slate-300 text-sm">{type}</span>
                          <span className="text-slate-400 text-sm">{count}</span>
                        </div>
                        <div className="w-full bg-slate-800 rounded-full h-2">
                          <div 
                            className="bg-gradient-to-r from-blue-500 to-purple-600 h-2 rounded-full transition-all duration-500" 
                            style={{ width: `${percentage}%` }}
                          ></div>
                        </div>
                      </div>
                    );
                  })}
                </div>
              </div>
            </div>

            {/* Main Content - Events List */}
            <div className="xl:col-span-3">
              <div className="bg-slate-900/50 border border-slate-800 rounded-xl overflow-hidden">
                <div className="p-6 border-b border-slate-800">
                  <div className="flex items-center justify-between">
                    <div>
                      <h3 className="text-lg font-semibold text-white flex items-center gap-2">
                        <Clock className="h-5 w-5 text-blue-500" />
                        Recent Events
                      </h3>
                      {eventsData?.events?.totalCount && (
                        <p className="text-slate-400 text-sm mt-1">{eventsData.events.totalCount} total events</p>
                      )}
                    </div>
                    <button className="text-slate-400 hover:text-white">
                      <Settings className="h-5 w-5" />
                    </button>
                  </div>
                </div>

                {eventsLoading ? (
                  <div className="p-6">
                    <div className="animate-pulse space-y-4">
                      {[...Array(5)].map((_, i) => (
                        <div key={i} className="bg-slate-800/50 rounded-lg p-4">
                          <div className="h-4 bg-slate-700 rounded mb-2"></div>
                          <div className="h-3 bg-slate-700 rounded mb-2"></div>
                          <div className="h-3 bg-slate-700 rounded"></div>
                        </div>
                      ))}
                    </div>
                  </div>
                ) : eventsError ? (
                  <div className="p-6 text-center text-red-400">
                    Error loading events: {eventsError.message}
                  </div>
                ) : events.length === 0 ? (
                  <div className="p-12 text-center">
                    <Database className="h-12 w-12 text-slate-600 mx-auto mb-4" />
                    <h3 className="text-slate-400 font-medium mb-2">No events found</h3>
                    <p className="text-slate-500 text-sm">No events match your current filters.</p>
                  </div>
                ) : (
                  <div className="divide-y divide-slate-800">
                    {events.map((event: Event, index: number) => (
                      <div key={event.id} className="p-6 hover:bg-slate-800/30 transition-colors">
                        <div className="flex items-start justify-between mb-3">
                          <div className="flex items-center gap-3">
                            <div className="w-8 h-8 bg-gradient-to-br from-blue-500 to-purple-600 rounded-lg flex items-center justify-center text-white text-xs font-bold">
                              {index + 1}
                            </div>
                            <div>
                              <div className="flex items-center gap-2 mb-1">
                                <span className="px-2 py-1 bg-blue-900/50 text-blue-300 text-xs font-medium rounded-full border border-blue-800">
                                  {event.eventType}
                                </span>
                                <span className="text-slate-500 text-sm">
                                  Block #{parseInt(event.blockNumber).toLocaleString()}
                                </span>
                              </div>
                              <div className="text-slate-400 text-sm">
                                {new Date(event.timestamp).toLocaleString()}
                              </div>
                            </div>
                          </div>
                          <button className="text-slate-500 hover:text-slate-300">
                            <ExternalLink className="h-4 w-4" />
                          </button>
                        </div>
                        
                        <div className="ml-11">
                          <div className="flex items-center gap-2 text-sm text-slate-400 mb-3">
                            <span>Tx:</span>
                            <code className="bg-slate-800 px-2 py-1 rounded text-slate-300 font-mono text-xs">
                              {event.transactionHash.slice(0, 10)}...{event.transactionHash.slice(-8)}
                            </code>
                            <button 
                              onClick={() => copyToClipboard(event.transactionHash)}
                              className="text-slate-500 hover:text-slate-300"
                            >
                              <Copy className="h-3 w-3" />
                            </button>
                          </div>
                          
                          {event.data && (
                            <div className="bg-slate-800/50 rounded-lg p-4 border border-slate-700">
                              <div className="text-sm font-medium text-slate-300 mb-2">Decoded Data</div>
                              <pre className="text-xs text-slate-400 overflow-x-auto">
                                {JSON.stringify(event.data, null, 2)}
                              </pre>
                            </div>
                          )}
                        </div>
                      </div>
                    ))}
                  </div>
                )}
              </div>
            </div>
          </div>
        )}

        {/* Empty State */}
        {!filters.contractAddress && (
          <div className="text-center py-20">
            <div className="w-20 h-20 bg-slate-800 rounded-full flex items-center justify-center mx-auto mb-6">
              <Hash className="h-10 w-10 text-slate-600" />
            </div>
            <h3 className="text-xl font-semibold text-white mb-2">No Contract Selected</h3>
            <p className="text-slate-400 mb-8 max-w-md mx-auto">
              Enter a contract address above to start exploring deployment information and events.
            </p>
          </div>
        )}
      </div>
    </div>
  );
}
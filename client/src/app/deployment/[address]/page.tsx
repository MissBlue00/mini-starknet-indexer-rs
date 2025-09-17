'use client';

import { useState } from 'react';
import { useQuery } from '@apollo/client';
import { useParams } from 'next/navigation';
import { GET_EVENTS, GET_CONTRACT } from '@/lib/graphql/queries';
import { Filter, Hash, Activity, Clock, TrendingUp, Copy, ExternalLink, Settings, ArrowLeft, BarChart3, Database, CheckCircle, XCircle, Code, Activity as ActivityIcon } from 'lucide-react';
import { clsx } from 'clsx';
import Link from 'next/link';

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
  eventTypes: string[];
  fromBlock: string;
  toBlock: string;
  fromTimestamp: string;
  toTimestamp: string;
}

export default function DeploymentDetailPage() {
  const params = useParams();
  const contractAddress = decodeURIComponent(params.address as string);
  
  const [filters, setFilters] = useState<DeploymentFilters>({
    eventTypes: [],
    fromBlock: '',
    toBlock: '',
    fromTimestamp: '',
    toTimestamp: '',
  });

  const [showFilters, setShowFilters] = useState(false);

  // Query events for this specific contract
  const { data: eventsData, loading: eventsLoading, error: eventsError } = useQuery(GET_EVENTS, {
    variables: {
      contractAddress,
      eventTypes: filters.eventTypes.length > 0 ? filters.eventTypes : undefined,
      fromBlock: filters.fromBlock || undefined,
      toBlock: filters.toBlock || undefined,
      fromTimestamp: filters.fromTimestamp || undefined,
      toTimestamp: filters.toTimestamp || undefined,
      first: 50,
      orderBy: 'BLOCK_NUMBER_DESC',
    },
  });

  // Query contract details
  const { data: contractData } = useQuery(GET_CONTRACT, {
    variables: { address: contractAddress },
  });

  const events = eventsData?.events?.edges?.map((edge: { node: Event }) => edge.node) || [];
  const contract = contractData?.contract;

  const handleFilterChange = (key: keyof DeploymentFilters, value: string | string[]) => {
    setFilters(prev => ({
      ...prev,
      [key]: value,
    }));
  };

  const clearFilters = () => {
    setFilters({
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
            <div className="flex items-center gap-4">
              <Link 
                href="/deployments"
                className="p-2 text-slate-400 hover:text-white hover:bg-slate-800 rounded-lg transition-colors"
              >
                <ArrowLeft className="h-5 w-5" />
              </Link>
              <div>
                <h1 className="text-2xl font-bold text-white flex items-center gap-3">
                  <Code className="h-8 w-8 text-blue-500" />
                  {contract?.name || 'Contract Deployment'}
                </h1>
                <p className="text-slate-400 text-sm mt-1">
                  Monitor events and activity for this contract deployment
                </p>
              </div>
            </div>
            <div className="flex items-center gap-3">
              <div className={clsx(
                "flex items-center gap-2 px-3 py-2 rounded-lg text-sm font-medium",
                contract?.verified
                  ? "bg-green-900/50 text-green-400 border border-green-800"
                  : "bg-red-900/50 text-red-400 border border-red-800"
              )}>
                {contract?.verified ? (
                  <CheckCircle className="h-4 w-4" />
                ) : (
                  <XCircle className="h-4 w-4" />
                )}
                {contract?.verified ? 'Verified' : 'Unverified'}
              </div>
            </div>
          </div>
        </div>
      </div>

      <div className="max-w-7xl mx-auto px-6 py-8">
        {/* Contract Info Card */}
        <div className="bg-slate-900/50 border border-slate-800 rounded-xl p-6 mb-8">
          <div className="flex items-start justify-between">
            <div className="flex items-center gap-4">
              <div className="w-16 h-16 bg-gradient-to-br from-blue-500 to-purple-600 rounded-xl flex items-center justify-center">
                <Code className="h-8 w-8 text-white" />
              </div>
              <div>
                <h2 className="text-xl font-bold text-white mb-2">
                  {contract?.name || 'Unnamed Contract'}
                </h2>
                <div className="flex items-center gap-2 mb-3">
                  <Hash className="h-4 w-4 text-slate-500" />
                  <code className="text-sm text-slate-300 bg-slate-800 px-3 py-1 rounded font-mono">
                    {contractAddress}
                  </code>
                  <button 
                    onClick={() => copyToClipboard(contractAddress)}
                    className="text-slate-500 hover:text-slate-300 transition-colors"
                  >
                    <Copy className="h-4 w-4" />
                  </button>
                  <button className="text-slate-500 hover:text-slate-300 transition-colors">
                    <ExternalLink className="h-4 w-4" />
                  </button>
                </div>
                {contract?.events && (
                  <div className="text-sm text-slate-400">
                    {contract.events.length} event types defined in ABI
                  </div>
                )}
              </div>
            </div>
          </div>
        </div>

        {/* Stats Cards */}
        <div className="grid grid-cols-1 md:grid-cols-4 gap-4 mb-8">
          <div className="bg-slate-900/50 border border-slate-800 rounded-xl p-6">
            <div className="flex items-center justify-between mb-3">
              <div className="text-slate-400 text-sm font-medium">TOTAL EVENTS</div>
              <ActivityIcon className="h-5 w-5 text-slate-500" />
            </div>
            <div className="text-2xl font-bold text-white">{stats.totalEvents.toLocaleString()}</div>
            <div className="text-slate-400 text-xs mt-1">Events indexed</div>
          </div>

          <div className="bg-slate-900/50 border border-slate-800 rounded-xl p-6">
            <div className="flex items-center justify-between mb-3">
              <div className="text-slate-400 text-sm font-medium">EVENT TYPES</div>
              <BarChart3 className="h-5 w-5 text-slate-500" />
            </div>
            <div className="text-2xl font-bold text-white">{stats.uniqueEventTypes}</div>
            <div className="text-slate-400 text-xs mt-1">Unique event types</div>
          </div>

          <div className="bg-slate-900/50 border border-slate-800 rounded-xl p-6">
            <div className="flex items-center justify-between mb-3">
              <div className="text-slate-400 text-sm font-medium">LATEST BLOCK</div>
              <Clock className="h-5 w-5 text-slate-500" />
            </div>
            <div className="text-2xl font-bold text-white">{stats.latestBlock.toLocaleString()}</div>
            <div className="text-slate-400 text-xs mt-1">Most recent activity</div>
          </div>

          <div className="bg-slate-900/50 border border-slate-800 rounded-xl p-6">
            <div className="flex items-center justify-between mb-3">
              <div className="text-slate-400 text-sm font-medium">BLOCK RANGE</div>
              <TrendingUp className="h-5 w-5 text-slate-500" />
            </div>
            <div className="text-2xl font-bold text-white">{(stats.latestBlock - stats.oldestBlock).toLocaleString()}</div>
            <div className="text-slate-400 text-xs mt-1">Blocks covered</div>
          </div>
        </div>

        {/* Filters Section */}
        <div className="bg-slate-900/50 border border-slate-800 rounded-xl p-6 mb-8">
          <div className="flex items-center justify-between mb-4">
            <h3 className="text-lg font-semibold text-white">Event Filters</h3>
            <button
              onClick={() => setShowFilters(!showFilters)}
              className={clsx(
                "px-4 py-2 border rounded-lg font-medium transition-colors flex items-center gap-2",
                showFilters 
                  ? "bg-slate-700 border-slate-600 text-white" 
                  : "border-slate-700 text-slate-400 hover:text-white hover:border-slate-600"
              )}
            >
              <Filter className="h-4 w-4" />
              {showFilters ? 'Hide Filters' : 'Show Filters'}
            </button>
          </div>

          {showFilters && (
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
              <div>
                <label className="block text-sm font-medium text-slate-300 mb-2">From Date</label>
                <input
                  type="datetime-local"
                  value={filters.fromTimestamp}
                  onChange={(e) => handleFilterChange('fromTimestamp', e.target.value ? new Date(e.target.value).toISOString() : '')}
                  className="w-full bg-slate-800 border border-slate-700 rounded-lg px-3 py-2 text-white focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-slate-300 mb-2">Actions</label>
                <button
                  onClick={clearFilters}
                  className="w-full px-4 py-2 text-slate-400 border border-slate-700 rounded-lg hover:text-white hover:border-slate-600 transition-colors"
                >
                  Clear Filters
                </button>
              </div>
            </div>
          )}
        </div>

        {/* Events List */}
        <div className="grid grid-cols-1 xl:grid-cols-4 gap-8">
          {/* Sidebar - Event Distribution */}
          <div className="xl:col-span-1 space-y-6">
            <div className="bg-slate-900/50 border border-slate-800 rounded-xl p-6">
              <h3 className="font-semibold text-white mb-4 flex items-center gap-2">
                <Activity className="h-5 w-5 text-purple-500" />
                Event Distribution
              </h3>
              <div className="space-y-3">
                {availableEventTypes.slice(0, 6).map((type) => {
                  const count = events.filter((e: Event) => e.eventType === type).length;
                  const percentage = stats.totalEvents > 0 ? (count / stats.totalEvents) * 100 : 0;
                  return (
                    <div key={type} className="space-y-2">
                      <div className="flex justify-between items-center">
                        <span className="text-slate-300 text-sm truncate">{type}</span>
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
                <div className="divide-y divide-slate-800 max-h-[800px] overflow-y-auto">
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
      </div>
    </div>
  );
}

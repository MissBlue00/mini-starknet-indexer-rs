import Link from "next/link";
import { Database, Zap, BarChart3, Shield, ArrowRight, Play } from 'lucide-react';

export default function Home() {
  return (
    <div className="min-h-screen bg-slate-950 text-white">
      {/* Hero Section */}
      <div className="relative overflow-hidden">
        {/* Background gradient */}
        <div className="absolute inset-0 bg-gradient-to-br from-blue-900/20 via-slate-950 to-purple-900/20"></div>
        <div className="absolute inset-0 opacity-20">
          <div className="absolute inset-0" style={{
            backgroundImage: `radial-gradient(circle at 1px 1px, rgba(255,255,255,0.15) 1px, transparent 0)`,
            backgroundSize: '20px 20px'
          }}></div>
        </div>
        
        <div className="relative max-w-7xl mx-auto px-6 py-20 sm:py-32">
          <div className="text-center">
            <div className="mb-8">
              <div className="inline-flex items-center gap-2 px-4 py-2 bg-blue-900/30 border border-blue-800/50 rounded-full text-blue-300 text-sm font-medium">
                <div className="w-2 h-2 bg-blue-400 rounded-full animate-pulse"></div>
                Live on Starknet
              </div>
            </div>
            
            <h1 className="text-5xl sm:text-7xl font-bold bg-gradient-to-r from-white via-blue-100 to-blue-300 bg-clip-text text-transparent mb-6">
              Starknet Indexer
            </h1>
            
            <p className="text-xl sm:text-2xl text-slate-300 mb-12 max-w-3xl mx-auto leading-relaxed">
              The most powerful way to monitor, analyze, and query your Starknet smart contract deployments with real-time insights
            </p>
            
            <div className="flex gap-4 items-center justify-center flex-col sm:flex-row">
              <Link
                href="/deployments"
                className="group relative inline-flex items-center justify-center px-8 py-4 bg-blue-600 hover:bg-blue-700 rounded-xl text-white font-semibold transition-all duration-200 transform hover:scale-105 hover:shadow-xl hover:shadow-blue-500/25"
              >
                <Database className="h-5 w-5 mr-2" />
                Explore Dashboard
                <ArrowRight className="h-4 w-4 ml-2 group-hover:translate-x-1 transition-transform" />
              </Link>
              
              <a
                href="http://localhost:3000/graphiql"
                target="_blank"
                rel="noopener noreferrer"
                className="group inline-flex items-center justify-center px-8 py-4 border border-slate-700 hover:border-slate-600 rounded-xl text-slate-300 hover:text-white font-semibold transition-all duration-200 hover:bg-slate-800/50"
              >
                <Play className="h-4 w-4 mr-2" />
                Try GraphQL API
              </a>
            </div>
          </div>
        </div>
      </div>

      {/* Features Section */}
      <div className="py-20 bg-slate-900/30">
        <div className="max-w-7xl mx-auto px-6">
          <div className="text-center mb-16">
            <h2 className="text-3xl sm:text-4xl font-bold text-white mb-4">
              Everything you need to monitor Starknet
            </h2>
            <p className="text-slate-400 text-lg max-w-2xl mx-auto">
              Professional-grade tools for indexing, querying, and analyzing smart contract events with enterprise reliability
            </p>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-8">
            <div className="bg-slate-900/50 border border-slate-800 rounded-xl p-6 hover:bg-slate-800/50 transition-all duration-200 group">
              <div className="w-12 h-12 bg-blue-600/10 border border-blue-500/20 rounded-lg flex items-center justify-center mb-4 group-hover:scale-110 transition-transform">
                <Database className="h-6 w-6 text-blue-400" />
              </div>
              <h3 className="text-lg font-semibold text-white mb-2">Real-time Indexing</h3>
              <p className="text-slate-400 text-sm">
                Monitor contract events as they happen with sub-second latency and automatic ABI decoding
              </p>
            </div>

            <div className="bg-slate-900/50 border border-slate-800 rounded-xl p-6 hover:bg-slate-800/50 transition-all duration-200 group">
              <div className="w-12 h-12 bg-purple-600/10 border border-purple-500/20 rounded-lg flex items-center justify-center mb-4 group-hover:scale-110 transition-transform">
                <BarChart3 className="h-6 w-6 text-purple-400" />
              </div>
              <h3 className="text-lg font-semibold text-white mb-2">Advanced Analytics</h3>
              <p className="text-slate-400 text-sm">
                Deep insights with filtering, aggregation, and visualization tools for contract behavior
              </p>
            </div>

            <div className="bg-slate-900/50 border border-slate-800 rounded-xl p-6 hover:bg-slate-800/50 transition-all duration-200 group">
              <div className="w-12 h-12 bg-green-600/10 border border-green-500/20 rounded-lg flex items-center justify-center mb-4 group-hover:scale-110 transition-transform">
                <Zap className="h-6 w-6 text-green-400" />
              </div>
              <h3 className="text-lg font-semibold text-white mb-2">GraphQL API</h3>
              <p className="text-slate-400 text-sm">
                Powerful, flexible queries with subscriptions for real-time updates and custom integrations
              </p>
            </div>

            <div className="bg-slate-900/50 border border-slate-800 rounded-xl p-6 hover:bg-slate-800/50 transition-all duration-200 group">
              <div className="w-12 h-12 bg-orange-600/10 border border-orange-500/20 rounded-lg flex items-center justify-center mb-4 group-hover:scale-110 transition-transform">
                <Shield className="h-6 w-6 text-orange-400" />
              </div>
              <h3 className="text-lg font-semibold text-white mb-2">Enterprise Ready</h3>
              <p className="text-slate-400 text-sm">
                Production-grade reliability with rate limiting, error handling, and scalable architecture
              </p>
            </div>
          </div>
        </div>
      </div>

      {/* Stats Section */}
      <div className="py-20">
        <div className="max-w-7xl mx-auto px-6">
          <div className="text-center">
            <h2 className="text-3xl font-bold text-white mb-12">Trusted by developers worldwide</h2>
            <div className="grid grid-cols-2 md:grid-cols-4 gap-8">
              <div className="text-center">
                <div className="text-3xl font-bold text-blue-400 mb-2">99.9%</div>
                <div className="text-slate-400 text-sm">Uptime</div>
              </div>
              <div className="text-center">
                <div className="text-3xl font-bold text-purple-400 mb-2">1M+</div>
                <div className="text-slate-400 text-sm">Events Indexed</div>
              </div>
              <div className="text-center">
                <div className="text-3xl font-bold text-green-400 mb-2">&lt;100ms</div>
                <div className="text-slate-400 text-sm">Query Response</div>
              </div>
              <div className="text-center">
                <div className="text-3xl font-bold text-orange-400 mb-2">24/7</div>
                <div className="text-slate-400 text-sm">Monitoring</div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

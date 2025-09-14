import Link from "next/link";

export default function Home() {
  return (
    <div className="font-sans min-h-screen bg-gradient-to-br from-blue-50 to-indigo-100 dark:from-gray-900 dark:to-gray-800">
      <div className="container mx-auto px-4 py-12">
        {/* Header */}
        <header className="text-center mb-16">
          <h1 className="text-5xl md:text-6xl font-bold text-gray-900 dark:text-white mb-6">
            Starknet Indexer
          </h1>
          <p className="text-xl md:text-2xl text-gray-600 dark:text-gray-300 max-w-3xl mx-auto mb-8">
            Real-time blockchain data indexing for Starknet. Fast, reliable, and scalable infrastructure for your dApps.
          </p>
          
          <div className="flex gap-4 items-center justify-center flex-col sm:flex-row">
            <Link
              href="/pricing"
              className="rounded-full bg-blue-600 hover:bg-blue-700 text-white font-semibold text-lg px-8 py-4 transition-colors shadow-lg hover:shadow-xl"
            >
              View Pricing
            </Link>
            <button className="rounded-full border-2 border-blue-600 text-blue-600 hover:bg-blue-600 hover:text-white font-semibold text-lg px-8 py-4 transition-colors">
              Get Started Free
            </button>
          </div>
        </header>

        {/* Features */}
        <section className="grid grid-cols-1 md:grid-cols-3 gap-8 mb-16">
          <div className="bg-white dark:bg-gray-800 rounded-xl p-6 shadow-lg">
            <div className="w-12 h-12 bg-blue-100 dark:bg-blue-900 rounded-lg flex items-center justify-center mb-4">
              <svg className="w-6 h-6 text-blue-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 10V3L4 14h7v7l9-11h-7z" />
              </svg>
            </div>
            <h3 className="text-xl font-semibold text-gray-900 dark:text-white mb-2">Real-time Updates</h3>
            <p className="text-gray-600 dark:text-gray-300">Get instant updates on blockchain events with WebSocket connections.</p>
          </div>

          <div className="bg-white dark:bg-gray-800 rounded-xl p-6 shadow-lg">
            <div className="w-12 h-12 bg-green-100 dark:bg-green-900 rounded-lg flex items-center justify-center mb-4">
              <svg className="w-6 h-6 text-green-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
              </svg>
            </div>
            <h3 className="text-xl font-semibold text-gray-900 dark:text-white mb-2">99.9% Uptime</h3>
            <p className="text-gray-600 dark:text-gray-300">Enterprise-grade reliability with comprehensive monitoring and alerts.</p>
          </div>

          <div className="bg-white dark:bg-gray-800 rounded-xl p-6 shadow-lg">
            <div className="w-12 h-12 bg-purple-100 dark:bg-purple-900 rounded-lg flex items-center justify-center mb-4">
              <svg className="w-6 h-6 text-purple-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M7 4V2a1 1 0 011-1h8a1 1 0 011 1v2m-9 0h10m-10 0l1 16a2 2 0 002 2h6a2 2 0 002-2l1-16" />
              </svg>
            </div>
            <h3 className="text-xl font-semibold text-gray-900 dark:text-white mb-2">GraphQL API</h3>
            <p className="text-gray-600 dark:text-gray-300">Flexible queries with GraphQL for efficient data fetching.</p>
          </div>
        </section>

        {/* Stats */}
        <section className="bg-white dark:bg-gray-800 rounded-2xl p-8 shadow-xl mb-16">
          <div className="grid grid-cols-1 md:grid-cols-4 gap-8 text-center">
            <div>
              <div className="text-3xl font-bold text-blue-600 mb-2">1M+</div>
              <div className="text-gray-600 dark:text-gray-300">Requests Served</div>
            </div>
            <div>
              <div className="text-3xl font-bold text-green-600 mb-2">50ms</div>
              <div className="text-gray-600 dark:text-gray-300">Average Response</div>
            </div>
            <div>
              <div className="text-3xl font-bold text-purple-600 mb-2">24/7</div>
              <div className="text-gray-600 dark:text-gray-300">Support Available</div>
            </div>
            <div>
              <div className="text-3xl font-bold text-orange-600 mb-2">100+</div>
              <div className="text-gray-600 dark:text-gray-300">Happy Developers</div>
            </div>
          </div>
        </section>

        {/* CTA */}
        <section className="text-center">
          <h2 className="text-3xl font-bold text-gray-900 dark:text-white mb-4">
            Ready to get started?
          </h2>
          <p className="text-xl text-gray-600 dark:text-gray-300 mb-8">
            Join thousands of developers building on Starknet with our indexing infrastructure.
          </p>
          <Link
            href="/pricing"
            className="inline-block rounded-full bg-blue-600 hover:bg-blue-700 text-white font-semibold text-lg px-8 py-4 transition-colors shadow-lg hover:shadow-xl"
          >
            Choose Your Plan
          </Link>
        </section>
      </div>
    </div>
  );
}

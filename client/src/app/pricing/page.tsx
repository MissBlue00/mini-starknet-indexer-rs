'use client';

import { useState, useEffect } from 'react';
import { PaymentModal } from '../../components/PaymentModal';

// Mock API function for pricing rates
const fetchPricingRates = async () => {
  // Simulate API call delay
  await new Promise(resolve => setTimeout(resolve, 500));
  
  return {
    payAsYouGo: {
      usdcPerRequest: 0.001, // $0.001 per request
      minimumAmount: 10, // $10 minimum
    },
    subscriptions: {
      plans: [
        {
          id: 'basic',
          name: 'Basic',
          requestsPerSecond: 10,
          monthlyPrice: 29,
          discount: 10,
          description: 'Perfect for small projects and testing'
        },
        {
          id: 'pro',
          name: 'Professional',
          requestsPerSecond: 100,
          monthlyPrice: 149,
          discount: 15,
          description: 'Great for growing applications'
        },
        {
          id: 'enterprise',
          name: 'Enterprise',
          requestsPerSecond: 1000,
          monthlyPrice: 499,
          discount: 20,
          description: 'For high-traffic applications'
        },
        {
          id: 'custom',
          name: 'Custom',
          requestsPerSecond: 'Unlimited',
          monthlyPrice: 'Contact us',
          discount: 0,
          description: 'Tailored solutions for your needs'
        }
      ],
      discounts: {
        quarterly: 0.1, // 10% discount
        semiannual: 0.15, // 15% discount
        annual: 0.2, // 20% discount
      }
    }
  };
};

type BillingPeriod = 'monthly' | 'quarterly' | 'semiannual' | 'annual';
type PricingMode = 'payAsYouGo' | 'subscription';

export default function PricingPage() {
  const [pricingMode, setPricingMode] = useState<PricingMode>('payAsYouGo');
  const [usdcAmount, setUsdcAmount] = useState<string>('100');
  const [billingPeriod, setBillingPeriod] = useState<BillingPeriod>('monthly');
  const [pricingData, setPricingData] = useState<{
    payAsYouGo: {
      usdcPerRequest: number;
      minimumAmount: number;
    };
    subscriptions: {
      plans: Array<{
        id: string;
        name: string;
        requestsPerSecond: number | string;
        monthlyPrice: number | string;
        discount: number;
        description: string;
      }>;
      discounts: {
        quarterly: number;
        semiannual: number;
        annual: number;
      };
    };
  } | null>(null);
  const [loading, setLoading] = useState(true);
  const [paymentModal, setPaymentModal] = useState<{
    isOpen: boolean;
    amount: number;
    planName?: string;
  }>({
    isOpen: false,
    amount: 0,
    planName: undefined,
  });

  useEffect(() => {
    const loadPricingData = async () => {
      try {
        const data = await fetchPricingRates();
        setPricingData(data);
      } catch (error) {
        console.error('Failed to load pricing data:', error);
      } finally {
        setLoading(false);
      }
    };

    loadPricingData();
  }, []);

  const calculateRequests = (amount: number) => {
    if (!pricingData) return 0;
    return Math.floor(amount / pricingData.payAsYouGo.usdcPerRequest);
  };

  const calculateDiscountedPrice = (basePrice: number | string) => {
    if (typeof basePrice !== 'number') return basePrice;
    
    const discounts = pricingData?.subscriptions.discounts || {
      quarterly: 0,
      semiannual: 0,
      annual: 0
    };
    let discount = 0;
    
    switch (billingPeriod) {
      case 'quarterly':
        discount = discounts.quarterly || 0;
        break;
      case 'semiannual':
        discount = discounts.semiannual || 0;
        break;
      case 'annual':
        discount = discounts.annual || 0;
        break;
      default:
        discount = 0;
    }
    
    return basePrice * (1 - discount);
  };

  const getBillingMultiplier = () => {
    switch (billingPeriod) {
      case 'quarterly': return 3;
      case 'semiannual': return 6;
      case 'annual': return 12;
      default: return 1;
    }
  };

  const openPaymentModal = (amount: number, planName?: string) => {
    setPaymentModal({
      isOpen: true,
      amount,
      planName,
    });
  };

  const closePaymentModal = () => {
    setPaymentModal({
      isOpen: false,
      amount: 0,
      planName: undefined,
    });
  };

  const handlePaymentSuccess = (txHash: string) => {
    console.log('Payment successful:', txHash);
    // You can add additional success handling here
    // e.g., updating user credits, showing success message, etc.
    closePaymentModal();
  };

  if (loading) {
    return (
      <div className="min-h-screen bg-gradient-to-br from-blue-50 to-indigo-100 dark:from-gray-900 dark:to-gray-800 flex items-center justify-center">
        <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600"></div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-gradient-to-br from-blue-50 to-indigo-100 dark:from-gray-900 dark:to-gray-800">
      <div className="container mx-auto px-4 py-12">
        {/* Header */}
        <div className="text-center mb-12">
          <h1 className="text-4xl md:text-5xl font-bold text-gray-900 dark:text-white mb-4">
            Simple, Transparent Pricing
          </h1>
          <p className="text-xl text-gray-600 dark:text-gray-300 max-w-2xl mx-auto">
            Choose the perfect plan for your Starknet indexing needs. Scale as you grow.
          </p>
        </div>

        {/* Pricing Mode Toggle */}
        <div className="flex justify-center mb-12">
          <div className="bg-white dark:bg-gray-800 p-1 rounded-lg shadow-lg">
            <button
              onClick={() => setPricingMode('payAsYouGo')}
              className={`px-6 py-3 rounded-md font-medium transition-all ${
                pricingMode === 'payAsYouGo'
                  ? 'bg-blue-600 text-white shadow-md'
                  : 'text-gray-600 dark:text-gray-300 hover:text-gray-900 dark:hover:text-white'
              }`}
            >
              Pay as You Go
            </button>
            <button
              onClick={() => setPricingMode('subscription')}
              className={`px-6 py-3 rounded-md font-medium transition-all ${
                pricingMode === 'subscription'
                  ? 'bg-blue-600 text-white shadow-md'
                  : 'text-gray-600 dark:text-gray-300 hover:text-gray-900 dark:hover:text-white'
              }`}
            >
              Subscription
            </button>
          </div>
        </div>

        {/* Pay as You Go Section */}
        {pricingMode === 'payAsYouGo' && (
          <div className="max-w-2xl mx-auto">
            <div className="bg-white dark:bg-gray-800 rounded-2xl shadow-xl p-8">
              <div className="text-center mb-8">
                <h2 className="text-3xl font-bold text-gray-900 dark:text-white mb-2">
                  Pay as You Go
                </h2>
                <p className="text-gray-600 dark:text-gray-300">
                  Only pay for what you use. Perfect for variable workloads.
                </p>
              </div>

              <div className="space-y-6">
                <div>
                  <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                    Amount in USDC
                  </label>
                  <div className="relative">
                    <input
                      type="number"
                      value={usdcAmount}
                      onChange={(e) => setUsdcAmount(e.target.value)}
                      min={pricingData?.payAsYouGo.minimumAmount || 10}
                      className="w-full px-4 py-3 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent bg-white dark:bg-gray-700 text-gray-900 dark:text-white [appearance:textfield] [&::-webkit-outer-spin-button]:appearance-none [&::-webkit-inner-spin-button]:appearance-none"
                      placeholder="Enter amount"
                    />
                    <div className="absolute inset-y-0 right-0 pr-3 flex items-center">
                      <span className="text-gray-500 dark:text-gray-400 font-medium">USDC</span>
                    </div>
                  </div>
                  <p className="text-sm text-gray-500 dark:text-gray-400 mt-1">
                    Minimum amount: ${pricingData?.payAsYouGo.minimumAmount || 10} USDC
                  </p>
                </div>

                <div className="bg-blue-50 dark:bg-blue-900/20 rounded-lg p-6">
                  <div className="flex justify-between items-center mb-4">
                    <span className="text-lg font-medium text-gray-900 dark:text-white">
                      Estimated Requests
                    </span>
                    <span className="text-2xl font-bold text-blue-600 dark:text-blue-400">
                      {calculateRequests(parseFloat(usdcAmount) || 0).toLocaleString()}
                    </span>
                  </div>
                  <div className="text-sm text-gray-600 dark:text-gray-300 space-y-1">
                    <p>Rate: ${pricingData?.payAsYouGo.usdcPerRequest || 0.001} USDC per request</p>
                    <p>Total cost: ${parseFloat(usdcAmount) || 0} USDC</p>
                  </div>
                </div>

                <button 
                  onClick={() => openPaymentModal(parseFloat(usdcAmount) || 0, 'Pay as You Go')}
                  className="w-full bg-blue-600 hover:bg-blue-700 text-white font-semibold py-3 px-6 rounded-lg transition-colors"
                >
                  Fund Account
                </button>
              </div>
            </div>
          </div>
        )}

        {/* Subscription Section */}
        {pricingMode === 'subscription' && (
          <div>
            {/* Billing Period Toggle */}
            <div className="flex justify-center mb-8">
              <div className="bg-white dark:bg-gray-800 p-1 rounded-lg shadow-lg">
                {(['monthly', 'quarterly', 'semiannual', 'annual'] as BillingPeriod[]).map((period) => {
                  const getDiscountForPeriod = () => {
                    const discounts = pricingData?.subscriptions.discounts || {
                      quarterly: 0,
                      semiannual: 0,
                      annual: 0
                    };
                    switch (period) {
                      case 'quarterly': return (discounts.quarterly || 0) * 100;
                      case 'semiannual': return (discounts.semiannual || 0) * 100;
                      case 'annual': return (discounts.annual || 0) * 100;
                      default: return 0;
                    }
                  };

                  return (
                    <button
                      key={period}
                      onClick={() => setBillingPeriod(period)}
                      className={`px-4 py-2 rounded-md font-medium transition-all capitalize relative ${
                        billingPeriod === period
                          ? 'bg-blue-600 text-white shadow-md'
                          : 'text-gray-600 dark:text-gray-300 hover:text-gray-900 dark:hover:text-white'
                      }`}
                    >
                      {period}
                      {period !== 'monthly' && (
                        <span className="absolute -top-2 -right-2 bg-green-500 text-white text-xs px-2 py-1 rounded-full">
                          -{getDiscountForPeriod()}%
                        </span>
                      )}
                    </button>
                  );
                })}
              </div>
            </div>

            {/* Subscription Plans */}
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
              {pricingData?.subscriptions.plans.map((plan, index: number) => (
                <div
                  key={plan.id}
                  className={`bg-white dark:bg-gray-800 rounded-2xl shadow-xl p-6 relative ${
                    index === 1 ? 'ring-2 ring-blue-500 scale-105' : ''
                  }`}
                >
                  {index === 1 && (
                    <div className="absolute -top-4 left-1/2 transform -translate-x-1/2">
                      <span className="bg-blue-500 text-white px-4 py-1 rounded-full text-sm font-medium">
                        Most Popular
                      </span>
                    </div>
                  )}


                  <div className="text-center mb-6">
                    <h3 className="text-2xl font-bold text-gray-900 dark:text-white mb-2">
                      {plan.name}
                    </h3>
                    <p className="text-gray-600 dark:text-gray-300 text-sm mb-4">
                      {plan.description}
                    </p>
                    
                    <div className="mb-2">
                      <span className="text-3xl font-bold text-gray-900 dark:text-white">
                        {typeof plan.monthlyPrice === 'number'
                          ? `$${(Number(calculateDiscountedPrice(plan.monthlyPrice)) * Number(getBillingMultiplier())).toFixed(0)}`
                          : plan.monthlyPrice
                        }
                      </span>
                      {typeof plan.monthlyPrice === 'number' && (
                        <span className="text-gray-500 dark:text-gray-400">
                          /{billingPeriod === 'monthly' ? 'mo' : billingPeriod.slice(0, -2)}
                        </span>
                      )}
                    </div>

                    {typeof plan.monthlyPrice === 'number' && billingPeriod !== 'monthly' && (
                      <div className="text-sm text-gray-500 dark:text-gray-400">
                        <span className="line-through">
                          ${(plan.monthlyPrice * getBillingMultiplier()).toFixed(0)}
                        </span>
                        <span className="ml-2 text-green-600 dark:text-green-400 font-medium">
                          Save ${((Number(plan.monthlyPrice) * Number(getBillingMultiplier())) - (Number(calculateDiscountedPrice(plan.monthlyPrice)) * Number(getBillingMultiplier()))).toFixed(0)}
                        </span>
                      </div>
                    )}
                  </div>

                  <div className="space-y-4 mb-6">
                    <div className="flex items-center justify-between">
                      <span className="text-gray-600 dark:text-gray-300">Requests/second</span>
                      <span className="font-semibold text-gray-900 dark:text-white">
                        {plan.requestsPerSecond}
                      </span>
                    </div>
                    <div className="flex items-center justify-between">
                      <span className="text-gray-600 dark:text-gray-300">Support</span>
                      <span className="font-semibold text-gray-900 dark:text-white">
                        {plan.id === 'basic' ? 'Email' : plan.id === 'pro' ? 'Priority' : 'Dedicated'}
                      </span>
                    </div>
                    <div className="flex items-center justify-between">
                      <span className="text-gray-600 dark:text-gray-300">SLA</span>
                      <span className="font-semibold text-gray-900 dark:text-white">
                        {plan.id === 'basic' ? '99.5%' : plan.id === 'pro' ? '99.9%' : '99.99%'}
                      </span>
                    </div>
                  </div>

                  <button 
                    onClick={() => {
                      if (plan.id === 'custom') {
                        // Handle custom plan contact
                        console.log('Contact sales for custom plan');
                      } else {
                        const planPrice = Number(calculateDiscountedPrice(plan.monthlyPrice)) * Number(getBillingMultiplier());
                        openPaymentModal(planPrice, plan.name);
                      }
                    }}
                    className={`w-full py-3 px-6 rounded-lg font-semibold transition-colors ${
                      index === 1
                        ? 'bg-blue-600 hover:bg-blue-700 text-white'
                        : 'bg-gray-100 hover:bg-gray-200 dark:bg-gray-700 dark:hover:bg-gray-600 text-gray-900 dark:text-white'
                    }`}
                  >
                    {plan.id === 'custom' ? 'Contact Sales' : 'Get Started'}
                  </button>
                </div>
              ))}
            </div>

            <div className="text-center mt-12">
              <p className="text-gray-600 dark:text-gray-300 mb-4">
                All plans include unlimited bandwidth, real-time updates, and GraphQL API access
              </p>
              <p className="text-sm text-gray-500 dark:text-gray-400">
                Need a custom solution? <a href="#" className="text-blue-600 hover:text-blue-700 font-medium">Contact our sales team</a>
              </p>
            </div>
          </div>
        )}

        {/* Payment Modal */}
        <PaymentModal
          isOpen={paymentModal.isOpen}
          onClose={closePaymentModal}
          amount={paymentModal.amount}
          planName={paymentModal.planName}
          onPaymentSuccess={handlePaymentSuccess}
        />
      </div>
    </div>
  );
}

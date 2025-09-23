// CPU Pricing API service
export interface CpuPricingTier {
  id: string;
  name: string;
  cpu_units_per_request: number;
  price_per_cpu_unit_usdc: number;
  minimum_charge_usdc: number;
  is_active: boolean;
  created_at: string;
  updated_at: string;
}

export interface CpuUsageStats {
  api_call_id: string;
  cpu_units_used: number;
  cpu_time_ms: number;
  memory_usage_mb: number;
  cost_usdc: number;
  timestamp: string;
  deployment_id?: string;
  endpoint: string;
  method: string;
}

export interface CpuPricingRates {
  payAsYouGo: {
    usdcPerCpuUnit: number;
    minimumCharge: number;
  };
  tiers: CpuPricingTier[];
  subscriptions: {
    plans: Array<{
      id: string;
      name: string;
      cpuUnitsPerSecond: number | string;
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
}

class CpuPricingApi {
  private baseUrl: string;

  constructor() {
    this.baseUrl = process.env.NODE_ENV === 'production' 
      ? 'https://your-api-domain.com' 
      : 'http://localhost:3000';
  }

  async fetchCpuPricingRates(): Promise<CpuPricingRates> {
    try {
      const response = await fetch(`${this.baseUrl}/graphql`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          query: `
            query GetCpuPricingRates {
              cpuPricingTiers(activeOnly: true) {
                id
                name
                cpuUnitsPerRequest
                pricePerCpuUnitUsdc
                minimumChargeUsdc
                isActive
                createdAt
                updatedAt
              }
            }
          `,
        }),
      });

      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }

      const data = await response.json();
      
      if (data.errors) {
        throw new Error(`GraphQL errors: ${JSON.stringify(data.errors)}`);
      }

      const tiers = data.data.cpuPricingTiers || [];
      
      // Calculate pay-as-you-go pricing from the first tier
      const payAsYouGoTier = tiers[0] || {
        pricePerCpuUnitUsdc: 0.0001,
        minimumChargeUsdc: 0.001
      };

      // Create subscription plans based on CPU tiers
      const subscriptionPlans = tiers.map((tier: CpuPricingTier, index: number) => ({
        id: tier.id,
        name: tier.name,
        cpuUnitsPerSecond: tier.cpu_units_per_request * 10, // Assume 10 requests per second
        monthlyPrice: Math.round(tier.price_per_cpu_unit_usdc * tier.cpu_units_per_request * 10 * 30 * 24 * 60 * 60), // Monthly cost
        discount: index * 5, // Increasing discount for higher tiers
        description: `Up to ${tier.cpu_units_per_request} CPU units per request`
      }));

      return {
        payAsYouGo: {
          usdcPerCpuUnit: payAsYouGoTier.pricePerCpuUnitUsdc,
          minimumCharge: payAsYouGoTier.minimumChargeUsdc,
        },
        tiers,
        subscriptions: {
          plans: subscriptionPlans,
          discounts: {
            quarterly: 0.1,
            semiannual: 0.15,
            annual: 0.2,
          }
        }
      };
    } catch (error) {
      console.error('Failed to fetch CPU pricing rates:', error);
      
      // Fallback to default pricing
      return {
        payAsYouGo: {
          usdcPerCpuUnit: 0.0001,
          minimumCharge: 0.001,
        },
        tiers: [],
        subscriptions: {
          plans: [
            {
              id: 'basic',
              name: 'Basic',
              cpuUnitsPerSecond: 10,
              monthlyPrice: 29,
              discount: 10,
              description: 'Perfect for small projects and testing'
            },
            {
              id: 'pro',
              name: 'Professional',
              cpuUnitsPerSecond: 100,
              monthlyPrice: 149,
              discount: 15,
              description: 'Great for growing applications'
            },
            {
              id: 'enterprise',
              name: 'Enterprise',
              cpuUnitsPerSecond: 1000,
              monthlyPrice: 499,
              discount: 20,
              description: 'For high-traffic applications'
            }
          ],
          discounts: {
            quarterly: 0.1,
            semiannual: 0.15,
            annual: 0.2,
          }
        }
      };
    }
  }

  async fetchCpuUsageStats(deploymentId?: string, fromDate?: string, toDate?: string): Promise<CpuUsageStats[]> {
    try {
      const response = await fetch(`${this.baseUrl}/graphql`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          query: `
            query GetCpuUsageStats($deploymentId: String, $fromDate: String, $toDate: String) {
              cpuUsageStats(deploymentId: $deploymentId, fromDate: $fromDate, toDate: $toDate) {
                apiCallId
                cpuUnitsUsed
                cpuTimeMs
                memoryUsageMb
                costUsdc
                timestamp
                deploymentId
                endpoint
                method
              }
            }
          `,
          variables: {
            deploymentId,
            fromDate,
            toDate,
          },
        }),
      });

      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }

      const data = await response.json();
      
      if (data.errors) {
        throw new Error(`GraphQL errors: ${JSON.stringify(data.errors)}`);
      }

      return data.data.cpuUsageStats || [];
    } catch (error) {
      console.error('Failed to fetch CPU usage stats:', error);
      return [];
    }
  }

  async createCpuPricingTier(
    name: string,
    cpuUnitsPerRequest: number,
    pricePerCpuUnitUsdc: number,
    minimumChargeUsdc: number
  ): Promise<string> {
    try {
      const response = await fetch(`${this.baseUrl}/graphql`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          query: `
            mutation CreateCpuPricingTier($name: String!, $cpuUnitsPerRequest: Int!, $pricePerCpuUnitUsdc: Float!, $minimumChargeUsdc: Float!) {
              createCpuPricingTier(
                name: $name,
                cpuUnitsPerRequest: $cpuUnitsPerRequest,
                pricePerCpuUnitUsdc: $pricePerCpuUnitUsdc,
                minimumChargeUsdc: $minimumChargeUsdc
              )
            }
          `,
          variables: {
            name,
            cpuUnitsPerRequest,
            pricePerCpuUnitUsdc,
            minimumChargeUsdc,
          },
        }),
      });

      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }

      const data = await response.json();
      
      if (data.errors) {
        throw new Error(`GraphQL errors: ${JSON.stringify(data.errors)}`);
      }

      return data.data.createCpuPricingTier;
    } catch (error) {
      console.error('Failed to create CPU pricing tier:', error);
      throw error;
    }
  }
}

export const cpuPricingApi = new CpuPricingApi();

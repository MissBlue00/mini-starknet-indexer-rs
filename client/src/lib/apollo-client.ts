import { ApolloClient, InMemoryCache, createHttpLink, ApolloLink, Observable } from '@apollo/client';
import { GET_EVENTS, GET_CONTRACT } from './graphql/queries';

// Mock data for demonstration
const mockEvents = [
  {
    id: "event_1",
    contractAddress: "0x049d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7",
    eventType: "Transfer",
    blockNumber: "456789",
    transactionHash: "0xabc123def456789abc123def456789abc123def456789abc123def456789abc123",
    logIndex: 0,
    timestamp: "2024-01-15T10:30:00Z",
    data: {
      from: "0x123abc...",
      to: "0x456def...",
      amount: "1000000000000000000"
    },
    rawData: ["0x123abc", "0x456def", "0x1000000000000000000"],
    rawKeys: ["0x99cd8bde557814842a3121e8ddfd433a539b8c9f14bf31ebf108d12e6196e9"],
  },
  {
    id: "event_2",
    contractAddress: "0x049d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7",
    eventType: "Approval",
    blockNumber: "456790",
    transactionHash: "0xdef456789abc123def456789abc123def456789abc123def456789abc123def456",
    logIndex: 1,
    timestamp: "2024-01-15T10:35:00Z",
    data: {
      owner: "0x123abc...",
      spender: "0x789ghi...",
      amount: "500000000000000000"
    },
    rawData: ["0x123abc", "0x789ghi", "0x500000000000000000"],
    rawKeys: ["0x8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b925"],
  },
  {
    id: "event_3",
    contractAddress: "0x049d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7",
    eventType: "Transfer",
    blockNumber: "456791",
    transactionHash: "0x789ghi123jkl456mno789pqr123stu456vwx789yz123abc456def789ghi123jkl",
    logIndex: 0,
    timestamp: "2024-01-15T10:40:00Z",
    data: {
      from: "0x456def...",
      to: "0x789ghi...",
      amount: "2500000000000000000"
    },
    rawData: ["0x456def", "0x789ghi", "0x2500000000000000000"],
    rawKeys: ["0x99cd8bde557814842a3121e8ddfd433a539b8c9f14bf31ebf108d12e6196e9"],
  },
  {
    id: "event_4",
    contractAddress: "0x049d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7",
    eventType: "Mint",
    blockNumber: "456792",
    transactionHash: "0x456def789abc123ghi456jkl789mno123pqr456stu789vwx123yz456abc789def",
    logIndex: 2,
    timestamp: "2024-01-15T10:45:00Z",
    data: {
      to: "0xabc123...",
      amount: "10000000000000000000"
    },
    rawData: ["0xabc123", "0x10000000000000000000"],
    rawKeys: ["0x0f6798a560793a54c3bcfe86a93cde1e73087d944c0ea20544137d4121396885"],
  },
  {
    id: "event_5",
    contractAddress: "0x049d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7",
    eventType: "Burn",
    blockNumber: "456793",
    transactionHash: "0x123abc456def789ghi123jkl456mno789pqr123stu456vwx789yz123abc456def",
    logIndex: 1,
    timestamp: "2024-01-15T10:50:00Z",
    data: {
      from: "0x789ghi...",
      amount: "750000000000000000"
    },
    rawData: ["0x789ghi", "0x750000000000000000"],
    rawKeys: ["0xcc16f5dbb4873280815c1ee09dbd06736cffcc184412cf7a71a0fdb75d397ca5"],
  }
];

const mockContract = {
  address: "0x049d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7",
  name: "Ether Token",
  verified: true,
  abi: '{"type":"contract","contract_name":"ERC20","events":[{"name":"Transfer","type":"event"},{"name":"Approval","type":"event"}]}',
  events: [
    {
      name: "Transfer",
      type: "event",
      inputs: [
        { name: "from", type: "felt", indexed: true },
        { name: "to", type: "felt", indexed: true },
        { name: "value", type: "Uint256", indexed: false }
      ],
      anonymous: false
    },
    {
      name: "Approval",
      type: "event",
      inputs: [
        { name: "owner", type: "felt", indexed: true },
        { name: "spender", type: "felt", indexed: true },
        { name: "value", type: "Uint256", indexed: false }
      ],
      anonymous: false
    },
    {
      name: "Mint",
      type: "event",
      inputs: [
        { name: "to", type: "felt", indexed: true },
        { name: "amount", type: "Uint256", indexed: false }
      ],
      anonymous: false
    },
    {
      name: "Burn",
      type: "event",
      inputs: [
        { name: "from", type: "felt", indexed: true },
        { name: "amount", type: "Uint256", indexed: false }
      ],
      anonymous: false
    }
  ]
};

// Create mock responses
const mocks = [
  {
    request: {
      query: GET_EVENTS,
      variables: {
        contractAddress: "0x049d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7",
        first: 50,
        orderBy: 'BLOCK_NUMBER_DESC',
      },
    },
    result: {
      data: {
        events: {
          edges: mockEvents.map(event => ({
            node: event,
            cursor: event.id,
          })),
          pageInfo: {
            hasNextPage: false,
            hasPreviousPage: false,
            startCursor: "event_1",
            endCursor: "event_5",
          },
          totalCount: mockEvents.length,
        },
      },
    },
  },
  {
    request: {
      query: GET_CONTRACT,
      variables: {
        address: "0x049d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7",
      },
    },
    result: {
      data: {
        contract: mockContract,
      },
    },
  },
];

// Create a simple mock link that returns our mock data
const mockLink = new ApolloLink((operation) => {
  return new Observable((observer) => {
    const { operationName, variables } = operation;
    
    // Find matching mock response
    const mockResponse = mocks.find(mock => {
      const mockVars = mock.request.variables || {};
      const opVars = variables || {};
      
      // Check if operation matches and key variables match
      if (mock.request.query === operation.query) {
        if (operationName === 'GetEvents') {
          return mockVars.contractAddress === opVars.contractAddress;
        }
        if (operationName === 'GetContract') {
          return mockVars.address === opVars.address;
        }
        return true;
      }
      return false;
    });

    setTimeout(() => {
      if (mockResponse) {
        observer.next(mockResponse.result);
      } else {
        // Default empty response for unmatched queries
        if (operationName === 'GetEvents') {
          observer.next({
            data: {
              events: {
                edges: [],
                pageInfo: {
                  hasNextPage: false,
                  hasPreviousPage: false,
                  startCursor: null,
                  endCursor: null,
                },
                totalCount: 0,
              },
            },
          });
        } else if (operationName === 'GetContract') {
          observer.next({
            data: {
              contract: null,
            },
          });
        }
      }
      observer.complete();
    }, 100); // Simulate network delay
  });
});

const httpLink = createHttpLink({
  uri: 'http://localhost:3000/graphql',
});

// Use mock data in development, real GraphQL in production
const link = process.env.NODE_ENV === 'development' ? mockLink : httpLink;

const client = new ApolloClient({
  link,
  cache: new InMemoryCache(),
  defaultOptions: {
    watchQuery: {
      errorPolicy: 'all',
    },
    query: {
      errorPolicy: 'all',
    },
  },
});

export default client;
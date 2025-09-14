export interface Event {
  id: string;
  contractAddress: string;
  eventType: string;
  blockNumber: string;
  transactionHash: string;
  logIndex: number;
  timestamp: string;
  data?: Record<string, unknown>;
  rawData: string[];
  rawKeys: string[];
}

export interface EventEdge {
  node: Event;
  cursor: string;
}

export interface PageInfo {
  hasNextPage: boolean;
  hasPreviousPage: boolean;
  startCursor?: string;
  endCursor?: string;
}

export interface EventConnection {
  edges: EventEdge[];
  pageInfo: PageInfo;
  totalCount: number;
}

export interface Contract {
  address: string;
  name?: string;
  abi?: string;
  verified: boolean;
  events: EventSchema[];
}

export interface EventInput {
  name: string;
  type: string;
  indexed: boolean;
}

export interface EventSchema {
  name: string;
  type: string;
  inputs: EventInput[];
  anonymous: boolean;
}

export enum EventOrderBy {
  BLOCK_NUMBER_DESC = 'BLOCK_NUMBER_DESC',
  BLOCK_NUMBER_ASC = 'BLOCK_NUMBER_ASC',
  TIMESTAMP_DESC = 'TIMESTAMP_DESC',
  TIMESTAMP_ASC = 'TIMESTAMP_ASC',
}

export interface EventsQueryVariables {
  contractAddress?: string;
  contractAddresses?: string[];
  eventTypes?: string[];
  eventKeys?: string[];
  fromBlock?: string;
  toBlock?: string;
  fromTimestamp?: string;
  toTimestamp?: string;
  transactionHash?: string;
  first?: number;
  after?: string;
  orderBy?: EventOrderBy;
}

export interface EventsQueryResult {
  events: EventConnection;
}

export interface ContractQueryVariables {
  address: string;
}

export interface ContractQueryResult {
  contract: Contract;
}

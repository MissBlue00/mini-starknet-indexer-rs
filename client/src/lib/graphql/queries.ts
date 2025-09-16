import { gql } from '@apollo/client';

export const GET_EVENTS = gql`
  query GetEvents(
    $contractAddress: String
    $contractAddresses: [String!]
    $eventTypes: [String!]
    $eventKeys: [String!]
    $fromBlock: String
    $toBlock: String
    $fromTimestamp: String
    $toTimestamp: String
    $transactionHash: String
    $first: Int
    $after: String
    $orderBy: EventOrderBy
  ) {
    events(
      contractAddress: $contractAddress
      contractAddresses: $contractAddresses
      eventTypes: $eventTypes
      eventKeys: $eventKeys
      fromBlock: $fromBlock
      toBlock: $toBlock
      fromTimestamp: $fromTimestamp
      toTimestamp: $toTimestamp
      transactionHash: $transactionHash
      first: $first
      after: $after
      orderBy: $orderBy
    ) {
      edges {
        node {
          id
          contractAddress
          eventType
          blockNumber
          transactionHash
          logIndex
          timestamp
          data
          rawData
          rawKeys
        }
        cursor
      }
      pageInfo {
        hasNextPage
        hasPreviousPage
        startCursor
        endCursor
      }
      totalCount
    }
  }
`;

export const GET_CONTRACT = gql`
  query GetContract($address: String!) {
    contract(address: $address) {
      address
      name
      abi
      verified
      events {
        name
        type
        inputs {
          name
          type
          indexed
        }
        anonymous
      }
    }
  }
`;

export const GET_DEPLOYMENTS = gql`
  query GetDeployments($first: Int, $after: String) {
    deployments(first: $first, after: $after) {
      address
      name
      verified
      events {
        name
        type
        inputs {
          name
          type
          indexed
        }
        anonymous
      }
    }
  }
`;

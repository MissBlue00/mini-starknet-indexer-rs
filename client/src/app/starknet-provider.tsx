'use client';

import { StarknetConfig, publicProvider } from '@starknet-react/core';
import { mainnet, sepolia } from '@starknet-react/chains';
import { ReactNode } from 'react';

interface StarknetProviderProps {
  children: ReactNode;
}

export function StarknetProvider({ children }: StarknetProviderProps) {
  return (
    <StarknetConfig
      chains={[mainnet, sepolia]}
      provider={publicProvider()}
      autoConnect
    >
      {children}
    </StarknetConfig>
  );
}
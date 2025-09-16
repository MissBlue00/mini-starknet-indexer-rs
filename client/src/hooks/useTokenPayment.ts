'use client';

import { useAccount, useSendTransaction } from '@starknet-react/core';
import { cairo, CallData } from 'starknet';

// Token addresses on Starknet mainnet
export const TOKEN_ADDRESSES = {
  USDC: '0x053c91253bc9682c04929ca02ed00b3e423f6710d2ee7e0d5ebb06f3ecf368a8',
  USDT: '0x068f5c6a61780768455de69077e07e89787839bf8166decfbf92b645209c0fb8',
  STRK: '0x04718f5a0fc34cc1af16a1cdee98ffb20c31f5cd61d6ab07201858f4287c938d'
} as const;

export const RECIPIENT_ADDRESS = '0x05e01dB693CBF7461a016343042786DaC5A6000104813cF134a1E8B1D0a6810b';

export type TokenType = keyof typeof TOKEN_ADDRESSES;

interface UseTokenPaymentProps {
  tokenType: TokenType;
}

export function useTokenPayment({ tokenType }: UseTokenPaymentProps) {
  const { address, isConnected } = useAccount();
  const tokenAddress = TOKEN_ADDRESSES[tokenType];

  const { send, isPending: sendPending, error: sendError } = useSendTransaction({
    calls: []
  });

  const transferToken = async (amount: string) => {
    if (!isConnected || !address) {
      throw new Error('Wallet not connected');
    }

    try {
      // Convert amount to proper format (assuming 6 decimals for USDC/USDT, 18 for STRK)
      const decimals = tokenType === 'STRK' ? 18 : 6;
      const amountBigInt = BigInt(Math.floor(parseFloat(amount) * Math.pow(10, decimals)));
      
      // Use proper uint256 format for Starknet
      const amountInWei = cairo.uint256(amountBigInt);

      // Create the call data for the transfer function
      const transferCall = {
        contractAddress: tokenAddress,
        entrypoint: 'transfer',
        calldata: CallData.compile([
          RECIPIENT_ADDRESS,
          amountInWei.low,
          amountInWei.high
        ])
      };

      console.log('Transfer call:', {
        contractAddress: tokenAddress,
        recipient: RECIPIENT_ADDRESS,
        amount: amount,
        amountInWei,
        decimals
      });

      send([transferCall]);

      // Return a mock transaction hash for now - in real implementation this would come from the transaction result
      return { transaction_hash: 'pending' };
    } catch (err) {
      console.error('Transfer failed:', err);
      throw err;
    }
  };

  return {
    transferToken,
    isPending: sendPending,
    error: sendError,
    isConnected,
    userAddress: address,
    tokenAddress,
  };
}

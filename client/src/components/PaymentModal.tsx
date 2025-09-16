'use client';

import { useState } from 'react';
import { useAccount, useConnect, useDisconnect } from '@starknet-react/core';
import { useStarknetkitConnectModal, disconnect } from 'starknetkit';
import { useTokenPayment, TokenType, TOKEN_ADDRESSES } from '../hooks/useTokenPayment';
import { X, Wallet, ExternalLink, CheckCircle, AlertCircle } from 'lucide-react';

interface PaymentModalProps {
  isOpen: boolean;
  onClose: () => void;
  amount: number;
  planName?: string;
  onPaymentSuccess?: (txHash: string) => void;
}

export function PaymentModal({ 
  isOpen, 
  onClose, 
  amount, 
  planName,
  onPaymentSuccess 
}: PaymentModalProps) {
  const [selectedToken, setSelectedToken] = useState<TokenType>('USDC');
  const [paymentStatus, setPaymentStatus] = useState<'idle' | 'pending' | 'success' | 'error'>('idle');
  const [transactionHash, setTransactionHash] = useState<string>('');
  const [errorMessage, setErrorMessage] = useState<string>('');

  const { address, isConnected } = useAccount();
  const { connect, connectors } = useConnect();
  const { disconnect: disconnectWallet } = useDisconnect();
  
  const { starknetkitConnectModal } = useStarknetkitConnectModal({
    connectors: connectors,
  });
  
  const { transferToken, isPending, error } = useTokenPayment({ 
    tokenType: selectedToken 
  });

  const handleConnect = async () => {
    try {
      const { connector } = await starknetkitConnectModal();
      if (!connector) {
        return;
      }
      await connect({ connector });
    } catch (err) {
      console.error('Failed to connect:', err);
      setErrorMessage('Failed to connect wallet');
    }
  };

  const handlePayment = async () => {
    if (!isConnected || !address) {
      setErrorMessage('Please connect your wallet first');
      return;
    }

    try {
      setPaymentStatus('pending');
      setErrorMessage('');
      
      const result = await transferToken(amount.toString());
      
      if (result?.transaction_hash) {
        setTransactionHash(result.transaction_hash);
        setPaymentStatus('success');
        onPaymentSuccess?.(result.transaction_hash);
      }
    } catch (err: unknown) {
      console.error('Payment failed:', err);
      setPaymentStatus('error');
      setErrorMessage(
        (err as Error)?.message || 'Payment failed. Please try again.'
      );
    }
  };

  const resetModal = () => {
    setPaymentStatus('idle');
    setTransactionHash('');
    setErrorMessage('');
    onClose();
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50 p-4">
      <div className="bg-white dark:bg-gray-800 rounded-2xl shadow-xl max-w-md w-full max-h-[90vh] overflow-y-auto">
        {/* Header */}
        <div className="flex items-center justify-between p-6 border-b border-gray-200 dark:border-gray-700">
          <h2 className="text-2xl font-bold text-gray-900 dark:text-white">
            {paymentStatus === 'success' ? 'Payment Successful!' : 'Complete Payment'}
          </h2>
          <button
            onClick={resetModal}
            className="text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 transition-colors"
          >
            <X size={24} />
          </button>
        </div>

        <div className="p-6">
          {paymentStatus === 'success' ? (
            // Success State
            <div className="text-center space-y-4">
              <CheckCircle className="mx-auto text-green-500" size={64} />
              <div>
                <h3 className="text-xl font-semibold text-gray-900 dark:text-white mb-2">
                  Payment Completed!
                </h3>
                <p className="text-gray-600 dark:text-gray-300">
                  Your payment of {amount} {selectedToken} has been processed successfully.
                </p>
                {planName && (
                  <p className="text-sm text-gray-500 dark:text-gray-400 mt-2">
                    Plan: {planName}
                  </p>
                )}
              </div>
              
              {transactionHash && (
                <div className="bg-gray-50 dark:bg-gray-700 rounded-lg p-4">
                  <p className="text-sm text-gray-600 dark:text-gray-300 mb-2">Transaction Hash:</p>
                  <div className="flex items-center space-x-2">
                    <code className="text-xs bg-gray-100 dark:bg-gray-600 px-2 py-1 rounded flex-1 truncate">
                      {transactionHash}
                    </code>
                    <a
                      href={`https://starkscan.co/tx/${transactionHash}`}
                      target="_blank"
                      rel="noopener noreferrer"
                      className="text-blue-600 hover:text-blue-700 dark:text-blue-400"
                    >
                      <ExternalLink size={16} />
                    </a>
                  </div>
                </div>
              )}
              
              <button
                onClick={resetModal}
                className="w-full bg-green-600 hover:bg-green-700 text-white font-semibold py-3 px-6 rounded-lg transition-colors"
              >
                Done
              </button>
            </div>
          ) : (
            // Payment Form
            <div className="space-y-6">
              {/* Payment Summary */}
              <div className="bg-blue-50 dark:bg-blue-900/20 rounded-lg p-4">
                <div className="flex justify-between items-center">
                  <span className="text-lg font-medium text-gray-900 dark:text-white">
                    {planName ? `${planName} Plan` : 'Account Credit'}
                  </span>
                  <span className="text-2xl font-bold text-blue-600 dark:text-blue-400">
                    ${amount}
                  </span>
                </div>
              </div>

              {/* Token Selection */}
              <div>
                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-3">
                  Select Payment Token
                </label>
                <div className="grid grid-cols-2 gap-3">
                  {(Object.keys(TOKEN_ADDRESSES) as TokenType[]).map((token) => (
                    <button
                      key={token}
                      onClick={() => setSelectedToken(token)}
                      className={`p-4 rounded-lg border-2 transition-all ${
                        selectedToken === token
                          ? 'border-blue-500 bg-blue-50 dark:bg-blue-900/20'
                          : 'border-gray-200 dark:border-gray-600 hover:border-gray-300 dark:hover:border-gray-500'
                      }`}
                    >
                      <div className="text-center">
                        <div className="font-semibold text-gray-900 dark:text-white">
                          {token}
                        </div>
                        <div className="text-sm text-gray-500 dark:text-gray-400 mt-1">
                          {token === 'STRK' ? 'Starknet Token' : token === 'USDC' ? 'USD Coin' : 'Tether USD'}
                        </div>
                      </div>
                    </button>
                  ))}
                </div>
              </div>

              {/* Wallet Connection */}
              {!isConnected ? (
                <div className="space-y-4">
                  <div className="text-center">
                    <Wallet className="mx-auto text-gray-400 mb-3" size={48} />
                    <p className="text-gray-600 dark:text-gray-300 mb-4">
                      Connect your Starknet wallet to proceed with payment
                    </p>
                  </div>
                  <button
                    onClick={handleConnect}
                    className="w-full bg-blue-600 hover:bg-blue-700 text-white font-semibold py-3 px-6 rounded-lg transition-colors flex items-center justify-center space-x-2"
                  >
                    <Wallet size={20} />
                    <span>Connect Wallet</span>
                  </button>
                </div>
              ) : (
                <div className="space-y-4">
                  <div className="bg-green-50 dark:bg-green-900/20 rounded-lg p-4">
                    <div className="flex items-center space-x-2">
                      <CheckCircle className="text-green-500" size={20} />
                      <div>
                        <p className="text-sm font-medium text-green-800 dark:text-green-300">
                          Wallet Connected
                        </p>
                        <p className="text-xs text-green-600 dark:text-green-400 font-mono">
                          {address?.slice(0, 6)}...{address?.slice(-4)}
                        </p>
                      </div>
                    </div>
                    <button
                      onClick={() => {
                        disconnect();
                        disconnectWallet();
                      }}
                      className="mt-2 text-xs text-green-600 dark:text-green-400 hover:text-green-700 dark:hover:text-green-300"
                    >
                      Disconnect
                    </button>
                  </div>

                  {/* Error Message */}
                  {(errorMessage || error) && (
                    <div className="bg-red-50 dark:bg-red-900/20 rounded-lg p-4 flex items-start space-x-2">
                      <AlertCircle className="text-red-500 flex-shrink-0 mt-0.5" size={16} />
                      <p className="text-sm text-red-800 dark:text-red-300">
                        {errorMessage || error?.message || 'An error occurred'}
                      </p>
                    </div>
                  )}

                  {/* Payment Button */}
                  <button
                    onClick={handlePayment}
                    disabled={isPending || paymentStatus === 'pending'}
                    className="w-full bg-blue-600 hover:bg-blue-700 disabled:bg-gray-400 disabled:cursor-not-allowed text-white font-semibold py-3 px-6 rounded-lg transition-colors"
                  >
                    {isPending || paymentStatus === 'pending' ? (
                      <div className="flex items-center justify-center space-x-2">
                        <div className="animate-spin rounded-full h-5 w-5 border-b-2 border-white"></div>
                        <span>Processing Payment...</span>
                      </div>
                    ) : (
                      `Pay ${amount} ${selectedToken}`
                    )}
                  </button>

                  <p className="text-xs text-gray-500 dark:text-gray-400 text-center">
                    By proceeding, you agree to transfer {amount} {selectedToken} tokens to our service address.
                  </p>
                </div>
              )}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

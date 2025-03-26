/* eslint-disable @typescript-eslint/no-unused-vars */
"use client";

import { useState } from 'react';
import { useWallet, useConnection } from '@solana/wallet-adapter-react';
import { LAMPORTS_PER_SOL } from '@solana/web3.js';
import { useAnchorProgram, findStateAddress } from '@/utils/anchor-client';
import { formatSol } from '@/utils/format';

const BuySellForm = () => {
  const { publicKey } = useWallet();
  const { connection } = useConnection();
  const program = useAnchorProgram();
  
  const [action, setAction] = useState<'buy' | 'sell'>('buy');
  const [amount, setAmount] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [txSignature, setTxSignature] = useState<string | null>(null);
  
  const handleAmountChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    // Allow only numbers and decimals
    const value = e.target.value.replace(/[^0-9.]/g, '');
    setAmount(value);
  };
  
  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!program || !publicKey) return;
    
    try {
      setLoading(true);
      setError(null);
      setTxSignature(null);
      
      const [stateAddress] = await findStateAddress();
      const amountValue = parseFloat(amount);
      
      if (isNaN(amountValue) || amountValue <= 0) {
        throw new Error('Please enter a valid amount');
      }
      
      if (action === 'buy') {
        // Buy EGGS with SOL
        const lamports = amountValue * LAMPORTS_PER_SOL;
        
        const tx = await program.methods
          .buy(lamports)
          .accounts({
            authority: publicKey,
            state: stateAddress,
            stateAccount: stateAddress,
            mint: (await program.account.eggsState.fetch(stateAddress)).mint,
            receiver: publicKey,
            // Note: receiverTokenAccount, feeAddressAccount, etc. would need to be derived 
            // in a real implementation
          })
          .rpc();
        
        setTxSignature(tx);
      } else {
        // Sell EGGS for SOL
        // Note: This is a placeholder implementation
        const tx = await program.methods
          .sell(amountValue)
          .accounts({
            authority: publicKey,
            state: stateAddress,
            stateAccount: stateAddress,
            mint: (await program.account.eggsState.fetch(stateAddress)).mint,
            receiver: publicKey,
            // Note: receiverTokenAccount, feeAddressAccount, etc. would need to be derived
            // in a real implementation
          })
          .rpc();
        
        setTxSignature(tx);
      }
    } catch (err) {
      console.error('Transaction error:', err);
      setError(err instanceof Error ? err.message : 'Transaction failed');
    } finally {
      setLoading(false);
    }
  };
  
  return (
    <div>
      <div className="flex space-x-4 mb-6">
        <button
          type="button"
          className={`flex-1 py-2 rounded-lg font-medium ${
            action === 'buy'
              ? 'bg-green-500 text-white'
              : 'bg-gray-700 text-gray-300 hover:bg-gray-600'
          }`}
          onClick={() => setAction('buy')}
        >
          Buy EGGS
        </button>
        <button
          type="button"
          className={`flex-1 py-2 rounded-lg font-medium ${
            action === 'sell'
              ? 'bg-red-500 text-white'
              : 'bg-gray-700 text-gray-300 hover:bg-gray-600'
          }`}
          onClick={() => setAction('sell')}
        >
          Sell EGGS
        </button>
      </div>
      
      <form onSubmit={handleSubmit}>
        <div className="mb-4">
          <label className="block text-gray-400 mb-2">
            {action === 'buy' ? 'SOL Amount' : 'EGGS Amount'}
          </label>
          <div className="relative">
            <input
              type="text"
              value={amount}
              onChange={handleAmountChange}
              className="w-full bg-gray-700 text-white px-4 py-3 rounded-lg focus:outline-none focus:ring-2 focus:ring-amber-500"
              placeholder={action === 'buy' ? 'Enter SOL amount' : 'Enter EGGS amount'}
              disabled={loading}
            />
            <div className="absolute right-3 top-1/2 transform -translate-y-1/2 text-gray-400">
              {action === 'buy' ? 'SOL' : 'EGGS'}
            </div>
          </div>
        </div>
        
        <button
          type="submit"
          className={`w-full py-3 px-4 rounded-lg font-medium ${
            action === 'buy'
              ? 'bg-green-500 hover:bg-green-600'
              : 'bg-red-500 hover:bg-red-600'
          } text-white disabled:opacity-50`}
          disabled={loading || !amount}
        >
          {loading
            ? 'Processing...'
            : action === 'buy'
            ? 'Buy EGGS'
            : 'Sell EGGS'}
        </button>
      </form>
      
      {error && (
        <div className="mt-4 p-3 bg-red-900/50 border border-red-500 rounded-lg text-red-200">
          {error}
        </div>
      )}
      
      {txSignature && (
        <div className="mt-4 p-3 bg-green-900/50 border border-green-500 rounded-lg text-green-200">
          <p>Transaction successful!</p>
          <a
            href={`https://explorer.solana.com/tx/${txSignature}?cluster=devnet`}
            target="_blank"
            rel="noopener noreferrer"
            className="text-amber-400 hover:underline"
          >
            View on Solana Explorer
          </a>
        </div>
      )}
    </div>
  );
};

export default BuySellForm; 
/* eslint-disable @typescript-eslint/no-explicit-any */
/* eslint-disable @typescript-eslint/no-unused-vars */
"use client";

import { useState, useEffect } from 'react';
import { useWallet, useConnection } from '@solana/wallet-adapter-react';
import { LAMPORTS_PER_SOL } from '@solana/web3.js';
import { BN } from '@coral-xyz/anchor';
import { useAnchorProgram, findStateAddress, findLoanAddress } from '@/utils/anchor-client';
import { formatSol, formatTokenAmount } from '@/utils/format';

const LeverageForm = () => {
  const { publicKey } = useWallet();
  const { connection } = useConnection();
  const program = useAnchorProgram();

  const [solAmount, setSolAmount] = useState('');
  const [days, setDays] = useState('7');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [txSignature, setTxSignature] = useState<string | null>(null);
  const [loanInfo, setLoanInfo] = useState<any>(null);
  const [fetchingLoan, setFetchingLoan] = useState(false);

  // Fetch the user's loan information
  const fetchLoanInfo = async () => {
    if (!program || !publicKey) return;

    try {
      setFetchingLoan(true);
      const [loanAddress] = await findLoanAddress(publicKey);
      const loanData = await program.account.loan.fetch(loanAddress).catch(() => null);
      
      if (loanData) {
        setLoanInfo(loanData);
      } else {
        setLoanInfo(null);
      }
    } catch (err) {
      console.error('Error fetching loan info:', err);
      setLoanInfo(null);
    } finally {
      setFetchingLoan(false);
    }
  };

  // Fetch loan info when component mounts
  useEffect(() => {
    fetchLoanInfo();
  }, [program, publicKey]);

  const handleSolAmountChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const value = e.target.value.replace(/[^0-9.]/g, '');
    setSolAmount(value);
  };

  const handleDaysChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const value = e.target.value.replace(/[^0-9]/g, '');
    setDays(value);
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!program || !publicKey) return;

    try {
      setLoading(true);
      setError(null);
      setTxSignature(null);

      const solValue = parseFloat(solAmount);
      const daysValue = parseInt(days);

      if (isNaN(solValue) || solValue <= 0) {
        throw new Error('Please enter a valid SOL amount');
      }

      if (isNaN(daysValue) || daysValue < 1 || daysValue > 365) {
        throw new Error('Please enter a valid number of days (1-365)');
      }

      // Get necessary addresses
      const [stateAddress] = await findStateAddress();

      // Submit leverage transaction
      const tx = await program.methods
        .leverage(
          new BN(solValue * LAMPORTS_PER_SOL),
          new BN(daysValue)
        )
        .accounts({
          authority: publicKey,
          state: stateAddress,
          // Additional accounts would be needed for a full implementation
        })
        .rpc();

      setTxSignature(tx);

      // Refresh loan info after transaction
      await fetchLoanInfo();

    } catch (err) {
      console.error('Transaction error:', err);
      setError(err instanceof Error ? err.message : 'Transaction failed');
    } finally {
      setLoading(false);
    }
  };

  return (
    <div>
      {loanInfo ? (
        <div className="mb-6 p-4 bg-gray-700 rounded-lg">
          <h3 className="text-lg font-medium mb-3">Your Active Loan</h3>
          <div className="grid grid-cols-2 gap-4">
            <div>
              <div className="text-gray-400 text-sm">Borrowed</div>
              <div className="font-medium">{formatSol(loanInfo.borrowed)} SOL</div>
            </div>
            <div>
              <div className="text-gray-400 text-sm">Collateral</div>
              <div className="font-medium">{formatTokenAmount(loanInfo.collateral)} EGGS</div>
            </div>
            <div>
              <div className="text-gray-400 text-sm">End Date</div>
              <div className="font-medium">
                {new Date(loanInfo.endDate * 1000).toLocaleDateString()}
              </div>
            </div>
            <div>
              <div className="text-gray-400 text-sm">Duration</div>
              <div className="font-medium">{loanInfo.numberOfDays} days</div>
            </div>
          </div>
        </div>
      ) : (
        <div className="mb-6 p-4 bg-gray-700 rounded-lg text-center">
          {fetchingLoan ? (
            <p>Loading loan information...</p>
          ) : (
            <p>You have no active loans</p>
          )}
        </div>
      )}

      <form onSubmit={handleSubmit}>
        <div className="mb-4">
          <label className="block text-gray-400 mb-2">SOL to Borrow</label>
          <div className="relative">
            <input
              type="text"
              value={solAmount}
              onChange={handleSolAmountChange}
              className="w-full bg-gray-700 text-white px-4 py-3 rounded-lg focus:outline-none focus:ring-2 focus:ring-amber-500"
              placeholder="Enter SOL amount to borrow"
              disabled={loading || !!loanInfo}
            />
            <div className="absolute right-3 top-1/2 transform -translate-y-1/2 text-gray-400">
              SOL
            </div>
          </div>
        </div>

        <div className="mb-4">
          <label className="block text-gray-400 mb-2">Loan Duration (Days)</label>
          <input
            type="text"
            value={days}
            onChange={handleDaysChange}
            className="w-full bg-gray-700 text-white px-4 py-3 rounded-lg focus:outline-none focus:ring-2 focus:ring-amber-500"
            placeholder="Number of days (1-365)"
            disabled={loading || !!loanInfo}
          />
        </div>

        <button
          type="submit"
          className="w-full py-3 px-4 rounded-lg font-medium bg-amber-500 hover:bg-amber-600 text-black disabled:opacity-50"
          disabled={loading || !solAmount || !days || !!loanInfo}
        >
          {loading ? 'Processing...' : 'Leverage EGGS'}
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

export default LeverageForm; 
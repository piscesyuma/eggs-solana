/* eslint-disable @typescript-eslint/no-explicit-any */
"use client";

import { useState, useEffect } from 'react';
import { useConnection } from '@solana/wallet-adapter-react';
import { formatSol, formatTokenAmount } from '@/utils/format';
import { useAnchorProgram, findStateAddress } from '@/utils/anchor-client';

const EggsInfo = () => {
  const { connection } = useConnection();
  const program = useAnchorProgram();
  const [loading, setLoading] = useState(true);
  const [stateData, setStateData] = useState<any>(null);
  const [solBalance, setSolBalance] = useState<number | null>(null);

  useEffect(() => {
    if (!program) return;

    const fetchStateData = async () => {
      try {
        setLoading(true);
        const [stateAddress] = await findStateAddress();
        
        // Fetch the state account data
        const state = await program.account.eggsState.fetch(stateAddress);
        setStateData(state);
        
        // Fetch the SOL balance
        const balance = await connection.getBalance(stateAddress);
        setSolBalance(balance);
      } catch (error) {
        console.error('Error fetching state data:', error);
      } finally {
        setLoading(false);
      }
    };

    fetchStateData();
    
    // Set up an interval to refresh data every 30 seconds
    const interval = setInterval(fetchStateData, 30000);
    return () => clearInterval(interval);
  }, [program, connection]);

  if (loading || !stateData) {
    return (
      <div className="bg-gray-800 p-6 rounded-lg shadow-lg">
        <h2 className="text-xl font-semibold mb-4">Eggs Protocol Info</h2>
        <div className="animate-pulse">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            {[...Array(6)].map((_, i) => (
              <div key={i} className="bg-gray-700 h-8 rounded"></div>
            ))}
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="bg-gray-800 p-6 rounded-lg shadow-lg">
      <h2 className="text-xl font-semibold mb-4">Eggs Protocol Info</h2>
      
      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        <div>
          <div className="mb-4">
            <div className="text-gray-400 text-sm">Total EGGS Minted</div>
            <div className="text-xl font-medium">
              {formatTokenAmount(stateData.totalMinted)}
            </div>
          </div>
          
          <div className="mb-4">
            <div className="text-gray-400 text-sm">Protocol SOL Balance</div>
            <div className="text-xl font-medium">
              {solBalance !== null ? formatSol(solBalance) : '0'} SOL
            </div>
          </div>
          
          <div>
            <div className="text-gray-400 text-sm">Trading Started</div>
            <div className="text-xl font-medium">
              {stateData.start ? (
                <span className="text-green-400">Active</span>
              ) : (
                <span className="text-red-400">Not Started</span>
              )}
            </div>
          </div>
        </div>
        
        <div>
          <div className="mb-4">
            <div className="text-gray-400 text-sm">Total Borrowed</div>
            <div className="text-xl font-medium">
              {formatSol(stateData.totalBorrowed)} SOL
            </div>
          </div>
          
          <div className="mb-4">
            <div className="text-gray-400 text-sm">Total Collateral</div>
            <div className="text-xl font-medium">
              {formatTokenAmount(stateData.totalCollateral)} EGGS
            </div>
          </div>
          
          <div>
            <div className="text-gray-400 text-sm">Last Price</div>
            <div className="text-xl font-medium">
              {stateData.lastPrice > 0 
                ? `${formatSol(stateData.lastPrice)} SOL` 
                : 'No trades yet'}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};

export default EggsInfo;
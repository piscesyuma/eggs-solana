"use client";

import { useState } from 'react';
import { useWallet } from '@solana/wallet-adapter-react';
import { WalletMultiButton } from '@solana/wallet-adapter-react-ui';
import Header from '@/components/Header';
import EggsInfo from '@/components/EggsInfo';
import BuySellForm from '@/components/BuySellForm';
import LeverageForm from '@/components/LeverageForm';

export default function Home() {
  const { publicKey } = useWallet();
  const [activeTab, setActiveTab] = useState<'buy-sell' | 'leverage'>('buy-sell');

  return (
    <main className="min-h-screen bg-gradient-to-b from-gray-900 to-gray-800 text-white">
      <Header />
      
      <div className="container mx-auto px-4 py-8">
        <div className="flex flex-col items-center justify-center">
          <h1 className="text-4xl font-bold mb-8 text-center text-amber-400">
            Eggs Protocol
          </h1>
          
          <div className="w-full max-w-md mb-8">
            <WalletMultiButton className="!bg-amber-500 hover:!bg-amber-600 !text-black font-semibold rounded-lg w-full py-3" />
          </div>
          
          {!publicKey ? (
            <div className="bg-gray-800 p-8 rounded-lg shadow-lg w-full max-w-2xl text-center">
              <p className="text-xl mb-4">Connect your wallet to interact with the Eggs protocol</p>
              <p className="text-gray-400">
                Buy, sell, or leverage trade the EGGS token on Solana
              </p>
            </div>
          ) : (
            <div className="w-full max-w-4xl">
              <EggsInfo />
              
              <div className="bg-gray-800 rounded-lg shadow-lg mt-8 overflow-hidden">
                <div className="flex border-b border-gray-700">
                  <button
                    className={`flex-1 py-3 text-center font-medium ${
                      activeTab === 'buy-sell'
                        ? 'bg-amber-500 text-black'
                        : 'text-gray-300 hover:bg-gray-700'
                    }`}
                    onClick={() => setActiveTab('buy-sell')}
                  >
                    Buy / Sell
                  </button>
                  <button
                    className={`flex-1 py-3 text-center font-medium ${
                      activeTab === 'leverage'
                        ? 'bg-amber-500 text-black'
                        : 'text-gray-300 hover:bg-gray-700'
                    }`}
                    onClick={() => setActiveTab('leverage')}
                  >
                    Leverage
                  </button>
                </div>
                
                <div className="p-6">
                  {activeTab === 'buy-sell' ? (
                    <BuySellForm />
                  ) : (
                    <LeverageForm />
                  )}
                </div>
              </div>
            </div>
          )}
        </div>
      </div>
    </main>
  );
} 
"use client";

import { useWallet } from '@solana/wallet-adapter-react';
import { WalletMultiButton } from '@solana/wallet-adapter-react-ui';
import { truncateAddress } from '@/utils/format';

const Header = () => {
  const { publicKey } = useWallet();

  return (
    <header className="bg-gray-900 border-b border-gray-800">
      <div className="container mx-auto px-4 py-4">
        <div className="flex items-center justify-between">
          <div className="flex items-center">
            <span className="text-2xl font-bold text-amber-400">🥚 EggsExampleApp</span>
          </div>
          
          <div className="flex items-center space-x-4">
            {publicKey && (
              <div className="hidden md:block bg-gray-800 px-3 py-1 rounded-lg">
                <span className="text-gray-400 text-sm">
                  {truncateAddress(publicKey.toString())}
                </span>
              </div>
            )}
            
            <WalletMultiButton className="!bg-amber-500 hover:!bg-amber-600 !text-black font-semibold" />
          </div>
        </div>
      </div>
    </header>
  );
};

export default Header; 
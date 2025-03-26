/* eslint-disable @typescript-eslint/no-unused-vars */
import { AnchorProvider, Idl, Program } from '@coral-xyz/anchor';
import { Connection, Keypair, PublicKey } from '@solana/web3.js';
import { useAnchorWallet, useConnection } from '@solana/wallet-adapter-react';
import { useMemo } from 'react';
import idl from '../idl/eggs.json';

export const EGGS_PROGRAM_ID = new PublicKey(idl.metadata.address);

export function useAnchorProgram() {
  const { connection } = useConnection();
  const wallet = useAnchorWallet();

  return useMemo(() => {
    if (!wallet) return null;
    
    const provider = new AnchorProvider(
      connection,
      wallet,
      AnchorProvider.defaultOptions()
    );
    
    return new Program(idl as Idl, EGGS_PROGRAM_ID, provider);
  }, [connection, wallet]);
}

// Helper function to derive PDA for state account
export const findStateAddress = async (): Promise<[PublicKey, number]> => {
  return PublicKey.findProgramAddressSync(
    [Buffer.from('state')],
    EGGS_PROGRAM_ID
  );
};

// Helper function to derive PDA for loan account
export const findLoanAddress = async (userPubkey: PublicKey): Promise<[PublicKey, number]> => {
  return PublicKey.findProgramAddressSync(
    [Buffer.from('loan'), userPubkey.toBuffer()],
    EGGS_PROGRAM_ID
  );
};

// Helper function to derive PDA for escrow token account
export const findEscrowAddress = async (userPubkey: PublicKey): Promise<[PublicKey, number]> => {
  return PublicKey.findProgramAddressSync(
    [Buffer.from('escrow'), userPubkey.toBuffer()],
    EGGS_PROGRAM_ID
  );
};

// Helper function to derive PDA for daily loan data
export const findDailyLoanDataAddress = async (date: number): Promise<[PublicKey, number]> => {
  return PublicKey.findProgramAddressSync(
    [Buffer.from('loan_data'), dateToBytes(date)],
    EGGS_PROGRAM_ID
  );
};

// Helper function to convert a date timestamp to bytes for seeds
function dateToBytes(date: number): Buffer {
  const buffer = Buffer.alloc(8);
  buffer.writeBigInt64LE(BigInt(date));
  return buffer;
} 
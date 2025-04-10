import * as dotenv from 'dotenv';
import * as path from 'path';
import { PublicKey } from '@solana/web3.js';

// Path to .env file
const envPath = path.resolve(__dirname, '../.env');

// Load environment variables
dotenv.config({ path: envPath });

console.log('Current environment variables:');
console.log('PROGRAM_ID:', process.env.PROGRAM_ID || 'Not set');
console.log('ECLIPSE_TOKEN_MINT:', process.env.ECLIPSE_TOKEN_MINT || 'Not set');

// Verify we can use these values to create PublicKey objects
if (process.env.PROGRAM_ID && process.env.ECLIPSE_TOKEN_MINT) {
  try {
    const programId = new PublicKey(process.env.PROGRAM_ID);
    const tokenMint = new PublicKey(process.env.ECLIPSE_TOKEN_MINT);
    console.log('\nSuccessfully created PublicKey objects from environment variables.');
    console.log('Program ID PublicKey:', programId.toString());
    console.log('Token Mint PublicKey:', tokenMint.toString());
  } catch (error) {
    console.error('\nError creating PublicKey objects:', error);
  }
} else {
  console.log('\nOne or both environment variables are not set.');
}

console.log('\nTest complete!'); 
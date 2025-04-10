import { PublicKey, LAMPORTS_PER_SOL, Connection } from '@solana/web3.js'
import * as fs from 'fs';
import * as path from 'path';

export function delay(ms: number) {
    return new Promise( resolve => setTimeout(resolve, ms) );
}

export async function safeAirdrop(address: PublicKey, connection: Connection) {
    const acctInfo = await connection.getAccountInfo(address, "confirmed")

    if (acctInfo == null || acctInfo.lamports < 2 * LAMPORTS_PER_SOL) {
        let signature = await connection.requestAirdrop(
            address,
            2 * LAMPORTS_PER_SOL
        )
        await connection.confirmTransaction(signature)
    }
}

/**
 * Updates or creates a .env file with the provided key-value pairs
 * @param envFilePath Path to the .env file
 * @param envVars Object containing key-value pairs to add to the .env file
 * @returns boolean indicating success or failure
 */
export function updateEnvFile(envFilePath: string, envVars: Record<string, string>): boolean {
  try {
    let envContent = '';
    
    // Read existing .env file if it exists
    if (fs.existsSync(envFilePath)) {
      try {
        envContent = fs.readFileSync(envFilePath, 'utf8');
      } catch (error) {
        console.warn(`Unable to read .env file at ${envFilePath}: ${error instanceof Error ? error.message : String(error)}`);
        // Continue with empty content if we can't read the file
      }
    }

    // Parse existing content into key-value pairs
    const envLines = envContent.split('\n').filter(line => line.trim() !== '');
    const existingVars: Record<string, string> = {};
    
    for (const line of envLines) {
      if (!line.startsWith('#') && line.includes('=')) {
        const [key, ...valueParts] = line.split('=');
        const value = valueParts.join('='); // Handle values that might contain = signs
        existingVars[key.trim()] = value.trim();
      }
    }

    // Merge existing vars with new vars
    const updatedVars = { ...existingVars, ...envVars };
    
    // Convert back to .env format
    const updatedContent = Object.entries(updatedVars)
      .map(([key, value]) => `${key}=${value}`)
      .join('\n');
    
    // Write to file
    try {
      fs.writeFileSync(envFilePath, updatedContent + '\n');
      console.log(`Updated .env file at ${envFilePath}`);
      return true;
    } catch (error) {
      console.warn(`Unable to write to .env file at ${envFilePath}: ${error instanceof Error ? error.message : String(error)}`);
      console.log('Would have written the following environment variables:');
      Object.entries(envVars).forEach(([key, value]) => {
        console.log(`${key}=${value}`);
      });
      return false;
    }
  } catch (error) {
    console.error(`Error updating .env file: ${error instanceof Error ? error.message : String(error)}`);
    return false;
  }
}
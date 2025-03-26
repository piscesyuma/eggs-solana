import { Program, BN, web3 } from '@coral-xyz/anchor';
import { PublicKey, SystemProgram, LAMPORTS_PER_SOL, Connection } from '@solana/web3.js';
import { TOKEN_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID } from '@solana/spl-token';
import { findStateAddress } from '../utils/anchor-client';

export class EggsService {
  constructor(private program: Program, private connection: Connection) {}

  // Get the Eggs state
  async getState() {
    const [stateAddress] = await findStateAddress();
    return this.program.account.eggsState.fetch(stateAddress);
  }

  // Buy EGGS tokens with SOL
  async buyEggs(amount: number) {
    const [stateAddress, stateBump] = await findStateAddress();
    const state = await this.program.account.eggsState.fetch(stateAddress);
    
    // Get the mint from the state
    const mint = state.mint;
    
    // Get our wallet public key
    const walletPubkey = this.program.provider.publicKey;
    
    // Find the associated token account for the wallet
    const tokenAccount = await this.getOrCreateAssociatedTokenAccount(walletPubkey, mint);
    
    // Prepare the transaction
    const tx = await this.program.methods
      .buy(new BN(amount * LAMPORTS_PER_SOL))
      .accounts({
        authority: walletPubkey,
        state: stateAddress,
        stateAccount: stateAddress,
        mint: mint,
        receiver: walletPubkey,
        receiverTokenAccount: tokenAccount,
        feeAddressAccount: state.feeAddress,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        rent: web3.SYSVAR_RENT_PUBKEY,
      })
      .rpc();
    
    return tx;
  }

  // Sell EGGS tokens for SOL
  async sellEggs(amount: number) {
    const [stateAddress, stateBump] = await findStateAddress();
    const state = await this.program.account.eggsState.fetch(stateAddress);
    
    // Get the mint from the state
    const mint = state.mint;
    
    // Get our wallet public key
    const walletPubkey = this.program.provider.publicKey;
    
    // Find the associated token account for the wallet
    const tokenAccount = await this.getOrCreateAssociatedTokenAccount(walletPubkey, mint);
    
    // Prepare the transaction
    const tx = await this.program.methods
      .sell(new BN(amount))
      .accounts({
        authority: walletPubkey,
        state: stateAddress,
        stateAccount: stateAddress,
        mint: mint,
        receiver: walletPubkey,
        receiverTokenAccount: tokenAccount,
        feeAddressAccount: state.feeAddress,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        rent: web3.SYSVAR_RENT_PUBKEY,
      })
      .rpc();
    
    return tx;
  }

  // Helper method to get or create an associated token account
  private async getOrCreateAssociatedTokenAccount(
    owner: PublicKey,
    mint: PublicKey
  ): Promise<PublicKey> {
    // This is a simplified version - ideally you'd use the actual SPL token methods
    // to check if the account exists and create it if needed
    return this.findAssociatedTokenAddress(owner, mint);
  }

  // Helper method to find associated token address
  private async findAssociatedTokenAddress(
    owner: PublicKey,
    mint: PublicKey
  ): Promise<PublicKey> {
    return PublicKey.findProgramAddressSync(
      [owner.toBuffer(), TOKEN_PROGRAM_ID.toBuffer(), mint.toBuffer()],
      ASSOCIATED_TOKEN_PROGRAM_ID
    )[0];
  }
}
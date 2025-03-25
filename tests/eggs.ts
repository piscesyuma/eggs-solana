import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Eggs } from "../target/types/eggs";
import { PublicKey } from "@solana/web3.js";
import { assert } from "chai";

describe("eggs", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.Eggs as Program<Eggs>;
  const wallet = provider.wallet;

  let statePda: PublicKey;
  let stateAccount: PublicKey;
  let mint: PublicKey;
  let bump: number;
  
  // Token Metadata Program ID from Metaplex
  const TOKEN_METADATA_PROGRAM_ID = new PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s");

  it("Is initialized", async () => {
    // Find the PDA for the program state
    const [_statePda, _bump] = await PublicKey.findProgramAddressSync(
      [Buffer.from("state")],
      program.programId
    );
    statePda = _statePda;
    bump = _bump;
    
    // Create a new keypair for the mint
    const mintKeypair = anchor.web3.Keypair.generate();
    mint = mintKeypair.publicKey;
    
    // Derive the metadata account PDA
    const [metadataAccount] = await PublicKey.findProgramAddressSync(
      [
        Buffer.from("metadata"),
        TOKEN_METADATA_PROGRAM_ID.toBuffer(),
        mint.toBuffer(),
      ],
      TOKEN_METADATA_PROGRAM_ID
    );

    // Get authority token account
    const authorityTokenAccount = await anchor.utils.token.associatedAddress({
      mint: mint,
      owner: wallet.publicKey,
    });

    // Initialize the program
    const tx = await program.methods
      .initialize(bump)
      .accounts({
        authority: wallet.publicKey,
        state: statePda,
        stateAccount: statePda,
        mint: mint,
        authorityTokenAccount: authorityTokenAccount,
        metadataProgram: TOKEN_METADATA_PROGRAM_ID,
        metadataAccount: metadataAccount,
        systemProgram: anchor.web3.SystemProgram.programId,
        tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
        associatedTokenProgram: anchor.utils.token.ASSOCIATED_PROGRAM_ID,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      })
      .signers([mintKeypair])
      .rpc();
    
    console.log("Transaction signature", tx);

    // Fetch the program state
    const state = await program.account.eggsState.fetch(statePda);
    console.log("Program state:", state);
    
    // Verify initial values
    assert.equal(state.authority.toString(), wallet.publicKey.toString());
    assert.equal(state.mint.toString(), mint.toString());
    assert.equal(state.buyFee, 975);
    assert.equal(state.sellFee, 975);
    assert.equal(state.start, false);
  });

  it("Sets fee address", async () => {
    // Set the fee address to the wallet for testing
    const tx = await program.methods
      .setFeeAddress(wallet.publicKey)
      .accounts({
        authority: wallet.publicKey,
        state: statePda,
      })
      .rpc();
    
    console.log("Transaction signature", tx);

    // Fetch the program state
    const state = await program.account.eggsState.fetch(statePda);
    
    // Verify fee address was set
    assert.equal(state.feeAddress.toString(), wallet.publicKey.toString());
  });

  // Additional tests would be added for the other functions
  // Including buy, sell, leverage, borrow, etc.
}); 
import * as anchor from "@coral-xyz/anchor";
import { web3 } from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { MushiProgram } from "../target/types/mushi_program";
import { MainStateInfo, GlobalStateInfo, sleep, MushiProgramRpc, getCurrentDateString } from "./mushiProgramRpc";
import { getAssociatedTokenAddressSync, createAssociatedTokenAccount, TOKEN_2022_PROGRAM_ID } from "@solana/spl-token";
import { safeAirdrop } from "./utils";
import { createAssociatedTokenAccountInstruction } from "@solana/spl-token";
import * as path from 'path';
import * as dotenv from 'dotenv';

// Load environment variables from .env file
dotenv.config({ path: path.resolve(__dirname, '../.env') });

const log = console.log;
describe("mushi_program_buy_with_referral", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());
  const provider = anchor.AnchorProvider.env();
  const connection = provider.connection;
  const rpc = connection.rpcEndpoint;
  const programId = new web3.PublicKey(
    "HF5x1bCgynzEnBL7ATMFYPNFjBaqfxgMASyUJL2ud6Xi"
  );
  let mainStateInfo: MainStateInfo | null = null;
  let globalInfo: GlobalStateInfo | null = null;
  const connectivity = new MushiProgramRpc({
    rpc,
    wallet: provider.wallet,
    programId,
  });
  const user = provider.publicKey;
  // const referralPubkey = new web3.PublicKey("HxEx3porEpbGa3PvmocLqooc6VPAAULkYxcr7vSm2hAn");
  const referral = anchor.web3.Keypair.generate();
  const referralPubkey = referral.publicKey;
  // Parameters for the buy operation
  const solAmount = 1; // Amount of SOL to buy tokens with

  it ("Airdrop SOL to referral", async () => {
    await safeAirdrop(referralPubkey, connection);
    await sleep(5000);

    const quoteToken = new web3.PublicKey(process.env.ECLIPSE_TOKEN_MINT!);
    console.log("Quote token:", quoteToken.toBase58());

    // const user1Ata = await createAssociatedTokenAccount (
    //   provider.connection,
    //   referral,
    //   quoteToken,
    //   referral.publicKey,
    //   undefined,
    //   TOKEN_2022_PROGRAM_ID,
    //   undefined,
    //   false,
    // )
    // console.log("Referral quote token account:", user1Ata.toBase58());
  });

  it("Get initial state info", async () => {
    mainStateInfo = await connectivity.getMainStateInfo();
    if (!mainStateInfo) throw "Failed to get main state info";
    // log({ mainStateInfo });

    globalInfo = await connectivity.getGlobalInfo();
    // log({ globalInfo });

    if (!globalInfo) throw "Failed to get global state info";

    // Check if the protocol has been started
    if (!globalInfo.started) {
      log("The protocol has not been started yet. Please run the start test first.");
      return;
    }
    
    // Log the current date string for reference
    // log(`Current date: ${getCurrentDateString()}`);
  });

  it("Buy tokens with SOL", async () => {
    if (!globalInfo || !mainStateInfo) throw "Global state info is not available";
    
    // Perform the buy operation with debug=true to show date strings
    const buyRes = await connectivity.buy_with_referral(solAmount, referral);
    if (!buyRes.isPass) throw "Failed to buy tokens";
    
    log({ buyRes: buyRes.info });

    // Wait for the transaction to be processed
    await sleep(20_000);
    
    // Verify the operation by getting updated state
    const updatedGlobalInfo = await connectivity.getGlobalInfo();
    if (!updatedGlobalInfo) throw "Failed to get updated global state info";
    log({ updatedGlobalInfo });
    
    // You might want to add additional verification here
    // For example, check that token supply has increased
    if (updatedGlobalInfo.tokenSupply <= globalInfo?.tokenSupply!) {
      log("Warning: Token supply did not increase as expected");
    } else {
      log(`Token supply increased from ${globalInfo?.tokenSupply} to ${updatedGlobalInfo.tokenSupply}`);
    }
  });
}); 
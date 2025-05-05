import * as anchor from "@coral-xyz/anchor";
import { web3 } from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { MushiProgram } from "../target/types/mushi_program";
import { MainStateInfo, GlobalStateInfo, sleep, MushiProgramRpc } from "./mushiProgramRpc";
import * as path from 'path';
import * as dotenv from 'dotenv';

// Load environment variables from .env file
dotenv.config({ path: path.resolve(__dirname, '../.env') });
const log = console.log;
describe("mushi_program_sell", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());
  const provider = anchor.AnchorProvider.env();
  const connection = provider.connection;
  const rpc = connection.rpcEndpoint;
  // Define programId - use from env if available or use default
  const programId = process.env.PROGRAM_ID 
    ? new web3.PublicKey(process.env.PROGRAM_ID) 
    : new web3.PublicKey("HF5x1bCgynzEnBL7ATMFYPNFjBaqfxgMASyUJL2ud6Xi");
  

  let mainStateInfo: MainStateInfo | null = null;
  let globalInfo: GlobalStateInfo | null = null;
  const connectivity = new MushiProgramRpc({
    rpc,
    wallet: provider.wallet,
    programId,
  });
  const user = provider.publicKey;

  // Parameters for the sell operation
  const tokenAmount = 49; // Amount of tokens to sell for ECLIPSE

  it("Get initial state info", async () => {
    mainStateInfo = await connectivity.getMainStateInfo();
    if (!mainStateInfo) throw "Failed to get main state info";
    log({ mainStateInfo });

    globalInfo = await connectivity.getGlobalInfo();
    if (!globalInfo) throw "Failed to get global state info";
    log({ globalInfo });

    // Check if the protocol has been started
    if (!mainStateInfo.started) {
      log("The protocol has not been started yet. Please run the start test first.");
      return;
    }
  });

  it("Sell tokens for ECLIPSE", async () => {
    if (!globalInfo) throw "Global state info is not available";

    // Perform the sell operation
    const sellRes = await connectivity.sell(tokenAmount);
    log({ sellRes: sellRes.info });
    
    if (!sellRes.isPass) throw "Failed to sell tokens";
    
    // Wait for the transaction to be processed
    await sleep(7_000);
    
    // Verify the operation by getting updated state
    const updatedGlobalInfo = await connectivity.getGlobalInfo();
    if (!updatedGlobalInfo) throw "Failed to get updated global state info";
    log({ updatedGlobalInfo });
    
    // You might want to add additional verification here
    // For example, check that token supply has decreased
    if (updatedGlobalInfo.tokenSupply >= globalInfo.tokenSupply) {
      log("Warning: Token supply did not decrease as expected");
    } else {
      log(`Token supply decreased from ${globalInfo.tokenSupply} to ${updatedGlobalInfo.tokenSupply}`);
    }
  });
}); 
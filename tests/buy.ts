import * as anchor from "@coral-xyz/anchor";
import { web3 } from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { MushiProgram } from "../target/types/mushi_program";
import { MainStateInfo, GlobalStateInfo, sleep, MushiProgramRpc, getCurrentDateString } from "./mushiProgramRpc";
import * as dotenv from 'dotenv';
import * as path from 'path';

// Load environment variables from .env file
dotenv.config({ path: path.resolve(__dirname, '../.env') });
const log = console.log;
describe("mushi_program_buy", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());
  const provider = anchor.AnchorProvider.env();
  const connection = provider.connection;
  const rpc = connection.rpcEndpoint;
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

  // Parameters for the buy operation
  const esAmount = 100; // Amount of ECLIPSE to buy tokens with

  it("Get initial state info", async () => {
    mainStateInfo = await connectivity.getMainStateInfo();
    if (!mainStateInfo) throw "Failed to get main state info";
    log({ mainStateInfo });

    globalInfo = await connectivity.getGlobalInfo();
    log({ globalInfo });

    if (!globalInfo) throw "Failed to get global state info";

    // Check if the protocol has been started
    if (!mainStateInfo.started) {
      log("The protocol has not been started yet. Please run the start test first.");
      return;
    }
    
    // Log the current date string for reference
    log(`Current date: ${getCurrentDateString()}`);
  });

  it("Buy tokens with ECLIPSE", async () => {
    if (!globalInfo) throw "Global state info is not available";

    // Perform the buy operation with debug=true to show date strings
    const buyRes = await connectivity.buy(esAmount, true);
    if (!buyRes.isPass) throw "Failed to buy tokens";
    
    log({ buyRes: buyRes.info });

    // Wait for the transaction to be processed
    await sleep(10_000);
    
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
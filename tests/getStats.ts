import * as anchor from "@coral-xyz/anchor";
import { web3 } from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { MushiProgram } from "../target/types/mushi_program";
import { MainStateInfo, GlobalStateInfo, sleep, MushiProgramRpc, getCurrentDateString, UserLoanInfo } from "./mushiProgramRpc";

const log = console.log;
describe("mushi_program_getStats", () => {
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
  let userLoanInfo: UserLoanInfo | null = null;
  
  const connectivity = new MushiProgramRpc({
    rpc,
    wallet: provider.wallet,
    programId,
  });
  const user = provider.publicKey;

  // Parameters for the buy operation
  const solAmount = 0.1; // Amount of SOL to buy tokens with

  it("Get initial state info", async () => {
    mainStateInfo = await connectivity.getMainStateInfo();
    if (!mainStateInfo) throw "Failed to get main state info";
    log({ mainStateInfo });

    // globalInfo = await connectivity.getGlobalInfo();
    // log({ globalInfo });

    // if (!globalInfo) throw "Failed to get global state info";

    // userLoanInfo = await connectivity.getUserLoanInfo(user);
    // if (!userLoanInfo) throw "Failed to get user loan info";
    // log({ userLoanInfo });

    // // Check if the protocol has been started
    // if (!globalInfo.started) {
    //   log("The protocol has not been started yet. Please run the start test first.");
    //   return;
    // }
    
    // Log the current date string for reference
    log(`Current date: ${getCurrentDateString()}`);
  });
}); 
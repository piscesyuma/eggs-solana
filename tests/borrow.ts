import * as anchor from "@coral-xyz/anchor";
import { web3 } from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { MushiProgram } from "../target/types/mushi_program";
import { MainStateInfo, GlobalStateInfo, sleep, MushiProgramRpc, getCurrentDateString } from "./mushiProgramRpc";

const log = console.log;
describe("mushi_program_borrow", () => {
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

  // Parameters for the borrow operation
  const solAmount = 1; // Amount of SOL to borrow
  const numberOfDays = 7; // Loan duration in days

  it("Get initial state info", async () => {
    mainStateInfo = await connectivity.getMainStateInfo();
    if (!mainStateInfo) throw "Failed to get main state info";
    log({ mainStateInfo });

    globalInfo = await connectivity.getGlobalInfo();
    log({ globalInfo });

    if (!globalInfo) throw "Failed to get global state info";

    // Check if the protocol has been started
    if (!globalInfo.started) {
      log("The protocol has not been started yet. Please run the start test first.");
      return;
    }
    
    // Log the current date string for reference
    log(`Current date: ${getCurrentDateString()}`);
  });

  it("Borrow SOL", async () => {
    if (!globalInfo) throw "Global state info is not available";

    // Perform the borrow operation with debug=true to show date strings
    const borrowRes = await connectivity.borrow(solAmount, numberOfDays, true);
    if (!borrowRes.isPass) throw "Failed to borrow SOL";
    
    log({ borrowRes: borrowRes.info });

    // Wait for the transaction to be processed
    await sleep(10_000);
    
    // Verify the operation by getting updated state
    const updatedGlobalInfo = await connectivity.getGlobalInfo();
    if (!updatedGlobalInfo) throw "Failed to get updated global state info";
    log({ updatedGlobalInfo });
    
    // Log the transaction was successful
    log("Successfully borrowed SOL");
  });
}); 
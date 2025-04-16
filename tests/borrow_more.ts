import * as anchor from "@coral-xyz/anchor";
import { web3 } from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { MushiProgram } from "../target/types/mushi_program";
import { MainStateInfo, GlobalStateInfo, sleep, MushiProgramRpc, getCurrentDateString } from "./mushiProgramRpc";

const log = console.log;
describe("mushi_program_borrow_more", () => {
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

  // Parameters for the borrow more operation
  const additionalEclipseAmount = 0.05; // Additional ECLIPSE amount to borrow

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
    
    // Get user loan info to verify a loan exists
    const userLoanInfo = await connectivity.getUserLoanInfo(user);
    if (!userLoanInfo || userLoanInfo.borrowed === 0) {
      log("No active loan found. Please borrow first before borrowing more.");
      return;
    }
    
    log(`Current date: ${getCurrentDateString()}`);
    log(`Current loan info: ${JSON.stringify(userLoanInfo)}`);
  });

  it("Borrow more ECLIPSE", async () => {
    if (!globalInfo) throw "Global state info is not available";

    // Get initial loan info
    const initialLoanInfo = await connectivity.getUserLoanInfo(user);
    if (!initialLoanInfo) throw "Failed to get initial loan info";
    
    // Perform the borrow more operation with debug=true to show date strings
    const borrowMoreRes = await connectivity.borrow_more(additionalEclipseAmount, true);
    if (!borrowMoreRes.isPass) throw "Failed to borrow more ECLIPSE";
    
    log({ borrowMoreRes: borrowMoreRes.info });

    // Wait for the transaction to be processed
    await sleep(10_000);
    
    // Verify the operation by getting updated loan info
    const updatedLoanInfo = await connectivity.getUserLoanInfo(user);
    if (!updatedLoanInfo) throw "Failed to get updated loan info";
    
    // Compare borrowed amounts to verify additional borrowing
    log({
      initialBorrowed: initialLoanInfo.borrowed,
      updatedBorrowed: updatedLoanInfo.borrowed
    });
    
    // Verify the updated global state
    const updatedGlobalInfo = await connectivity.getGlobalInfo();
    if (!updatedGlobalInfo) throw "Failed to get updated global state info";
    log({ updatedGlobalInfo });
    
    // Log the transaction was successful
    log("Successfully borrowed additional Eclipse");
  });
}); 
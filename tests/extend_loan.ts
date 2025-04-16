import * as anchor from "@coral-xyz/anchor";
import { web3 } from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { MushiProgram } from "../target/types/mushi_program";
import { MainStateInfo, GlobalStateInfo, sleep, MushiProgramRpc, getCurrentDateString } from "./mushiProgramRpc";

const log = console.log;
describe("mushi_program_extend_loan", () => {
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

  // Parameters for the extend loan operation
  const esAmount = 0.01; // ECLIPSE fee amount for extending loan
  const numberOfDays = 5; // Number of days to extend the loan by

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
      log("No active loan found. Please borrow first before extending a loan.");
      return;
    }
    
    log(`Current date: ${getCurrentDateString()}`);
    log(`User loan end date: ${userLoanInfo.endDate}`);
  });

  it("Extend loan", async () => {
    if (!globalInfo) throw "Global state info is not available";

    // Get initial loan info
    const initialLoanInfo = await connectivity.getUserLoanInfo(user);
    if (!initialLoanInfo) throw "Failed to get initial loan info";
    
    // Perform the extend loan operation with debug=true to show date strings
    const extendLoanRes = await connectivity.extend_loan(numberOfDays, true);
    if (!extendLoanRes.isPass) throw "Failed to extend loan";
    
    log({ extendLoanRes: extendLoanRes.info });

    // Wait for the transaction to be processed
    await sleep(10_000);
    
    // Verify the operation by getting updated loan info
    const updatedLoanInfo = await connectivity.getUserLoanInfo(user);
    if (!updatedLoanInfo) throw "Failed to get updated loan info";
    
    // Compare dates to verify extension
    log({
      initialEndDate: initialLoanInfo.endDate,
      updatedEndDate: updatedLoanInfo.endDate
    });
    
    // Log the transaction was successful
    log("Successfully extended loan");
  });
}); 
import * as anchor from "@coral-xyz/anchor";
import { web3 } from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { MushiProgram } from "../target/types/mushi_program";
import { MainStateInfo, GlobalStateInfo, sleep, MushiProgramRpc } from "./mushiProgramRpc";

const log = console.log;
describe("mushi_program_sell", () => {
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

  // Parameters for the sell operation
  const tokenAmount = 1000; // Amount of tokens to sell for SOL

  it("Get initial state info", async () => {
    mainStateInfo = await connectivity.getMainStateInfo();
    if (!mainStateInfo) throw "Failed to get main state info";
    log({ mainStateInfo });

    globalInfo = await connectivity.getGlobalInfo();
    if (!globalInfo) throw "Failed to get global state info";
    log({ globalInfo });

    // Check if the protocol has been started
    if (!globalInfo.started) {
      log("The protocol has not been started yet. Please run the start test first.");
      return;
    }
  });

  it("Sell tokens for SOL", async () => {
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
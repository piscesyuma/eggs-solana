import * as anchor from "@coral-xyz/anchor";
import { web3 } from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { MushiProgram } from "../target/types/mushi_program";
import { MainStateInfo, sleep, MushiProgramRpc } from "./mushiProgramRpc";
import * as dotenv from 'dotenv';
import * as path from 'path';

// Load environment variables from .env file
dotenv.config({ path: path.resolve(__dirname, '../.env') });

const log = console.log;
describe("sonic_program", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());
  const provider = anchor.AnchorProvider.env();
  const connection = provider.connection;
  const rpc = connection.rpcEndpoint;

  console.log("env program id", process.env.PROGRAM_ID);
  console.log("env eclipse token mint", process.env.ECLIPSE_TOKEN_MINT);
  
  const programId = process.env.PROGRAM_ID 
    ? new web3.PublicKey(process.env.PROGRAM_ID) 
    : new web3.PublicKey("HF5x1bCgynzEnBL7ATMFYPNFjBaqfxgMASyUJL2ud6Xi");
  console.log("programId", programId);
  let mainStateInfo: MainStateInfo;
  const connectivity = new MushiProgramRpc({
    rpc,
    wallet: provider.wallet,
    programId,
  });
  const user = provider.publicKey;

  const tokenName = "Mushi";
  const tokenSymbol = "MUSHI";
  const tokenUri = "https://mushi.xyz";

  it("set stake token", async () => {
    const info = await connectivity.getMainStateInfo();
    console.log("main state info", { info });
    if (info) {
      const updateRes = await connectivity.updateMainState({
        stakeToken: new web3.PublicKey("2222222222222222222222222222222222222222")
      });
      if (!updateRes.isPass) throw "failed to update main state";
      await sleep(7_000);
      const _info = await connectivity.getMainStateInfo();
      if (!_info) throw "failed to get main state info";
      console.log({ _info });
    }
  });
});

import * as anchor from "@coral-xyz/anchor";
import { web3 } from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { MushiProgram } from "../target/types/mushi_program";
import { MainStateInfo, sleep, MushiProgramRpc } from "./mushiProgramRpc";

const log = console.log;
describe("sonic_program", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());
  const provider = anchor.AnchorProvider.env();
  const connection = provider.connection;
  const rpc = connection.rpcEndpoint;
  const programId = new web3.PublicKey(
    "9eykXRhjtB3PXZSd4ZwYVyajyACC5D3iGgTBvjYNbFpK"
  );
  let mainStateInfo: MainStateInfo;
  const connectivity = new MushiProgramRpc({
    rpc,
    wallet: provider.wallet,
    programId,
  });
  const user = provider.publicKey;

  const tokenName = "Mushi BaBara";
  const tokenSymbol = "MUSHI";
  const tokenUri = "https://mushi.xyz";

  it("start", async () => {
    const info = await connectivity.getMainStateInfo();
    if (info) {
      const startRes = await connectivity.start({
        solAmount: 100,
        tokenName,
        tokenSymbol,
        tokenUri,
      });
      log({ startRes: startRes.info });
      if (!startRes.isPass) throw "failed to start";
      await sleep(7_000);
      const _info = await connectivity.getGlobalInfo();
      if (!_info) throw "failed to get mainstate info";
      console.log({ _info });
    }
  });
});

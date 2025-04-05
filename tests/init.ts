import * as anchor from "@coral-xyz/anchor";
import { web3 } from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { MushiProgram } from "../target/types/mushi_program";
import { MainStateInfo, sleep, MushiProgramRpc } from "./mushiProgramRpc";

const log = console.log;
describe("mushi_program", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());
  const provider = anchor.AnchorProvider.env();
  const connection = provider.connection;
  const rpc = connection.rpcEndpoint;
  const programId = new web3.PublicKey(
    "HF5x1bCgynzEnBL7ATMFYPNFjBaqfxgMASyUJL2ud6Xi"
  );
  let mainStateInfo: MainStateInfo;
  const connectivity = new MushiProgramRpc({
    rpc,
    wallet: provider.wallet,
    programId,
  });
  const user = provider.publicKey;

    const feeReceiver = new web3.PublicKey("8CHNnNzHme7hVv2Qw2WHbxX54EWJ6NMkjJ1zRTEkNvsg");
  const tokenAmount = 100_000;
  const tokenName = "mushi";
  const tokenSymbol = "MUSHI";
  const tokenUri = "sss";

  it("init", async () => {
    const info = await connectivity.getMainStateInfo();
    if (!info) {
      const initRes = await connectivity.initialize({
        sellFee: 975,
        buyFee: 975,
        buyFeeLeverage: 10,
        feeReceiver,
      });
      log({ initRes: initRes.info });
      if (!initRes.isPass) throw "failed to init mainstate";
      await sleep(15_000);
      const _info = await connectivity.getMainStateInfo();
      if (!_info) throw "failed to get mainstate info";
      mainStateInfo = _info;
    } else mainStateInfo = info;
    console.log({ mainStateInfo });
  });
});

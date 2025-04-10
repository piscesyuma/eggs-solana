import * as anchor from "@coral-xyz/anchor";
import { web3 } from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { MushiProgram } from "../target/types/mushi_program";
import { MainStateInfo, sleep, MushiProgramRpc } from "./mushiProgramRpc";
import { createAssociatedTokenAccount, createMint, getAssociatedTokenAddress, mintTo, TOKEN_2022_PROGRAM_ID } from "@solana/spl-token";
import { PublicKey } from "@solana/web3.js";
import { delay, safeAirdrop } from "./utils";

const log = console.log;
describe("mushi_program", () => {
  let eclipseTokenMint: PublicKey;
  
  anchor.setProvider(anchor.AnchorProvider.env());
  const provider = anchor.AnchorProvider.env();
  const connection = provider.connection;
  const rpc = connection.rpcEndpoint;
  let user1Ata: PublicKey;
  let user1StakeAta: PublicKey;
  // test accounts
  const payer = anchor.web3.Keypair.generate()
  
  // it("create quote token", async () => {
  //   await safeAirdrop(payer.publicKey, provider.connection)
  //   delay(10000)
  //   // create new token mint  
  //   eclipseTokenMint = await createMint(
  //     provider.connection,
  //     payer,
  //     payer.publicKey,
  //     null,
  //     6,
  //     undefined,
  //     undefined,
  //     TOKEN_2022_PROGRAM_ID 
  //   )
  //   console.log("Test token mint: ", eclipseTokenMint.toBase58())

  //   // create test token ata of test user
  //   user1StakeAta = await getAssociatedTokenAddress(eclipseTokenMint, payer.publicKey, false, TOKEN_2022_PROGRAM_ID)

  //   user1Ata = await createAssociatedTokenAccount (
  //     provider.connection,
  //     payer,
  //     eclipseTokenMint,
  //     payer.publicKey,
  //     undefined,
  //     TOKEN_2022_PROGRAM_ID
  //   )
  //   console.log("Test user associated token account: ", user1Ata.toBase58())

  //   // mint 1000 tokens to test user
  //   const mintTx = await mintTo(
  //     provider.connection,
  //     payer,
  //     eclipseTokenMint,
  //     user1Ata,
  //     payer,
  //     1000,
  //     undefined,
  //     undefined,
  //     TOKEN_2022_PROGRAM_ID
  //   )
  //   console.log("Mint tx: ", mintTx)
  // });

    
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

  it("init", async () => {
    // const info = await connectivity.getMainStateInfo();
    // if (!info) {
      const initRes = await connectivity.initialize({
        sellFee: 975,
        buyFee: 975,
        buyFeeLeverage: 10,
        feeReceiver,
        quoteToken: new web3.PublicKey("4WhHyNda5YdjV4HXCVM9iSGrZkCegGMhkkjyXnAL51G5"),
      });
      log({ initRes: initRes.info });
      if (!initRes.isPass) throw "failed to init mainstate";
      await sleep(15_000);
      const _info = await connectivity.getMainStateInfo();
      if (!_info) throw "failed to get mainstate info";
      mainStateInfo = _info;
    // } else mainStateInfo = info;
    console.log({ mainStateInfo });
  });
});

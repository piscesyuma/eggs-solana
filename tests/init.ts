import * as anchor from "@coral-xyz/anchor";
import { web3 } from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { MushiProgram } from "../target/types/mushi_program";
import { MainStateInfo, sleep, MushiProgramRpc } from "./mushiProgramRpc";
import { createAssociatedTokenAccount, createMint, getAssociatedTokenAddress, mintTo, TOKEN_2022_PROGRAM_ID } from "@solana/spl-token";
import { PublicKey } from "@solana/web3.js";
import { delay, safeAirdrop, updateEnvFile } from "./utils";
import * as path from 'path';
import * as dotenv from 'dotenv';

// Load environment variables from .env file
dotenv.config({ path: path.resolve(__dirname, '../.env') });

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
  const payer = anchor.web3.Keypair.fromSecretKey(Uint8Array.from(JSON.parse("[157,240,92,76,195,141,93,96,254,82,125,255,121,252,119,222,204,237,129,1,29,220,187,136,192,180,151,220,183,128,142,249,106,229,182,154,1,30,129,103,127,143,233,25,142,116,81,183,178,109,187,36,90,196,130,120,226,191,179,212,93,75,47,135]")));
  console.log("payer", payer.publicKey.toBase58());
  const admin = provider.publicKey;

  // Define programId - use from env if available or use default
  const programId = process.env.PROGRAM_ID 
    ? new web3.PublicKey(process.env.PROGRAM_ID) 
    : new web3.PublicKey("HF5x1bCgynzEnBL7ATMFYPNFjBaqfxgMASyUJL2ud6Xi");
  
  // Save programId to .env file
  const rootDir = path.resolve(__dirname, '..');
  updateEnvFile(path.join(rootDir, '.env'), {
    PROGRAM_ID: programId.toBase58()
  });
  
  let mainStateInfo: MainStateInfo;
  const connectivity = new MushiProgramRpc({
    rpc,
    wallet: provider.wallet,
    programId,
  });
  const user = provider.publicKey;
  // const feeReceiver = new web3.PublicKey("8CHNnNzHme7hVv2Qw2WHbxX54EWJ6NMkjJ1zRTEkNvsg");
  const feeReceiver = anchor.web3.Keypair.generate();

  
  it("create quote token", async () => {
    // const info = await connectivity.getMainStateInfo();
    // if (info) {
    //   // If mainstate already exists, try to use the quoteToken from it or from env
    //   eclipseTokenMint = process.env.ECLIPSE_TOKEN_MINT 
    //     ? new web3.PublicKey(process.env.ECLIPSE_TOKEN_MINT) 
    //     : info.quoteToken;
    //   return;
    // }
    
    // await safeAirdrop(payer.publicKey, provider.connection)
    // delay(10000)
    // create new token mint  
    
    // transfer 0.1 SOL to payer

    //////////////////////////////////////////////////////////////
    
    // eclipseTokenMint = await createMint(
    //   provider.connection,
    //   payer,
    //   payer.publicKey,
    //   null,
    //   9,
    //   undefined,
    //   undefined,
    //   TOKEN_2022_PROGRAM_ID 
    // )
    // console.log("Test token mint: ", eclipseTokenMint.toBase58())

    eclipseTokenMint = new web3.PublicKey("6Y9vfMCya4NbzEw34rXsFYDnbbUnkfRZPsD2qqmmepMH");
    const toUserAddr = new web3.PublicKey("7NWaCnWUr6qwmqGYmcv1r3cTtbtw1W6KKpyJtwWvPqEB");
    // create test token ata of test user
    user1Ata = await createAssociatedTokenAccount (
      provider.connection,
      payer,
      eclipseTokenMint,
      // admin,
      toUserAddr,
      undefined,
      TOKEN_2022_PROGRAM_ID,
      undefined,
      false,
    )
    console.log("Test user associated token account: ", user1Ata.toBase58())

    // mint 1000 tokens to test user
    const mintTx = await mintTo(
      provider.connection,
      payer,
      eclipseTokenMint,
      user1Ata,
      payer.publicKey,
      10000000000000, // 10000 tokens
      undefined,
      undefined,
      TOKEN_2022_PROGRAM_ID
    )
    console.log("Mint tx: ", mintTx)

    // Save eclipseTokenMint to .env file
    // updateEnvFile(path.join(rootDir, '.env'), {
    //   ECLIPSE_TOKEN_MINT: eclipseTokenMint.toBase58()
    // });
  });

  // eclipseTokenMint = new web3.PublicKey("51GWmkWaZbqPdeojuKUBu5M2cR2LjK54n6xLLh9yijod");

  // it("init", async () => {
  //   const info = await connectivity.getMainStateInfo();
  //   if (!info) {
  //     const initRes = await connectivity.initialize({
  //       sellFee: 975,
  //       buyFee: 975,
  //       buyFeeLeverage: 10,
  //       feeReceiver: feeReceiver.publicKey,
  //       quoteToken: eclipseTokenMint,
  //     });
  //     log({ initRes: initRes.info });
  //     if (!initRes.isPass) throw "failed to init mainstate";
  //     await sleep(15_000);
  //     const _info = await connectivity.getMainStateInfo();
  //     if (!_info) throw "failed to get mainstate info";
  //     mainStateInfo = _info;
  //   } else mainStateInfo = info;
  //   console.log({ mainStateInfo });
  // });
});

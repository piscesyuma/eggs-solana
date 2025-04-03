import { BN, Program, web3 } from "@coral-xyz/anchor";
import { IDL, MushiProgram } from "../target/types/mushi_program";
import { AnchorProvider, Wallet } from "@coral-xyz/anchor/dist/cjs/provider";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getAssociatedTokenAddressSync,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";

const ONE_BASIS_POINTS = 100_000;
const Seeds = {
  mainState: Buffer.from("main_state"),
  vault: Buffer.from("vault"),
};
const log = console.log;
export type Result<T, E = string> =
  | { isPass: true; info: T }
  | { isPass: false; info: E };
export type SendTxResult = Result<{ txSignature: string }, string>;
export const TOKEN_DECIMALS_HELPER = 1_000_000; // 6 decimals
export const SOL_DECIMALS_HELPER = 1_000_000_000; // 9 decimals
const associatedTokenProgram = ASSOCIATED_TOKEN_PROGRAM_ID;
const mplProgram = new web3.PublicKey(
  "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
);
const systemProgram = web3.SystemProgram.programId;
const sysvarRent = web3.SYSVAR_RENT_PUBKEY;
const tokenProgram = TOKEN_PROGRAM_ID;

export async function sleep(ms: number) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}
export type MainStateInfo = {
  admin: web3.PublicKey;
  feeOnBuy: number;
  feeReceiver: web3.PublicKey;
  token: web3.PublicKey;
  initOn: number;
};

export class MushiProgramRpc {
  private program: Program<MushiProgram>;
  private connection: web3.Connection;
  private programId: web3.PublicKey;
  private mainState: web3.PublicKey;
  private vaultOwner: web3.PublicKey;
  private provider: AnchorProvider;

  constructor({
    rpc,
    wallet,
    programId,
  }: {
    rpc: string;
    wallet: Wallet;
    programId: web3.PublicKey;
  }) {
    this.connection = new web3.Connection(rpc);
    const provider = new AnchorProvider(this.connection, wallet, {
      commitment: "confirmed",
    });
    this.provider = provider;
    this.programId = programId;
    this.program = new Program(IDL, programId, provider);
    this.mainState = web3.PublicKey.findProgramAddressSync(
      [Seeds.mainState],
      this.programId
    )[0];
    this.vaultOwner = web3.PublicKey.findProgramAddressSync(
      [Seeds.vault],
      this.programId
    )[0];
  }

  async sendTx(
    ixs: web3.TransactionInstruction[],
    signers?: web3.Keypair[]
  ): Promise<string | null> {
    try {
      const payerKey = this.provider.publicKey;
      const recentBlockhash = (await this.connection.getLatestBlockhash())
        .blockhash;
      const msg = new web3.TransactionMessage({
        instructions: ixs,
        payerKey,
        recentBlockhash,
      }).compileToV0Message();
      const tx = new web3.VersionedTransaction(msg);
      signers && tx.sign(signers);
      const signedTx = await this.provider.wallet
        .signTransaction(tx)
        .catch(() => null);
      if (!signedTx) throw "failed to sign tx";
      const txSignature = await this.connection.sendRawTransaction(
        signedTx.serialize(),
        { skipPreflight: true }
      );
      let expireCount = 0;
      for (let i = 0; i < 50; ++i) {
        await sleep(2_000);
        const res = await this.connection
          .getSignatureStatus(txSignature)
          .catch(() => null);
        if (res) {
          if (res.value?.err) {
            const simRes = await this.connection
              .simulateTransaction(tx, {
                replaceRecentBlockhash: true,
              })
              .catch(() => null)
              .then((res) => res?.value);
            log({ txSignature });
            log({ simRes });
            log({ txSignatureRes: res.value });
            throw "tx failed";
          }
          return txSignature;
        }
        const isValid = await this.connection
          .isBlockhashValid(recentBlockhash)
          .catch(() => null)
          .then((res) => res?.value);
        if (isValid == false) expireCount += 1;
        if (expireCount >= 2) {
          log({ txSignature });
          throw "tx expired";
        }
      }
      log({ txSignature });
      return null;
    } catch (sendTxError) {
      log({ sendTxError });
      return null;
    }
  }

  async getMainStateInfo(): Promise<MainStateInfo | null> {
    try {
      const mainStateData = await this.program.account.mainState.fetch(this.mainState);
      return {
        admin: mainStateData.admin,
        token: mainStateData.token,
        initOn: Number(mainStateData.lastLiquidationDate?.toString() || "0"),
        feeOnBuy: Number(mainStateData.buyFee?.toString() || "0") / ONE_BASIS_POINTS,
        feeReceiver: mainStateData.feeReceiver,
      };
    } catch (getMainStateInfoError) {
      log({ getMainStateInfoError });
      return null;
    }
  }

  async initialize(input: {
    tokenName: string;
    tokenSymbol: string;
    tokenUri: string;
    solAmount: number;
    feeReceiver: web3.PublicKey;
    feeOnBuy: number;
  }): Promise<SendTxResult> {
    try {
      const { tokenName, tokenSymbol, tokenUri } = input;
      const tokenKp = web3.Keypair.generate();
      const token = tokenKp.publicKey;
      const admin = this.provider.publicKey;
      const tokenVault = getAssociatedTokenAddressSync(
        token,
        this.vaultOwner,
        true
      );
      const tokenMetadataAccount = web3.PublicKey.findProgramAddressSync(
        [Buffer.from("metadata"), mplProgram.toBuffer(), token.toBuffer()],
        mplProgram
      )[0];

      const ix = await this.program.methods
        .initialize({
          feeReceiver: input.feeReceiver,
          tokenName,
          tokenSymbol,
          tokenUri,
          solAmount: new BN(input.solAmount * SOL_DECIMALS_HELPER),
        })
        .accounts({
          admin,
          mainState: this.mainState,
          tokenVault: tokenVault,
          tokenVaultOwner: this.vaultOwner,
          associatedTokenProgram,
          mplProgram,
          systemProgram,
          sysvarRent,
          tokenProgram,
          token,
          tokenMetadataAccount,
        })
        .instruction();
      const ixs = [
        web3.ComputeBudgetProgram.setComputeUnitLimit({ units: 300_000 }),
        ix,
      ];
      const txSignature = await this.sendTx(ixs, [tokenKp]);
      if (!txSignature) throw "failed to send tx";
      return { isPass: true, info: { txSignature } };
    } catch (initializeError) {
      log({ initializeError });
      return { isPass: false, info: "failed to process the input" };
    }
  }

  async updateMainState(input: {
    feeReceiver?: web3.PublicKey;
    admin?: web3.PublicKey;
  }): Promise<SendTxResult> {
    try {
      const admin = this.provider.publicKey;
      const feeReceiver = input.feeReceiver ?? null;
      const newAdmin = input.admin ?? null;
      const ix = await this.program.methods
        .updateMainState({
          feeReceiver, admin: newAdmin,
          buyFee: new BN(975),
          sellFee: new BN(975),
          buyFeeLeverage: new BN(10),
        })
        .accounts({ admin, mainState: this.mainState })
        .instruction();
      const ixs = [
        web3.ComputeBudgetProgram.setComputeUnitLimit({ units: 100_000 }),
        ix,
      ];
      const updateTxRes = await this.sendTx(ixs);
      if (!updateTxRes) return { isPass: false, info: "failed to send tx" };
      return { isPass: true, info: { txSignature: updateTxRes } };
    } catch (updateMainStateError) {
      log({ updateMainStateError });
      return { isPass: false, info: "failed to process input" };
    }
  }

  async buy(
    solAmount: number,
    mainStateInfo: MainStateInfo
  ): Promise<SendTxResult> {
    try {
      const { token, feeReceiver } = mainStateInfo;
      const rawSolAmount = Math.trunc(solAmount * SOL_DECIMALS_HELPER);
      const user = this.provider.publicKey;
      const userAta = getAssociatedTokenAddressSync(token, user);
      const tokenVault = getAssociatedTokenAddressSync(
        token,
        this.vaultOwner,
        true
      );
      const ix = await this.program.methods
        .buy(new BN(rawSolAmount))
        .accounts({
          mainState: this.mainState,
          user,
          userAta,
          associatedTokenProgram,
          token,
          tokenProgram,
          tokenVault,
          tokenVaultOwner: this.vaultOwner,
          feeReceiver,
          systemProgram,
        })
        .instruction();
      const ixs = [
        web3.ComputeBudgetProgram.setComputeUnitLimit({ units: 150_000 }),
        ix,
      ];
      const txRes = await this.sendTx(ixs);
      if (!txRes) throw "failed to send tx";
      return { isPass: true, info: { txSignature: txRes } };
    } catch (buyError) {
      log({ buyError });
      return { isPass: false, info: "failed to process input" };
    }
  }

  async sell(
    tokenAmount: number,
    mainStateInfo: MainStateInfo
  ): Promise<SendTxResult> {
    try {
      const { token, feeReceiver } = mainStateInfo;
      const rawTokenAmount = Math.trunc(tokenAmount * TOKEN_DECIMALS_HELPER);
      const user = this.provider.publicKey;
      const userAta = getAssociatedTokenAddressSync(token, user);
      const tokenVault = getAssociatedTokenAddressSync(
        token,
        this.vaultOwner,
        true
      );
      const ix = await this.program.methods
        .sell(new BN(rawTokenAmount))
        .accounts({
          mainState: this.mainState,
          user,
          userAta,
          associatedTokenProgram,
          token,
          tokenProgram,
          tokenVault,
          tokenVaultOwner: this.vaultOwner,
          feeReceiver,
          systemProgram,
        })
        .instruction();
      const ixs = [
        web3.ComputeBudgetProgram.setComputeUnitLimit({ units: 150_000 }),
        ix,
      ];
      const txRes = await this.sendTx(ixs);
      if (!txRes) throw "failed to send tx";
      return { isPass: true, info: { txSignature: txRes } };
    } catch (sellError) {
      log({ sellError });
      return { isPass: false, info: "failed to process input" };
    }
  }
}

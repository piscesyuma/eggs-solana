import { BN, Program, web3 } from "@coral-xyz/anchor";
import { IDL, MushiProgram } from "../target/types/mushi_program";
import { AnchorProvider, Wallet } from "@coral-xyz/anchor/dist/cjs/provider";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getAssociatedTokenAddressSync,
  TOKEN_2022_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";

const ONE_BASIS_POINTS = 1;  //100_000;
const Seeds = {
  mainState: Buffer.from("main_state"),
  globalState: Buffer.from("global_stats"),
  vault: Buffer.from("vault"),
};
const log = console.log;
export type Result<T, E = string> =
  | { isPass: true; info: T }
  | { isPass: false; info: E };
export type SendTxResult = Result<{ txSignature: string }, string>;
export const TOKEN_DECIMALS_HELPER = 1_000_000_000; // 9 decimals
export const SOL_DECIMALS_HELPER = 1_000_000_000; // 9 decimals
const SECONDS_IN_A_DAY = 86400;
const associatedTokenProgram = ASSOCIATED_TOKEN_PROGRAM_ID;
const mplProgram = new web3.PublicKey(
  "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
);
const systemProgram = web3.SystemProgram.programId;
const sysvarRent = web3.SYSVAR_RENT_PUBKEY;
const tokenProgram = TOKEN_PROGRAM_ID;
const quoteTokenProgram = TOKEN_2022_PROGRAM_ID;

const quoteTokenMint = new web3.PublicKey("4WhHyNda5YdjV4HXCVM9iSGrZkCegGMhkkjyXnAL51G5");

export async function sleep(ms: number) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}
export type MainStateInfo = {
  admin: web3.PublicKey;
  feeReceiver: web3.PublicKey;
  sellFee: number;
  buyFee: number;
  buyFeeLeverage: number;
  quoteToken: web3.PublicKey;
};
export type GlobalStateInfo = {
  started: boolean;
  tokenSupply: number;
  baseToken: web3.PublicKey;
  lastLiquidationDate: number;
  totalBorrowed: number;
  totalCollateral: number;
  lastPrice: number;
};
export type UserLoanInfo = {
  endDate: string;
  borrowed: number;
  collateral: number;
};

/**
 * Converts a Unix timestamp to YYYY-MM-DD format
 * This is a direct port of the Rust implementation to ensure compatibility
 * @param timestamp Unix timestamp in seconds
 * @returns Formatted date string YYYY-MM-DD
 */
export function getDateStringFromTimestamp(timestamp: number): string {
  // Normalize to midnight
  const normalizedTimestamp = timestamp - (timestamp % SECONDS_IN_A_DAY);
  
  // Calculate days since Unix epoch (1970-01-01)
  const daysSinceEpoch = Math.floor(normalizedTimestamp / SECONDS_IN_A_DAY);
  
  // Initialize with epoch year
  let year = 1970;
  let daysRemaining = daysSinceEpoch;
  
  // Advance through years
  while (true) {
    const daysInYear = isLeapYear(year) ? 366 : 365;
    if (daysRemaining < daysInYear) {
      break;
    }
    daysRemaining -= daysInYear;
    year++;
  }
  
  // Determine month and day
  const daysInMonths = isLeapYear(year) 
    ? [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    : [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
  
  let month = 0;
  for (const daysInMonth of daysInMonths) {
    if (daysRemaining < daysInMonth) {
      break;
    }
    daysRemaining -= daysInMonth;
    month++;
  }
  
  // Month is 0-based in our calculation, but we want 1-based
  month++;
  // Day is 0-based, need to add 1
  const day = daysRemaining + 1;
  
  // Format as YYYY-MM-DD
  return `${year.toString().padStart(4, '0')}-${month.toString().padStart(2, '0')}-${day.toString().padStart(2, '0')}`;
}

// Helper function to determine if a year is a leap year (identical to Rust implementation)
function isLeapYear(year: number): boolean {
  return (year % 4 === 0) && (year % 100 !== 0 || year % 400 === 0);
}

/**
 * Gets the current date at midnight in YYYY-MM-DD format
 * @returns Formatted date string YYYY-MM-DD
 */
export function getCurrentDateString(): string {
  const now = Math.floor(Date.now() / 1000);
  return getDateStringFromTimestamp(now);
}

export class MushiProgramRpc {
  private program: Program<MushiProgram>;
  private connection: web3.Connection;
  private programId: web3.PublicKey;
  private mainState: web3.PublicKey;
  private globalState: web3.PublicKey;
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
    this.globalState = web3.PublicKey.findProgramAddressSync(
      [Seeds.globalState],
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
      const { admin, feeReceiver, sellFee, buyFee, buyFeeLeverage, quoteToken } =
        await this.program.account.mainState.fetch(this.mainState);
      return {
        admin,
        sellFee: Number(sellFee.toString()) / ONE_BASIS_POINTS,
        buyFee: Number(buyFee.toString()) / ONE_BASIS_POINTS,
        buyFeeLeverage: Number(buyFeeLeverage.toString()) / ONE_BASIS_POINTS,
        feeReceiver,
        quoteToken,
      };
    } catch (getMainStateInfoError) {
      return null;
    }
  }

  async getGlobalInfo(): Promise<GlobalStateInfo | null> {
    try {
      const { tokenSupply, baseToken, started, lastLiquidationDate, totalBorrowed, totalCollateral, lastPrice } =
        await this.program.account.globalStats.fetch(this.globalState);
      return {
        tokenSupply: Number(tokenSupply.toString()),
        baseToken,
        started,
        lastLiquidationDate: Number(lastLiquidationDate.toString()),
        totalBorrowed: Number(totalBorrowed.toString()),
        totalCollateral: Number(totalCollateral.toString()),
        lastPrice: Number(lastPrice.toString()),
      };
    } catch (getGlobalStateInfoError) {
      log({ getGlobalStateInfoError });
      return null;
    }
  }

  async getUserLoanInfo(user: web3.PublicKey): Promise<UserLoanInfo | null> {
    try {
      const userLoanAddress = web3.PublicKey.findProgramAddressSync(
        [Buffer.from("user-loan"), user.toBuffer()],
        this.programId
      )[0];

      const userLoanData = await this.program.account.userLoan.fetch(userLoanAddress);
      log({ userLoanData: userLoanData, userEndDate: getDateStringFromTimestamp(Number(userLoanData.endDate.toString())) });
      return {
        endDate: userLoanData.endDate.toString(),
        borrowed: Number(userLoanData.borrowed.toString()),
        collateral: Number(userLoanData.collateral.toString()),
      };
    } catch (getUserLoanInfoError) {
      log({ getUserLoanInfoError });
      return null;
    }
  }

  async initialize(input: {
    feeReceiver: web3.PublicKey;
    sellFee: number;
    buyFee: number;
    buyFeeLeverage: number;
    quoteToken: web3.PublicKey;
  }): Promise<SendTxResult> {
    try {
      log(Math.trunc(input.sellFee * ONE_BASIS_POINTS));
      const admin = this.provider.publicKey;
      const ix = await this.program.methods
        .initMainState({
          feeReceiver: input.feeReceiver,
          quoteToken: input.feeReceiver,
          sellFee: new BN(Math.trunc(input.sellFee * ONE_BASIS_POINTS)),
          buyFee: new BN(Math.trunc(input.buyFee * ONE_BASIS_POINTS)),
          buyFeeLeverage: new BN(Math.trunc(input.buyFeeLeverage * ONE_BASIS_POINTS)),
        })
        .accounts({
          admin,
          mainState: this.mainState,
          globalState: this.globalState,
          systemProgram,
        })
        .instruction();
      const ixs = [
        web3.ComputeBudgetProgram.setComputeUnitLimit({ units: 300_000 }),
        ix,
      ];
      const txSignature = await this.sendTx(ixs, []);
      if (!txSignature) throw "failed to send tx";
      return { isPass: true, info: { txSignature } };
    } catch (initializeError) {
      log({ initializeError });
      return { isPass: false, info: "failed to process the input" };
    }
  }

  async start(input: {
    tokenName: string;
    tokenSymbol: string;
    tokenUri: string;
    solAmount: number;
  }): Promise<SendTxResult> {
    try {
      const { tokenName, tokenSymbol, tokenUri, solAmount } = input;
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

      const quoteTokenVault = getAssociatedTokenAddressSync(
        quoteTokenMint,
        this.vaultOwner,
        true
      );
      const adminQuoteAta = getAssociatedTokenAddressSync(
        quoteTokenMint,
        admin,
        true
      );
      
      const ix = await this.program.methods
        .start({
          solAmount: new BN(
            Math.trunc(solAmount * SOL_DECIMALS_HELPER)
          ),
          tokenName,
          tokenSymbol,
          tokenUri,
        })
        .accounts({
          admin,
          mainState: this.mainState,
          globalState: this.globalState,
          tokenVault: tokenVault,
          tokenVaultOwner: this.vaultOwner,
          associatedTokenProgram,
          mplProgram,
          systemProgram,
          sysvarRent,
          baseTokenProgram: tokenProgram,
          quoteTokenProgram: quoteTokenProgram,
          baseToken: token,
          quoteMint: quoteTokenMint,
          quoteVault: quoteTokenVault,
          adminQuoteAta: adminQuoteAta,
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

  // async updateMainState(input: {
  //   feeReceiver?: web3.PublicKey;
  //   admin?: web3.PublicKey;
  // }): Promise<SendTxResult> {
  //   try {
  //     const admin = this.provider.publicKey;
  //     const feeReceiver = input.feeReceiver ?? null;
  //     const newAdmin = input.admin ?? null;
  //     const ix = await this.program.methods
  //       .updateMainState({ feeReceiver, admin: newAdmin })
  //       .accounts({ admin, mainState: this.mainState })
  //       .instruction();
  //     const ixs = [
  //       web3.ComputeBudgetProgram.setComputeUnitLimit({ units: 100_000 }),
  //       ix,
  //     ];
  //     const updateTxRes = await this.sendTx(ixs);
  //     if (!updateTxRes) return { isPass: false, info: "failed to send tx" };
  //     return { isPass: true, info: { txSignature: updateTxRes } };
  //   } catch (updateMainStateError) {
  //     log({ updateMainStateError });
  //     return { isPass: false, info: "failed to process input" };
  //   }
  // }

  async buy(
    solAmount: number,
    debug: boolean = false
  ): Promise<SendTxResult> {
    try {
      const admin = this.provider.publicKey;
      const globalInfo = await this.getGlobalInfo();
      if (!globalInfo) throw "Failed to get global state info";
      const { token } = globalInfo;
      const mainStateInfo = await this.getMainStateInfo();
      if (!mainStateInfo) throw "Failed to get main state info";
      const { feeReceiver } = mainStateInfo;

      // Get the global state directly to access last_liquidation_date
      const globalState = await this.program.account.globalStats.fetch(this.globalState);
      const lastLiquidationDate = globalState.lastLiquidationDate;

      const rawSolAmount = Math.trunc(solAmount * SOL_DECIMALS_HELPER);
      const user = this.provider.publicKey;
      const userAta = getAssociatedTokenAddressSync(token, user);
      const tokenVault = getAssociatedTokenAddressSync(
        token,
        this.vaultOwner,
        true
      );
      
      // Calculate the midnight timestamp in seconds (Unix timestamp) as the program does
      const now = Math.floor(Date.now() / 1000); // Current time in seconds
      const midnightTimestamp = now - (now % SECONDS_IN_A_DAY);
      
      // Get the date strings correctly formatted
      // const currentDateString = getDateStringFromTimestamp(midnightTimestamp);
      const currentDateString = getCurrentDateString();
      const liquidationDateString = getDateStringFromTimestamp(Number(lastLiquidationDate));
      
      const ix = await this.program.methods
        .buy(new BN(rawSolAmount))
        .accounts({
          user,
          mainState: this.mainState,
          globalState: this.globalState,
          dailyState: web3.PublicKey.findProgramAddressSync(
            [Buffer.from("daily-stats"), Buffer.from(currentDateString)],
            this.programId
          )[0],
          userLoan: web3.PublicKey.findProgramAddressSync(
            [Buffer.from("user-loan"), user.toBuffer()],
            this.programId
          )[0],
          lastLiquidationDateState: web3.PublicKey.findProgramAddressSync(
            [Buffer.from("daily-stats"), Buffer.from(liquidationDateString)],
            this.programId
          )[0],
          feeReceiver,
          token,
          userAta,
          tokenVaultOwner: this.vaultOwner,
          tokenVault,
          associatedTokenProgram,
          tokenProgram,
          systemProgram,
        })
        .instruction();
      
      const ixs = [
        web3.ComputeBudgetProgram.setComputeUnitLimit({ units: 150_000 }),
        ix,
      ];
      
      const txSignature = await this.sendTx(ixs);
      if (!txSignature) throw "failed to send tx";
      return { isPass: true, info: { txSignature } };
    } catch (buyError) {
      log({ buyError });
      return { isPass: false, info: "failed to process input" };
    }
  }

  async buy_with_referral(
    solAmount: number,
    referralPubkey: web3.PublicKey
  ): Promise<SendTxResult> {
    try {
      const admin = this.provider.publicKey;
      const globalInfo = await this.getGlobalInfo();
      if (!globalInfo) throw "Failed to get global state info";
      const { token } = globalInfo;
      const mainStateInfo = await this.getMainStateInfo();
      if (!mainStateInfo) throw "Failed to get main state info";
      const { feeReceiver } = mainStateInfo;

      // Get the global state directly to access last_liquidation_date
      const globalState = await this.program.account.globalStats.fetch(this.globalState);
      const lastLiquidationDate = globalState.lastLiquidationDate;

      const rawSolAmount = Math.trunc(solAmount * SOL_DECIMALS_HELPER);
      const user = this.provider.publicKey;
      const userAta = getAssociatedTokenAddressSync(token, user);
      const tokenVault = getAssociatedTokenAddressSync(
        token,
        this.vaultOwner,
        true
      );
      
      // Calculate the midnight timestamp in seconds (Unix timestamp) as the program does
      const now = Math.floor(Date.now() / 1000); // Current time in seconds
      const midnightTimestamp = now - (now % SECONDS_IN_A_DAY);
      
      // Get the date strings correctly formatted
      const currentDateString = getDateStringFromTimestamp(midnightTimestamp);
      const liquidationDateString = getDateStringFromTimestamp(Number(lastLiquidationDate));
      
      const ix = await this.program.methods
        .buyWithReferral({
          solAmount: new BN(rawSolAmount),
          referralPubkey: referralPubkey
        })
        .accounts({
          common: {
            user,
            mainState: this.mainState,
            globalState: this.globalState,
            dailyState: web3.PublicKey.findProgramAddressSync(
              [Buffer.from("daily-stats"), Buffer.from(currentDateString)],
              this.programId
            )[0],
            userLoan: web3.PublicKey.findProgramAddressSync(
              [Buffer.from("user-loan"), user.toBuffer()],
              this.programId
            )[0],
            lastLiquidationDateState: web3.PublicKey.findProgramAddressSync(
              [Buffer.from("daily-stats"), Buffer.from(liquidationDateString)],
              this.programId
            )[0],
            feeReceiver,
            token,
            userAta,
            tokenVaultOwner: this.vaultOwner,
            tokenVault,
            associatedTokenProgram,
            tokenProgram,
            systemProgram,
          },
          referral: referralPubkey,
        })
        .instruction();
      
      const ixs = [
        web3.ComputeBudgetProgram.setComputeUnitLimit({ units: 150_000 }),
        ix,
      ];
      
      const txSignature = await this.sendTx(ixs);
      if (!txSignature) throw "failed to send tx";
      return { isPass: true, info: { txSignature } };
    } catch (buyError) {
      log({ buyError });
      return { isPass: false, info: "failed to process input" };
    }
  }

  async sell(
    tokenAmount: number,
    debug: boolean = false
  ): Promise<SendTxResult> {
    try {
      const admin = this.provider.publicKey;
      const globalInfo = await this.getGlobalInfo();
      if (!globalInfo) throw "Failed to get global state info";
      const { token } = globalInfo;
      const mainStateInfo = await this.getMainStateInfo();
      if (!mainStateInfo) throw "Failed to get main state info";
      const { feeReceiver } = mainStateInfo;

      // Get the global state directly to access last_liquidation_date
      const globalState = await this.program.account.globalStats.fetch(this.globalState);
      const lastLiquidationDate = globalState.lastLiquidationDate;

      const rawTokenAmount = Math.trunc(tokenAmount * TOKEN_DECIMALS_HELPER);
      const user = this.provider.publicKey;
      const userAta = getAssociatedTokenAddressSync(token, user);
      const tokenVault = getAssociatedTokenAddressSync(
        token,
        this.vaultOwner,
        true
      );
      
      // Calculate the midnight timestamp in seconds (Unix timestamp) as the program does
      const now = Math.floor(Date.now() / 1000); // Current time in seconds
      const midnightTimestamp = now - (now % SECONDS_IN_A_DAY);
      
      // Get the date strings correctly formatted
      const currentDateString = getDateStringFromTimestamp(midnightTimestamp);
      const liquidationDateString = getDateStringFromTimestamp(Number(lastLiquidationDate));
      
      // For debugging - print the date strings
      if (debug) {
        log({
          currentDate: currentDateString,
          liquidationDate: liquidationDateString,
          currentTimestamp: midnightTimestamp, 
          liquidationTimestamp: Number(lastLiquidationDate)
        });
      }
      
      const ix = await this.program.methods
        .sell(new BN(rawTokenAmount))
        .accounts({
          user,
          mainState: this.mainState,
          globalState: this.globalState,
          dailyState: web3.PublicKey.findProgramAddressSync(
            [Buffer.from("daily-stats"), Buffer.from(currentDateString)],
            this.programId
          )[0],
          lastLiquidationDateState: web3.PublicKey.findProgramAddressSync(
            [Buffer.from("daily-stats"), Buffer.from(liquidationDateString)],
            this.programId
          )[0],
          userLoan: web3.PublicKey.findProgramAddressSync(
            [Buffer.from("user-loan"), user.toBuffer()],
            this.programId
          )[0],
          feeReceiver,
          token,
          userAta,
          tokenVaultOwner: this.vaultOwner,
          tokenVault,
          associatedTokenProgram,
          tokenProgram,
          systemProgram,
        })
        .instruction();
      
      const ixs = [
        web3.ComputeBudgetProgram.setComputeUnitLimit({ units: 150_000 }),
        ix,
      ];
      
      const txSignature = await this.sendTx(ixs);
      if (!txSignature) throw "failed to send tx";
      return { isPass: true, info: { txSignature } };
    } catch (sellError) {
      log({ sellError });
      return { isPass: false, info: "failed to process input" };
    }
  }

  async borrow(
    solAmount: number,
    numberOfDays: number,
    debug: boolean = false
  ): Promise<SendTxResult> {
    try {
      const globalInfo = await this.getGlobalInfo();
      if (!globalInfo) throw "Failed to get global state info";
      const { token } = globalInfo;
      const mainStateInfo = await this.getMainStateInfo();
      if (!mainStateInfo) throw "Failed to get main state info";
      const { feeReceiver } = mainStateInfo;

      // Get the global state directly to access last_liquidation_date
      const globalState = await this.program.account.globalStats.fetch(this.globalState);
      const lastLiquidationDate = globalState.lastLiquidationDate;

      const rawSolAmount = Math.trunc(solAmount * SOL_DECIMALS_HELPER);
      const user = this.provider.publicKey;
      const userAta = getAssociatedTokenAddressSync(token, user);
      const tokenVault = getAssociatedTokenAddressSync(
        token,
        this.vaultOwner,
        true
      );
      
      // Calculate the midnight timestamp in seconds (Unix timestamp) as the program does
      const now = Math.floor(Date.now() / 1000); // Current time in seconds
      const midnightTimestamp = now - (now % SECONDS_IN_A_DAY);
      
      // Get the date strings correctly formatted
      const currentDateString = getDateStringFromTimestamp(midnightTimestamp);
      const liquidationDateString = getDateStringFromTimestamp(Number(lastLiquidationDate));
      const endDate = now + (numberOfDays * SECONDS_IN_A_DAY) + SECONDS_IN_A_DAY;
      const endDateString = getDateStringFromTimestamp(endDate);
      
      // For debugging - print the date strings
      if (debug) {
        log({
          currentDate: currentDateString,
          liquidationDate: liquidationDateString,
          currentTimestamp: midnightTimestamp, 
          liquidationTimestamp: Number(lastLiquidationDate),
          endDate: endDateString,
          endDateTimestamp: endDate,
        });
      }
      
      const ix = await this.program.methods
        .borrow(new BN(numberOfDays), new BN(rawSolAmount))
        .accounts({
          common: {
            user,
            mainState: this.mainState,
            globalState: this.globalState,
            dailyState: web3.PublicKey.findProgramAddressSync(
              [Buffer.from("daily-stats"), Buffer.from(currentDateString)],
              this.programId
            )[0],
            lastLiquidationDateState: web3.PublicKey.findProgramAddressSync(
              [Buffer.from("daily-stats"), Buffer.from(liquidationDateString)],
              this.programId
            )[0],
            
            userLoan: web3.PublicKey.findProgramAddressSync(
              [Buffer.from("user-loan"), user.toBuffer()],
              this.programId
            )[0],
            feeReceiver,
            token,
            userAta,
            tokenVaultOwner: this.vaultOwner,
            tokenVault,
            associatedTokenProgram,
            tokenProgram,
            systemProgram,
          },
          user,
          systemProgram,
          dailyStateEndDate: web3.PublicKey.findProgramAddressSync(
            [Buffer.from("daily-stats"), Buffer.from(endDateString)],
            this.programId
          )[0],
        })
        .instruction();
      
      const ixs = [
        web3.ComputeBudgetProgram.setComputeUnitLimit({ units: 150_000 }),
        ix,
      ];
      
      const txSignature = await this.sendTx(ixs);
      if (!txSignature) throw "failed to send tx";
      return { isPass: true, info: { txSignature } };
    } catch (borrowError) {
      log({ borrowError });
      return { isPass: false, info: "failed to borrow" };
    }
  }

  async leverage(
    solAmount: number,
    numberOfDays: number,
    debug: boolean = false
  ): Promise<SendTxResult> {
    try {
      const globalInfo = await this.getGlobalInfo();
      if (!globalInfo) throw "Failed to get global state info";
      const { token } = globalInfo;
      const mainStateInfo = await this.getMainStateInfo();
      if (!mainStateInfo) throw "Failed to get main state info";
      const { feeReceiver } = mainStateInfo;

      // Get the global state directly to access last_liquidation_date
      const globalState = await this.program.account.globalStats.fetch(this.globalState);
      const lastLiquidationDate = globalState.lastLiquidationDate;

      const rawSolAmount = Math.trunc(solAmount * SOL_DECIMALS_HELPER);
      const user = this.provider.publicKey;
      const userAta = getAssociatedTokenAddressSync(token, user);
      const tokenVault = getAssociatedTokenAddressSync(
        token,
        this.vaultOwner,
        true
      );
      
      // Calculate the midnight timestamp in seconds (Unix timestamp) as the program does
      const now = Math.floor(Date.now() / 1000); // Current time in seconds
      const midnightTimestamp = now - (now % SECONDS_IN_A_DAY);
      
      // Get the date strings correctly formatted
      const currentDateString = getDateStringFromTimestamp(midnightTimestamp);
      const liquidationDateString = getDateStringFromTimestamp(Number(lastLiquidationDate));
      
      const endDate = now + (numberOfDays * SECONDS_IN_A_DAY) + SECONDS_IN_A_DAY;
      const endDateString = getDateStringFromTimestamp(endDate);

      // For debugging - print the date strings
      if (debug) {
        log({
          currentDate: currentDateString,
          liquidationDate: liquidationDateString,
          currentTimestamp: midnightTimestamp, 
          liquidationTimestamp: Number(lastLiquidationDate)
        });
      }
      
      const ix = await this.program.methods
        .leverage(new BN(numberOfDays), new BN(rawSolAmount) )
        .accounts({
          common: {
            user,
            mainState: this.mainState,
            globalState: this.globalState,
            dailyState: web3.PublicKey.findProgramAddressSync(
            [Buffer.from("daily-stats"), Buffer.from(currentDateString)],
              this.programId
            )[0],
            lastLiquidationDateState: web3.PublicKey.findProgramAddressSync(
              [Buffer.from("daily-stats"), Buffer.from(liquidationDateString)],
              this.programId
            )[0],
            userLoan: web3.PublicKey.findProgramAddressSync(
              [Buffer.from("user-loan"), user.toBuffer()],
              this.programId
            )[0],
            feeReceiver,
            token,
            userAta,
            tokenVaultOwner: this.vaultOwner,
            tokenVault,
            associatedTokenProgram,
            tokenProgram,
            systemProgram,
          },
          user,
          systemProgram,
          dailyStateEndDate: web3.PublicKey.findProgramAddressSync(
            [Buffer.from("daily-stats"), Buffer.from(endDateString)],
            this.programId
          )[0],
        })
        .instruction();
      
      const txSignature = await this.sendTx([ix]);
      if (!txSignature) throw "failed to send tx";
      return { isPass: true, info: { txSignature } };
    } catch (leverageError) {
      log({ leverageError });
      return { isPass: false, info: "failed to leverage" };
    }
  }

  async repay(
    solAmount: number,
    debug: boolean = false
  ): Promise<SendTxResult> {
    try {
      const globalInfo = await this.getGlobalInfo();
      if (!globalInfo) throw "Failed to get global state info";
      const { token } = globalInfo;
      const mainStateInfo = await this.getMainStateInfo();
      if (!mainStateInfo) throw "Failed to get main state info";
      const { feeReceiver } = mainStateInfo;

      const userLoanInfo = await this.getUserLoanInfo(this.provider.publicKey);
      if (!userLoanInfo) throw "Failed to get user loan info";
      const { endDate } = userLoanInfo;

      // Get the global state directly to access last_liquidation_date
      const globalState = await this.program.account.globalStats.fetch(this.globalState);
      const lastLiquidationDate = globalState.lastLiquidationDate;

      const rawSolAmount = Math.trunc(solAmount * SOL_DECIMALS_HELPER);
      const user = this.provider.publicKey;
      const userAta = getAssociatedTokenAddressSync(token, user);
      const tokenVault = getAssociatedTokenAddressSync(
        token,
        this.vaultOwner,
        true
      );
      
      // Calculate the midnight timestamp in seconds (Unix timestamp) as the program does
      const now = Math.floor(Date.now() / 1000); // Current time in seconds
      const midnightTimestamp = now - (now % SECONDS_IN_A_DAY);
      
      // Get the date strings correctly formatted
      const currentDateString = getDateStringFromTimestamp(midnightTimestamp);
      const liquidationDateString = getDateStringFromTimestamp(Number(lastLiquidationDate));
      
      // For debugging - print the date strings
      if (debug) {
        log({
          currentDate: currentDateString,
          liquidationDate: liquidationDateString,
          currentTimestamp: midnightTimestamp, 
          liquidationTimestamp: Number(lastLiquidationDate)
        });
      }
      
      const ix = await this.program.methods
        .repay(new BN(rawSolAmount))
        .accounts({
          common: {
            user,
            mainState: this.mainState,
            globalState: this.globalState,
            dailyState: web3.PublicKey.findProgramAddressSync(
            [Buffer.from("daily-stats"), Buffer.from(currentDateString)],
              this.programId
            )[0],
            lastLiquidationDateState: web3.PublicKey.findProgramAddressSync(
              [Buffer.from("daily-stats"), Buffer.from(liquidationDateString)],
              this.programId
            )[0],
            userLoan: web3.PublicKey.findProgramAddressSync(
              [Buffer.from("user-loan"), user.toBuffer()],
              this.programId
            )[0],
            feeReceiver,
            token,
            userAta,
            tokenVaultOwner: this.vaultOwner,
            tokenVault,
            associatedTokenProgram,
            tokenProgram,
            systemProgram,
          },
          dailyStateOldEndDate: web3.PublicKey.findProgramAddressSync(
            [Buffer.from("daily-stats"), Buffer.from(getDateStringFromTimestamp(Number(endDate)))],
            this.programId
          )[0],
        })
        .instruction();
      const ixs = [
        web3.ComputeBudgetProgram.setComputeUnitLimit({ units: 150_000 }),
        ix,
      ];
      
      const txSignature = await this.sendTx(ixs);
      if (!txSignature) throw "failed to send tx";
      return { isPass: true, info: { txSignature } };
    } catch (repayError) {
      log({ repayError });
      return { isPass: false, info: "failed to repay" };
    }
  }

  async remove_collateral(
    amount: number,
    debug: boolean = false
  ): Promise<SendTxResult> {
    try {
      const globalInfo = await this.getGlobalInfo();
      if (!globalInfo) throw "Failed to get global state info";
      const { token } = globalInfo;
      const mainStateInfo = await this.getMainStateInfo();
      if (!mainStateInfo) throw "Failed to get main state info";
      const { feeReceiver } = mainStateInfo;

      const userLoanInfo = await this.getUserLoanInfo(this.provider.publicKey);
      if (!userLoanInfo) throw "Failed to get user loan info";
      const { endDate } = userLoanInfo;
      
      // Get the global state directly to access last_liquidation_date
      const globalState = await this.program.account.globalStats.fetch(this.globalState);
      const lastLiquidationDate = globalState.lastLiquidationDate;

      const rawAmount = Math.trunc(amount * TOKEN_DECIMALS_HELPER);
      const user = this.provider.publicKey;
      const userAta = getAssociatedTokenAddressSync(token, user);
      const tokenVault = getAssociatedTokenAddressSync(
        token,
        this.vaultOwner,
        true
      );
      
      // Calculate the midnight timestamp in seconds (Unix timestamp) as the program does
      const now = Math.floor(Date.now() / 1000); // Current time in seconds
      const midnightTimestamp = now - (now % SECONDS_IN_A_DAY);
      
      // Get the date strings correctly formatted
      const currentDateString = getDateStringFromTimestamp(midnightTimestamp);
      const liquidationDateString = getDateStringFromTimestamp(Number(lastLiquidationDate));
      
      // For debugging - print the date strings
      if (debug) {
        log({
          currentDate: currentDateString,
          liquidationDate: liquidationDateString,
          currentTimestamp: midnightTimestamp, 
          liquidationTimestamp: Number(lastLiquidationDate)
        });
      }
      
      const ix = await this.program.methods
        .removeCollateral(new BN(rawAmount))
        .accounts({
          common: {
            user,
            mainState: this.mainState,
            globalState: this.globalState,
            dailyState: web3.PublicKey.findProgramAddressSync(
            [Buffer.from("daily-stats"), Buffer.from(currentDateString)],
              this.programId
            )[0],
            lastLiquidationDateState: web3.PublicKey.findProgramAddressSync(
              [Buffer.from("daily-stats"), Buffer.from(liquidationDateString)],
              this.programId
            )[0],
            userLoan: web3.PublicKey.findProgramAddressSync(
              [Buffer.from("user-loan"), user.toBuffer()],
              this.programId
            )[0],
            feeReceiver,
            token,
            userAta,
            tokenVaultOwner: this.vaultOwner,
            tokenVault,
            associatedTokenProgram,
            tokenProgram,
            systemProgram,
          },
          dailyStateOldEndDate: web3.PublicKey.findProgramAddressSync(
            [Buffer.from("daily-stats"), Buffer.from(getDateStringFromTimestamp(Number(endDate)))],
            this.programId
          )[0],
        })
        .instruction();
      
      const ixs = [
        web3.ComputeBudgetProgram.setComputeUnitLimit({ units: 150_000 }),
        ix,
      ];
      
      const txSignature = await this.sendTx(ixs);
      if (!txSignature) throw "failed to send tx";
      return { isPass: true, info: { txSignature } };
    } catch (removeCollateralError) {
      log({ removeCollateralError });
      return { isPass: false, info: "failed to remove collateral" };
    }
  }

  async close_position(
    solAmount: number,
    debug: boolean = false
  ): Promise<SendTxResult> {
    try {
      const globalInfo = await this.getGlobalInfo();
      if (!globalInfo) throw "Failed to get global state info";
      const { token } = globalInfo;
      const mainStateInfo = await this.getMainStateInfo();
      if (!mainStateInfo) throw "Failed to get main state info";
      const { feeReceiver } = mainStateInfo;

      const userLoanInfo = await this.getUserLoanInfo(this.provider.publicKey);
      if (!userLoanInfo) throw "Failed to get user loan info";
      const { endDate } = userLoanInfo;

      // Get the global state directly to access last_liquidation_date
      const globalState = await this.program.account.globalStats.fetch(this.globalState);
      const lastLiquidationDate = globalState.lastLiquidationDate;

      const rawSolAmount = Math.trunc(solAmount * SOL_DECIMALS_HELPER);
      const user = this.provider.publicKey;
      const userAta = getAssociatedTokenAddressSync(token, user);
      const tokenVault = getAssociatedTokenAddressSync(
        token,
        this.vaultOwner,
        true
      );
      
      // Calculate the midnight timestamp in seconds (Unix timestamp) as the program does
      const now = Math.floor(Date.now() / 1000); // Current time in seconds
      const midnightTimestamp = now - (now % SECONDS_IN_A_DAY);
      
      // Get the date strings correctly formatted
      const currentDateString = getDateStringFromTimestamp(midnightTimestamp);
      const liquidationDateString = getDateStringFromTimestamp(Number(lastLiquidationDate));
      
      // For debugging - print the date strings
      if (debug) {
        log({
          currentDate: currentDateString,
          liquidationDate: liquidationDateString,
          currentTimestamp: midnightTimestamp, 
          liquidationTimestamp: Number(lastLiquidationDate)
        });
      }
      
      const ix = await this.program.methods
        .closePosition(new BN(rawSolAmount))
        .accounts({
          common: {
            user,
            mainState: this.mainState,
            globalState: this.globalState,
            dailyState: web3.PublicKey.findProgramAddressSync(
            [Buffer.from("daily-stats"), Buffer.from(currentDateString)],
              this.programId
            )[0],
            lastLiquidationDateState: web3.PublicKey.findProgramAddressSync(
              [Buffer.from("daily-stats"), Buffer.from(liquidationDateString)],
              this.programId
            )[0],
            userLoan: web3.PublicKey.findProgramAddressSync(
              [Buffer.from("user-loan"), user.toBuffer()],
              this.programId
            )[0],
            feeReceiver,
            token,
            userAta,
            tokenVaultOwner: this.vaultOwner,
            tokenVault,
            associatedTokenProgram,
            tokenProgram,
            systemProgram,
          },
          dailyStateOldEndDate: web3.PublicKey.findProgramAddressSync(
            [Buffer.from("daily-stats"), Buffer.from(getDateStringFromTimestamp(Number(endDate)))],
            this.programId
          )[0],
        })
        .instruction();
      
      const ixs = [
        web3.ComputeBudgetProgram.setComputeUnitLimit({ units: 150_000 }),
        ix,
      ];
      
      const txSignature = await this.sendTx(ixs);
      if (!txSignature) throw "failed to send tx";
      return { isPass: true, info: { txSignature } };
    } catch (closePositionError) {
      log({ closePositionError });
      return { isPass: false, info: "failed to close position" };
    }
  }

  async flash_close_position(
    debug: boolean = false
  ): Promise<SendTxResult> {
    try {
      const globalInfo = await this.getGlobalInfo();
      if (!globalInfo) throw "Failed to get global state info";
      const { token } = globalInfo;
      const mainStateInfo = await this.getMainStateInfo();
      if (!mainStateInfo) throw "Failed to get main state info";
      const { feeReceiver } = mainStateInfo;

      const userLoanInfo = await this.getUserLoanInfo(this.provider.publicKey);
      if (!userLoanInfo) throw "Failed to get user loan info";
      const { endDate } = userLoanInfo;

      // Get the global state directly to access last_liquidation_date
      const globalState = await this.program.account.globalStats.fetch(this.globalState);
      const lastLiquidationDate = globalState.lastLiquidationDate;

      const user = this.provider.publicKey;
      const userAta = getAssociatedTokenAddressSync(token, user);
      const tokenVault = getAssociatedTokenAddressSync(
        token,
        this.vaultOwner,
        true
      );
      
      // Calculate the midnight timestamp in seconds (Unix timestamp) as the program does
      const now = Math.floor(Date.now() / 1000); // Current time in seconds
      const midnightTimestamp = now - (now % SECONDS_IN_A_DAY);
      
      // Get the date strings correctly formatted
      const currentDateString = getDateStringFromTimestamp(midnightTimestamp);
      const liquidationDateString = getDateStringFromTimestamp(Number(lastLiquidationDate));
      
      // For debugging - print the date strings
      if (debug) {
        log({
          currentDate: currentDateString,
          liquidationDate: liquidationDateString,
          currentTimestamp: midnightTimestamp, 
          liquidationTimestamp: Number(lastLiquidationDate)
        });
      }
      
      const ix = await this.program.methods
        .flashClosePosition()
        .accounts({
          common: {
            user,
            mainState: this.mainState,
            globalState: this.globalState,
            dailyState: web3.PublicKey.findProgramAddressSync(
            [Buffer.from("daily-stats"), Buffer.from(currentDateString)],
            this.programId
            )[0],
            lastLiquidationDateState: web3.PublicKey.findProgramAddressSync(
              [Buffer.from("daily-stats"), Buffer.from(liquidationDateString)],
              this.programId
            )[0],
            userLoan: web3.PublicKey.findProgramAddressSync(
              [Buffer.from("user-loan"), user.toBuffer()],
              this.programId
            )[0],
            feeReceiver,
            token,
            userAta,
            tokenVaultOwner: this.vaultOwner,
            tokenVault,
            associatedTokenProgram,
            tokenProgram,
            systemProgram,
          },
          dailyStateOldEndDate: web3.PublicKey.findProgramAddressSync(
            [Buffer.from("daily-stats"), Buffer.from(getDateStringFromTimestamp(Number(endDate)))],
            this.programId
          )[0],
        })
        .instruction();
      
      const ixs = [
        web3.ComputeBudgetProgram.setComputeUnitLimit({ units: 150_000 }),
        ix,
      ];
      
      const txSignature = await this.sendTx(ixs);
      if (!txSignature) throw "failed to send tx";
      return { isPass: true, info: { txSignature } };
    } catch (flashClosePositionError) {
      log({ flashClosePositionError });
      return { isPass: false, info: "failed to flash close position" };
    }
  }

  async extend_loan(
    numberOfDays: number,
    debug: boolean = false
  ): Promise<SendTxResult> {
    try {
      const globalInfo = await this.getGlobalInfo();
      if (!globalInfo) throw "Failed to get global state info";
      const { token } = globalInfo;
      const mainStateInfo = await this.getMainStateInfo();
      if (!mainStateInfo) throw "Failed to get main state info";
      const { feeReceiver } = mainStateInfo;

      const userLoanInfo = await this.getUserLoanInfo(this.provider.publicKey);
      if (!userLoanInfo) throw "Failed to get user loan info";
      const { endDate } = userLoanInfo;

      // Get the global state directly to access last_liquidation_date
      const globalState = await this.program.account.globalStats.fetch(this.globalState);
      const lastLiquidationDate = globalState.lastLiquidationDate;

      const user = this.provider.publicKey;
      const userAta = getAssociatedTokenAddressSync(token, user);
      const tokenVault = getAssociatedTokenAddressSync(
        token,
        this.vaultOwner,
        true
      );
      
      // Calculate the midnight timestamp in seconds (Unix timestamp) as the program does
      const now = Math.floor(Date.now() / 1000); // Current time in seconds
      const midnightTimestamp = now - (now % SECONDS_IN_A_DAY);
      
      // Get the date strings correctly formatted
      const currentDateString = getDateStringFromTimestamp(midnightTimestamp);
      const liquidationDateString = getDateStringFromTimestamp(Number(lastLiquidationDate));
      
      const newEndDate = Number(endDate) + ((numberOfDays) * SECONDS_IN_A_DAY);
      const newEndDateString = getDateStringFromTimestamp(newEndDate);
      // For debugging - print the date strings
      if (debug) {
        log({
          currentDate: currentDateString,
          liquidationDate: liquidationDateString,
          currentTimestamp: midnightTimestamp, 
          liquidationTimestamp: Number(lastLiquidationDate)
        });
      }
      
      const ix = await this.program.methods
        .extendLoan(new BN(numberOfDays))
        .accounts({ 
          common: {
            user,
            mainState: this.mainState,
            globalState: this.globalState,
            dailyState: web3.PublicKey.findProgramAddressSync(
            [Buffer.from("daily-stats"), Buffer.from(currentDateString)],
            this.programId
            )[0],
            lastLiquidationDateState: web3.PublicKey.findProgramAddressSync(
              [Buffer.from("daily-stats"), Buffer.from(liquidationDateString)],
              this.programId
            )[0],
            
            userLoan: web3.PublicKey.findProgramAddressSync(
              [Buffer.from("user-loan"), user.toBuffer()],
              this.programId
            )[0],
            feeReceiver,
            token,
            userAta,
            tokenVaultOwner: this.vaultOwner,
            tokenVault,
            associatedTokenProgram,
            tokenProgram,
            systemProgram,
          },
          user,
          systemProgram,
          dailyStateOldEndDate: web3.PublicKey.findProgramAddressSync(
            [Buffer.from("daily-stats"), Buffer.from(getDateStringFromTimestamp(Number(endDate)))],
            this.programId
          )[0],
          dailyStateNewEndDate: web3.PublicKey.findProgramAddressSync(
            [Buffer.from("daily-stats"), Buffer.from(newEndDateString)],
            this.programId
          )[0],
        })
        .instruction();
      
      const ixs = [
        web3.ComputeBudgetProgram.setComputeUnitLimit({ units: 150_000 }),
        ix,
      ];
      
      const txSignature = await this.sendTx(ixs);
      if (!txSignature) throw "failed to send tx";
      return { isPass: true, info: { txSignature } };
    } catch (extendLoanError) {
      log({ extendLoanError });
      return { isPass: false, info: "failed to extend loan" };
    }
  }

  async borrow_more(
    solAmount: number,
    debug: boolean = false
  ): Promise<SendTxResult> {
    try {
      const globalInfo = await this.getGlobalInfo();
      if (!globalInfo) throw "Failed to get global state info";
      const { token } = globalInfo;
      const mainStateInfo = await this.getMainStateInfo();
      if (!mainStateInfo) throw "Failed to get main state info";
      const { feeReceiver } = mainStateInfo;

      const userLoanInfo = await this.getUserLoanInfo(this.provider.publicKey);
      if (!userLoanInfo) throw "Failed to get user loan info";
      const { endDate } = userLoanInfo;

      // Get the global state directly to access last_liquidation_date
      const globalState = await this.program.account.globalStats.fetch(this.globalState);
      const lastLiquidationDate = globalState.lastLiquidationDate;

      const rawSolAmount = Math.trunc(solAmount * SOL_DECIMALS_HELPER);
      const user = this.provider.publicKey;
      const userAta = getAssociatedTokenAddressSync(token, user);
      const tokenVault = getAssociatedTokenAddressSync(
        token,
        this.vaultOwner,
        true
      );
      
      // Calculate the midnight timestamp in seconds (Unix timestamp) as the program does
      const now = Math.floor(Date.now() / 1000); // Current time in seconds
      const midnightTimestamp = now - (now % SECONDS_IN_A_DAY);
      
      // Get the date strings correctly formatted
      const currentDateString = getDateStringFromTimestamp(midnightTimestamp);
      const liquidationDateString = getDateStringFromTimestamp(Number(lastLiquidationDate));
      
      // For debugging - print the date strings
      if (debug) {
        log({
          currentDate: currentDateString,
          liquidationDate: liquidationDateString,
          currentTimestamp: midnightTimestamp, 
          liquidationTimestamp: Number(lastLiquidationDate)
        });
      }
      
      const ix = await this.program.methods
        .borrowMore(new BN(rawSolAmount))
        .accounts({
          common: {
            user,
            mainState: this.mainState,
            globalState: this.globalState,
            dailyState: web3.PublicKey.findProgramAddressSync(
              [Buffer.from("daily-stats"), Buffer.from(currentDateString)],
              this.programId
            )[0],
            lastLiquidationDateState: web3.PublicKey.findProgramAddressSync(
              [Buffer.from("daily-stats"), Buffer.from(liquidationDateString)],
              this.programId
            )[0],
            userLoan: web3.PublicKey.findProgramAddressSync(
              [Buffer.from("user-loan"), user.toBuffer()],
              this.programId
            )[0],
            feeReceiver,
            token,
            userAta,
            tokenVaultOwner: this.vaultOwner,
            tokenVault,
            associatedTokenProgram,
            tokenProgram,
            systemProgram,
          },
          dailyStateOldEndDate: web3.PublicKey.findProgramAddressSync(
            [Buffer.from("daily-stats"), Buffer.from(getDateStringFromTimestamp(Number(endDate)))],
            this.programId
          )[0],
        })
        .instruction();
      
      const ixs = [
        web3.ComputeBudgetProgram.setComputeUnitLimit({ units: 150_000 }),
        ix,
      ];
      
      const txSignature = await this.sendTx(ixs);
      if (!txSignature) throw "failed to send tx";
      return { isPass: true, info: { txSignature } };
    } catch (borrowMoreError) {
      log({ borrowMoreError });
      return { isPass: false, info: "failed to borrow more" };
    }
  }
}

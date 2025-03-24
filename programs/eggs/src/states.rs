use anchor_lang::prelude::*;

// Constants from the Solidity contract
pub const FEE_BASE_1000: u16 = 1000;
pub const MIN: u64 = 1000;
pub const FEES_BUY: u16 = 125;
pub const FEES_SELL: u16 = 125;
pub const SONIC_DECIMALS: u8 = 9;  // Solana's native token (SOL) has 9 decimals
pub const EGGS_DECIMALS: u8 = 9;
pub const MAX_SUPPLY: u128 = 10_000_000_000_000_000_000_000_000_000;  // 10e28 in Solidity

#[account]
pub struct EggsState {
    pub fee_address: Pubkey,
    pub sell_fee: u16,
    pub buy_fee: u16,
    pub buy_fee_leverage: u16,
    pub start: bool,
    pub total_borrowed: u64,
    pub total_collateral: u64,
    pub total_minted: u64,
    pub last_price: u64,
    pub last_liquidation_date: i64,
    // The mint address for the EGGS token
    pub mint: Pubkey,
    // The authority that can update program parameters
    pub authority: Pubkey,
    // The bump seed for PDA derivation
    pub bump: u8,
}

impl EggsState {
    pub const LEN: usize = 8 + // discriminator
                          32 + // fee_address
                          2 +  // sell_fee
                          2 +  // buy_fee
                          2 +  // buy_fee_leverage
                          1 +  // start
                          8 +  // total_borrowed
                          8 +  // total_collateral
                          8 +  // total_minted
                          8 +  // last_price
                          8 +  // last_liquidation_date
                          32 + // mint address
                          32 + // authority
                          1;   // bump
}

#[account]
pub struct Loan {
    pub user: Pubkey,          // The borrower's address
    pub collateral: u64,       // Amount of collateral
    pub borrowed: u64,         // Amount borrowed
    pub end_date: i64,         // End date as timestamp
    pub number_of_days: u64,   // Duration in days
}

impl Loan {
    pub const LEN: usize = 8 + // discriminator
                          32 + // user
                          8 +  // collateral
                          8 +  // borrowed
                          8 +  // end_date
                          8;   // number_of_days
}

#[account]
pub struct DailyLoanData {
    pub date: i64,            // The date this data represents (midnight timestamp)
    pub borrowed: u64,        // Total amount borrowed for this date
    pub collateral: u64,      // Total collateral for this date
}

impl DailyLoanData {
    pub const LEN: usize = 8 + // discriminator
                          8 +  // date
                          8 +  // borrowed
                          8;   // collateral
} 
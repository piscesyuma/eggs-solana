use anchor_lang::prelude::*;

use crate::{
    constants::{FEE_BASE_1000, SECONDS_IN_A_DAY},
    error::MushiProgramError,
};

#[account]
pub struct MainState {
    pub admin: Pubkey,
    pub fee_receiver: Pubkey,
    pub buy_fee: u64,
    pub sell_fee: u64,
    pub buy_fee_leverage: u64,
    pub quote_token: Pubkey,
    pub stake_token: Pubkey,
    pub stake_vault_program: Pubkey,
    pub stake_enabled: bool,
    pub started: bool,
}

impl MainState {
    pub const PREFIX_SEED: &'static [u8] = b"main_state";
    pub const MAX_SIZE: usize = std::mem::size_of::<Self>();
}

#[account]
pub struct UserLoan {
    pub collateral: u64,
    pub borrowed: u64,
    pub end_date: i64,
    pub number_of_days: u64,
}

impl UserLoan {
    pub const PREFIX_SEED: &'static [u8] = b"user_loan";
    pub const MAX_SIZE: usize = std::mem::size_of::<Self>();
}

#[account]
pub struct DailyStats {
    pub date: i64,
    pub borrowed: u64,
    pub collateral: u64,
}

impl DailyStats {
    pub const PREFIX_SEED: &'static [u8] = b"daily_stats";
    pub const MAX_SIZE: usize = std::mem::size_of::<Self>();
}

#[account]
pub struct GlobalStats {
    pub last_liquidation_date: i64,
    pub total_borrowed: u64,
    pub total_collateral: u64,
    pub total_eclipse_token_staked: u64,
    pub token_supply: u64,
    pub last_price: u64,
    pub base_token: Pubkey,
}

impl GlobalStats {
    pub const PREFIX_SEED: &'static [u8] = b"global_stats";
    pub const MAX_SIZE: usize = std::mem::size_of::<Self>();
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct DailyStatsResult {
    pub date: i64,
    pub borrowed: u64,
    pub collateral: u64,
}

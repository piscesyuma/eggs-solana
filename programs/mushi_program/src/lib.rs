#![allow(unused)]

use anchor_lang::prelude::*;

pub mod context;
use context::*;
pub mod ixs;
use ixs::*;

pub mod state;
use state::*;

mod constants;
mod error;
mod utils;

declare_id!("HF5x1bCgynzEnBL7ATMFYPNFjBaqfxgMASyUJL2ud6Xi");

#[program]
pub mod mushi_program {
    use super::*;

    pub fn init_main_state(ctx: Context<AInitializeState>, input: InitializeInput) -> Result<()> {
        ixs::init_main_state(ctx, input)
    }

    pub fn update_main_state(
        ctx: Context<AUpdateMainState>,
        input: UpdateMainStateInput,
    ) -> Result<()> {
        ixs::update_main_state(ctx, input)
    }

    pub fn start(ctx: Context<AStart>, input: StartInput) -> Result<()> {
        ixs::start(ctx, input)
    }

    pub fn buy(ctx: Context<ACommon>, es_amount: u64) -> Result<()> {
        ixs::buy(ctx, es_amount)
    }

    pub fn buy_with_referral(ctx: Context<ACommonExtReferral>, referral_address: Pubkey, es_amount: u64) -> Result<()> {
        ixs::buy_with_referral(ctx, referral_address, es_amount)
    }

    pub fn sell(ctx: Context<ACommon>, token_amount: u64) -> Result<()> {
        ixs::sell(ctx, token_amount)
    }

    pub fn borrow(ctx: Context<ACommonExtLoan>, number_of_days: u64, es_amount: u64) -> Result<()> {
        ixs::borrow(ctx, number_of_days, es_amount)
    }

    pub fn borrow_more(ctx: Context<ACommonExtSubLoan>, es_amount: u64) -> Result<()> {
        ixs::borrow_more(ctx, es_amount)
    }

    pub fn repay(ctx: Context<ACommonExtSubLoan>, es_amount: u64) -> Result<()> {
        ixs::repay(ctx, es_amount)
    }

    pub fn leverage(ctx: Context<ACommonExtLoan>, number_of_days: u64, es_amount: u64) -> Result<()> {
        ixs::leverage(ctx, number_of_days, es_amount)
    }

    pub fn remove_collateral(ctx: Context<ACommonExtSubLoan>, amount: u64) -> Result<()> {
        ixs::remove_collateral(ctx, amount)
    }

    pub fn extend_loan(ctx: Context<ACommonExtExtendLoan>, number_of_days: u64) -> Result<()> {
        ixs::extend_loan(ctx, number_of_days)
    }
    
    pub fn close_position(ctx: Context<ACommonExtSubLoan>, es_amount: u64) -> Result<()> {
        ixs::close_position(ctx, es_amount)
    }

    pub fn flash_close_position(ctx: Context<ACommonExtSubLoan>) -> Result<()> {
        ixs::flash_close_position(ctx)
    }

    pub fn stake(ctx: Context<Stake>, amount: u64) -> Result<()> {
        ixs::stake(ctx, amount)
    }

    pub fn unstake(ctx: Context<Unstake>, amount: u64) -> Result<()> {
        ixs::unstake(ctx, amount)
    }

    pub fn liquidate(ctx: Context<ACommon>) -> Result<()> {
        ixs::liquidate_expired_loans(ctx)
    }
}

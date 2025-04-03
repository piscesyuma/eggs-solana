#![allow(unused)]

use anchor_lang::prelude::*;

pub mod ixs;
use ixs::*;

pub mod state;
use state::*;

mod constants;
mod error;
mod utils;

declare_id!("65zNCEhvCtWo6DcphN6omP5Cz3hFo6zjUkHZfEauMDXr");

#[program]
pub mod mushi_program {
    use super::*;

    pub fn initialize(ctx: Context<AInitialize>, input: InitializeInput) -> Result<()> {
        ixs::initialize(ctx, input)
    }

    pub fn update_main_state(
        ctx: Context<AUpdateMainState>,
        input: UpdateMainStateInput,
    ) -> Result<()> {
        ixs::update_main_state(ctx, input)
    }

    pub fn buy(ctx: Context<ABuySell>, sol_amount_in: u64) -> Result<()> {
        ixs::buy(ctx, sol_amount_in)
    }

    pub fn sell(ctx: Context<ABuySell>, token_amount: u64) -> Result<()> {
        ixs::sell(ctx, token_amount)
    }

    pub fn borrow(ctx: Context<ABorrow>, sol_amount: u64, number_of_days: u64) -> Result<()> {
        ixs::borrow(ctx, sol_amount, number_of_days)
    }

    pub fn borrow_more(ctx: Context<ABorrow>, sol_amount: u64) -> Result<()> {
        ixs::borrow_more(ctx, sol_amount)
    }

    pub fn close_position(ctx: Context<ABorrow>, sol_amount: u64) -> Result<()> {
        ixs::close_position(ctx, sol_amount)
    }

    pub fn flash_close_position(ctx: Context<ABorrow>) -> Result<()> {
        ixs::flash_close_position(ctx)
    }

    pub fn remove_collateral(ctx: Context<ABorrow>, amount: u64) -> Result<()> {
        ixs::remove_collateral(ctx, amount)
    }

    pub fn extend_loan(ctx: Context<ABorrow>, sol_amount: u64, number_of_days: u64) -> Result<u64> {
        ixs::extend_loan(ctx, sol_amount, number_of_days)
    }

    pub fn repay(ctx: Context<ABorrow>, sol_amount: u64) -> Result<()> {
        ixs::repay(ctx, sol_amount)
    }

    pub fn leverage(ctx: Context<ALeverage>, sol_amount_in: u64, sol_amount:u64, number_of_days: u64)->Result<()>{
        ixs::leverage(ctx, sol_amount_in, sol_amount, number_of_days)
    }
}

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

    pub fn buy(ctx: Context<ACommon>, sol_amount: u64) -> Result<()> {
        ixs::buy(ctx, sol_amount)
    }

    pub fn buy_with_referral(ctx: Context<ACommon>, input: BuyWithReferralInput) -> Result<()> {
        ixs::buy_with_referral(ctx, input)
    }

    pub fn sell(ctx: Context<ACommon>, token_amount: u64) -> Result<()> {
        ixs::sell(ctx, token_amount)
    }
}

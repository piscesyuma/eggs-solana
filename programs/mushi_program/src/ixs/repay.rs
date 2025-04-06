use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token_interface};

use crate::{
    constants::{
        FEES_BUY, FEES_SELL, FEE_BASE_1000, MIN, SECONDS_IN_A_DAY, VAULT_SEED
    }, 
    error::MushiProgramError, 
    utils::{
        burn_tokens, 
        get_interest_fee, 
        get_midnight_timestamp, 
        liquidate, 
        mint_to_tokens_by_main_state, 
        transfer_tokens, 
        transfer_sol
    },
};
use crate::context::common::ACommon;

pub fn repay(ctx:Context<ACommon>, sol_amount: u64)->Result<()>{
    let user_loan = & ctx.accounts.user_loan;
    let borrowed = user_loan.borrowed;
    if borrowed <= sol_amount {
        return Err(MushiProgramError::InvalidSolAmount.into());
    }
    if sol_amount == 0 {
        return Err(MushiProgramError::InvalidSolAmount.into());
    }
    ctx.accounts.sub_loans_by_date(sol_amount, 0, user_loan.end_date)?;
    let new_borrow = borrowed - sol_amount;
    let user_loan = &mut ctx.accounts.user_loan;
    user_loan.borrowed = new_borrow;
    ctx.accounts.safety_check()?;
    Ok(())
}
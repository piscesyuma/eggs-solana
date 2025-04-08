use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token_interface};

use crate::{
    constants::{
        FEES_BUY, FEES_SELL, FEE_BASE_1000, MIN, SECONDS_IN_A_DAY, VAULT_SEED
    }, context::{ACommonExtLoan, ACommonExtSubLoan}, error::MushiProgramError, utils::{
        burn_tokens, get_interest_fee, get_midnight_timestamp, liquidate, mint_to_tokens_by_main_state, sub_loans_by_date, transfer_sol, transfer_tokens
    }
};
use crate::context::common::ACommon;

pub fn repay(ctx:Context<ACommonExtSubLoan>, sol_amount: u64)->Result<()>{
    let user_loan = & ctx.accounts.common.user_loan;
    let borrowed = user_loan.borrowed;
    require!(borrowed > sol_amount, MushiProgramError::InvalidSolAmount);
    require!(sol_amount != 0, MushiProgramError::InvalidSolAmount);

    transfer_sol(
        ctx.accounts.common.user.to_account_info(), 
        ctx.accounts.common.token_vault_owner.to_account_info(), 
        ctx.accounts.common.system_program.to_account_info(), 
        sol_amount, 
        None)?;

    sub_loans_by_date(&mut ctx.accounts.common.global_state, &mut ctx.accounts.daily_state_old_end_date, sol_amount, 0)?;
    let new_borrow = borrowed - sol_amount;
    let user_loan = &mut ctx.accounts.common.user_loan;
    user_loan.borrowed = new_borrow;
    ctx.accounts.common.safety_check()?;
    Ok(())
}
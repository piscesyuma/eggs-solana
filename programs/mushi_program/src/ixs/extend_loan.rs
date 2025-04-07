use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token_interface};

use crate::{
    constants::{
        FEES_BUY, FEES_SELL, FEE_BASE_1000, MIN, SECONDS_IN_A_DAY, VAULT_SEED
    }, context::{ACommonExtLoan, ACommonExtLoan2}, error::MushiProgramError, utils::{
        burn_tokens, get_interest_fee, get_midnight_timestamp, liquidate, mint_to_tokens_by_main_state, transfer_sol, transfer_tokens
    }
};
use crate::context::common::ACommon;

pub fn extend_loan(ctx:Context<ACommonExtLoan2>, number_of_days: u64, sol_amount: u64)->Result<()>{
    let user_loan = & ctx.accounts.common.user_loan;
    let old_end_date = user_loan.end_date;
    let _number_of_days = user_loan.number_of_days;
    let borrowed = user_loan.borrowed;
    let collateral = user_loan.collateral;

    let new_end_date = old_end_date + number_of_days as i64 * SECONDS_IN_A_DAY;
    let loan_fee = get_interest_fee(borrowed, number_of_days);
    
    require!(!ctx.accounts.common.is_loan_expired()?, MushiProgramError::LoanExpired);
    require!(loan_fee == sol_amount, MushiProgramError::InvalidSolAmount);

    let fee_address_fee = sol_amount.checked_mul(3).unwrap().checked_div(10).unwrap();
    require!(fee_address_fee > MIN, MushiProgramError::InvalidFeeAmount);

    transfer_sol(
        ctx.accounts.common.user.to_account_info(), 
        ctx.accounts.common.token_vault_owner.to_account_info(), 
        ctx.accounts.common.system_program.to_account_info(), 
        sol_amount, 
        None)?;

    let signer_seeds:&[&[&[u8]]] = &[&[VAULT_SEED, &[*ctx.bumps.get("token_vault_owner").unwrap()]]];
    transfer_sol(
        ctx.accounts.common.token_vault_owner.to_account_info(), 
        ctx.accounts.common.fee_receiver.to_account_info(), 
        ctx.accounts.common.system_program.to_account_info(), 
        fee_address_fee, 
        Some(signer_seeds))?;
    ctx.accounts.sub_loans_by_date(borrowed, collateral, old_end_date)?;
    ctx.accounts.add_loans_by_date(borrowed, collateral)?;
    let user_loan = &mut ctx.accounts.common.user_loan;
    user_loan.end_date = new_end_date;
    user_loan.number_of_days = number_of_days + _number_of_days;

    let current_timestamp = Clock::get()?.unix_timestamp;
    if (new_end_date - current_timestamp) / SECONDS_IN_A_DAY >= 366 {
        return Err(MushiProgramError::InvalidNumberOfDays.into());
    }
    ctx.accounts.common.safety_check()?;
    
    Ok(())
}
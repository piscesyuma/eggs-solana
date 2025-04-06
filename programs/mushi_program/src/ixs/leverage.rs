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

pub fn leverage(ctx:Context<ACommon>, sol_amount:u64, number_of_days: u64)->Result<()>{
    let is_started = ctx.accounts.global_state.started;
    if !is_started {
        return Err(MushiProgramError::NotStarted.into());
    }
    if number_of_days >= 366 {
        return Err(MushiProgramError::InvalidNumberOfDays.into());
    }
    let is_expired = ctx.accounts.is_loan_expired()?;
    let sol_fee = ctx.accounts.leverage_fee(sol_amount, number_of_days)?;
    let global_state = &mut ctx.accounts.global_state;
    liquidate(
        &mut ctx.accounts.last_liquidation_date_state,
        global_state,
        ctx.accounts.token_vault.to_account_info(),
        ctx.accounts.token.to_account_info(),
        ctx.accounts.token_vault_owner.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        *ctx.bumps.get("token_vault_owner").unwrap(),
    )?;
    let current_timestamp = Clock::get()?.unix_timestamp;
    let end_date = get_midnight_timestamp(current_timestamp + number_of_days as i64 * SECONDS_IN_A_DAY);
    let user_sol = sol_amount.checked_sub(sol_fee).unwrap();

    let fee_address_amount = sol_fee.checked_mul(3).unwrap().checked_div(10).unwrap();
    let user_borrow = user_sol.checked_mul(99).unwrap().checked_div(100).unwrap();
    let over_collateralization_amount = user_sol/100;
    let sub_value = fee_address_amount.checked_add(over_collateralization_amount).unwrap();
    let total_fee = sol_fee.checked_add(over_collateralization_amount).unwrap();
    let mut fee_overage = 0;

    let signer_seeds:&[&[&[u8]]] = &[&[VAULT_SEED, &[*ctx.bumps.get("token_vault_owner").unwrap()]]];
    if sol_amount > total_fee {
        fee_overage = sol_amount.checked_sub(total_fee).unwrap();
        transfer_sol(
            ctx.accounts.token_vault_owner.to_account_info(),
            ctx.accounts.user.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
            fee_overage,
            Some(signer_seeds)
        )?;
    }
    if sol_amount - fee_overage != total_fee {
        return Err(MushiProgramError::InvalidFeeAmount.into());
    }
    
    let user_mushi = ctx.accounts.sol_to_mushi_lev(user_sol, sub_value)?;
    mint_to_tokens_by_main_state(
        ctx.accounts.token.to_account_info(),
        ctx.accounts.main_state.to_account_info(),
        ctx.accounts.token_vault.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        user_mushi,
        *ctx.bumps.get("main_state").unwrap(),
    )?;

    if fee_address_amount <= MIN {
        return Err(MushiProgramError::InvalidFeeAmount.into());
    }
    let signer_seeds:&[&[&[u8]]] = &[&[VAULT_SEED, &[*ctx.bumps.get("token_vault_owner").unwrap()]]];
    transfer_sol(
        ctx.accounts.token_vault_owner.to_account_info(),
        ctx.accounts.fee_receiver.to_account_info(),
        ctx.accounts.system_program.to_account_info(),
        fee_address_amount,
        Some(signer_seeds)
    )?;
    ctx.accounts.add_loans_by_date(user_borrow, user_mushi, end_date)?;
    let user_loan = &mut ctx.accounts.user_loan;
 
    if user_loan.borrowed != 0 {
        if is_expired {
            user_loan.borrowed = 0;
            user_loan.collateral = 0;
            user_loan.end_date = 0;
            user_loan.number_of_days = 0;
        }
        if user_loan.borrowed != 0 {
            return Err(MushiProgramError::InvalidLoanAmount.into());
        }
    }
    user_loan.borrowed = user_borrow;
    user_loan.collateral = user_mushi;
    user_loan.end_date = end_date;
    user_loan.number_of_days = number_of_days;
    ctx.accounts.safety_check()?;
    
    Ok(())
}
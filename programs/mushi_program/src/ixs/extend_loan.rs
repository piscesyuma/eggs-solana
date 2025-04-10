use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token_interface};

use crate::{
    constants::{
        FEES_BUY, FEES_SELL, FEE_BASE_1000, MIN, SECONDS_IN_A_DAY, VAULT_SEED
    }, context::{ACommonExtExtendLoan, ACommonExtLoan}, error::MushiProgramError, utils::{
        add_loans_by_date, burn_tokens, get_interest_fee, get_midnight_timestamp, liquidate, mint_to_tokens_by_main_state, sub_loans_by_date, transfer_tokens, transfer_tokens_checked
    }
};
use crate::context::common::ACommon;

pub fn extend_loan(ctx:Context<ACommonExtExtendLoan>, number_of_days: u64 )->Result<()>{
    let user_loan = & ctx.accounts.common.user_loan;
    let old_end_date = user_loan.end_date;
    let _number_of_days = user_loan.number_of_days;
    let borrowed = user_loan.borrowed;
    let collateral = user_loan.collateral;

    let new_end_date = old_end_date + number_of_days as i64 * SECONDS_IN_A_DAY;
    let loan_fee = get_interest_fee(borrowed, number_of_days);
    
    require!(!ctx.accounts.common.is_loan_expired()?, MushiProgramError::LoanExpired);

    let fee_address_fee = loan_fee.checked_mul(3).unwrap().checked_div(10).unwrap();
    require!(fee_address_fee > MIN, MushiProgramError::InvalidFeeAmount);

    let quote_mint = ctx.accounts.common.quote_mint.to_account_info();
    let quote_token_program = ctx.accounts.common.quote_token_program.to_account_info();
    let decimals = ctx.accounts.common.quote_mint.decimals;

    transfer_tokens_checked(
        ctx.accounts.common.user_quote_ata.to_account_info(),
        ctx.accounts.common.quote_vault.to_account_info(),
        ctx.accounts.common.user.to_account_info(),
        quote_mint.clone(),
        quote_token_program.clone(),
        loan_fee, 
        decimals,
        None,
    )?;

    let signer_seeds: &[&[&[u8]]] =
        &[&[VAULT_SEED, &[*ctx.bumps.get("token_vault_owner").unwrap()]]];

    transfer_tokens_checked(
        ctx.accounts.common.quote_vault.to_account_info(),
        ctx.accounts.common.fee_receiver_quote_ata.to_account_info(),
        ctx.accounts.common.token_vault_owner.to_account_info(),
        quote_mint.clone(),
        quote_token_program.clone(),
        fee_address_fee, 
        decimals,
        Some(signer_seeds),
    )?;

    sub_loans_by_date(&mut ctx.accounts.common.global_state, &mut ctx.accounts.daily_state_old_end_date, borrowed, collateral)?;
    add_loans_by_date(&mut ctx.accounts.common.global_state, &mut ctx.accounts.daily_state_new_end_date, borrowed, collateral)?;
    let user_loan = &mut ctx.accounts.common.user_loan;
    user_loan.end_date = new_end_date;
    user_loan.number_of_days = number_of_days + _number_of_days;

    let current_timestamp = Clock::get()?.unix_timestamp;
    require!((new_end_date - current_timestamp) / SECONDS_IN_A_DAY < 366, MushiProgramError::InvalidNumberOfDays);
    ctx.accounts.common.safety_check()?;
    
    Ok(())
}
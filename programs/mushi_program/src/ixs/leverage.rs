use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token_interface};

use crate::{
    constants::{
        FEES_BUY, FEES_SELL, FEE_BASE_1000, MAX_SUPPLY, MIN, SECONDS_IN_A_DAY, VAULT_SEED
    }, context::ACommonExtLoan, error::MushiProgramError, utils::{
        add_loans_by_date, burn_tokens, get_interest_fee, get_midnight_timestamp, liquidate, mint_to_tokens_by_main_state, transfer_tokens, transfer_tokens_checked
    }
};
use crate::context::common::ACommon;

pub fn leverage(ctx:Context<ACommonExtLoan>, number_of_days: u64, es_amount:u64)->Result<()>{
    let is_started = ctx.accounts.common.global_state.started;
    require!(is_started, MushiProgramError::NotStarted);
    require!(number_of_days < 366, MushiProgramError::InvalidNumberOfDays);
    require!(es_amount != 0, MushiProgramError::InvalidEclipseAmount);
    
    let is_expired = ctx.accounts.common.is_loan_expired()?;
    
    // Reset loan if expired
    if is_expired {
        ctx.accounts.common.user_loan.borrowed = 0;
        ctx.accounts.common.user_loan.collateral = 0;
        ctx.accounts.common.user_loan.end_date = 0;
        ctx.accounts.common.user_loan.number_of_days = 0;
    }
    
    // Check borrowed amount
    require!(ctx.accounts.common.user_loan.borrowed == 0, MushiProgramError::InvalidLoanAmount);
    
    // Extract values before further operations to avoid multiple borrows
    let eclipse_fee = ctx.accounts.common.leverage_fee(es_amount, number_of_days)?;
    let bump = *ctx.bumps.get("token_vault_owner").unwrap();
    let main_state_bump = *ctx.bumps.get("main_state").unwrap();
    
    // Liquidate
    liquidate(
        &mut ctx.accounts.common.last_liquidation_date_state,
        &mut ctx.accounts.common.global_state,
        ctx.accounts.common.token_vault.to_account_info(),
        ctx.accounts.common.token.to_account_info(),
        ctx.accounts.common.token_vault_owner.to_account_info(),
        ctx.accounts.common.base_token_program.to_account_info(),
        bump,
    )?;
    
    let current_timestamp = Clock::get()?.unix_timestamp;
    let end_date = get_midnight_timestamp(current_timestamp + number_of_days as i64 * SECONDS_IN_A_DAY);
    let user_eclipse = es_amount.checked_sub(eclipse_fee).unwrap();

    let fee_address_amount = eclipse_fee.checked_mul(3).unwrap().checked_div(10).unwrap();
    let user_borrow = user_eclipse.checked_mul(99).unwrap().checked_div(100).unwrap();
    let over_collateralization_amount = user_eclipse/100;
    let sub_value = fee_address_amount.checked_add(over_collateralization_amount).unwrap();
    let total_fee = eclipse_fee.checked_add(over_collateralization_amount).unwrap();

    // Calculate user_mushi before borrowing ctx.accounts mutably again
    let user_mushi = ctx.accounts.common.eclipse_to_mushi_lev(user_eclipse, sub_value, es_amount)?;
    
    require!(ctx.accounts.common.global_state.token_supply + user_mushi <= MAX_SUPPLY, MushiProgramError::MaxSupplyExceeded);

    // Mint tokens
    mint_to_tokens_by_main_state(
        ctx.accounts.common.token.to_account_info(),
        ctx.accounts.common.main_state.to_account_info(),
        ctx.accounts.common.token_vault.to_account_info(),
        ctx.accounts.common.base_token_program.to_account_info(),
        user_mushi,
        main_state_bump,
    )?;
    
    ctx.accounts.common.global_state.token_supply += user_mushi;

    require!(fee_address_amount > MIN, MushiProgramError::InvalidFeeAmount);
    
    let quote_mint = ctx.accounts.common.quote_mint.to_account_info();
    let quote_token_program = ctx.accounts.common.quote_token_program.to_account_info();
    let decimals = ctx.accounts.common.quote_mint.decimals;

    transfer_tokens_checked(
        ctx.accounts.common.user_quote_ata.to_account_info(),
        ctx.accounts.common.quote_vault.to_account_info(),
        ctx.accounts.common.user.to_account_info(),
        quote_mint.clone(),
        quote_token_program.clone(),
        total_fee, 
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
        fee_address_amount, 
        decimals,
        Some(signer_seeds),
    )?;
    
    // Update loans by date
    add_loans_by_date(&mut ctx.accounts.common.global_state, &mut ctx.accounts.daily_state_end_date, user_borrow, user_mushi)?;
 
    // Update user loan data at the end to avoid borrowing conflicts
    let user_loan = &mut ctx.accounts.common.user_loan;
    user_loan.borrowed = user_borrow;
    user_loan.collateral = user_mushi;
    user_loan.end_date = end_date;
    user_loan.number_of_days = number_of_days;
    
    ctx.accounts.common.safety_check(total_fee - fee_address_amount, true)?;
    
    Ok(())
}
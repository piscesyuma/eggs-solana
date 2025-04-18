use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token_interface};

use crate::{
    constants::{
        FEES_BUY, FEES_SELL, FEE_BASE_1000, MIN, SECONDS_IN_A_DAY, VAULT_SEED
    }, context::{ACommonExtLoan, ACommonExtSubLoan}, error::MushiProgramError, utils::{
        add_loans_by_date, burn_tokens, get_interest_fee, get_midnight_timestamp, liquidate, mint_to_tokens_by_main_state, transfer_tokens, transfer_tokens_checked
    }
};
use crate::context::common::ACommon;

pub fn borrow(ctx:Context<ACommonExtLoan>, number_of_days: u64, es_amount:u64)->Result<()>{
    let is_expired = ctx.accounts.common.is_loan_expired()?;
    let user_mushi = ctx.accounts.common.eclipse_to_mushi_no_trade_ceil(es_amount)?;
    let user_loan = &mut ctx.accounts.common.user_loan;
    
    require!(number_of_days < 366, MushiProgramError::InvalidNumberOfDays);
    require!(es_amount != 0, MushiProgramError::InvalidEclipseAmount);

    if is_expired {
        user_loan.borrowed = 0;
        user_loan.collateral = 0;
        user_loan.end_date = 0;
        user_loan.number_of_days = 0;
    }

    require!(user_loan.borrowed == 0, MushiProgramError::InvalidLoanAmount);

    let global_state = &mut ctx.accounts.common.global_state;
    liquidate(
        &mut ctx.accounts.common.last_liquidation_date_state,
        global_state,
        ctx.accounts.common.token_vault.to_account_info(),
        ctx.accounts.common.token.to_account_info(),
        ctx.accounts.common.token_vault_owner.to_account_info(),
        ctx.accounts.common.base_token_program.to_account_info(),   
        *ctx.bumps.get("token_vault_owner").unwrap(),
    )?;

    let current_timestamp = Clock::get()?.unix_timestamp;
    let end_date = get_midnight_timestamp(current_timestamp + number_of_days as i64 * SECONDS_IN_A_DAY);
    let eclipse_fee = get_interest_fee(es_amount, number_of_days);
    let fee_address_fee = eclipse_fee.checked_mul(3).unwrap().checked_div(10).unwrap();
    // AUDIT: mushi required from user round up?
    let new_user_borrow = es_amount.checked_mul(99).unwrap().checked_div(100).unwrap();
    
    user_loan.collateral = user_mushi;
    user_loan.borrowed = new_user_borrow;
    user_loan.end_date = end_date;
    user_loan.number_of_days = number_of_days;

    transfer_tokens(
        ctx.accounts.common.user_ata.to_account_info(),
        ctx.accounts.common.token_vault.to_account_info(),
        ctx.accounts.user.to_account_info(),
        ctx.accounts.common.base_token_program.to_account_info(),
        user_mushi,
        None,
    )?;
    
    require!(fee_address_fee > MIN, MushiProgramError::InvalidFeeAmount);

    let from = ctx.accounts.common.quote_vault.to_account_info();
    let authority = ctx.accounts.common.token_vault_owner.to_account_info();
    let quote_mint = ctx.accounts.common.quote_mint.to_account_info();
    let quote_token_program = ctx.accounts.common.quote_token_program.to_account_info();
    let decimals = ctx.accounts.common.quote_mint.decimals;
    let token_vault_owner_bump = *ctx.bumps.get("token_vault_owner").unwrap();
    transfer_tokens_checked(
        from.clone(),
        ctx.accounts.common.user_quote_ata.to_account_info(),
        authority.clone(),
        quote_mint.clone(),
        quote_token_program.clone(),
        new_user_borrow - eclipse_fee, 
        decimals,
        Some(&[&[VAULT_SEED, &[token_vault_owner_bump]]]),
    )?;

    transfer_tokens_checked(
        from.clone(),
        ctx.accounts.common.fee_receiver_quote_ata.to_account_info(),
        authority.clone(),
        quote_mint.clone(),
        quote_token_program.clone(),
        fee_address_fee, 
        decimals,
        Some(&[&[VAULT_SEED, &[token_vault_owner_bump]]]),
    )?;
    
    // ctx.accounts.add_loans_by_date( new_user_borrow, user_mushi)?;
    add_loans_by_date(&mut ctx.accounts.common.global_state, &mut ctx.accounts.daily_state_end_date, new_user_borrow, user_mushi)?;

    ctx.accounts.common.safety_check(new_user_borrow - eclipse_fee + fee_address_fee, false)?;
    Ok(())
}

pub fn borrow_more(ctx:Context<ACommonExtSubLoan>, es_amount:u64)->Result<()>{
    let is_expired = ctx.accounts.common.is_loan_expired()?;
    require!(!is_expired, MushiProgramError::LoanExpired);
    require!(es_amount != 0, MushiProgramError::InvalidEclipseAmount);

    let user_mushi = ctx.accounts.common.eclipse_to_mushi_no_trade_ceil(es_amount)?;
    let user_loan = & ctx.accounts.common.user_loan;
 
    let global_state = &mut ctx.accounts.common.global_state;
    liquidate(
        &mut ctx.accounts.common.last_liquidation_date_state,
        global_state,
        ctx.accounts.common.token_vault.to_account_info(),
        ctx.accounts.common.token.to_account_info(),
        ctx.accounts.common.token_vault_owner.to_account_info(),
        ctx.accounts.common.base_token_program.to_account_info(),
        *ctx.bumps.get("token_vault_owner").unwrap(),
    )?;
    let user_borrowed = user_loan.borrowed;
    let user_collateral = user_loan.collateral;
    let user_end_date = user_loan.end_date;

    let today_midnight = get_midnight_timestamp(Clock::get()?.unix_timestamp);
    let new_borrow_length = (user_end_date - today_midnight) / SECONDS_IN_A_DAY;
    let eclipse_fee = get_interest_fee(es_amount, new_borrow_length as u64);

    let fee_address_fee = eclipse_fee.checked_mul(3).unwrap().checked_div(10).unwrap();
    let new_user_borrow = es_amount.checked_mul(99).unwrap().checked_div(100).unwrap();
    let user_borrowed_in_mushi = ctx.accounts.common.eclipse_to_mushi_no_trade(user_borrowed)?;
    let user_excess_in_mushi = user_collateral.checked_mul(99).unwrap().checked_div(100).unwrap().checked_sub(user_borrowed_in_mushi).unwrap();
    let mut require_collateral_from_user = user_mushi;

    if user_excess_in_mushi >= user_mushi {
        require_collateral_from_user = 0;
    } else {
        require_collateral_from_user -= user_excess_in_mushi;
    }

    let new_user_borrow_total = user_borrowed.checked_add(new_user_borrow).unwrap();
    let new_user_collateral_total = user_collateral + require_collateral_from_user;

    let user_loan = &mut ctx.accounts.common.user_loan;
    user_loan.borrowed = new_user_borrow_total;
    user_loan.collateral = new_user_collateral_total;
    user_loan.end_date = user_end_date;
    user_loan.number_of_days = new_borrow_length as u64;

    if require_collateral_from_user != 0 {
        transfer_tokens(
            ctx.accounts.common.user_ata.to_account_info(),
            ctx.accounts.common.token_vault.to_account_info(),
            ctx.accounts.common.user.to_account_info(),
            ctx.accounts.common.base_token_program.to_account_info(),
            require_collateral_from_user,
            None,
        )?;
    }
    
    require!(fee_address_fee > MIN, MushiProgramError::InvalidFeeAmount);

    let from = ctx.accounts.common.quote_vault.to_account_info();
    let authority = ctx.accounts.common.token_vault_owner.to_account_info();
    let quote_mint = ctx.accounts.common.quote_mint.to_account_info();
    let quote_token_program = ctx.accounts.common.quote_token_program.to_account_info();
    let decimals = ctx.accounts.common.quote_mint.decimals;

    let token_vault_owner_bump = *ctx.bumps.get("token_vault_owner").unwrap();


    transfer_tokens_checked(
        from.clone(),
        ctx.accounts.common.user_quote_ata.to_account_info(),
        authority.clone(),
        quote_mint.clone(),
        quote_token_program.clone(),
        new_user_borrow - eclipse_fee, 
        decimals,
        Some(&[&[VAULT_SEED, &[token_vault_owner_bump]]]),
    )?;

    transfer_tokens_checked(
        from.clone(),
        ctx.accounts.common.fee_receiver_quote_ata.to_account_info(),
        authority.clone(),
        quote_mint.clone(),
        quote_token_program.clone(),
        fee_address_fee, 
        decimals,
        Some(&[&[VAULT_SEED, &[token_vault_owner_bump]]]),
    )?;

    add_loans_by_date(&mut ctx.accounts.common.global_state, &mut ctx.accounts.daily_state_old_end_date, new_user_borrow, user_mushi)?;

    ctx.accounts.common.safety_check(new_user_borrow - eclipse_fee + fee_address_fee, false)?;
    Ok(())
}
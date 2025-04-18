use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token_interface};

use crate::{
    constants::{
        FEES_BUY, FEES_SELL, FEE_BASE_1000, MIN, SECONDS_IN_A_DAY, VAULT_SEED
    }, context::{ACommonExtLoan, ACommonExtLoan2, ACommonExtSubLoan}, error::MushiProgramError, utils::{
        burn_tokens, get_interest_fee, get_midnight_timestamp, liquidate, mint_to_tokens_by_main_state, sub_loans_by_date, transfer_tokens, transfer_tokens_checked
    }
};
use crate::context::common::ACommon;

pub fn close_position(ctx:Context<ACommonExtSubLoan>, es_amount: u64)->Result<()>{
    let user_loan = & ctx.accounts.common.user_loan;
    let borrowed = user_loan.borrowed;
    let collateral = user_loan.collateral;
    require!(!ctx.accounts.common.is_loan_expired()?, MushiProgramError::LoanExpired);
    require!(borrowed == es_amount, MushiProgramError::InvalidLoanAmount);

    let quote_mint = ctx.accounts.common.quote_mint.to_account_info();
    let quote_token_program = ctx.accounts.common.quote_token_program.to_account_info();
    let decimals = ctx.accounts.common.quote_mint.decimals;

    transfer_tokens_checked(
        ctx.accounts.common.user_quote_ata.to_account_info(),
        ctx.accounts.common.quote_vault.to_account_info(),
        ctx.accounts.common.user.to_account_info(),
        quote_mint.clone(),
        quote_token_program.clone(),
        es_amount, 
        decimals,
        None,
    )?;
            
    let signer_seeds:&[&[&[u8]]] = &[&[VAULT_SEED, &[*ctx.bumps.get("token_vault_owner").unwrap()]]];
    transfer_tokens(
        ctx.accounts.common.token_vault.to_account_info(),
        ctx.accounts.common.user_ata.to_account_info(),
        ctx.accounts.common.token_vault_owner.to_account_info(),
        ctx.accounts.common.base_token_program.to_account_info(),
        collateral,
        Some(signer_seeds)
    )?;
    sub_loans_by_date(&mut ctx.accounts.common.global_state, &mut ctx.accounts.daily_state_old_end_date, borrowed, collateral)?;

    let user_loan = &mut ctx.accounts.common.user_loan;
    user_loan.borrowed = 0;
    user_loan.collateral = 0;
    user_loan.end_date = 0;
    user_loan.number_of_days = 0;
    ctx.accounts.common.safety_check(es_amount, true)?;
    
    Ok(())
}

pub fn flash_close_position(ctx:Context<ACommonExtSubLoan>)->Result<()>{
    require!(!ctx.accounts.common.is_loan_expired()?, MushiProgramError::LoanExpired);
    let global_state = &mut ctx.accounts.common.global_state;
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
    let user_loan = & ctx.accounts.common.user_loan;
    let borrowed = user_loan.borrowed;
    let collateral = user_loan.collateral;

    let collateral_in_eclipse = ctx.accounts.common.mushi_to_eclipse(collateral)?;
    let signer_seeds:&[&[&[u8]]] = &[&[VAULT_SEED, &[*ctx.bumps.get("token_vault_owner").unwrap()]]];
    burn_tokens(
        ctx.accounts.common.token_vault.to_account_info(),
        ctx.accounts.common.token.to_account_info(),
        ctx.accounts.common.token_vault_owner.to_account_info(),
        ctx.accounts.common.base_token_program.to_account_info(),
        collateral,
        Some(signer_seeds)
    )?;
    ctx.accounts.common.global_state.token_supply = ctx.accounts.common.global_state.token_supply.checked_sub(collateral).unwrap();
    
    let collateral_in_eclipse_after_fee = collateral_in_eclipse.checked_mul(99).unwrap().checked_div(100).unwrap();
    let fee = collateral_in_eclipse / 100;

    require!(collateral_in_eclipse_after_fee >= borrowed, MushiProgramError::InvalidCollateralAmount);
    
    let to_user = collateral_in_eclipse_after_fee.checked_sub(borrowed).unwrap();
    let fee_address_fee = fee.checked_mul(3).unwrap().checked_div(10).unwrap();

    require!(fee_address_fee > MIN, MushiProgramError::InvalidFeeAmount);

    let quote_mint = ctx.accounts.common.quote_mint.to_account_info();
    let quote_token_program = ctx.accounts.common.quote_token_program.to_account_info();
    let decimals = ctx.accounts.common.quote_mint.decimals;

    let signer_seeds: &[&[&[u8]]] =
        &[&[VAULT_SEED, &[*ctx.bumps.get("token_vault_owner").unwrap()]]];

    transfer_tokens_checked(
        ctx.accounts.common.quote_vault.to_account_info(),
        ctx.accounts.common.user_quote_ata.to_account_info(),
        ctx.accounts.common.token_vault_owner.to_account_info(),
        quote_mint.clone(),
        quote_token_program.clone(),
        to_user, 
        decimals,
        Some(signer_seeds),
    )?;

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
    let user_loan = &mut ctx.accounts.common.user_loan;
    user_loan.borrowed = 0;
    user_loan.collateral = 0;
    user_loan.end_date = 0;
    user_loan.number_of_days = 0;
    ctx.accounts.common.safety_check(to_user + fee_address_fee, false)?;
    Ok(())
}
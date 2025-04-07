use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token_interface};

use crate::{
    constants::{
        FEES_BUY, FEES_SELL, FEE_BASE_1000, MIN, SECONDS_IN_A_DAY, VAULT_SEED
    }, context::{ACommonExtLoan, ACommonExtLoan2, ACommonExtSubLoan}, error::MushiProgramError, utils::{
        burn_tokens, get_interest_fee, get_midnight_timestamp, liquidate, mint_to_tokens_by_main_state, transfer_sol, transfer_tokens
    }
};
use crate::context::common::ACommon;

pub fn close_position(ctx:Context<ACommonExtSubLoan>, sol_amount: u64)->Result<()>{
    let user_loan = & ctx.accounts.common.user_loan;
    let borrowed = user_loan.borrowed;
    let collateral = user_loan.collateral;
    require!(!ctx.accounts.common.is_loan_expired()?, MushiProgramError::LoanExpired);
    require!(borrowed == sol_amount, MushiProgramError::InvalidLoanAmount);

    transfer_sol(
        ctx.accounts.common.user.to_account_info(), 
        ctx.accounts.common.token_vault_owner.to_account_info(), 
        ctx.accounts.common.system_program.to_account_info(), 
        sol_amount, 
        None)?;
            
    let signer_seeds:&[&[&[u8]]] = &[&[VAULT_SEED, &[*ctx.bumps.get("token_vault_owner").unwrap()]]];
    transfer_tokens(
        ctx.accounts.common.token_vault.to_account_info(),
        ctx.accounts.common.user_ata.to_account_info(),
        ctx.accounts.common.token_vault_owner.to_account_info(),
        ctx.accounts.common.token_program.to_account_info(),
        collateral,
        Some(signer_seeds)
    )?;
    ctx.accounts.sub_loans_by_date(borrowed, collateral, user_loan.end_date)?;

    let user_loan = &mut ctx.accounts.common.user_loan;
    user_loan.borrowed = 0;
    user_loan.collateral = 0;
    user_loan.end_date = 0;
    user_loan.number_of_days = 0;
    ctx.accounts.common.safety_check()?;
    
    Ok(())
}

pub fn flash_close_position(ctx:Context<ACommonExtSubLoan>)->Result<()>{
    if ctx.accounts.common.is_loan_expired()? {
        return Err(MushiProgramError::LoanExpired.into());
    }
    let global_state = &mut ctx.accounts.common.global_state;
    liquidate(
        &mut ctx.accounts.common.last_liquidation_date_state,
        global_state,
        ctx.accounts.common.token_vault.to_account_info(),
        ctx.accounts.common.token.to_account_info(),
        ctx.accounts.common.token_vault_owner.to_account_info(),
        ctx.accounts.common.token_program.to_account_info(),
        *ctx.bumps.get("token_vault_owner").unwrap(),
    )?;
    let user_loan = & ctx.accounts.common.user_loan;
    let borrowed = user_loan.borrowed;
    let collateral = user_loan.collateral;

    let collateral_in_sol = ctx.accounts.common.mushi_to_sol(collateral)?;
    let signer_seeds:&[&[&[u8]]] = &[&[VAULT_SEED, &[*ctx.bumps.get("token_vault_owner").unwrap()]]];
    burn_tokens(
        ctx.accounts.common.token_vault.to_account_info(),
        ctx.accounts.common.token.to_account_info(),
        ctx.accounts.common.token_vault_owner.to_account_info(),
        ctx.accounts.common.token_program.to_account_info(),
        collateral,
        Some(signer_seeds)
    )?;
    let collateral_in_sonic_after_fee = collateral_in_sol.checked_mul(99).unwrap().checked_div(100).unwrap();
    let fee = collateral_in_sol / 100;
    if collateral_in_sonic_after_fee < borrowed {
        return Err(MushiProgramError::InvalidCollateralAmount.into());
    }
    let to_user = collateral_in_sonic_after_fee.checked_sub(borrowed).unwrap();
    let fee_address_fee = fee.checked_mul(3).unwrap().checked_div(10).unwrap();
    transfer_sol(
        ctx.accounts.common.token_vault_owner.to_account_info(),
        ctx.accounts.common.user.to_account_info(),
        ctx.accounts.common.system_program.to_account_info(),
        to_user,
        Some(signer_seeds)
    )?;
    if fee_address_fee <= MIN {
        return Err(MushiProgramError::InvalidFeeAmount.into());
    }
    transfer_sol(
        ctx.accounts.common.token_vault_owner.to_account_info(),
        ctx.accounts.common.fee_receiver.to_account_info(),
        ctx.accounts.common.system_program.to_account_info(),
        fee_address_fee,
            Some(signer_seeds))?;
    ctx.accounts.sub_loans_by_date(borrowed, collateral, user_loan.end_date)?;
    let user_loan = &mut ctx.accounts.common.user_loan;
    user_loan.borrowed = 0;
    user_loan.collateral = 0;
    user_loan.end_date = 0;
    user_loan.number_of_days = 0;
    ctx.accounts.common.safety_check()?;
    Ok(())
}
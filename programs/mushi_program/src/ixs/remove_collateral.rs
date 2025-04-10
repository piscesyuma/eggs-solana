use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token_interface};

use crate::{
    constants::{
        FEES_BUY, FEES_SELL, FEE_BASE_1000, MIN, SECONDS_IN_A_DAY, VAULT_SEED
    }, context::{ACommonExtLoan, ACommonExtSubLoan}, error::MushiProgramError, utils::{
        burn_tokens, get_interest_fee, get_midnight_timestamp, liquidate, mint_to_tokens_by_main_state, sub_loans_by_date, transfer_tokens
    }
};
use crate::context::common::ACommon;

pub fn remove_collateral(ctx:Context<ACommonExtSubLoan>, amount: u64)->Result<()>{
    require!(!ctx.accounts.common.is_loan_expired()?, MushiProgramError::LoanExpired);

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
    let collateral = user_loan.collateral;

    require!(
        user_loan.borrowed <= 
        ctx.accounts.common.mushi_to_sol((collateral - amount)* 99)?/ 100, 
        MushiProgramError::RemoveCollateralFailed);
        
    sub_loans_by_date(&mut ctx.accounts.common.global_state, &mut ctx.accounts.daily_state_old_end_date, 0, amount)?;
    let user_loan = &mut ctx.accounts.common.user_loan;
    user_loan.collateral -= amount;
    let signer_seeds:&[&[&[u8]]] = &[&[VAULT_SEED, &[*ctx.bumps.get("token_vault_owner").unwrap()]]];
    transfer_tokens(
        ctx.accounts.common.token_vault.to_account_info(),
        ctx.accounts.common.user_ata.to_account_info(),
        ctx.accounts.common.token_vault_owner.to_account_info(),
        ctx.accounts.common.base_token_program.to_account_info(),
        amount,
        Some(signer_seeds)
    )?;
    ctx.accounts.common.safety_check(0, true)?;
    Ok(())
}
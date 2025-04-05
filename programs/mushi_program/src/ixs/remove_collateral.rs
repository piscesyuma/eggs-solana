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

pub fn remove_collateral(ctx:Context<ACommon>, amount: u64)->Result<()>{
    if ctx.accounts.is_loan_expired()? {
        return Err(MushiProgramError::LoanExpired.into());
    }
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
    let user_loan = & ctx.accounts.user_loan;
    let collateral = user_loan.collateral;
    if user_loan.borrowed > ctx.accounts.mushi_to_sol((collateral - amount)* 99)?/ 100 {
        return Err(MushiProgramError::RemoveCollateralFailed.into());
    }
    ctx.accounts.sub_loans_by_date(0, amount, user_loan.end_date)?;
    let user_loan = &mut ctx.accounts.user_loan;
    user_loan.collateral -= amount;
    let signer_seeds:&[&[&[u8]]] = &[&[VAULT_SEED, &[*ctx.bumps.get("token_vault_owner").unwrap()]]];
    transfer_tokens(
        ctx.accounts.token_vault.to_account_info(),
        ctx.accounts.user_ata.to_account_info(),
        ctx.accounts.token_vault_owner.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        amount,
        Some(signer_seeds)
    )?;
    ctx.accounts.safety_check()?;
    Ok(())
}
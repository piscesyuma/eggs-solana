use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token_interface};

use crate::{
    constants::{
        FEES_BUY, FEES_SELL, FEE_BASE_1000, MIN, SECONDS_IN_A_DAY, VAULT_SEED
    }, context::{ACommonExtLoan, ACommonExtSubLoan}, error::MushiProgramError, utils::{
        burn_tokens, get_interest_fee, get_midnight_timestamp, liquidate, mint_to_tokens_by_main_state, sub_loans_by_date, transfer_tokens, transfer_tokens_checked
    }
};
use crate::context::common::ACommon;

pub fn repay(ctx:Context<ACommonExtSubLoan>, es_amount: u64)->Result<()>{
    let user_loan = & ctx.accounts.common.user_loan;
    let borrowed = user_loan.borrowed;
    require!(borrowed > es_amount, MushiProgramError::InvalidSolAmount);
    require!(es_amount != 0, MushiProgramError::InvalidSolAmount);

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

    sub_loans_by_date(&mut ctx.accounts.common.global_state, &mut ctx.accounts.daily_state_old_end_date, es_amount, 0)?;
    let new_borrow = borrowed - es_amount;
    let user_loan = &mut ctx.accounts.common.user_loan;
    user_loan.borrowed = new_borrow;
    ctx.accounts.common.safety_check(es_amount, true)?;
    Ok(())
}
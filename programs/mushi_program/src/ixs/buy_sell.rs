use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token_interface};

use crate::{
    constants::{FEES_BUY, FEES_SELL, FEE_BASE_1000, MIN, VAULT_SEED}, 
    error::MushiProgramError, 
    utils::{burn_tokens, liquidate, mint_to_tokens_by_main_state, trasnfer_sol}, 
};
use crate::context::common::ACommon;

pub fn buy(ctx:Context<ACommon>, sol_amount:u64) -> Result<()> {
    let mushi = ctx.accounts.sol_to_mushi(sol_amount)?;
    let global_state =&mut ctx.accounts.global_state;
    liquidate(
        &mut ctx.accounts.last_liquidation_date_state,
        global_state,
        ctx.accounts.token_vault.to_account_info(),
        ctx.accounts.token.to_account_info(),
        ctx.accounts.token_vault_owner.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        *ctx.bumps.get("token_vault_owner").unwrap(),
    )?;
    let is_started = global_state.started;
    if !is_started {
        return Err(MushiProgramError::NotStarted.into());
    }
    // minting tokens
    mint_to_tokens_by_main_state(
        ctx.accounts.token.to_account_info(), 
    ctx.accounts.main_state.to_account_info(), 
        ctx.accounts.user_ata.to_account_info(), 
        ctx.accounts.token_program.to_account_info(), 
        mushi * ctx.accounts.main_state.buy_fee / FEE_BASE_1000, 
        *ctx.bumps.get("main_state").unwrap()
    )?;
    global_state.token_supply += mushi * ctx.accounts.main_state.buy_fee / FEE_BASE_1000;
    // calc sender SOLs
    let fee: u64 = sol_amount.checked_div(FEES_BUY).unwrap();
    if fee <= MIN {
        return Err(MushiProgramError::TooSmallTeamFee.into());
    }
    let left_sol_amount = sol_amount.checked_sub(fee).unwrap();
    trasnfer_sol(
        ctx.accounts.user.to_account_info(), 
    ctx.accounts.fee_receiver.to_account_info(), 
    ctx.accounts.system_program.to_account_info(), 
    fee, 
    None)?;
    trasnfer_sol(
        ctx.accounts.user.to_account_info(), 
        ctx.accounts.token_vault_owner.to_account_info(), 
        ctx.accounts.system_program.to_account_info(), 
        left_sol_amount, 
        None)?;
    ctx.accounts.safety_check()?;
    Ok(())
}

pub fn sell(ctx:Context<ACommon>, token_amount:u64)->Result<()>{
    let sol_amount = ctx.accounts.mushi_to_sol(token_amount)?;
    let global_state =&mut ctx.accounts.global_state;
    liquidate(
        &mut ctx.accounts.last_liquidation_date_state,
        global_state,
        ctx.accounts.token_vault.to_account_info(),
        ctx.accounts.token.to_account_info(),
        ctx.accounts.token_vault_owner.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        *ctx.bumps.get("token_vault_owner").unwrap(),
    )?;
    let seller = ctx.accounts.user.to_account_info();
    // burn tokens
    burn_tokens(
        ctx.accounts.user_ata.to_account_info(), 
        ctx.accounts.token.to_account_info(), 
        seller.clone(), 
        ctx.accounts.token_program.to_account_info(), 
        token_amount, 
        None)?; 
    
    global_state.token_supply -= token_amount;
    // calc & sending sol
    let system_program = ctx.accounts.system_program.to_account_info();
    let vault_owner = ctx.accounts.token_vault_owner.to_account_info();
    let signer_seeds:&[&[&[u8]]] = &[&[VAULT_SEED, &[*ctx.bumps.get("token_vault_owner").unwrap()]]];
    // Payment to seller
    trasnfer_sol(
        vault_owner.to_account_info(), 
        seller.to_account_info(), 
        system_program.to_account_info(), 
        sol_amount * ctx.accounts.main_state.sell_fee / FEE_BASE_1000, 
        Some(signer_seeds))?;
    // team fee
    if sol_amount / FEES_SELL <= MIN {
        return Err(MushiProgramError::TooSmallInputAmount.into());
    }
    trasnfer_sol(
        vault_owner.to_account_info(), 
        ctx.accounts.fee_receiver.to_account_info(), 
        system_program.to_account_info(), 
        sol_amount / FEES_SELL, 
        Some(signer_seeds))?;
    ctx.accounts.safety_check()?;
    Ok(())
}

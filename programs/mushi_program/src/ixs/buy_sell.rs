use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token_interface};

use crate::{
    constants::{FEES_BUY, FEES_BUY_REFERRAL, FEES_SELL, FEE_BASE_1000, MIN, VAULT_SEED}, 
    error::MushiProgramError, 
    utils::{burn_tokens, liquidate, mint_to_tokens_by_main_state, trasnfer_sol, trasnfer_sol_to_pubkey}, 
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
    
    let fee: u64 = sol_amount.checked_mul(FEES_BUY+FEES_BUY_REFERRAL).unwrap().checked_div(10_000).unwrap();
    // let fee: u64 = sol_amount.checked_div(FEES_BUY).unwrap();
    
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

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct BuyWithReferralInput {
    pub sol_amount: u64,
    pub referral_pubkey: Pubkey,
}

pub fn buy_with_referral(ctx:Context<ACommon>,  input: BuyWithReferralInput ) -> Result<()> {
    let sol_amount = input.sol_amount;
    let referral = &mut ctx.accounts.referral;
    

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

    if ctx.accounts.referral.is_none() {
        return Err(MushiProgramError::ReferralNotFound.into());
    }
    
    let referral_account = ctx.accounts.referral.as_ref().unwrap();
    if referral_account.key() != input.referral_pubkey {
        return Err(MushiProgramError::InvalidReferralAccount.into());
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
    
    let fee_treasury: u64 = sol_amount.checked_mul(FEES_BUY).unwrap().checked_div(10_000).unwrap();
    let fee_referral: u64 = sol_amount.checked_mul(FEES_BUY_REFERRAL).unwrap().checked_div(10_000).unwrap();
    // let fee: u64 = sol_amount.checked_div(FEES_BUY).unwrap();
    
    if fee_treasury <= MIN {
        return Err(MushiProgramError::TooSmallTeamFee.into());
    }
    if fee_referral <= MIN {
        return Err(MushiProgramError::TooSmallTeamFee.into());
    }

    let left_sol_amount = sol_amount.checked_sub(fee_treasury+fee_referral).unwrap();
    trasnfer_sol(
        ctx.accounts.user.to_account_info(), 
    ctx.accounts.fee_receiver.to_account_info(), 
    ctx.accounts.system_program.to_account_info(), 
    fee_treasury, 
    None)?;
    
    trasnfer_sol(
        ctx.accounts.user.to_account_info(), 
        ctx.accounts.referral.as_ref().unwrap().to_account_info(), 
        ctx.accounts.system_program.to_account_info(), 
        fee_referral, 
        None
    )?;
    
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
    
    let sol_fee_amount = sol_amount.checked_mul(FEES_SELL).unwrap().checked_div(10_000).unwrap();

    require!(sol_fee_amount > MIN, MushiProgramError::TooSmallInputAmount);

    // Payment to seller
    trasnfer_sol(
        vault_owner.to_account_info(), 
        seller.to_account_info(), 
        system_program.to_account_info(), 
        sol_amount * ctx.accounts.main_state.sell_fee / FEE_BASE_1000, 
        Some(signer_seeds))?;
    // team fee
    
    // if sol_amount / FEES_SELL <= MIN {
    //     return Err(MushiProgramError::TooSmallInputAmount.into());
    // }

    trasnfer_sol(
        vault_owner.to_account_info(), 
        ctx.accounts.fee_receiver.to_account_info(), 
        system_program.to_account_info(), 
        sol_fee_amount, 
        Some(signer_seeds))?;
    ctx.accounts.safety_check()?;
    Ok(())
}

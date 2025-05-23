use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token_interface};

use crate::constants::MAX_SUPPLY;
use crate::context::common::ACommon;
use crate::utils::transfer_tokens_checked;
use crate::{
    constants::{FEES_BUY, FEES_BUY_REFERRAL, FEES_SELL, FEE_BASE_1000, MIN, VAULT_SEED},
    context::ACommonExtReferral,
    error::MushiProgramError,
    utils::{
        burn_tokens, liquidate, mint_to_tokens_by_main_state,
    },
};

pub fn buy(ctx: Context<ACommon>, es_amount: u64, min_mushi_out: Option<u64>) -> Result<()> {
    let mushi = ctx.accounts.eclipse_to_mushi(es_amount)?;

    let min_mushi_out = min_mushi_out.unwrap_or(mushi);
    require!(mushi >= min_mushi_out, MushiProgramError::TooSmallOutputAmount);

    let global_state = &mut ctx.accounts.global_state;
    liquidate(
        &mut ctx.accounts.last_liquidation_date_state,
        global_state,
        ctx.accounts.token_vault.to_account_info(),
        ctx.accounts.token.to_account_info(),
        ctx.accounts.token_vault_owner.to_account_info(),
        ctx.accounts.base_token_program.to_account_info(),
        *ctx.bumps.get("token_vault_owner").unwrap(),
    )?;
    let is_started = ctx.accounts.main_state.started;
    require!(is_started, MushiProgramError::NotStarted);

    require!(global_state.token_supply + mushi * ctx.accounts.main_state.buy_fee / FEE_BASE_1000 <= MAX_SUPPLY, MushiProgramError::MaxSupplyExceeded);
    // minting tokens
    mint_to_tokens_by_main_state(
        ctx.accounts.token.to_account_info(),
        ctx.accounts.main_state.to_account_info(),
        ctx.accounts.user_ata.to_account_info(),
        ctx.accounts.base_token_program.to_account_info(),
        mushi * ctx.accounts.main_state.buy_fee / FEE_BASE_1000,
        *ctx.bumps.get("main_state").unwrap(),
    )?;
    global_state.token_supply += mushi * ctx.accounts.main_state.buy_fee / FEE_BASE_1000;

    // calc sender Eclipses
    let fee: u64 = es_amount
        .checked_mul(FEES_BUY + FEES_BUY_REFERRAL)
        .unwrap()
        .checked_div(10_000)
        .unwrap();
    // let fee: u64 = es_amount.checked_div(FEES_BUY).unwrap();

    require!(fee > MIN, MushiProgramError::TooSmallTeamFee);
    let left_es_amount = es_amount.checked_sub(fee).unwrap();

    // sending quote token
    let authority = ctx.accounts.user.to_account_info();
    let from = ctx.accounts.user_quote_ata.to_account_info();
    let decimals = ctx.accounts.quote_mint.decimals;
    let mint = ctx.accounts.quote_mint.to_account_info();
    let token_program = ctx.accounts.quote_token_program.to_account_info();
    // paying fees(in quote)
    transfer_tokens_checked(
        from.clone(),
        ctx.accounts.fee_receiver_quote_ata.to_account_info(),
        authority.clone(),
        mint.clone(),
        token_program.clone(),
        fee,
        decimals,
        None,
    )?;
    // paying quotes
    transfer_tokens_checked(
        from.clone(),
        ctx.accounts.quote_vault.to_account_info(),
        authority.clone(),
        mint.clone(),
        token_program.clone(),
        left_es_amount,
        decimals,
        None,
    )?;

    ctx.accounts.safety_check(left_es_amount, true)?;
    Ok(())
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct BuyWithReferralInput {
    pub es_amount: u64,
    pub referral_pubkey: Pubkey,
}

pub fn buy_with_referral(
    ctx: Context<ACommonExtReferral>,
    referral_address: Pubkey,
    es_amount: u64,
    min_mushi_out: Option<u64>,
) -> Result<()> {
    let mushi = ctx.accounts.common.eclipse_to_mushi(es_amount)?;
    
    let min_mushi_out = min_mushi_out.unwrap_or(mushi);
    require!(mushi >= min_mushi_out, MushiProgramError::TooSmallOutputAmount);

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
    let is_started = ctx.accounts.common.main_state.started;
    require!(is_started, MushiProgramError::NotStarted);

    require!(global_state.token_supply + mushi * ctx.accounts.common.main_state.buy_fee / FEE_BASE_1000 <= MAX_SUPPLY, MushiProgramError::MaxSupplyExceeded);
    // minting tokens
    mint_to_tokens_by_main_state(
        ctx.accounts.common.token.to_account_info(),
        ctx.accounts.common.main_state.to_account_info(),
        ctx.accounts.common.user_ata.to_account_info(),
        ctx.accounts.common.base_token_program.to_account_info(),
        mushi * ctx.accounts.common.main_state.buy_fee / FEE_BASE_1000,
        *ctx.bumps.get("main_state").unwrap(),
    )?;
    global_state.token_supply += mushi * ctx.accounts.common.main_state.buy_fee / FEE_BASE_1000;

    // calc sender Eclipses

    let fee_treasury: u64 = es_amount
        .checked_mul(FEES_BUY)
        .unwrap()
        .checked_div(10_000)
        .unwrap();
    let fee_referral: u64 = es_amount
        .checked_mul(FEES_BUY_REFERRAL)
        .unwrap()
        .checked_div(10_000)
        .unwrap();
    // let fee: u64 = es_amount.checked_div(FEES_BUY).unwrap();

    // require!(fee_treasury > MIN, MushiProgramError::TooSmallTeamFee);
    // require!(fee_referral > MIN, MushiProgramError::TooSmallTeamFee);
    require!(fee_treasury + fee_referral > MIN, MushiProgramError::TooSmallTeamFee);

    let left_es_amount = es_amount.checked_sub(fee_treasury + fee_referral).unwrap();
    {
        // sending quote token
        let authority = ctx.accounts.common.user.to_account_info();
        let from = ctx.accounts.common.user_quote_ata.to_account_info();
        let decimals = ctx.accounts.common.quote_mint.decimals;
        let mint = ctx.accounts.common.quote_mint.to_account_info();
        let token_program = ctx.accounts.common.quote_token_program.to_account_info();
        // paying fees(in quote)
        transfer_tokens_checked(
            from.clone(),
            ctx.accounts.common.fee_receiver_quote_ata.to_account_info(),
            authority.clone(),
            mint.clone(),
            token_program.clone(),
            fee_treasury,
            decimals,
            None,
        )?;
        
        transfer_tokens_checked(
            from.clone(),
            ctx.accounts.referral_quote_ata.to_account_info(),
            authority.clone(),
            mint.clone(),
            token_program.clone(),
            fee_referral,
            decimals,
            None,
        )?;

        transfer_tokens_checked(
            from.clone(),
            ctx.accounts.common.quote_vault.to_account_info(),
            authority.clone(),
            mint.clone(),
            token_program.clone(),
            left_es_amount,
            decimals,
            None,
        )?;
    }
    ctx.accounts.common.safety_check(left_es_amount, true)?;
    Ok(())
}

pub fn sell(ctx: Context<ACommon>, token_amount: u64, min_eclipse_out: Option<u64>) -> Result<()> {
    let es_amount = ctx.accounts.mushi_to_eclipse(token_amount)?;

    let min_eclipse_out = min_eclipse_out.unwrap_or(es_amount);
    require!(es_amount >= min_eclipse_out, MushiProgramError::TooSmallOutputAmount);

    let global_state = &mut ctx.accounts.global_state;
    liquidate(
        &mut ctx.accounts.last_liquidation_date_state,
        global_state,
        ctx.accounts.token_vault.to_account_info(),
        ctx.accounts.token.to_account_info(),
        ctx.accounts.token_vault_owner.to_account_info(),
        ctx.accounts.base_token_program.to_account_info(),
        *ctx.bumps.get("token_vault_owner").unwrap(),
    )?;
    let seller = ctx.accounts.user.to_account_info();
    // burn tokens
    burn_tokens(
        ctx.accounts.user_ata.to_account_info(),
        ctx.accounts.token.to_account_info(),
        seller.clone(),
        ctx.accounts.base_token_program.to_account_info(),
        token_amount,
        None,
    )?;

    global_state.token_supply -= token_amount;
    // calc & sending eclipses
    let system_program = ctx.accounts.system_program.to_account_info();
    let vault_owner = ctx.accounts.token_vault_owner.to_account_info();
    let signer_seeds: &[&[&[u8]]] =
        &[&[VAULT_SEED, &[*ctx.bumps.get("token_vault_owner").unwrap()]]];

    let eclipse_fee_amount = es_amount
        .checked_mul(FEES_SELL)
        .unwrap()
        .checked_div(10_000)
        .unwrap();

    require!(eclipse_fee_amount > MIN, MushiProgramError::TooSmallInputAmount);

    {
        // sending quote tokens
        let authority = ctx.accounts.token_vault_owner.to_account_info();
        let from = ctx.accounts.quote_vault.to_account_info();
        let mint = ctx.accounts.quote_mint.to_account_info();
        let decimals = ctx.accounts.quote_mint.decimals;
        let token_program = ctx.accounts.quote_token_program.to_account_info();
        // Payment to seller
        transfer_tokens_checked(
            from.clone(),
            ctx.accounts.user_quote_ata.to_account_info(),
            authority.clone(),
            mint.clone(),
            token_program.clone(),
            es_amount * ctx.accounts.main_state.sell_fee / FEE_BASE_1000,
            decimals,
            Some(signer_seeds),
        )?;

        // team fee
        if es_amount / FEES_SELL <= MIN {
            return Err(MushiProgramError::TooSmallInputAmount.into());
        }
        transfer_tokens_checked(
            from.clone(),
            ctx.accounts.fee_receiver_quote_ata.to_account_info(),
            authority.clone(),
            mint.clone(),
            token_program.clone(),
            eclipse_fee_amount,
            decimals,
            Some(signer_seeds),
        )?;
    }

    ctx.accounts.safety_check(es_amount, false)?;
    Ok(())
}

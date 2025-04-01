use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token_interface};

use crate::{
    constants::{FEES_BUY, FEES_SELL, FEE_BASE_1000, MIN, VAULT_SEED}, 
    error::MushiProgramError, 
    utils::{burn_tokens, calc_fee_amount, liquidate, mint_to_tokens_by_main_state, safety_check, trasnfer_sol}, 
    MainState
};

pub fn buy(ctx:Context<ABuySell>, sol_amount_in:u64) -> Result<()> {
    liquidate(
        ctx.accounts.main_state.to_account_info(),
        ctx.accounts.token.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        ctx.accounts.token_vault.to_account_info(),
        ctx.accounts.token_vault_owner.to_account_info(),
        *ctx.bumps.get("token_vault_owner").unwrap()
    )?;
    let fee = calc_fee_amount(sol_amount_in);
    if fee <= MIN {
        return Err(MushiProgramError::TooSmallInputAmount.into());
    }
    let buyer = ctx.accounts.user.to_account_info();
    let main_state =&mut ctx.accounts.main_state;
    let token_amount = main_state.calc_buy_amount(sol_amount_in)?;
    // minting tokens
    mint_to_tokens_by_main_state(
        ctx.accounts.token.to_account_info(), 
        main_state.to_account_info(), 
        ctx.accounts.user_ata.to_account_info(), 
        ctx.accounts.token_program.to_account_info(), 
        token_amount * main_state.buy_fee / FEE_BASE_1000, 
        *ctx.bumps.get("main_state").unwrap()
    )?;
    main_state.token_supply += token_amount * main_state.buy_fee / FEE_BASE_1000;
    // calc sender SOLs
    let left_sol_amount = sol_amount_in.checked_sub(fee).unwrap();
    let system_program = ctx.accounts.system_program.to_account_info();
    trasnfer_sol(buyer.to_account_info(), 
    ctx.accounts.fee_receiver.to_account_info(), 
    system_program.to_account_info(), 
    fee, 
    None)?;
    trasnfer_sol(
        buyer.to_account_info(), 
        ctx.accounts.token_vault_owner.to_account_info(), 
        system_program.to_account_info(), 
        left_sol_amount, 
        None)?;
    safety_check(
        main_state.to_account_info(),
        ctx.accounts.token_vault_owner.to_account_info(),
    )?;
    Ok(())
}

pub fn sell(ctx:Context<ABuySell>, token_amount:u64)->Result<()>{
    liquidate(
        ctx.accounts.main_state.to_account_info(),
        ctx.accounts.token.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        ctx.accounts.token_vault.to_account_info(),
        ctx.accounts.token_vault_owner.to_account_info(),
        *ctx.bumps.get("token_vault_owner").unwrap()
    )?;
    let main_state =&mut ctx.accounts.main_state;
    let seller = ctx.accounts.user.to_account_info();
    let sol_amount = main_state.calc_sell_amount(token_amount)?;
    // burn tokens
    burn_tokens(
        ctx.accounts.user_ata.to_account_info(), 
        ctx.accounts.token.to_account_info(), 
        seller.to_account_info(), 
        ctx.accounts.token_program.to_account_info(), 
        token_amount, 
        None)?; 
    
    main_state.token_supply -= token_amount;
    // calc & sending sol
    let system_program = ctx.accounts.system_program.to_account_info();
    let vault_owner = ctx.accounts.token_vault_owner.to_account_info();
    let signer_seeds:&[&[&[u8]]] = &[&[VAULT_SEED, &[*ctx.bumps.get("token_vault_owner").unwrap()]]];
    // payment to sender
    trasnfer_sol(
        vault_owner.to_account_info(), 
        ctx.accounts.fee_receiver.to_account_info(), 
        system_program.to_account_info(), 
        sol_amount * main_state.sell_fee / FEE_BASE_1000, 
        Some(signer_seeds))?;
    // team fee
    if sol_amount / FEES_SELL <= MIN {
        return Err(MushiProgramError::TooSmallInputAmount.into());
    }
    trasnfer_sol(
        vault_owner.to_account_info(), 
        seller.to_account_info(), 
        system_program.to_account_info(), 
        sol_amount / FEES_SELL, 
        Some(signer_seeds))?;
    safety_check(
        main_state.to_account_info(),
        ctx.accounts.token_vault_owner.to_account_info(),
    )?;
    Ok(())
}

#[derive(Accounts)]
pub struct ABuySell<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(    
        mut,
        seeds = [MainState::PREFIX_SEED],
        bump,
    )]
    pub main_state: Box<Account<'info, MainState>>,
    #[account(
        mut,
        address=main_state.fee_receiver,
    )]
    pub fee_receiver:SystemAccount<'info>,

    #[account(
        mut,
        address = main_state.token,
    )]
    pub token: Box<InterfaceAccount<'info, token_interface::Mint>>,
    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = token,
        associated_token::authority = user,
    )]
    pub user_ata: Box<InterfaceAccount<'info, token_interface::TokenAccount>>,
    #[account(
        mut,
        seeds = [VAULT_SEED],
        bump,
    )]
    pub token_vault_owner: SystemAccount<'info>,
    #[account(
        mut,
        token::mint = token,
        token::authority = token_vault_owner,
    )]
    pub token_vault: Box<InterfaceAccount<'info, token_interface::TokenAccount>>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, token_interface::TokenInterface>,
    pub system_program: Program<'info, System>,
}

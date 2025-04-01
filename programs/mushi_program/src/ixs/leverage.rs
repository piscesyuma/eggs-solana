use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token_interface};

use crate::{
    constants::{FEES_BUY, FEES_SELL, FEE_BASE_1000, MIN, VAULT_SEED, SECONDS_IN_A_DAY}, 
    error::MushiProgramError, 
    utils::{burn_tokens, calc_fee_amount, liquidate, mint_to_tokens_by_main_state, safety_check, trasnfer_sol}, 
    MainState,
    LoanState,
};

pub fn leverage(ctx:Context<ALeverage>, sol_amount_in: u64, sol_amount:u64, number_of_days: u64)->Result<()>{
    if number_of_days >= 366 {
        return Err(MushiProgramError::InvalidNumberOfDays.into());
    }
    
    let main_state = &mut ctx.accounts.main_state;
    let loan_state = &mut ctx.accounts.loan_state;
    let user = ctx.accounts.user.to_account_info();
    let fee_receiver = ctx.accounts.fee_receiver.to_account_info();
    let token = ctx.accounts.token.to_account_info();

    // Get user's loan information using their address
    let user_key = user.key();
    let (collateral, borrowed, end_date, number_of_days) = loan_state.get_loan_by_address(user_key);
    
    if borrowed != 0 {
        if loan_state.is_loan_expired(user_key) {
            // Delete the expired loan
            loan_state.delete_loan(user_key);
        }
    }

    // Calculate fees and amounts
    let sonic_fee = loan_state.leverage_fee(
        sol_amount,
        number_of_days,
        main_state.buy_fee_leverage
    )?;

    liquidate(
        main_state.to_account_info(),
        ctx.accounts.token.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        ctx.accounts.token_vault.to_account_info(),
        ctx.accounts.token_vault_owner.to_account_info(),
        *ctx.bumps.get("token_vault_owner").unwrap()
    )?;
    let end_date = main_state.get_midnight_timestamp(
        (number_of_days as u64 * SECONDS_IN_A_DAY as u64) + Clock::get().unwrap().unix_timestamp as u64
    )?;
    let sonic_fee = loan_state.leverage_fee(
        sol_amount,
        number_of_days,
        main_state.buy_fee_leverage
    )?;
    let user_sonic = sol_amount.checked_sub(sonic_fee).unwrap();
    let fee_address_amount = sonic_fee.checked_mul(3).unwrap().checked_div(10).unwrap();
    let user_borrow = user_sonic.checked_mul(99).unwrap().checked_div(100).unwrap();
    let over_collateralization_amount = user_sonic.checked_div(100).unwrap();
    let sub_value = fee_address_amount.checked_add(over_collateralization_amount).unwrap();
    let total_fee = sonic_fee.checked_add(over_collateralization_amount).unwrap();
    let mut fee_overage = total_fee.checked_sub(fee_address_amount).unwrap();
    if sol_amount_in > total_fee {
        fee_overage = sol_amount_in.checked_sub(total_fee).unwrap();
        let signer_seeds:&[&[&[u8]]] = &[&[VAULT_SEED, &[*ctx.bumps.get("token_vault_owner").unwrap()]]];
        trasnfer_sol(
            ctx.accounts.token_vault_owner.to_account_info(), 
            ctx.accounts.user.to_account_info(), 
            ctx.accounts.system_program.to_account_info(), 
            fee_overage, 
            Some(signer_seeds))?;
    }
    if sol_amount_in - fee_overage != total_fee {
        return Err(MushiProgramError::InvalidFeeAmount.into());
    }
    // AUDIT: to user round 
    let user_eggs = main_state.calc_buy_amount_lev(user_sonic, sub_value)?;
    let mint = ctx.accounts.token.to_account_info();
    mint_to_tokens_by_main_state(
        mint.to_account_info(),
        main_state.to_account_info(),
        ctx.accounts.token_vault.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        user_eggs,
        *ctx.bumps.get("main_state").unwrap(),
    )?;
    main_state.token_supply += user_eggs;
    if fee_address_amount <= MIN {
        return Err(MushiProgramError::InvalidFeeAmount.into());
    }
    trasnfer_sol(
        ctx.accounts.token_vault_owner.to_account_info(), 
        ctx.accounts.fee_receiver.to_account_info(), 
        ctx.accounts.system_program.to_account_info(), 
        fee_address_amount, 
        None)?;
    main_state.add_loans_by_date(user_borrow, user_eggs, end_date)?;
    loan_state.set_loan(user_key, user_eggs, user_borrow, end_date, number_of_days);
    safety_check(
        main_state.to_account_info(),
        ctx.accounts.token_vault_owner.to_account_info(),
    )?;
    Ok(())
}
#[derive(Accounts)]
pub struct ALeverage<'info> {
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
        seeds = [LoanState::PREFIX_SEED],
        bump,
    )]
    pub loan_state: Box<Account<'info, LoanState>>,
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
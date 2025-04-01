use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token_interface};

use crate::{
    constants::{FEES_BUY, FEES_SELL, FEE_BASE_1000, MIN, VAULT_SEED, SECONDS_IN_A_DAY}, 
    error::MushiProgramError, 
    utils::{burn_tokens, calc_fee_amount, liquidate, mint_to_tokens_by_main_state, safety_check, trasnfer_sol, transfer_tokens}, 
    MainState,
    LoanState,
};

pub fn borrow(ctx:Context<ABorrow>, sol_amount:u64, number_of_days: u64)->Result<()>{
    if number_of_days >= 366 {
        return Err(MushiProgramError::InvalidNumberOfDays.into());
    }
    
    if sol_amount == 0 {
        return Err(MushiProgramError::InvalidSolAmount.into());
    }
    let main_state = &mut ctx.accounts.main_state;
    let loan_state = &mut ctx.accounts.loan_state;
    let user = ctx.accounts.user.to_account_info();
    let fee_receiver = ctx.accounts.fee_receiver.to_account_info();
    let token = ctx.accounts.token.to_account_info();

    if loan_state.is_loan_expired(user.key()) {
        loan_state.delete_loan(user.key());
    }

    // Get user's loan information using their address
    let user_key = user.key();
    let (collateral, borrowed, end_date, number_of_days) = loan_state.get_loan_by_address(user_key);

    if borrowed != 0 {
        return Err(MushiProgramError::InvalidLoanAmount.into());
    }
    
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
    let sonic_fee = loan_state.get_interest_fee(sol_amount, number_of_days)?;
    let fee_address_fee = sonic_fee.checked_mul(3).unwrap().checked_div(10).unwrap();

    //AUDIT: eggs required from user round up?
    let user_eggs = main_state.calc_buy_amount_no_trade_ceil(sol_amount)?;
    let new_user_borrow = sol_amount.checked_mul(99).unwrap().checked_div(100).unwrap();
    loan_state.set_loan(user.key(), user_eggs, new_user_borrow, end_date, number_of_days);

    transfer_tokens(
        ctx.accounts.user_ata.to_account_info(),
        ctx.accounts.token_vault_owner.to_account_info(),
        main_state.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        user_eggs,
        None,
    )?;
    
    if fee_address_fee <= MIN {
        return Err(MushiProgramError::InvalidFeeAmount.into());
    }
    let signer_seeds:&[&[&[u8]]] = &[&[VAULT_SEED, &[*ctx.bumps.get("token_vault_owner").unwrap()]]];
    trasnfer_sol(
        ctx.accounts.token_vault_owner.to_account_info(), 
        ctx.accounts.user.to_account_info(), 
        ctx.accounts.system_program.to_account_info(), 
        new_user_borrow - sonic_fee, 
        Some(signer_seeds))?;
    trasnfer_sol(
        ctx.accounts.token_vault_owner.to_account_info(), 
        ctx.accounts.fee_receiver.to_account_info(), 
        ctx.accounts.system_program.to_account_info(), 
        fee_address_fee, 
        Some(signer_seeds))?;
    main_state.add_loans_by_date(new_user_borrow, user_eggs, end_date)?;
    safety_check(
        main_state.to_account_info(),
        ctx.accounts.token_vault_owner.to_account_info(),
    )?;
    Ok(())
}

pub fn borrow_more(ctx:Context<ABorrow>, sol_amount:u64)->Result<()>{
    let main_state = &mut ctx.accounts.main_state;
    let loan_state = &mut ctx.accounts.loan_state;
    let user = ctx.accounts.user.to_account_info();
    let fee_receiver = ctx.accounts.fee_receiver.to_account_info();
    let token = ctx.accounts.token.to_account_info();

    if loan_state.is_loan_expired(user.key()) {
        return Err(MushiProgramError::InvalidLoanAmount.into());
    }
    if sol_amount == 0 {
        return Err(MushiProgramError::InvalidSolAmount.into());
    }
    liquidate(
        main_state.to_account_info(),
        ctx.accounts.token.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        ctx.accounts.token_vault.to_account_info(),
        ctx.accounts.token_vault_owner.to_account_info(),
        *ctx.bumps.get("token_vault_owner").unwrap()
    )?;
    let user_key = user.key();
    let (user_collateral, user_borrowed, user_end_date, number_of_days) = loan_state.get_loan_by_address(user_key);

    let today_midnight = main_state.get_midnight_timestamp(Clock::get().unwrap().unix_timestamp as u64)?;
    let new_borrow_length = (user_end_date as u64).checked_sub(today_midnight).unwrap().checked_div(SECONDS_IN_A_DAY as u64).unwrap();
    let sonic_fee = loan_state.get_interest_fee(sol_amount, new_borrow_length)?;

    // AUDIT: eggs required from user round up?
    let user_eggs = main_state.calc_buy_amount_no_trade_ceil(sol_amount)?;
    let user_borrowed_in_eggs = main_state.calc_buy_amount_no_trade(user_borrowed)?;
    let user_excess_in_eggs = (user_collateral.checked_mul(99).unwrap().checked_div(100).unwrap()).checked_sub(user_borrowed_in_eggs).unwrap();
    let mut require_collateral_from_user = user_eggs;
    
    if user_excess_in_eggs >= user_eggs {
        require_collateral_from_user = 0;
    } else {
        require_collateral_from_user -= user_excess_in_eggs;
    }
    let fee_address_fee = sonic_fee.checked_mul(3).unwrap().checked_div(10).unwrap();
    let new_user_borrow = sol_amount.checked_mul(99).unwrap().checked_div(100).unwrap();
    let new_user_borrow_total = user_borrowed.checked_add(new_user_borrow).unwrap();
    let new_user_collateral_total = user_collateral.checked_add(require_collateral_from_user).unwrap();

    loan_state.set_loan(user.key(), new_user_collateral_total, new_user_borrow_total, user_end_date, new_borrow_length);
    
    if require_collateral_from_user != 0 {
        transfer_tokens(
            ctx.accounts.user_ata.to_account_info(),
            ctx.accounts.token_vault_owner.to_account_info(),
            main_state.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
            require_collateral_from_user,
            None,
        )?;
    }
    if fee_address_fee <= MIN {
        return Err(MushiProgramError::InvalidFeeAmount.into());
    }
    let signer_seeds:&[&[&[u8]]] = &[&[VAULT_SEED, &[*ctx.bumps.get("token_vault_owner").unwrap()]]];
    trasnfer_sol(
        ctx.accounts.token_vault_owner.to_account_info(), 
        ctx.accounts.user.to_account_info(), 
        ctx.accounts.system_program.to_account_info(), 
        new_user_borrow - sonic_fee, 
        Some(signer_seeds))?;
    trasnfer_sol(
        ctx.accounts.token_vault_owner.to_account_info(), 
        ctx.accounts.fee_receiver.to_account_info(), 
        ctx.accounts.system_program.to_account_info(), 
        fee_address_fee, 
        Some(signer_seeds))?;
    main_state.add_loans_by_date(new_user_borrow, require_collateral_from_user, user_end_date)?;
    safety_check(
        main_state.to_account_info(),
        ctx.accounts.token_vault_owner.to_account_info(),
    )?;
    Ok(())
}

pub fn remove_collateral(ctx:Context<ABorrow>, amount:u64)->Result<()>{
    let main_state = &mut ctx.accounts.main_state;
    let loan_state = &mut ctx.accounts.loan_state;
    let user = ctx.accounts.user.to_account_info();
    let fee_receiver = ctx.accounts.fee_receiver.to_account_info();
    let token = ctx.accounts.token.to_account_info();

    if loan_state.is_loan_expired(user.key()) {
        return Err(MushiProgramError::InvalidLoanAmount.into());
    }
    if amount == 0 {
        return Err(MushiProgramError::InvalidSolAmount.into());
    }
    liquidate(
        main_state.to_account_info(),
        ctx.accounts.token.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        ctx.accounts.token_vault.to_account_info(),
        ctx.accounts.token_vault_owner.to_account_info(),
        *ctx.bumps.get("token_vault_owner").unwrap()
    )?;
    let user_key = user.key();
    let (collateral, borrowed, end_date, number_of_days) = loan_state.get_loan_by_address(user_key);
    if borrowed > main_state.calc_sell_amount(collateral - amount).unwrap() * 99 / 100 {
        return Err(MushiProgramError::InvalidLoanAmount.into());
    }
    loan_state.set_loan(user.key(), collateral - amount, borrowed, end_date, number_of_days);

    let signer_seeds:&[&[&[u8]]] = &[&[VAULT_SEED, &[*ctx.bumps.get("token_vault_owner").unwrap()]]];
    transfer_tokens(
        ctx.accounts.token_vault_owner.to_account_info(),
        ctx.accounts.user_ata.to_account_info(),
        main_state.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        amount,
        Some(signer_seeds),
    )?;

    main_state.sub_loans_by_date(0, amount, end_date)?;
    safety_check(
        main_state.to_account_info(),
        ctx.accounts.token_vault_owner.to_account_info(),
    )?;
    Ok(())
}


pub fn repay(ctx:Context<ABorrow>, sol_amount:u64)->Result<()>{
    let main_state = &mut ctx.accounts.main_state;
    let loan_state = &mut ctx.accounts.loan_state;
    let user = ctx.accounts.user.to_account_info();
    let fee_receiver = ctx.accounts.fee_receiver.to_account_info();
    let token = ctx.accounts.token.to_account_info();

    if loan_state.is_loan_expired(user.key()) {
        return Err(MushiProgramError::InvalidLoanAmount.into());
    }
    if sol_amount == 0 {
        return Err(MushiProgramError::InvalidSolAmount.into());
    }
    let user_key = user.key();
    let (collateral, borrowed, end_date, number_of_days) = loan_state.get_loan_by_address(user_key);
    if borrowed <= sol_amount {
        return Err(MushiProgramError::InvalidLoanAmount.into());
    }
    let new_borrow = borrowed - sol_amount;
    loan_state.set_loan(user.key(), collateral, new_borrow, end_date, number_of_days);
    main_state.sub_loans_by_date(sol_amount, 0, end_date)?;
    safety_check(
        main_state.to_account_info(),
        ctx.accounts.token_vault_owner.to_account_info(),
    )?;
    Ok(())
}

pub fn close_position(ctx:Context<ABorrow>, sol_amount:u64)->Result<()>{
    let main_state = &mut ctx.accounts.main_state;
    let loan_state = &mut ctx.accounts.loan_state;
    let user = ctx.accounts.user.to_account_info();
    let fee_receiver = ctx.accounts.fee_receiver.to_account_info();
    let token = ctx.accounts.token.to_account_info();

    if loan_state.is_loan_expired(user.key()) {
        return Err(MushiProgramError::InvalidLoanAmount.into());
    }
    let user_key = user.key();
    let (collateral, borrowed, end_date, number_of_days) = loan_state.get_loan_by_address(user_key);
    if borrowed != sol_amount {
        return Err(MushiProgramError::InvalidLoanAmount.into());
    }
    let signer_seeds:&[&[&[u8]]] = &[&[VAULT_SEED, &[*ctx.bumps.get("token_vault_owner").unwrap()]]];
    transfer_tokens(
        ctx.accounts.token_vault_owner.to_account_info(),
        ctx.accounts.user_ata.to_account_info(),
        main_state.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        collateral,
        Some(signer_seeds),
    )?;
    main_state.sub_loans_by_date(borrowed, collateral, end_date)?;
    loan_state.delete_loan(user_key);
    safety_check(
        main_state.to_account_info(),
        ctx.accounts.token_vault_owner.to_account_info(),
    )?;
    Ok(())
}

pub fn flash_close_position(ctx:Context<ABorrow>)->Result<()>{
    let main_state = &mut ctx.accounts.main_state;
    let loan_state = &mut ctx.accounts.loan_state;
    let user = ctx.accounts.user.to_account_info();
    let fee_receiver = ctx.accounts.fee_receiver.to_account_info();
    let token = ctx.accounts.token.to_account_info();

    if loan_state.is_loan_expired(user.key()) {
        return Err(MushiProgramError::InvalidLoanAmount.into());
    }
    liquidate(
        main_state.to_account_info(),
        ctx.accounts.token.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        ctx.accounts.token_vault.to_account_info(),
        ctx.accounts.token_vault_owner.to_account_info(),
        *ctx.bumps.get("token_vault_owner").unwrap()
    )?;
    let user_key = user.key();
    let (collateral, borrowed, end_date, number_of_days) = loan_state.get_loan_by_address(user_key);
    
    // AUDIT: from user round up
    let collateral_in_sol = main_state.calc_sell_amount(collateral)?;
    let signer_seeds:&[&[&[u8]]] = &[&[VAULT_SEED, &[*ctx.bumps.get("token_vault_owner").unwrap()]]];
    burn_tokens(
        ctx.accounts.token_vault.to_account_info(), 
        ctx.accounts.token.to_account_info(), 
        ctx.accounts.token_vault_owner.to_account_info(), 
        ctx.accounts.token_program.to_account_info(), 
        collateral, 
        Some(signer_seeds))?;
    main_state.token_supply -= collateral;
    let collateral_in_sol_after_fee = (collateral_in_sol * 99) / 100;
    let fee = collateral_in_sol / 100;
    if collateral_in_sol_after_fee < borrowed {
        return Err(MushiProgramError::InvalidLoanAmount.into());
    }
    let to_user = collateral_in_sol_after_fee - borrowed;
    let fee_address_fee = fee.checked_mul(3).unwrap().checked_div(10).unwrap();
    let signer_seeds:&[&[&[u8]]] = &[&[VAULT_SEED, &[*ctx.bumps.get("token_vault_owner").unwrap()]]];
    trasnfer_sol(
        ctx.accounts.token_vault_owner.to_account_info(), 
        ctx.accounts.user.to_account_info(), 
        ctx.accounts.system_program.to_account_info(), 
        to_user,
         Some(signer_seeds))?;
    if fee_address_fee <= MIN {
        return Err(MushiProgramError::InvalidFeeAmount.into());
    }
    trasnfer_sol(
        ctx.accounts.token_vault_owner.to_account_info(), 
        ctx.accounts.fee_receiver.to_account_info(), 
        ctx.accounts.system_program.to_account_info(), 
        fee_address_fee,
        Some(signer_seeds))?;
    main_state.sub_loans_by_date(borrowed, collateral, end_date)?;
    loan_state.delete_loan(user_key);
    safety_check(
        main_state.to_account_info(),
        ctx.accounts.token_vault_owner.to_account_info(),
    )?;
    Ok(())
}

pub fn extend_loan(ctx:Context<ABorrow>, sol_amount: u64,number_of_days:u64)->Result<u64>{
    let main_state = &mut ctx.accounts.main_state;
    let loan_state = &mut ctx.accounts.loan_state;
    let user = ctx.accounts.user.to_account_info();
    let fee_receiver = ctx.accounts.fee_receiver.to_account_info();
    let token = ctx.accounts.token.to_account_info();

    let user_key = user.key();
    let (collateral, borrowed, old_end_date, _number_of_days) = loan_state.get_loan_by_address(user_key);
    
    let new_end_date = old_end_date + (number_of_days as u64 * SECONDS_IN_A_DAY as u64);
    let loan_fee = loan_state.get_interest_fee(borrowed, number_of_days)?;
    if loan_state.is_loan_expired(user.key()) {
        return Err(MushiProgramError::InvalidLoanAmount.into());
    }
    if loan_fee != sol_amount {
        return Err(MushiProgramError::InvalidLoanAmount.into());
    }
    let fee_address_fee = loan_fee.checked_mul(3).unwrap().checked_div(10).unwrap();
    if fee_address_fee <= MIN { 
        return Err(MushiProgramError::InvalidFeeAmount.into());
    }
    let signer_seeds:&[&[&[u8]]] = &[&[VAULT_SEED, &[*ctx.bumps.get("token_vault_owner").unwrap()]]];
    trasnfer_sol(
        ctx.accounts.token_vault_owner.to_account_info(), 
        ctx.accounts.fee_receiver.to_account_info(), 
        ctx.accounts.system_program.to_account_info(), 
        fee_address_fee,
        Some(signer_seeds))?;
    main_state.sub_loans_by_date(borrowed, collateral, old_end_date)?;
    main_state.add_loans_by_date(borrowed, collateral, new_end_date)?;
    loan_state.set_loan(user_key, collateral, borrowed, new_end_date, number_of_days + _number_of_days);
    if (new_end_date as i64 - Clock::get().unwrap().unix_timestamp) / SECONDS_IN_A_DAY >= 366 {
        return Err(MushiProgramError::InvalidLoanAmount.into());
    }
    safety_check(
        main_state.to_account_info(),
        ctx.accounts.token_vault_owner.to_account_info(),
    )?;
    Ok(loan_fee)
}

#[derive(Accounts)]
pub struct ABorrow<'info> {
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
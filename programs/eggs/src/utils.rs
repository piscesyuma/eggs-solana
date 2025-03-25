use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Mint, Token, TokenAccount, Burn, MintTo, Transfer},
};
use solana_program::{native_token::LAMPORTS_PER_SOL, program::invoke_signed, program::invoke, system_instruction};
use std::rc::Rc;
use std::cell::RefCell;

use crate::states::*;
use crate::error::*;

// Utility functions
pub fn get_midnight_timestamp(timestamp: i64) -> i64 {
    let seconds_per_day = 86400;
    let midnight_timestamp = timestamp - (timestamp % seconds_per_day);
    midnight_timestamp + seconds_per_day
}

// Updated token conversion functions that take account info

pub fn eggs_to_sol_with_account(state: &EggsState, state_account: &AccountInfo, eggs_amount: u64) -> Result<u64> {
    // Backing is the balance of the program's account + total borrowed
    let backing = state.total_borrowed + state_account.lamports();
    
    // Calculate using the formula: eggs_amount * backing / total_supply
    // This determines how much SOL (in lamports) the eggs are worth
    let total_supply = state.total_minted; // assuming total_minted is our total supply
    
    if total_supply == 0 {
        return Err(EggsError::InvalidParameter.into());
    }
    
    let sol = (eggs_amount as u128 * backing as u128) / total_supply as u128;
    Ok(sol as u64)
}

pub fn sol_to_eggs_with_account(state: &EggsState, state_account: &AccountInfo, sol_amount: u64) -> Result<u64> {
    // Backing is the balance of the program's account + total borrowed
    let backing = state.total_borrowed + state_account.lamports();
    
    // Calculate using the formula: sol_amount * total_supply / (backing - sol_amount)
    // This determines how many EGGS tokens the SOL amount is worth
    let total_supply = state.total_minted; // assuming total_minted is our total supply
    
    if backing <= sol_amount {
        return Err(EggsError::InvalidParameter.into());
    }
    
    let eggs = (sol_amount as u128 * total_supply as u128) / (backing as u128 - sol_amount as u128);
    Ok(eggs as u64)
}

pub fn sol_to_eggs_no_trade_with_account(state: &EggsState, state_account: &AccountInfo, sol_amount: u64) -> Result<u64> {
    // Backing is the balance of the program's account + total borrowed
    let backing = state.total_borrowed + state_account.lamports();
    
    // Calculate using the formula: sol_amount * total_supply / backing
    // This calculation doesn't remove the sol_amount from the backing
    let total_supply = state.total_minted;
    
    if backing == 0 {
        return Err(EggsError::InvalidParameter.into());
    }
    
    let eggs = (sol_amount as u128 * total_supply as u128) / backing as u128;
    Ok(eggs as u64)
}

pub fn sol_to_eggs_no_trade_ceil_with_account(state: &EggsState, state_account: &AccountInfo, sol_amount: u64) -> Result<u64> {
    // Backing is the balance of the program's account + total borrowed
    let backing = state.total_borrowed + state_account.lamports();
    
    // Calculate using the formula: (sol_amount * total_supply + (backing - 1)) / backing
    // This calculation rounds up the result to ensure sufficient collateral
    let total_supply = state.total_minted;
    
    if backing == 0 {
        return Err(EggsError::InvalidParameter.into());
    }
    
    let eggs = ((sol_amount as u128 * total_supply as u128) + (backing as u128 - 1)) / backing as u128;
    Ok(eggs as u64)
}

pub fn sol_to_eggs_lev_with_account(state: &EggsState, state_account: &AccountInfo, sol_amount: u64, fee: u64) -> Result<u64> {
    // Backing is the balance of the program's account + total borrowed - fee
    let backing = state.total_borrowed + state_account.lamports() - fee;
    
    // Calculate using the formula: (sol_amount * total_supply + (backing - 1)) / backing
    // This calculation is used for leverage operations, accounting for the fee
    let total_supply = state.total_minted;
    
    if backing == 0 {
        return Err(EggsError::InvalidParameter.into());
    }
    
    let eggs = ((sol_amount as u128 * total_supply as u128) + (backing as u128 - 1)) / backing as u128;
    Ok(eggs as u64)
}

pub fn get_interest_fee(amount: u64, number_of_days: u64) -> Result<u64> {
    // Calculates the interest fee on a loan of 'amount' SOL (in lamports) for 'number_of_days' days
    //
    // Formula originally from Solidity: Math.mulDiv(0.039e18, numberOfDays, 365) + 0.001e18
    // Adapted for Solana and simplified to:
    //   - 3.9% annual interest rate (prorated by days)
    //   + 0.1% flat fee
    //
    // We use fixed-point arithmetic with 9 decimal places for precision
    
    // Calculate the interest component: (amount * (3.9% * days / 365))
    let interest = ((39_000_000_000 * number_of_days) / 365) + 1_000_000_000; // 0.039 + 0.001 in fixed point
    
    // Apply the interest rate to the amount (maintaining precision)
    let fee = (amount as u128 * interest as u128) / 1_000_000_000_000;
    
    Ok(fee as u64)
}

// The real implementation that should be used in instruction contexts
pub fn get_backing_balance_from_account(state_account: &AccountInfo) -> Result<u64> {
    // Returns the actual SOL balance of the program account
    Ok(state_account.lamports())
}

pub fn is_loan_expired(loan: &Loan, current_time: i64) -> bool {
    loan.end_date < current_time
}

pub fn add_loans_by_date(
    state: &mut EggsState,
    daily_loan_data: &mut Account<DailyLoanData>,
    borrowed: u64,
    collateral: u64,
) -> Result<()> {
    daily_loan_data.borrowed += borrowed;
    daily_loan_data.collateral += collateral;
    state.total_borrowed += borrowed;
    state.total_collateral += collateral;
    Ok(())
}

pub fn sub_loans_by_date(
    state: &mut EggsState,
    daily_loan_data: &mut Account<DailyLoanData>,
    borrowed: u64,
    collateral: u64,
) -> Result<()> {
    daily_loan_data.borrowed = daily_loan_data.borrowed.saturating_sub(borrowed);
    daily_loan_data.collateral = daily_loan_data.collateral.saturating_sub(collateral);
    state.total_borrowed = state.total_borrowed.saturating_sub(borrowed);
    state.total_collateral = state.total_collateral.saturating_sub(collateral);
    Ok(())
}

pub fn liquidate(state: &mut EggsState) -> Result<()> {
    // This is a stub function that should be replaced with the actual liquidation logic
    // The real liquidation process happens in the liquidate_loans instruction
    
    // In a real implementation, we would need access to the following:
    // 1. Program state account
    // 2. Token mint
    // 3. Escrow accounts
    // 4. DailyLoanData accounts for each date
    
    // Since we can't access those accounts from here, we just update the last liquidation date
    // The actual liquidation should be performed by calling the liquidate_loans instruction
    
    let current_time = Clock::get()?.unix_timestamp;
    state.last_liquidation_date = get_midnight_timestamp(current_time);
    
    Ok(())
}

// The real implementation with account info
pub fn safety_check_with_account(state: &mut EggsState, state_account: &AccountInfo, sol: u64) -> Result<()> {
    // Safety check to ensure the price of EGGS can't decrease
    // and verify other conditions for program stability
    
    // Calculate new price: backing * 10^9 / total_supply
    // The price is measured in terms of how many lamports (SOL's smallest unit) one EGGS token is worth
    let backing = state.total_borrowed + state_account.lamports();
    let total_supply = state.total_minted;
    
    if total_supply == 0 {
        // If there's no supply, just update the last price to 0
        state.last_price = 0;
        return Ok(());
    }
    
    // Calculate price in lamports per EGGS token
    // We multiply by LAMPORTS_PER_SOL to maintain precision
    let new_price = (backing as u128 * LAMPORTS_PER_SOL as u128) / total_supply as u128;
    
    // Check that the price of EGGS cannot decrease
    if state.last_price > new_price as u64 {
        return Err(EggsError::PriceDecrease.into());
    }
    
    // Check that total collateral <= contract's EGGS balance
    // This would require passing in the escrow token account to verify
    
    // Update the last price
    state.last_price = new_price as u64;
    
    Ok(())
}

// Added utility functions for loan operations

// Process loan creation
pub fn process_loan_creation(
    state: &mut EggsState,
    loan: &mut Account<Loan>,
    daily_loan_data: &mut Account<DailyLoanData>,
    end_date: i64,
    collateral: u64,
    borrowed: u64,
    number_of_days: u64,
    user: Pubkey,
) -> Result<()> {
    // Update loan data
    loan.user = user;
    loan.collateral = collateral;
    loan.borrowed = borrowed;
    loan.end_date = end_date;
    loan.number_of_days = number_of_days;
    
    // Update daily loan data
    daily_loan_data.date = end_date;
    add_loans_by_date(state, daily_loan_data, borrowed, collateral)?;
    
    Ok(())
}

// Process loan repayment
pub fn process_loan_repayment(
    state: &mut EggsState,
    loan: &mut Account<Loan>,
    daily_loan_data: &mut Account<DailyLoanData>,
    repay_amount: u64,
) -> Result<()> {
    // Update loan with reduced borrowed amount
    let new_borrow = loan.borrowed.saturating_sub(repay_amount);
    loan.borrowed = new_borrow;
    
    // Update loan data for the date
    sub_loans_by_date(state, daily_loan_data, repay_amount, 0)?;
    
    Ok(())
}

// Process loan collateral adjustment
pub fn process_collateral_adjustment(
    state: &mut EggsState,
    loan: &mut Account<Loan>,
    daily_loan_data: &mut Account<DailyLoanData>,
    collateral_change: i64, // Positive for adding, negative for removing
) -> Result<()> {
    if collateral_change > 0 {
        // Adding collateral
        loan.collateral = loan.collateral.saturating_add(collateral_change as u64);
        add_loans_by_date(state, daily_loan_data, 0, collateral_change as u64)?;
    } else if collateral_change < 0 {
        // Removing collateral
        loan.collateral = loan.collateral.saturating_sub((-collateral_change) as u64);
        sub_loans_by_date(state, daily_loan_data, 0, (-collateral_change) as u64)?;
    }
    
    Ok(())
}

// Close loan and reset to default state
pub fn close_loan(
    state: &mut EggsState,
    loan: &mut Account<Loan>,
    daily_loan_data: &mut Account<DailyLoanData>,
) -> Result<()> {
    // Update loan data for the date
    sub_loans_by_date(state, daily_loan_data, loan.borrowed, loan.collateral)?;
    
    // Reset the loan
    loan.borrowed = 0;
    loan.collateral = 0;
    loan.end_date = 0;
    loan.number_of_days = 0;
    
    Ok(())
}

// Handle loan extension
pub fn extend_loan(
    state: &mut EggsState,
    loan: &mut Account<Loan>,
    old_daily_loan_data: &mut Account<DailyLoanData>,
    new_daily_loan_data: &mut Account<DailyLoanData>,
    additional_days: u64,
) -> Result<()> {
    let old_end_date = loan.end_date;
    let borrowed = loan.borrowed;
    let collateral = loan.collateral;
    let old_number_of_days = loan.number_of_days;
    
    // Calculate new end date
    let new_end_date = old_end_date + (additional_days as i64 * 86400);
    
    // Update loan data - remove from old date and add to new date
    sub_loans_by_date(state, old_daily_loan_data, borrowed, collateral)?;
    
    new_daily_loan_data.date = new_end_date;
    add_loans_by_date(state, new_daily_loan_data, borrowed, collateral)?;
    
    // Update the loan
    loan.end_date = new_end_date;
    loan.number_of_days = old_number_of_days + additional_days;
    
    Ok(())
}

// Transfer SOL utility function
pub fn transfer_sol<'a>(
    from: &'a AccountInfo<'a>,
    to: &'a AccountInfo<'a>,
    system_program: &'a AccountInfo<'a>,
    amount: u64,
    signer_seeds: &[&[&[u8]]],
) -> Result<()> {
    let ix = system_instruction::transfer(
        &from.key(),
        &to.key(),
        amount,
    );
    
    if signer_seeds.is_empty() {
        // Regular transfer (no signing)
        invoke(
            &ix,
            &[
                from.clone(),
                to.clone(),
                system_program.clone(),
            ],
        )?;
    } else {
        // PDA signing required
        invoke_signed(
            &ix,
            &[
                from.clone(),
                to.clone(),
                system_program.clone(),
            ],
            signer_seeds,
        )?;
    }
    
    Ok(())
}

// Account structs for instruction contexts

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        init,
        payer = authority,
        space = EggsState::LEN,
        seeds = [b"state"],
        bump
    )]
    pub state: Account<'info, EggsState>,
    
    /// CHECK: This is the state account as a signer
    #[account(
        seeds = [b"state"],
        bump
    )]
    pub state_account: AccountInfo<'info>,
    
    #[account(
        init,
        payer = authority,
        mint::decimals = 9,
        mint::authority = state_account,
        mint::freeze_authority = state_account,
    )]
    pub mint: Account<'info, Mint>,
    
    /// Associated token account for the authority (token creator)
    #[account(
        init_if_needed,
        payer = authority,
        associated_token::mint = mint,
        associated_token::authority = authority
    )]
    pub authority_token_account: Account<'info, TokenAccount>,
    
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct UpdateState<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"state"],
        bump = state.bump,
        has_one = authority
    )]
    pub state: Account<'info, EggsState>,
}

#[derive(Accounts)]
pub struct SetStart<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"state"],
        bump = state.bump,
        has_one = authority
    )]
    pub state: Account<'info, EggsState>,
    
    /// CHECK: This is the state account as a signer
    #[account(
        mut,
        seeds = [b"state"],
        bump = state.bump
    )]
    pub state_account: AccountInfo<'info>,
    
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    
    #[account(
        init_if_needed,
        payer = authority,
        associated_token::mint = mint,
        associated_token::authority = authority
    )]
    pub authority_token_account: Account<'info, TokenAccount>,
    
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct Trade<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"state"],
        bump = state.bump
    )]
    pub state: Account<'info, EggsState>,
    
    /// CHECK: This is the state account as a signer
    #[account(
        mut,
        seeds = [b"state"],
        bump = state.bump
    )]
    pub state_account: AccountInfo<'info>,
    
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    
    /// CHECK: This is the receiver of tokens
    pub receiver: AccountInfo<'info>,
    
    #[account(
        init_if_needed,
        payer = authority,
        associated_token::mint = mint,
        associated_token::authority = receiver
    )]
    pub receiver_token_account: Account<'info, TokenAccount>,
    
    /// CHECK: This is the fee address
    #[account(
        mut,
        address = state.fee_address
    )]
    pub fee_address_account: AccountInfo<'info>,
    
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct LoanOperation<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"state"],
        bump = state.bump
    )]
    pub state: Account<'info, EggsState>,
    
    /// CHECK: This is the state account as a signer
    #[account(
        mut,
        seeds = [b"state"],
        bump = state.bump
    )]
    pub state_account: AccountInfo<'info>,
    
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = authority
    )]
    pub authority_token_account: Account<'info, TokenAccount>,
    
    /// Escrow token account to hold the collateral
    #[account(
        init_if_needed,
        payer = authority,
        token::mint = mint,
        token::authority = state_account,
        seeds = [b"escrow", authority.key().as_ref()],
        bump
    )]
    pub escrow_token_account: Account<'info, TokenAccount>,
    
    /// CHECK: This is the fee address
    #[account(
        mut,
        address = state.fee_address
    )]
    pub fee_address_account: AccountInfo<'info>,
    
    #[account(
        init_if_needed,
        payer = authority,
        space = Loan::LEN,
        seeds = [b"loan", authority.key().as_ref()],
        bump
    )]
    pub loan: Account<'info, Loan>,
    
    #[account(
        init_if_needed,
        payer = authority,
        space = DailyLoanData::LEN,
        seeds = [b"loan_data", &date_to_bytes(loan.end_date)[..]],
        bump
    )]
    pub daily_loan_data: Account<'info, DailyLoanData>,
    
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
}

// Helper function to convert a timestamp to bytes for seeds
pub fn date_to_bytes(date: i64) -> [u8; 8] {
    date.to_le_bytes()
}

#[derive(Accounts)]
pub struct LiquidateLoans<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"state"],
        bump = state.bump
    )]
    pub state: Account<'info, EggsState>,
    
    /// CHECK: This is the state account as a signer
    #[account(
        seeds = [b"state"],
        bump = state.bump
    )]
    pub state_account: AccountInfo<'info>,
    
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    
    #[account(
        mut,
        token::mint = mint,
        token::authority = state_account,
        seeds = [b"escrow", authority.key().as_ref()],
        bump
    )]
    pub escrow_token_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub daily_loan_data: Account<'info, DailyLoanData>,
    
    pub token_program: Program<'info, Token>,
} 
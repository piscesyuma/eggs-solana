use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Mint, Token, TokenAccount},
};
use solana_program::{native_token::LAMPORTS_PER_SOL, program::invoke_signed, system_instruction};

use crate::states::*;
use crate::error::*;

// Utility functions
pub fn get_midnight_timestamp(timestamp: i64) -> i64 {
    let seconds_per_day = 86400;
    let midnight_timestamp = timestamp - (timestamp % seconds_per_day);
    midnight_timestamp + seconds_per_day
}

pub fn eggs_to_sonic(state: &EggsState, eggs_amount: u64) -> Result<u64> {
    // Backing is the balance of the program's account + total borrowed
    let backing = state.total_borrowed + get_backing_balance(state)?;
    
    // Calculate using the formula: eggs_amount * backing / total_supply
    let total_supply = state.total_minted; // assuming total_minted is our total supply
    
    if total_supply == 0 {
        return Err(EggsError::InvalidParameter.into());
    }
    
    let sonic = (eggs_amount as u128 * backing as u128) / total_supply as u128;
    Ok(sonic as u64)
}

pub fn sonic_to_eggs(state: &EggsState, sonic_amount: u64) -> Result<u64> {
    // Backing is the balance of the program's account + total borrowed
    let backing = state.total_borrowed + get_backing_balance(state)?;
    
    // Calculate using the formula: sonic_amount * total_supply / (backing - sonic_amount)
    let total_supply = state.total_minted; // assuming total_minted is our total supply
    
    if backing <= sonic_amount {
        return Err(EggsError::InvalidParameter.into());
    }
    
    let eggs = (sonic_amount as u128 * total_supply as u128) / (backing as u128 - sonic_amount as u128);
    Ok(eggs as u64)
}

pub fn get_backing_balance(state: &EggsState) -> Result<u64> {
    // In a real implementation, you would get the actual balance of the program's account
    // For now we'll return a placeholder
    Ok(0) // This should be replaced with the actual logic
}

pub fn liquidate(_state: &mut EggsState) -> Result<()> {
    // Implementation of liquidate function
    // This would scan through expired loans and liquidate them
    // For now, this is a placeholder
    Ok(())
}

pub fn safety_check(state: &mut EggsState, sonic: u64) -> Result<()> {
    // Implementation of safety check
    // This would verify that the price doesn't decrease
    // For now, this is a placeholder
    let _new_price = 0; // Calculate new price
    
    // In a real implementation, we would check:
    // 1. Total collateral <= contract's EGGS balance
    // 2. The price of EGGS cannot decrease
    
    state.last_price = 0; // Update last price
    
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
    )]
    pub mint: Account<'info, Mint>,
    
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
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
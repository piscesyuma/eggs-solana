use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Mint, Token, TokenAccount, Transfer, Burn, MintTo},
};
use solana_program::{native_token::LAMPORTS_PER_SOL, program::invoke, program::invoke_signed, system_instruction};

pub mod states;
pub mod error;
pub mod utils;

use states::*;
use error::*;
use utils::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS"); // This will be replaced with the actual program ID

#[program]
pub mod eggs {
    use super::*;

    // Initialize the EGGS program with a new token mint
    pub fn initialize(ctx: Context<Initialize>, bump: u8) -> Result<()> {
        let state = &mut ctx.accounts.state;
        state.authority = ctx.accounts.authority.key();
        state.mint = ctx.accounts.mint.key();
        state.bump = bump;
        state.sell_fee = 975;
        state.buy_fee = 975;
        state.buy_fee_leverage = 10;
        state.start = false;
        state.last_liquidation_date = get_midnight_timestamp(Clock::get()?.unix_timestamp);
        
        Ok(())
    }

    // Set the fee address
    pub fn set_fee_address(ctx: Context<UpdateState>, fee_address: Pubkey) -> Result<()> {
        require!(fee_address != Pubkey::default(), EggsError::InvalidParameter);
        
        let state = &mut ctx.accounts.state;
        state.fee_address = fee_address;
        
        Ok(())
    }

    // Start trading
    pub fn set_start(ctx: Context<SetStart>, amount: u64) -> Result<()> {
        let state = &mut ctx.accounts.state;
        require!(state.fee_address != Pubkey::default(), EggsError::FeeAddressNotSet);
        require!(!state.start, EggsError::TradingNotInitialized); // Trading should not be already initialized
        require!(amount >= LAMPORTS_PER_SOL, EggsError::BelowMinimumValue);
        
        // Transfer SOL from the user to the program
        let amount_lamports = amount;
        let ix = system_instruction::transfer(
            &ctx.accounts.authority.key(),
            &ctx.accounts.state_account.key(),
            amount_lamports,
        );
        invoke(
            &ix,
            &[
                ctx.accounts.authority.to_account_info(),
                ctx.accounts.state_account.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;
        
        // Calculate team mint amount
        let team_mint_amount = amount_lamports * MIN;
        
        // Mint to the authority (team)
        token::mint_to(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    mint: ctx.accounts.mint.to_account_info(),
                    to: ctx.accounts.authority_token_account.to_account_info(),
                    authority: ctx.accounts.state_account.to_account_info(),
                },
            )
            .with_signer(&[&[
                b"state".as_ref(),
                &[state.bump],
            ]]),
            team_mint_amount,
        )?;
        
        // Update state
        state.start = true;
        state.total_minted += team_mint_amount;
        
        // Burns are handled by transferring to a dead address in Solana
        // This would be handled in a separate instruction for Solana
        
        Ok(())
    }

    // Set the buy fee
    pub fn set_buy_fee(ctx: Context<UpdateState>, amount: u16) -> Result<()> {
        require!(amount <= 992, EggsError::InvalidParameter); // must be greater than FEES_BUY
        require!(amount >= 975, EggsError::InvalidParameter); // must be less than 2.5%
        
        let state = &mut ctx.accounts.state;
        state.buy_fee = amount;
        
        Ok(())
    }

    // Set the buy fee leverage
    pub fn set_buy_fee_leverage(ctx: Context<UpdateState>, amount: u16) -> Result<()> {
        require!(amount <= 25, EggsError::InvalidParameter); // must be less than 2.5%
        // require!(amount >= 0, EggsError::InvalidParameter); // must be greater than 0%
        
        let state = &mut ctx.accounts.state;
        state.buy_fee_leverage = amount;
        
        Ok(())
    }

    // Set the sell fee
    pub fn set_sell_fee(ctx: Context<UpdateState>, amount: u16) -> Result<()> {
        require!(amount <= 992, EggsError::InvalidParameter); // must be greater than FEES_SELL
        require!(amount >= 975, EggsError::InvalidParameter); // must be less than 2.5%
        
        let state = &mut ctx.accounts.state;
        state.sell_fee = amount;
        
        Ok(())
    }

    // Buy EGGS tokens
    pub fn buy(ctx: Context<Trade>, amount: u64) -> Result<()> {
        let state = &mut ctx.accounts.state;
        // liquidate(&mut ctx.accounts.state)?;
        liquidate(state)?;
        require!(state.start, EggsError::TradingNotInitialized);
        require!(ctx.accounts.receiver.key() != Pubkey::default(), EggsError::InvalidParameter);
        
        // Transfer SOL from the user to the program
        let amount_lamports = amount;
        let ix = system_instruction::transfer(
            &ctx.accounts.authority.key(),
            &ctx.accounts.state_account.key(),
            amount_lamports,
        );
        invoke(
            &ix,
            &[
                ctx.accounts.authority.to_account_info(),
                ctx.accounts.state_account.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;
        
        // Calculate EGGS to mint
        let eggs = sonic_to_eggs(state, amount_lamports)?;
        let eggs_with_fee = (eggs * state.buy_fee as u64) / FEE_BASE_1000 as u64;
        
        // Mint EGGS to the receiver
        token::mint_to(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    mint: ctx.accounts.mint.to_account_info(),
                    to: ctx.accounts.receiver_token_account.to_account_info(),
                    authority: ctx.accounts.state_account.to_account_info(),
                },
            )
            .with_signer(&[&[
                b"state".as_ref(),
                &[state.bump],
            ]]),
            eggs_with_fee,
        )?;
        
        // Update state
        state.total_minted += eggs_with_fee;
        
        // Team fee
        let fee_address_amount = amount_lamports / FEES_BUY as u64;
        require!(fee_address_amount > MIN, EggsError::BelowMinimumValue);
        
        // Transfer fee to fee address
        let ix_fee = system_instruction::transfer(
            &ctx.accounts.state_account.key(),
            &state.fee_address,
            fee_address_amount,
        );
        invoke_signed(
            &ix_fee,
            &[
                ctx.accounts.state_account.to_account_info(),
                ctx.accounts.fee_address_account.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
            &[&[
                b"state".as_ref(),
                &[state.bump],
            ]],
        )?;
        
        safety_check(&mut ctx.accounts.state, amount_lamports)?;
        
        Ok(())
    }

    // Sell EGGS tokens
    pub fn sell(ctx: Context<Trade>, eggs_amount: u64) -> Result<()> {
        let state = &mut ctx.accounts.state;
        liquidate(state)?;
        
        // Calculate SOL to be sent
        let sonic = eggs_to_sonic(state, eggs_amount)?;
        
        // Burn EGGS from the seller
        token::burn(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Burn {
                    mint: ctx.accounts.mint.to_account_info(),
                    from: ctx.accounts.receiver_token_account.to_account_info(),
                    authority: ctx.accounts.authority.to_account_info(),
                },
            ),
            eggs_amount,
        )?;
        
        // Team fee
        let fee_address_amount = sonic / FEES_SELL as u64;
        require!(fee_address_amount > MIN, EggsError::BelowMinimumValue);
        
        // Transfer SOL to the seller
        let seller_amount = (sonic * state.sell_fee as u64) / FEE_BASE_1000 as u64;
        let ix_seller = system_instruction::transfer(
            &ctx.accounts.state_account.key(),
            &ctx.accounts.authority.key(),
            seller_amount,
        );
        invoke_signed(
            &ix_seller,
            &[
                ctx.accounts.state_account.to_account_info(),
                ctx.accounts.authority.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
            &[&[
                b"state".as_ref(),
                &[state.bump],
            ]],
        )?;
        
        // Transfer fee to fee address
        let ix_fee = system_instruction::transfer(
            &ctx.accounts.state_account.key(),
            &state.fee_address,
            fee_address_amount,
        );
        invoke_signed(
            &ix_fee,
            &[
                ctx.accounts.state_account.to_account_info(),
                ctx.accounts.fee_address_account.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
            &[&[
                b"state".as_ref(),
                &[state.bump],
            ]],
        )?;
        
        safety_check(&mut ctx.accounts.state, sonic)?;
        
        Ok(())
    }

    // Additional functions to implement:
    // - leverage
    // - borrow
    // - borrow_more
    // - remove_collateral
    // - repay
    // - close_position
    // - flash_close_position
    // - extend_loan
    // - liquidate
    
    // These functions would be implemented in a similar pattern to what's shown above,
    // adapting the Solidity contract logic to Solana's account model
} 
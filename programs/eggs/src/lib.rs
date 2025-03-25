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
        
        // Initialize the state with zero values for tracking
        state.total_borrowed = 0;
        state.total_collateral = 0;
        state.total_minted = 0;
        state.last_price = 0;
        state.fee_address = Pubkey::default(); // Will be set later with set_fee_address
        
        // The mint is already initialized in the Initialize struct with:
        // - 9 decimals (EGGS_DECIMALS)
        // - Mint authority set to state_account (PDA)
        // - Freeze authority set to None
        // No need to initialize it again, as it's handled by Anchor's constraints
        
        // Note: To add metadata to the token (name, symbol, etc.), you would need to:
        // 1. Add the Metaplex Token Metadata program as a dependency
        // 2. Create a CPI call to create_metadata_accounts_v3
        // 3. Pass in the required parameters (name: "EGGS", symbol: "EGGS", uri: "https://...")
        //
        // Example of that code would be:
        //
        // let seeds = &[b"state".as_ref(), &[bump]];
        // let signer = &[&seeds[..]];
        // 
        // let data = mpl_token_metadata::pda::DataV2 {
        //     name: "EGGS".to_string(),
        //     symbol: "EGGS".to_string(),
        //     uri: "https://...".to_string(),
        //     seller_fee_basis_points: 0,
        //     creators: None,
        //     collection: None,
        //     uses: None,
        // };
        //
        // invoke_signed(
        //     &mpl_token_metadata::instruction::create_metadata_accounts_v3(
        //         metadata_program_id,
        //         metadata_pda,
        //         mint_address,
        //         mint_authority,
        //         payer,
        //         update_authority,
        //         data.name,
        //         data.symbol,
        //         data.uri,
        //         data.creators,
        //         data.seller_fee_basis_points,
        //         true,
        //         true,
        //         data.collection,
        //         data.uses,
        //         None,
        //     ),
        //     &accounts_vec,
        //     signer,
        // )?;
        
        msg!("EGGS token initialized with:");
        msg!("- Decimals: {}", EGGS_DECIMALS);
        msg!("- Mint authority: {}", ctx.accounts.state_account.key());
        msg!("- Mint address: {}", ctx.accounts.mint.key());
        
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
        
        // Ensure we don't exceed MAX_SUPPLY
        require!(team_mint_amount <= MAX_SUPPLY as u64, EggsError::MaxSupplyExceeded);
        
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
        // Update to use the real implementation for token conversion
        let sol_amount = amount_lamports;
        let state_account = &ctx.accounts.state_account;
        
        // Get the backing (SOL balance + borrowed)
        let backing = state.total_borrowed + state_account.lamports() - sol_amount;
        let total_supply = state.total_minted;
        
        // Calculate eggs: sol_amount * total_supply / (backing)
        let eggs = if backing == 0 || total_supply == 0 {
            return Err(EggsError::InvalidParameter.into());
        } else {
            (sol_amount as u128 * total_supply as u128) / backing as u128
        };
        
        let eggs_with_fee = (eggs as u64 * state.buy_fee as u64) / FEE_BASE_1000 as u64;
        
        // Ensure we don't exceed MAX_SUPPLY
        require!(
            state.total_minted as u128 + eggs_with_fee as u128 <= MAX_SUPPLY,
            EggsError::MaxSupplyExceeded
        );
        
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
        
        // Use the real safety check with account info
        safety_check_with_account(state, &ctx.accounts.state_account, amount_lamports)?;
        
        Ok(())
    }

    // Sell EGGS tokens
    pub fn sell(ctx: Context<Trade>, eggs_amount: u64) -> Result<()> {
        let state = &mut ctx.accounts.state;
        liquidate(state)?;
        
        // Calculate SOL to be sent using real account info
        let state_account = &ctx.accounts.state_account;
        
        // Get the backing (SOL balance + borrowed)
        let backing = state.total_borrowed + state_account.lamports();
        let total_supply = state.total_minted;
        
        // Calculate sol: eggs_amount * backing / total_supply
        let sol = if total_supply == 0 {
            return Err(EggsError::InvalidParameter.into());
        } else {
            (eggs_amount as u128 * backing as u128) / total_supply as u128
        };
        
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
        let fee_address_amount = sol as u64 / FEES_SELL as u64;
        require!(fee_address_amount > MIN, EggsError::BelowMinimumValue);
        
        // Transfer SOL to the seller
        let seller_amount = (sol as u64 * state.sell_fee as u64) / FEE_BASE_1000 as u64;
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
        
        // Use the real safety check with account info
        safety_check_with_account(state, &ctx.accounts.state_account, sol as u64)?;
        
        Ok(())
    }

    // Leverage function - allows users to borrow SOL by providing collateral
    pub fn leverage(ctx: Context<LoanOperation>, sol: u64, number_of_days: u64) -> Result<()> {
        require!(ctx.accounts.state.start, EggsError::TradingNotInitialized);
        require!(number_of_days < 366, EggsError::LoanTooLong);

        let current_time = Clock::get()?.unix_timestamp;
        
        // Check if user already has a loan and if it's expired
        if ctx.accounts.loan.borrowed != 0 {
            if is_loan_expired(&ctx.accounts.loan, current_time) {
                // Reset loan if expired
                ctx.accounts.loan.collateral = 0;
                ctx.accounts.loan.borrowed = 0;
                ctx.accounts.loan.end_date = 0;
                ctx.accounts.loan.number_of_days = 0;
            } else {
                return Err(EggsError::UserHasActiveLoan.into());
            }
        }
        
        liquidate(&mut ctx.accounts.state)?;
        
        // Calculate end date
        let end_date = get_midnight_timestamp(current_time + (number_of_days as i64 * 86400));
        
        // Calculate fees
        let sol_fee = (sol * ctx.accounts.state.buy_fee_leverage as u64) / FEE_BASE_1000 as u64;
        let interest_fee = get_interest_fee(sol, number_of_days)?;
        let total_fee = sol_fee + interest_fee;
        
        let user_sol = sol - total_fee;
        
        let fee_address_amount = (total_fee * 3) / 10;
        let user_borrow = (user_sol * 99) / 100;
        let over_collateralization_amount = user_sol / 100;
        let total_required_payment = total_fee + over_collateralization_amount;
        
        // Check if user paid enough
        require!(ctx.accounts.authority.lamports() >= total_required_payment, EggsError::InsufficientFunds);
        
        // Transfer the fee from the user to the program
        let ix = system_instruction::transfer(
            &ctx.accounts.authority.key(),
            &ctx.accounts.state_account.key(),
            total_required_payment,
        );
        invoke(
            &ix,
            &[
                ctx.accounts.authority.to_account_info(),
                ctx.accounts.state_account.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;
        
        // Calculate sub value for collateral calculation
        let sub_value = fee_address_amount + over_collateralization_amount;
        
        // Calculate user eggs (collateral) using real account info
        let state_account = &ctx.accounts.state_account;
        let user_eggs = sol_to_eggs_lev_with_account(&ctx.accounts.state, state_account, user_sol, sub_value)?;
        
        // Ensure we don't exceed MAX_SUPPLY
        require!(
            ctx.accounts.state.total_minted as u128 + user_eggs as u128 <= MAX_SUPPLY,
            EggsError::MaxSupplyExceeded
        );
        
        // Mint eggs to the contract as collateral
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
                &[ctx.accounts.state.bump],
            ]]),
            user_eggs,
        )?;
        
        // Transfer fee to fee address
        require!(fee_address_amount > MIN, EggsError::BelowMinimumValue);
        let ix_fee = system_instruction::transfer(
            &ctx.accounts.state_account.key(),
            &ctx.accounts.state.fee_address,
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
                &[ctx.accounts.state.bump],
            ]],
        )?;
        
        // Transfer borrowed amount to user
        let ix_borrow = system_instruction::transfer(
            &ctx.accounts.state_account.key(),
            &ctx.accounts.authority.key(),
            user_borrow,
        );
        invoke_signed(
            &ix_borrow,
            &[
                ctx.accounts.state_account.to_account_info(),
                ctx.accounts.authority.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
            &[&[
                b"state".as_ref(),
                &[ctx.accounts.state.bump],
            ]],
        )?;
        
        // Update loan data for the date
        ctx.accounts.daily_loan_data.date = end_date;
        add_loans_by_date(
            &mut ctx.accounts.state, 
            &mut ctx.accounts.daily_loan_data, 
            user_borrow, 
            user_eggs
        )?;
        
        // Update user's loan
        ctx.accounts.loan.user = ctx.accounts.authority.key();
        ctx.accounts.loan.collateral = user_eggs;
        ctx.accounts.loan.borrowed = user_borrow;
        ctx.accounts.loan.end_date = end_date;
        ctx.accounts.loan.number_of_days = number_of_days;
        
        safety_check_with_account(&mut ctx.accounts.state, &ctx.accounts.state_account, sol)?;
        
        Ok(())
    }

    // Borrow EGGS tokens by providing collateral
    pub fn borrow(ctx: Context<LoanOperation>, sol: u64, number_of_days: u64) -> Result<()> {
        require!(number_of_days < 366, EggsError::LoanTooLong);
        require!(sol > 0, EggsError::InvalidParameter);
        
        let current_time = Clock::get()?.unix_timestamp;
        
        // Check if user has an expired loan
        if is_loan_expired(&ctx.accounts.loan, current_time) {
            // Reset loan if expired
            ctx.accounts.loan.collateral = 0;
            ctx.accounts.loan.borrowed = 0;
            ctx.accounts.loan.end_date = 0;
            ctx.accounts.loan.number_of_days = 0;
        }
        
        // Check if user already has an active loan
        require!(ctx.accounts.loan.borrowed == 0, EggsError::UserHasActiveLoan);
        
        liquidate(&mut ctx.accounts.state)?;
        
        // Calculate end date
        let end_date = get_midnight_timestamp(current_time + (number_of_days as i64 * 86400));
        
        // Calculate fees
        let sol_fee = get_interest_fee(sol, number_of_days)?;
        let fee_address_amount = (sol_fee * 3) / 10;
        
        // Calculate user eggs required (collateral) using real account info
        let state_account = &ctx.accounts.state_account;
        let user_eggs = sol_to_eggs_no_trade_ceil_with_account(&ctx.accounts.state, state_account, sol)?;
        
        // Calculate new user borrow amount
        let new_user_borrow = (sol * 99) / 100;
        
        // Check if user has enough tokens for collateral
        require!(
            ctx.accounts.authority_token_account.amount >= user_eggs,
            EggsError::InsufficientCollateral
        );
        
        // Transfer collateral from user to program escrow
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.authority_token_account.to_account_info(),
                    to: ctx.accounts.escrow_token_account.to_account_info(),
                    authority: ctx.accounts.authority.to_account_info(),
                },
            ),
            user_eggs,
        )?;
        
        // Update user's loan
        ctx.accounts.loan.user = ctx.accounts.authority.key();
        ctx.accounts.loan.collateral = user_eggs;
        ctx.accounts.loan.borrowed = new_user_borrow;
        ctx.accounts.loan.end_date = end_date;
        ctx.accounts.loan.number_of_days = number_of_days;
        
        // Update loan data for the date
        ctx.accounts.daily_loan_data.date = end_date;
        add_loans_by_date(
            &mut ctx.accounts.state, 
            &mut ctx.accounts.daily_loan_data, 
            new_user_borrow, 
            user_eggs
        )?;
        
        // Transfer borrowed amount minus fees to the user
        require!(fee_address_amount > MIN, EggsError::BelowMinimumValue);
        
        // Transfer fee to fee address
        let ix_fee = system_instruction::transfer(
            &ctx.accounts.state_account.key(),
            &ctx.accounts.state.fee_address,
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
                &[ctx.accounts.state.bump],
            ]],
        )?;
        
        // Transfer borrowed amount to user
        let ix_borrow = system_instruction::transfer(
            &ctx.accounts.state_account.key(),
            &ctx.accounts.authority.key(),
            new_user_borrow - sol_fee,
        );
        invoke_signed(
            &ix_borrow,
            &[
                ctx.accounts.state_account.to_account_info(),
                ctx.accounts.authority.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
            &[&[
                b"state".as_ref(),
                &[ctx.accounts.state.bump],
            ]],
        )?;
        
        safety_check_with_account(&mut ctx.accounts.state, &ctx.accounts.state_account, sol_fee)?;
        
        Ok(())
    }
    
    // Borrow more against existing collateral
    pub fn borrow_more(ctx: Context<LoanOperation>, sol: u64) -> Result<()> {
        require!(sol > 0, EggsError::InvalidParameter);
        
        let current_time = Clock::get()?.unix_timestamp;
        
        // Check if loan is expired
        require!(!is_loan_expired(&ctx.accounts.loan, current_time), EggsError::LoanExpired);
        
        liquidate(&mut ctx.accounts.state)?;
        
        // Get user's current loan details
        let user_borrowed = ctx.accounts.loan.borrowed;
        let user_collateral = ctx.accounts.loan.collateral;
        let user_end_date = ctx.accounts.loan.end_date;
        
        // Calculate remaining loan duration in days
        let today_midnight = get_midnight_timestamp(current_time);
        let new_borrow_length = (user_end_date - today_midnight) / 86400;
        
        // Calculate fees
        let sol_fee = get_interest_fee(sol, new_borrow_length as u64)?;
        
        // Get the state account for real calculations
        let state_account = &ctx.accounts.state_account;
        
        // Calculate required eggs for new loan
        let user_eggs = sol_to_eggs_no_trade_ceil_with_account(&ctx.accounts.state, state_account, sol)?;
        
        // Calculate current borrowed amount in eggs
        let user_borrowed_in_eggs = sol_to_eggs_no_trade_with_account(&ctx.accounts.state, state_account, user_borrowed)?;
        
        // Calculate excess collateral in eggs
        let user_excess_in_eggs = ((user_collateral * 99) / 100).saturating_sub(user_borrowed_in_eggs);
        
        // Calculate how much additional collateral is needed
        let require_collateral_from_user = if user_excess_in_eggs >= user_eggs {
            0
        } else {
            user_eggs - user_excess_in_eggs
        };
        
        let fee_address_fee = (sol_fee * 3) / 10;
        let new_user_borrow = (sol * 99) / 100;
        
        // Update user's loan
        let new_user_borrow_total = user_borrowed + new_user_borrow;
        let new_user_collateral_total = user_collateral + require_collateral_from_user;
        
        // Transfer additional collateral if needed
        if require_collateral_from_user > 0 {
            // Check if user has enough tokens for additional collateral
            require!(
                ctx.accounts.authority_token_account.amount >= require_collateral_from_user,
                EggsError::InsufficientCollateral
            );
            
            // Transfer additional collateral from user to program escrow
            token::transfer(
                CpiContext::new(
                    ctx.accounts.token_program.to_account_info(),
                    Transfer {
                        from: ctx.accounts.authority_token_account.to_account_info(),
                        to: ctx.accounts.escrow_token_account.to_account_info(),
                        authority: ctx.accounts.authority.to_account_info(),
                    },
                ),
                require_collateral_from_user,
            )?;
        }
        
        // Update user's loan with new values
        ctx.accounts.loan.borrowed = new_user_borrow_total;
        ctx.accounts.loan.collateral = new_user_collateral_total;
        
        // Update loan data for the date
        add_loans_by_date(
            &mut ctx.accounts.state, 
            &mut ctx.accounts.daily_loan_data, 
            new_user_borrow, 
            require_collateral_from_user
        )?;
        
        // Transfer fee to fee address
        require!(fee_address_fee > MIN, EggsError::BelowMinimumValue);
        let ix_fee = system_instruction::transfer(
            &ctx.accounts.state_account.key(),
            &ctx.accounts.state.fee_address,
            fee_address_fee,
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
                &[ctx.accounts.state.bump],
            ]],
        )?;
        
        // Transfer borrowed amount to user
        let ix_borrow = system_instruction::transfer(
            &ctx.accounts.state_account.key(),
            &ctx.accounts.authority.key(),
            new_user_borrow - sol_fee,
        );
        invoke_signed(
            &ix_borrow,
            &[
                ctx.accounts.state_account.to_account_info(),
                ctx.accounts.authority.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
            &[&[
                b"state".as_ref(),
                &[ctx.accounts.state.bump],
            ]],
        )?;
        
        safety_check_with_account(&mut ctx.accounts.state, &ctx.accounts.state_account, sol_fee)?;
        
        Ok(())
    }

    // Remove excess collateral from a loan
    pub fn remove_collateral(ctx: Context<LoanOperation>, amount: u64) -> Result<()> {
        let current_time = Clock::get()?.unix_timestamp;
        
        // Check if loan is expired
        require!(!is_loan_expired(&ctx.accounts.loan, current_time), EggsError::LoanExpired);
        
        liquidate(&mut ctx.accounts.state)?;
        
        let collateral = ctx.accounts.loan.collateral;
        
        // Check that removing collateral doesn't go below 99% collateralization
        let state_account = &ctx.accounts.state_account;
        let sol_value = eggs_to_sol_with_account(&ctx.accounts.state, state_account, collateral - amount)?;
        require!(
            ctx.accounts.loan.borrowed <= (sol_value * 99) / 100,
            EggsError::InsufficientCollateral
        );
        
        // Update loan with reduced collateral
        ctx.accounts.loan.collateral = ctx.accounts.loan.collateral - amount;
        
        // Transfer collateral back to the user from the escrow
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.escrow_token_account.to_account_info(),
                    to: ctx.accounts.authority_token_account.to_account_info(),
                    authority: ctx.accounts.state_account.to_account_info(),
                },
                &[&[
                    b"state".as_ref(),
                    &[ctx.accounts.state.bump],
                ]]
            ),
            amount,
        )?;
        
        // Update loan data for the date
        sub_loans_by_date(
            &mut ctx.accounts.state, 
            &mut ctx.accounts.daily_loan_data, 
            0, 
            amount
        )?;
        
        safety_check_with_account(&mut ctx.accounts.state, &ctx.accounts.state_account, 0)?;
        
        Ok(())
    }
    
    // Repay part of a loan
    pub fn repay(ctx: Context<LoanOperation>) -> Result<()> {
        let current_time = Clock::get()?.unix_timestamp;
        let borrowed = ctx.accounts.loan.borrowed;
        
        // Validate the repayment
        let repay_amount = ctx.accounts.authority.lamports();
        require!(borrowed > repay_amount, EggsError::RepayAmountTooLarge);
        require!(repay_amount > 0, EggsError::InvalidParameter);
        
        // Check if loan is expired
        require!(!is_loan_expired(&ctx.accounts.loan, current_time), EggsError::LoanExpired);
        
        // Transfer SOL from user to program
        let ix = system_instruction::transfer(
            &ctx.accounts.authority.key(),
            &ctx.accounts.state_account.key(),
            repay_amount,
        );
        invoke(
            &ix,
            &[
                ctx.accounts.authority.to_account_info(),
                ctx.accounts.state_account.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;
        
        // Update loan with reduced borrowed amount
        let new_borrow = borrowed - repay_amount;
        ctx.accounts.loan.borrowed = new_borrow;
        
        // Update loan data for the date
        sub_loans_by_date(
            &mut ctx.accounts.state, 
            &mut ctx.accounts.daily_loan_data, 
            repay_amount, 
            0
        )?;
        
        safety_check_with_account(&mut ctx.accounts.state, &ctx.accounts.state_account, 0)?;
        
        Ok(())
    }
    
    // Close a loan position by repaying the full amount
    pub fn close_position(ctx: Context<LoanOperation>) -> Result<()> {
        let current_time = Clock::get()?.unix_timestamp;
        let borrowed = ctx.accounts.loan.borrowed;
        let collateral = ctx.accounts.loan.collateral;
        
        // Check if loan is expired
        require!(!is_loan_expired(&ctx.accounts.loan, current_time), EggsError::LoanExpired);
        
        // Validate full repayment
        let repay_amount = ctx.accounts.authority.lamports();
        require!(borrowed == repay_amount, EggsError::IncorrectRepaymentAmount);
        
        // Transfer SOL from user to program
        let ix = system_instruction::transfer(
            &ctx.accounts.authority.key(),
            &ctx.accounts.state_account.key(),
            repay_amount,
        );
        invoke(
            &ix,
            &[
                ctx.accounts.authority.to_account_info(),
                ctx.accounts.state_account.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;
        
        // Transfer collateral back to the user from the escrow
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.escrow_token_account.to_account_info(),
                    to: ctx.accounts.authority_token_account.to_account_info(),
                    authority: ctx.accounts.state_account.to_account_info(),
                },
                &[&[
                    b"state".as_ref(),
                    &[ctx.accounts.state.bump],
                ]]
            ),
            collateral,
        )?;
        
        // Update loan data for the date
        sub_loans_by_date(
            &mut ctx.accounts.state, 
            &mut ctx.accounts.daily_loan_data, 
            borrowed, 
            collateral
        )?;
        
        // Reset the loan
        ctx.accounts.loan.borrowed = 0;
        ctx.accounts.loan.collateral = 0;
        ctx.accounts.loan.end_date = 0;
        ctx.accounts.loan.number_of_days = 0;
        
        safety_check_with_account(&mut ctx.accounts.state, &ctx.accounts.state_account, 0)?;
        
        Ok(())
    }
    
    // Close a loan position by using collateral
    pub fn flash_close_position(ctx: Context<LoanOperation>) -> Result<()> {
        let current_time = Clock::get()?.unix_timestamp;
        
        // Check if loan is expired
        require!(!is_loan_expired(&ctx.accounts.loan, current_time), EggsError::LoanExpired);
        
        liquidate(&mut ctx.accounts.state)?;
        
        let borrowed = ctx.accounts.loan.borrowed;
        let collateral = ctx.accounts.loan.collateral;
        
        // Burn the collateral from the escrow account
        token::burn(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Burn {
                    mint: ctx.accounts.mint.to_account_info(),
                    from: ctx.accounts.escrow_token_account.to_account_info(),
                    authority: ctx.accounts.state_account.to_account_info(),
                },
                &[&[
                    b"state".as_ref(),
                    &[ctx.accounts.state.bump],
                ]]
            ),
            collateral,
        )?;
        
        // Calculate values after fee using real account info
        let state_account = &ctx.accounts.state_account;
        let collateral_in_sol = eggs_to_sol_with_account(&ctx.accounts.state, state_account, collateral)?;
        let collateral_in_sol_after_fee = (collateral_in_sol * 99) / 100;
        let fee = collateral_in_sol / 100;
        
        // Check if there's enough collateral to cover the borrowed amount
        require!(
            collateral_in_sol_after_fee >= borrowed,
            EggsError::InsufficientCollateral
        );
        
        // Calculate amounts
        let to_user = collateral_in_sol_after_fee - borrowed;
        let fee_address_fee = (fee * 3) / 10;
        
        // Transfer remaining value to user
        let ix_user = system_instruction::transfer(
            &ctx.accounts.state_account.key(),
            &ctx.accounts.authority.key(),
            to_user,
        );
        invoke_signed(
            &ix_user,
            &[
                ctx.accounts.state_account.to_account_info(),
                ctx.accounts.authority.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
            &[&[
                b"state".as_ref(),
                &[ctx.accounts.state.bump],
            ]],
        )?;
        
        // Transfer fee to fee address
        require!(fee_address_fee > MIN, EggsError::BelowMinimumValue);
        let ix_fee = system_instruction::transfer(
            &ctx.accounts.state_account.key(),
            &ctx.accounts.state.fee_address,
            fee_address_fee,
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
                &[ctx.accounts.state.bump],
            ]],
        )?;
        
        // Update loan data for the date
        sub_loans_by_date(
            &mut ctx.accounts.state, 
            &mut ctx.accounts.daily_loan_data, 
            borrowed, 
            collateral
        )?;
        
        // Reset the loan
        ctx.accounts.loan.borrowed = 0;
        ctx.accounts.loan.collateral = 0;
        ctx.accounts.loan.end_date = 0;
        ctx.accounts.loan.number_of_days = 0;
        
        safety_check_with_account(&mut ctx.accounts.state, &ctx.accounts.state_account, borrowed)?;
        
        Ok(())
    }
    
    // Extend the duration of a loan
    pub fn extend_loan(ctx: Context<LoanOperation>, number_of_days: u64) -> Result<()> {
        let current_time = Clock::get()?.unix_timestamp;
        
        // Check if loan is expired
        require!(!is_loan_expired(&ctx.accounts.loan, current_time), EggsError::LoanExpired);
        
        let old_end_date = ctx.accounts.loan.end_date;
        let borrowed = ctx.accounts.loan.borrowed;
        let collateral = ctx.accounts.loan.collateral;
        let old_number_of_days = ctx.accounts.loan.number_of_days;
        
        // Calculate new end date
        let new_end_date = old_end_date + (number_of_days as i64 * 86400);
        
        // Calculate the loan extension fee
        let loan_fee = get_interest_fee(borrowed, number_of_days)?;
        
        // Transfer fee from user to program
        let ix = system_instruction::transfer(
            &ctx.accounts.authority.key(),
            &ctx.accounts.state_account.key(),
            loan_fee,
        );
        invoke(
            &ix,
            &[
                ctx.accounts.authority.to_account_info(),
                ctx.accounts.state_account.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;
        
        // Calculate fee address amount
        let fee_address_fee = (loan_fee * 3) / 10;
        require!(fee_address_fee > MIN, EggsError::BelowMinimumValue);
        
        // Transfer fee to fee address
        let ix_fee = system_instruction::transfer(
            &ctx.accounts.state_account.key(),
            &ctx.accounts.state.fee_address,
            fee_address_fee,
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
                &[ctx.accounts.state.bump],
            ]],
        )?;
        
        // Update loan data - remove from old date and add to new date
        sub_loans_by_date(
            &mut ctx.accounts.state, 
            &mut ctx.accounts.daily_loan_data, 
            borrowed, 
            collateral
        )?;
        
        // Get or create daily loan data for the new date
        // In a real implementation, you would look up or create the daily loan data for the new date
        ctx.accounts.daily_loan_data.date = new_end_date;
        add_loans_by_date(
            &mut ctx.accounts.state, 
            &mut ctx.accounts.daily_loan_data, 
            borrowed, 
            collateral
        )?;
        
        // Update the loan
        ctx.accounts.loan.end_date = new_end_date;
        ctx.accounts.loan.number_of_days = old_number_of_days + number_of_days;
        
        // Check that the loan duration is under 365 days
        require!(
            (new_end_date - current_time) / 86400 < 366,
            EggsError::LoanTooLong
        );
        
        safety_check_with_account(&mut ctx.accounts.state, &ctx.accounts.state_account, loan_fee)?;
        
        Ok(())
    }

    // Public liquidate function
    pub fn liquidate_loans(ctx: Context<LiquidateLoans>) -> Result<()> {
        // Get current time
        let current_time = Clock::get()?.unix_timestamp;
        let mut date = ctx.accounts.state.last_liquidation_date;
        
        // Process all expired loans up to the current midnight timestamp
        while date < current_time {
            // For a full implementation, you would fetch the daily loan data for each date
            // For each date that has expired loans:
            // 1. Get the collateral amount
            // 2. Burn the collateral tokens
            // 3. Update the state
            
            let midnight_timestamp = get_midnight_timestamp(date);
            
            // Process the liquidation for the current date (if data exists)
            if ctx.accounts.daily_loan_data.date == midnight_timestamp {
                let collateral = ctx.accounts.daily_loan_data.collateral;
                let borrowed = ctx.accounts.daily_loan_data.borrowed;
                
                if collateral > 0 {
                    // Burn the collateral tokens from the escrow
                    token::burn(
                        CpiContext::new_with_signer(
                            ctx.accounts.token_program.to_account_info(),
                            Burn {
                                mint: ctx.accounts.mint.to_account_info(),
                                from: ctx.accounts.escrow_token_account.to_account_info(),
                                authority: ctx.accounts.state_account.to_account_info(),
                            },
                            &[&[
                                b"state".as_ref(),
                                &[ctx.accounts.state.bump],
                            ]]
                        ),
                        collateral,
                    )?;
                    
                    // Update the state
                    ctx.accounts.state.total_collateral = ctx.accounts.state.total_collateral.saturating_sub(collateral);
                }
                
                if borrowed > 0 {
                    // Update the state
                    ctx.accounts.state.total_borrowed = ctx.accounts.state.total_borrowed.saturating_sub(borrowed);
                }
                
                // Reset the daily loan data
                ctx.accounts.daily_loan_data.collateral = 0;
                ctx.accounts.daily_loan_data.borrowed = 0;
            }
            
            // Move to the next day
            date += 86400;
        }
        
        // Update the last liquidation date
        ctx.accounts.state.last_liquidation_date = get_midnight_timestamp(current_time);
        
        Ok(())
    }
}

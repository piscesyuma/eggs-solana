use anchor_lang::prelude::*;
use anchor_spl::token_interface;

use crate::{error::MushiProgramError, state::MainState};

#[derive(AnchorDeserialize, AnchorSerialize, Debug, Clone)]
pub struct UpdateMainStateInput {
    admin: Option<Pubkey>,
    fee_receiver: Option<Pubkey>,
    sell_fee: Option<u64>,
    buy_fee: Option<u64>,
    buy_fee_leverage: Option<u64>,
    quote_token: Option<Pubkey>,
    stake_token: Option<Pubkey>,
    stake_vault_program: Option<Pubkey>,
    stake_enabled: Option<bool>,
    started: Option<bool>,
}

pub fn update_main_state(
    ctx: Context<AUpdateMainState>,
    input: UpdateMainStateInput,
) -> Result<()> {
    let state = &mut ctx.accounts.main_state;
    
    // Only update admin if provided and different from current value
    if let Some(admin) = input.admin {
        if admin != state.admin {
            state.admin = admin;
        }
    }
    
    // Only update fee_receiver if provided and different from current value
    if let Some(fee_receiver) = input.fee_receiver {
        if fee_receiver != state.fee_receiver {
            state.fee_receiver = fee_receiver;
        }
    }
    
    // Only update quote_token if provided and different from current value
    if let Some(quote_token) = input.quote_token {
        if quote_token != state.quote_token {
            state.quote_token = quote_token;
        }
    }
    // Only update stake_token if provided and different from current value
    if let Some(stake_token) = input.stake_token {
        if stake_token != state.stake_token {
            state.stake_token = stake_token;
        }
    }
    
    // Only update stake_vault_program if provided and different from current value
    if let Some(stake_vault_program) = input.stake_vault_program {
        if stake_vault_program != state.stake_vault_program {
            state.stake_vault_program = stake_vault_program;
        }
    }
    
    // Only update stake_enabled if provided and different from current value
    if let Some(stake_enabled) = input.stake_enabled {
        if stake_enabled != state.stake_enabled {
            state.stake_enabled = stake_enabled;
        }
    }
    
    // Only update buy_fee if provided and different from current value
    if let Some(buy_fee) = input.buy_fee {
        require!(
            buy_fee <= 992 && buy_fee >= 975,
            MushiProgramError::InvalidBuyFee
        );
        if buy_fee != state.buy_fee {
            state.buy_fee = buy_fee;
        }
    }
    
    // Only update sell_fee if provided and different from current value
    if let Some(sell_fee) = input.sell_fee {
        require!(
            sell_fee <= 992 && sell_fee >= 975,
            MushiProgramError::InvalidSellFee
        );
        if sell_fee != state.sell_fee {
            state.sell_fee = sell_fee;
        }
    }
    
    // Only update buy_fee_leverage if provided and different from current value
    if let Some(buy_fee_leverage) = input.buy_fee_leverage {
        require!(
            buy_fee_leverage <= 25,
            MushiProgramError::InvalidBuyFeeLeverage
        );
        if buy_fee_leverage != state.buy_fee_leverage {
            state.buy_fee_leverage = buy_fee_leverage;
        }
    }
    
    // Only update started if provided and different from current value
    if let Some(started) = input.started {
        if started != state.started {
            state.started = started;
        }
    }
    
    Ok(())
}

#[derive(Accounts)]
pub struct AUpdateMainState<'info> {
    #[account(address=main_state.admin @MushiProgramError::UnAuthorised)]
    pub admin: Signer<'info>,
    #[account(
        mut,
        seeds=[MainState::PREFIX_SEED],
        bump,
    )]
    pub main_state: Account<'info, MainState>,
    pub stake_token: InterfaceAccount<'info, token_interface::Mint>,
}

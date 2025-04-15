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
    stake_token: Option<Pubkey>,
}

pub fn update_main_state(
    ctx: Context<AUpdateMainState>,
    input: UpdateMainStateInput,
) -> Result<()> {
    let state = &mut ctx.accounts.main_state;
    state.admin = input.admin.unwrap_or(state.admin);
    state.fee_receiver = input.fee_receiver.unwrap_or(state.fee_receiver);

    let state_token = input.stake_token.unwrap_or(state.stake_token);
    state.stake_token = state_token;

    let buy_fee = input.buy_fee.unwrap_or(state.buy_fee);

    require!(
        buy_fee <= 992 && buy_fee >= 975,
        MushiProgramError::InvalidBuyFee
    );
    state.buy_fee = buy_fee;

    let sell_fee = input.sell_fee.unwrap_or(state.sell_fee);
    require!(
        sell_fee <= 992 && sell_fee >= 975,
        MushiProgramError::InvalidSellFee
    );
    state.sell_fee = input.sell_fee.unwrap_or(state.sell_fee);

    let buy_fee_leverage = input.buy_fee_leverage.unwrap_or(state.buy_fee_leverage);
    require!(
        buy_fee_leverage <= 25,
        MushiProgramError::InvalidBuyFeeLeverage
    );
    state.buy_fee_leverage = buy_fee_leverage;
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

use crate::state::{DailyStats, GlobalStats, MainState};
use crate::utils::get_midnight_timestamp;
use anchor_lang::prelude::*;
use anchor_spl::token_interface;
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct InitializeInput {
    pub fee_receiver: Pubkey,
    pub sell_fee: u64,
    pub buy_fee: u64,
    pub buy_fee_leverage: u64,
}

pub fn init_main_state(ctx: Context<AInitializeState>, input: InitializeInput) -> Result<()> {
    // main state
    let main_state = &mut ctx.accounts.main_state;
    main_state.admin = ctx.accounts.admin.key();
    main_state.fee_receiver = input.fee_receiver.key();
    main_state.sell_fee = input.sell_fee;
    main_state.buy_fee = input.buy_fee;
    main_state.buy_fee_leverage = input.buy_fee_leverage;
    main_state.quote_token = ctx.accounts.quote_token.key();

    // global state
    let global_state = &mut ctx.accounts.global_state;
    global_state.last_liquidation_date =
        get_midnight_timestamp(Clock::get().unwrap().unix_timestamp);
    Ok(())
}

#[derive(Accounts)]
pub struct AInitializeState<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        init,
        payer = admin,
        seeds = [GlobalStats::PREFIX_SEED],
        bump,
        space =  8 + GlobalStats::MAX_SIZE,
    )]
    pub global_state: Box<Account<'info, GlobalStats>>,
    #[account(
        init,
        payer = admin,
        seeds = [MainState::PREFIX_SEED],
        bump,
        space =  8 + MainState::MAX_SIZE,
    )]
    pub main_state: Box<Account<'info, MainState>>,
    pub quote_token: InterfaceAccount<'info, token_interface::Mint>,
    pub system_program: Program<'info, System>,
}

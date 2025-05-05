use anchor_lang::prelude::*;
use anchor_spl::token_interface;

use crate::{
    constants::VAULT_SEED,
    context::common::ACommon,
    utils::liquidate,
};

pub fn liquidate_expired_loans(ctx: Context<ACommon>) -> Result<()> {
    // Calculate current timestamp and last liquidation date
    let current_timestamp = Clock::get()?.unix_timestamp;
    let last_liquidation_date = ctx.accounts.global_state.last_liquidation_date;
    
    // We'll call liquidate in a loop to process multiple days
    if ctx.accounts.global_state.last_liquidation_date < current_timestamp {
        // Call the liquidate function to process one day at a time
        liquidate(
            &mut ctx.accounts.last_liquidation_date_state,
            &mut ctx.accounts.global_state,
            ctx.accounts.token_vault.to_account_info(),
            ctx.accounts.token.to_account_info(),
            ctx.accounts.token_vault_owner.to_account_info(),
            ctx.accounts.base_token_program.to_account_info(),
            *ctx.bumps.get("token_vault_owner").unwrap(),
        )?;
    }
    
    Ok(())
} 
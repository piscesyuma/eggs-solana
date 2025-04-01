use anchor_lang::{
    prelude::*,
    solana_program::{
        program::{invoke, invoke_signed},
        system_instruction::transfer,
    },
};
use anchor_spl::token::{self, Burn, MintTo, Token, TokenAccount, Transfer};

use crate::{
    constants::{FEES_BUY, SECONDS_IN_A_DAY, VAULT_SEED, LAMPORTS_PER_SOL}, 
    state::MainState,
    error::MushiProgramError,
};

pub fn calc_days_from_seconds(seconds: i64) -> i64 {
    seconds / 60 / 60 / 24
}

pub fn calc_fee_amount(amount: u64) -> u64 {
    amount
        .checked_div(FEES_BUY)
        .unwrap()
}

pub fn mint_to_tokens_by_main_state<'info>(
    mint: AccountInfo<'info>,
    main_state: AccountInfo<'info>,
    receiver_ata: AccountInfo<'info>,
    token_program: AccountInfo<'info>,
    amount: u64,
    bump: u8,
) -> Result<()> {
    let accounts = MintTo {
        authority: main_state,
        mint,
        to: receiver_ata,
    };
    token::mint_to(
        CpiContext::new_with_signer(
            token_program,
            accounts,
            &[&[MainState::PREFIX_SEED, &[bump]]],
        ),
        amount,
    )
}

pub fn burn_tokens<'info>(
    token_account: AccountInfo<'info>,
    mint: AccountInfo<'info>,
    authority: AccountInfo<'info>,
    token_program: AccountInfo<'info>,
    amount: u64,
    signer_seeds: Option<&[&[&[u8]]]>,
) -> Result<()> {
    let accounts = Burn {
        authority,
        from: token_account,
        mint,
    };
    if let Some(signer_seeds) = signer_seeds {
        token::burn(
            CpiContext::new_with_signer(token_program, accounts, signer_seeds),
            amount,
        )
    } else {
        token::burn(CpiContext::new(token_program, accounts), amount)
    }
}

pub fn transfer_tokens<'info>(
    from: AccountInfo<'info>,
    to: AccountInfo<'info>,
    authority: AccountInfo<'info>,
    token_program: AccountInfo<'info>,
    amount: u64,
    signer_seeds: Option<&[&[&[u8]]]>,
) -> Result<()> {
    let token_transfer_accounts = Transfer {
        from,
        to,
        authority,
    };
    if let Some(signer_seeds) = signer_seeds {
        token::transfer(
            CpiContext::new_with_signer(token_program.clone(), token_transfer_accounts, signer_seeds),
            amount,
        )?;
    } else {
        token::transfer(CpiContext::new(token_program, token_transfer_accounts), amount)?;
    }
    Ok(())
}

pub fn trasnfer_sol<'info>(
    sender: AccountInfo<'info>,
    receiver: AccountInfo<'info>,
    system_program: AccountInfo<'info>,
    amount: u64,
    signer_seeds: Option<&[&[&[u8]]]>,
) -> Result<()> {
    let ix = transfer(sender.key, receiver.key, amount);
    if let Some(signer_seeds) = signer_seeds {
        invoke_signed(&ix, &[sender, receiver, system_program], signer_seeds)?;
    } else {
        invoke(&ix, &[sender, receiver, system_program])?;
    }
    Ok(())
}

pub fn liquidate<'info>(
    main_state: AccountInfo<'info>,
    token: AccountInfo<'info>,
    token_program: AccountInfo<'info>,
    token_vault: AccountInfo<'info>,
    token_vault_owner: AccountInfo<'info>,
    vault_bump: u8,
) -> Result<()> {
    let mut borrowed: u64 = 0;
    let mut collateral: u64 = 0;
    let current_time = Clock::get().unwrap().unix_timestamp;

    let mut state = MainState::try_from_slice(&main_state.data.borrow())?;
    let days_since_last_liquidation = (current_time - state.last_liquidation_date) / SECONDS_IN_A_DAY;
    while (state.last_liquidation_date < current_time) {
        collateral += state.get_collateral(state.last_liquidation_date as u64);
        borrowed += state.get_borrowed(state.last_liquidation_date as u64);
        state.last_liquidation_date += SECONDS_IN_A_DAY;
    }
    if collateral != 0 {
        state.total_collateral -= collateral;
        burn_tokens(
            token_vault.to_account_info(),
            token.to_account_info(),
            token_vault_owner.to_account_info(),
            token_program.to_account_info(),
            collateral,
            Some(&[&[VAULT_SEED, &[vault_bump]]]),
        )?;
    }
    if borrowed != 0 {
        state.total_borrowed -= borrowed;
    }
    Ok(())
}

pub fn safety_check<'info>(
    main_state: AccountInfo<'info>,
    token_vault_owner: AccountInfo<'info>,
) -> Result<()> {
    let mut state = MainState::try_from_slice(&main_state.data.borrow())?;
    let new_price: u64 = state.get_backing().unwrap()
        .checked_mul(1 * LAMPORTS_PER_SOL).unwrap()
        .checked_div(state.token_supply).unwrap();
    let _total_collateral = token_vault_owner.lamports();
    if _total_collateral < state.total_collateral {
        return Err(MushiProgramError::SafetyCheckFailed.into());
    }
    if state.last_price > new_price {
        return Err(MushiProgramError::SafetyCheckFailed.into());
    }
    state.last_price = new_price;
    Ok(())
}

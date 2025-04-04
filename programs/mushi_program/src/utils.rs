use anchor_lang::{
    prelude::*,
    solana_program::{
        program::{invoke, invoke_signed},
        system_instruction::transfer,
    },
};
use anchor_spl::token::{self, Burn, MintTo, Token, TokenAccount, Transfer};
use crate::state::DailyStats;
use crate::{
    constants::{FEES_BUY, SECONDS_IN_A_DAY, VAULT_SEED, LAMPORTS_PER_SOL}, 
    state::{MainState, GlobalStats},
    error::MushiProgramError,
};

pub fn mint_to_tokens_by_main_state<'info>(
    mint: AccountInfo<'info>,
    authority: AccountInfo<'info>,
    receiver_ata: AccountInfo<'info>,
    token_program: AccountInfo<'info>,
    amount: u64,
    bump: u8,
) -> Result<()> {
    let accounts = MintTo {
        authority,
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
    last_liquidation_date_state: &mut DailyStats,
    global_state: &mut GlobalStats,
    token_vault: AccountInfo<'info>,
    token: AccountInfo<'info>,
    token_vault_owner: AccountInfo<'info>,
    token_program: AccountInfo<'info>,
    vault_owner_bump: u8,
) -> Result<()> {
    let mut borrowed: u64 = 0;
    let mut collateral: u64 = 0;
    let mut last_liquidation_date = global_state.last_liquidation_date;

    let current_timestamp = Clock::get().unwrap().unix_timestamp;
    while (global_state.last_liquidation_date < current_timestamp) {
        collateral += last_liquidation_date_state.collateral;
        borrowed += last_liquidation_date_state.borrowed;
        global_state.last_liquidation_date += SECONDS_IN_A_DAY;
    }

    if collateral != 0 {
        global_state.total_collateral -= collateral;
        burn_tokens(
            token_vault,
            token,
            token_vault_owner,
            token_program,
            collateral,
            Some(&[&[VAULT_SEED, &[vault_owner_bump]]]),
        )?;
    }

    if borrowed != 0 {
        global_state.total_borrowed -= borrowed;
    }
    Ok(())
}

// pub fn safety_check<'info>(
//     main_state: AccountInfo<'info>,
//     token_vault_owner: AccountInfo<'info>,
// ) -> Result<()> {
//     let mut state = MainState::try_from_slice(&main_state.data.borrow())?;
//     let new_price: u64 = state.get_backing().unwrap()
//         .checked_mul(1 * LAMPORTS_PER_SOL).unwrap()
//         .checked_div(state.token_supply).unwrap();
//     let _total_collateral = token_vault_owner.lamports();
//     if _total_collateral < state.total_collateral {
//         return Err(SonicProgramError::SafetyCheckFailed.into());
//     }
//     if state.last_price > new_price {
//         return Err(SonicProgramError::SafetyCheckFailed.into());
//     }
//     state.last_price = new_price;
//     Ok(())
// }

pub fn get_midnight_timestamp(timestamp: i64) -> i64 {
    timestamp - (timestamp % SECONDS_IN_A_DAY) + SECONDS_IN_A_DAY
}

pub fn get_date_from_timestamp(timestamp: i64) -> i64 {
    timestamp - (timestamp % SECONDS_IN_A_DAY)
}

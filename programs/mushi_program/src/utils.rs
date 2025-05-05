use crate::state::DailyStats;
use crate::{
    constants::{FEES_BUY, SECONDS_IN_A_DAY, VAULT_SEED},
    error::MushiProgramError,
    state::{GlobalStats, MainState},
};
use anchor_lang::{
    prelude::*,
    solana_program::{
        program::{invoke, invoke_signed},
        system_instruction::transfer,
    },
};
use anchor_spl::{
    token::{self, Burn, MintTo, Token, TokenAccount, Transfer},
    token_2022::{self, transfer_checked, TransferChecked},
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
            CpiContext::new_with_signer(
                token_program.clone(),
                token_transfer_accounts,
                signer_seeds,
            ),
            amount,
        )?;
    } else {
        token::transfer(
            CpiContext::new(token_program, token_transfer_accounts),
            amount,
        )?;
    }
    Ok(())
}

pub fn transfer_tokens_checked<'info>(
    from: AccountInfo<'info>,
    to: AccountInfo<'info>,
    authority: AccountInfo<'info>,
    mint: AccountInfo<'info>,
    token_program: AccountInfo<'info>,
    amount: u64,
    decimals: u8,
    signer_seeds: Option<&[&[&[u8]]]>,
) -> Result<()> {
    let token_transfer_accounts = TransferChecked {
        from,
        to,
        authority,
        mint,
    };
    if let Some(signer_seeds) = signer_seeds {
        token_2022::transfer_checked(
            CpiContext::new_with_signer(token_program, token_transfer_accounts, signer_seeds),
            amount,
            decimals,
        )?;
    } else {
        token_2022::transfer_checked(
            CpiContext::new(token_program, token_transfer_accounts),
            amount,
            decimals,
        )?;
    }
    Ok(())
}

// pub fn transfer_sol<'info>(
//     sender: AccountInfo<'info>,
//     receiver: AccountInfo<'info>,
//     system_program: AccountInfo<'info>,
//     amount: u64,
//     signer_seeds: Option<&[&[&[u8]]]>,
// ) -> Result<()> {
//     let ix = transfer(sender.key, receiver.key, amount);
//     if let Some(signer_seeds) = signer_seeds {
//         invoke_signed(&ix, &[sender, receiver, system_program], signer_seeds)?;
//     } else {
//         invoke(&ix, &[sender, receiver, system_program])?;
//     }
//     Ok(())
// }

// pub fn trasnfer_sol_to_pubkey<'info>(
//     sender: AccountInfo<'info>,
//     receiver: &Pubkey,
//     system_program: AccountInfo<'info>,
//     amount: u64,
// ) -> Result<()> {
//     let ix = transfer(sender.key, receiver, amount);
//     invoke(&ix, &[sender, system_program])?;
//     Ok(())
// }

pub fn liquidate<'info>(
    last_liquidation_date_state: &mut DailyStats,
    global_state: &mut GlobalStats,
    token_vault: AccountInfo<'info>,
    token: AccountInfo<'info>,
    token_vault_owner: AccountInfo<'info>,
    token_program: AccountInfo<'info>,
    vault_owner_bump: u8,
) -> Result<()> {
    let current_timestamp = Clock::get().unwrap().unix_timestamp;
    
    // Only process one day at a time
    if global_state.last_liquidation_date < current_timestamp {
        // Accumulate borrowed and collateral for the current day
        let borrowed = last_liquidation_date_state.borrowed;
        let collateral = last_liquidation_date_state.collateral;
        
        // Update global stats
        if collateral > 0 {
            global_state.total_collateral = global_state.total_collateral.saturating_sub(collateral);
            
            // Burn the collateral tokens
            burn_tokens(
                token_vault,
                token,
                token_vault_owner,
                token_program,
                collateral,
                Some(&[&[VAULT_SEED, &[vault_owner_bump]]]),
            )?;
        }
        
        if borrowed > 0 {
            global_state.total_borrowed = global_state.total_borrowed.saturating_sub(borrowed);
        }
        
        // Advance to the next day
        global_state.last_liquidation_date += SECONDS_IN_A_DAY;
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

pub fn get_interest_fee(amount: u64, number_of_days: u64) -> u64 {
    // Daily interest rate of 3.9% (0.039) plus base fee of 0.1% (0.001)
    // Using 1e9 as precision factor since we're working with u64 instead of u256
    let daily_rate = 39_000_000; // 0.039 * 1e9
    let base_fee = 1_000_000; // 0.001 * 1e9

    // Calculate total interest rate: (daily_rate * days / 365) + base_fee
    let total_interest = ((daily_rate as u128 * number_of_days as u128) / 365) + base_fee as u128;

    // Calculate final fee: (amount * total_interest) / 1e9
    ((amount as u128 * total_interest) / 1_000_000_000) as u64
}

/// Converts a Unix timestamp to a date string in YYYY-MM-DD format.
/// First normalizes the timestamp to midnight (00:00:00) of the day.
pub fn get_date_string_from_timestamp(timestamp: i64) -> Result<String> {
    // Normalize to midnight
    let normalized_timestamp = get_date_from_timestamp(timestamp);

    // Create a readable date in UTC
    let seconds = normalized_timestamp;
    let days_since_epoch = seconds / SECONDS_IN_A_DAY;

    // 1970-01-01 is the Unix epoch (day 0)
    let mut year = 1970;
    let mut days_remaining = days_since_epoch;

    // Advance through years
    loop {
        let days_in_year = if is_leap_year(year) { 366 } else { 365 };
        if days_remaining < days_in_year {
            break;
        }
        days_remaining -= days_in_year;
        year += 1;
    }

    // Determine month and day
    let days_in_months = if is_leap_year(year) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    let mut month = 0;
    for days_in_month in days_in_months.iter() {
        if days_remaining < *days_in_month {
            break;
        }
        days_remaining -= *days_in_month;
        month += 1;
    }

    // Month is 0-based in our calculation, but we want 1-based
    let month = month + 1;
    // Day is 0-based, need to add 1
    let day = days_remaining + 1;

    // Format as YYYY-MM-DD
    Ok(format!("{:04}-{:02}-{:02}", year, month, day))
}

// Helper function to determine if a year is a leap year
fn is_leap_year(year: i64) -> bool {
    (year % 4 == 0) && (year % 100 != 0 || year % 400 == 0)
}

pub fn add_loans_by_date(
    global_state: &mut Box<Account<GlobalStats>>,
    daily_state: &mut Box<Account<DailyStats>>,
    borrowed: u64,
    collateral: u64,
) -> Result<()> {
    daily_state.borrowed += borrowed;
    daily_state.collateral += collateral;
    global_state.total_borrowed += borrowed;
    global_state.total_collateral += collateral;
    Ok(())
}

pub fn sub_loans_by_date(
    global_state: &mut Box<Account<GlobalStats>>,
    daily_state: &mut Box<Account<DailyStats>>,
    borrowed: u64,
    collateral: u64,
) -> Result<()> {
    daily_state.borrowed -= borrowed;
    daily_state.collateral -= collateral;
    global_state.total_borrowed -= borrowed;
    global_state.total_collateral -= collateral;
    Ok(())
}

/// Returns the current date in YYYY-MM-DD format
pub fn get_current_date_string() -> Result<String> {
    get_date_string_from_timestamp(Clock::get()?.unix_timestamp)
}

/// Returns the global state's last liquidation date in YYYY-MM-DD format
pub fn get_liquidation_date_string(last_liquidation_date: i64) -> Result<String> {
    get_date_string_from_timestamp(last_liquidation_date)
}
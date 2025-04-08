use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token_interface};

use crate::{
    constants::{FEE_BASE_1000, LAMPORTS_PER_SOL, SECONDS_IN_A_DAY, VAULT_SEED}, error::MushiProgramError, state::{GlobalStats, MainState, UserLoan}, utils::{get_date_from_timestamp, get_date_string_from_timestamp, get_interest_fee}, DailyStats 
};

#[derive(Accounts)]
pub struct ACommon<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(    
        mut,
        seeds = [MainState::PREFIX_SEED],
        bump,
    )]
    pub main_state: Box<Account<'info, MainState>>,
    #[account(    
        mut,
        seeds = [GlobalStats::PREFIX_SEED],
        bump,
    )]
    pub global_state: Box<Account<'info, GlobalStats>>,
    #[account(
        init_if_needed,
        payer = user,
        space = 8 + DailyStats::MAX_SIZE,
        seeds = [
            b"daily-stats".as_ref(),
            get_date_string_from_timestamp(Clock::get()?.unix_timestamp).as_bytes()  
        ],
        bump
    )]
    pub daily_state: Box<Account<'info, DailyStats>>,
    #[account(
        init_if_needed,
        payer = user,
        space = 8 + DailyStats::MAX_SIZE,
        seeds = [
            b"daily-stats".as_ref(),
            get_date_string_from_timestamp(global_state.last_liquidation_date).as_bytes()
        ],
        bump
    )]
    pub last_liquidation_date_state: Box<Account<'info, DailyStats>>,
    #[account(
        init_if_needed,
        payer = user,
        space = 8 + UserLoan::MAX_SIZE,
        seeds = [
            b"user-loan".as_ref(),
            user.key().as_ref()
        ],
        bump
    )]
    pub user_loan: Box<Account<'info, UserLoan>>,
    #[account(
        mut,
        address=main_state.fee_receiver,
    )]
    pub fee_receiver:SystemAccount<'info>,
    #[account(
        mut,
        address = global_state.token,
    )]
    pub token: Box<InterfaceAccount<'info, token_interface::Mint>>,
    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = token,
        associated_token::authority = user,
    )]
    pub user_ata: Box<InterfaceAccount<'info, token_interface::TokenAccount>>,
    #[account(
        mut,
        seeds = [VAULT_SEED],
        bump,
    )]
    pub token_vault_owner: SystemAccount<'info>,
    
    #[account(
        mut,
        token::mint = token,
        token::authority = token_vault_owner,
    )]
    pub token_vault: Box<InterfaceAccount<'info, token_interface::TokenAccount>>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, token_interface::TokenInterface>,
    pub system_program: Program<'info, System>,
}

impl<'info> ACommon<'info> {
    /// Returns the current date in YYYY-MM-DD format
    pub fn get_current_date_string(&self) -> Result<String> {
        Ok(get_date_string_from_timestamp(Clock::get()?.unix_timestamp))
    }

    /// Returns the global state's last liquidation date in YYYY-MM-DD format
    pub fn get_liquidation_date_string(&self) -> Result<String> {
        Ok(get_date_string_from_timestamp(self.global_state.last_liquidation_date))
    }

    pub fn get_backing(&self) -> Result<u64> {
        Ok(self.global_state.total_borrowed + self.token_vault_owner.lamports())
    }
    pub fn sol_to_mushi(&self, sol_amount: u64) -> Result<u64>{
        Ok(
            ((sol_amount as u128).checked_mul(self.global_state.token_supply as u128).unwrap())
            .checked_div(
                self.get_backing()?.checked_sub(sol_amount).unwrap() as u128
            ).unwrap() as u64
        )
    }
    pub fn mushi_to_sol(&self, mushi_amount: u64) -> Result<u64>{
        Ok(
            ((mushi_amount as u128).checked_mul(self.get_backing()? as u128).unwrap())
            .checked_div(self.global_state.token_supply as u128).unwrap() as u64
        )
    }
    pub fn sol_to_mushi_lev(&self, sol_amount: u64, fee: u64) -> Result<u64>{
        let backing = self.get_backing()? - fee;
        Ok(
            ((sol_amount as u128).checked_mul(self.global_state.token_supply as u128).unwrap())
            .checked_add((backing - 1) as u128).unwrap()
            .checked_div(backing as u128).unwrap() as u64
        )
    }
    pub fn sol_to_mushi_no_trade_ceil(&self, sol_amount: u64) -> Result<u64>{
        Ok(
            ((sol_amount as u128).checked_mul(self.global_state.token_supply as u128).unwrap())
            .checked_add(self.get_backing()? as u128 - 1).unwrap()
            .checked_div(self.get_backing()? as u128).unwrap() as u64
        )
    }
    pub fn sol_to_mushi_no_trade(&self, sol_amount: u64) -> Result<u64>{
        Ok(
            ((sol_amount as u128).checked_mul(self.global_state.token_supply as u128).unwrap())
            .checked_div(self.get_backing()? as u128).unwrap() as u64
        )
    }
    pub fn safety_check(&mut self) -> Result<()> {
        let backing = self.get_backing()?;
        let new_price: u64 = (backing as u128).checked_mul(LAMPORTS_PER_SOL as u128).unwrap()
        .checked_div(self.global_state.token_supply as u128).unwrap() as u64;
        let _total_collateral = self.token_vault.amount;

        require!(
            _total_collateral >= self.global_state.total_collateral,
            MushiProgramError::SafetyCheckCollateralFailed
        );
        require!(
            new_price >= self.global_state.last_price,
            MushiProgramError::SafetyCheckPriceFailed
        );
        self.global_state.last_price = new_price;
        Ok(())
    }
    pub fn is_loan_expired(&self) -> Result<bool> {
        let end_date = self.user_loan.end_date;
        let current_date = Clock::get()?.unix_timestamp;
        Ok(end_date < current_date)
    }
    
    pub fn leverage_fee(&self, sol_amount: u64, number_of_days: u64) -> Result<u64> {
        let buy_fee_leverage = self.main_state.buy_fee_leverage;
        let mint_fee = sol_amount.checked_mul(buy_fee_leverage).unwrap().checked_div(FEE_BASE_1000).unwrap();
        let interest = get_interest_fee(sol_amount, number_of_days);
        Ok(mint_fee.checked_add(interest).unwrap())
    }
}

#[derive(Accounts)]
pub struct ACommonExtReferral<'info> {
    pub common: ACommon<'info>, // Embed the existing ACommon struct
    
    #[account(mut)]
    pub referral: Option<UncheckedAccount<'info>>,
}

impl<'info> ACommonExtReferral<'info> {
}

#[derive(Accounts)]
#[instruction(number_of_days: i64)]
pub struct ACommonExtLoan<'info> {
    pub common: ACommon<'info>, // Embed the existing ACommon struct
    
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(
        init_if_needed,
        payer = user,
        space = 8 + DailyStats::MAX_SIZE,
        seeds = [
            b"daily-stats".as_ref(),
            get_date_string_from_timestamp(Clock::get()?.unix_timestamp + (number_of_days+1) * SECONDS_IN_A_DAY).as_bytes()
        ],
        bump
    )]
    pub daily_state_end_date: Box<Account<'info, DailyStats>>, 
    pub system_program: Program<'info, System>,
}

impl<'info> ACommonExtLoan<'info> {
    pub fn add_loans_by_date(&mut self, borrowed: u64, collateral: u64) -> Result<()> {
        let daily_state = &mut self.daily_state_end_date;
        let global_state = &mut self.common.global_state;
        daily_state.borrowed += borrowed;
        daily_state.collateral += collateral;
        global_state.total_borrowed += borrowed;
        global_state.total_collateral += collateral;
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(number_of_days: i64)]
pub struct ACommonExtBorrowMore<'info> {
    pub common: ACommon<'info>, // Embed the existing ACommon struct
    
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(
        seeds = [
            b"daily-stats".as_ref(),
            get_date_string_from_timestamp(common.user_loan.end_date).as_bytes()
        ],
        bump
    )]
    pub daily_state_end_date: Box<Account<'info, DailyStats>>, 
}

impl<'info> ACommonExtBorrowMore<'info> {
    pub fn add_loans_by_date(&mut self, borrowed: u64, collateral: u64) -> Result<()> {
        let daily_state = &mut self.daily_state_end_date;
        let global_state = &mut self.common.global_state;
        daily_state.borrowed += borrowed;
        daily_state.collateral += collateral;
        global_state.total_borrowed += borrowed;
        global_state.total_collateral += collateral;
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(number_of_days: i64)]
pub struct ACommonExtLoan2<'info> {
    pub common: ACommon<'info>, // Embed the existing ACommon struct
    
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(
        seeds = [
            b"daily-stats".as_ref(),
            get_date_string_from_timestamp(common.user_loan.end_date).as_bytes()
        ],
        bump
    )]
    pub daily_state_old_end_date: Box<Account<'info, DailyStats>>, 

    #[account(
        init_if_needed,
        payer = user,
        space = 8 + DailyStats::MAX_SIZE,
        seeds = [
            b"daily-stats".as_ref(),
            get_date_string_from_timestamp(Clock::get()?.unix_timestamp + (number_of_days+1) * SECONDS_IN_A_DAY).as_bytes()
        ],
        bump
    )]
    pub daily_state_end_date: Box<Account<'info, DailyStats>>, 
    pub system_program: Program<'info, System>,
}


impl<'info> ACommonExtLoan2<'info> {
    pub fn add_loans_by_date(&mut self, borrowed: u64, collateral: u64) -> Result<()> {
        let daily_state = &mut self.daily_state_end_date;
        let global_state = &mut self.common.global_state;
        daily_state.borrowed += borrowed;
        daily_state.collateral += collateral;
        global_state.total_borrowed += borrowed;
        global_state.total_collateral += collateral;
        Ok(())
    }
    pub fn sub_loans_by_date(&mut self, borrowed: u64, collateral: u64, date: i64) -> Result<()> {
        let daily_state = &mut self.daily_state_old_end_date;
        let global_state = &mut self.common.global_state;
        daily_state.borrowed -= borrowed;
        daily_state.collateral -= collateral;
        global_state.total_borrowed -= borrowed;
        global_state.total_collateral -= collateral;
        Ok(())
    }
}


#[derive(Accounts)]
pub struct ACommonExtSubLoan<'info> {
    pub common: ACommon<'info>, // Embed the existing ACommon struct
    
    #[account(
        seeds = [
            b"daily-stats".as_ref(),
            get_date_string_from_timestamp(common.user_loan.end_date).as_bytes()
        ],
        bump
    )]
    pub daily_state_old_end_date: Box<Account<'info, DailyStats>>, 
}


impl<'info> ACommonExtSubLoan<'info> {
    pub fn sub_loans_by_date(&mut self, borrowed: u64, collateral: u64, date: i64) -> Result<()> {
        let daily_state = &mut self.daily_state_old_end_date;
        let global_state = &mut self.common.global_state;
        daily_state.borrowed -= borrowed;
        daily_state.collateral -= collateral;
        global_state.total_borrowed -= borrowed;
        global_state.total_collateral -= collateral;
        Ok(())
    }
}
use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token_interface};

use crate::{
    constants::{FEE_BASE_1000, LAMPORTS_PER_ECLIPSE, SECONDS_IN_A_DAY, VAULT_SEED}, error::MushiProgramError, state::{GlobalStats, MainState, UserLoan}, utils::{get_date_from_timestamp, get_date_string_from_timestamp, get_interest_fee}, DailyStats 
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
        address = global_state.base_token,
    )]
    pub token: Box<InterfaceAccount<'info, token_interface::Mint>>,
    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = token,
        associated_token::authority = user,
        associated_token::token_program = base_token_program,
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

    #[account(
        mut,
        address = main_state.quote_token,
    )]
    pub quote_mint: Box<InterfaceAccount<'info, token_interface::Mint>>,
    #[account(
        // init_if_needed,
        // payer = user,
        // associated_token::mint = quote_mint,
        // associated_token::authority = token_vault_owner,
        // associated_token::token_program = quote_token_program,

        mut,
        token::mint = quote_mint,
        token::authority = token_vault_owner,
        token::token_program = quote_token_program,
    )]
    pub quote_vault: Box<InterfaceAccount<'info, token_interface::TokenAccount>>,
    #[account(
        // init_if_needed,
        // payer = user,
        // associated_token::mint = quote_mint,
        // associated_token::authority = user,
        // associated_token::token_program = quote_token_program,

        mut,
        token::mint = quote_mint,
        token::authority = user,
        token::token_program = quote_token_program,
    )]
    pub user_quote_ata: Box<InterfaceAccount<'info, token_interface::TokenAccount>>,
    #[account(
        // init_if_needed,
        // payer = user,
        // associated_token::mint = quote_mint,
        // associated_token::authority = fee_receiver,
        // associated_token::token_program = quote_token_program,

        mut,
        token::mint = quote_mint,
        token::authority = fee_receiver,
        token::token_program = quote_token_program,
    )]
    pub fee_receiver_quote_ata: Box<InterfaceAccount<'info, token_interface::TokenAccount>>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub base_token_program: Interface<'info, token_interface::TokenInterface>,
    pub quote_token_program: Interface<'info, token_interface::TokenInterface>,
    pub system_program: Program<'info, System>,
}

impl<'info> ACommon<'info> {
    

    pub fn get_backing(&self, es_amount: u64) -> Result<u64> {
        Ok(self.global_state.total_borrowed + self.quote_vault.amount + es_amount + self.global_state.total_eclipse_token_staked)
    }
    pub fn eclipse_to_mushi(&self, es_amount: u64) -> Result<u64>{
        Ok(
            ((es_amount as u128).checked_mul(self.global_state.token_supply as u128).unwrap())
            .checked_div(
                self.get_backing(es_amount)?.checked_sub(es_amount).unwrap() as u128
            ).unwrap() as u64
        )
    }
    pub fn mushi_to_eclipse(&self, mushi_amount: u64) -> Result<u64>{
        Ok(
            ((mushi_amount as u128).checked_mul(self.get_backing(0)? as u128).unwrap())
            .checked_div(self.global_state.token_supply as u128).unwrap() as u64
        )
    }
    pub fn eclipse_to_mushi_lev(&self, es_amount: u64, fee: u64, total_es_amount: u64) -> Result<u64>{
        let backing = self.get_backing(total_es_amount)? - fee;
        Ok(
            ((es_amount as u128).checked_mul(self.global_state.token_supply as u128).unwrap())
            .checked_add((backing - 1) as u128).unwrap()
            .checked_div(backing as u128).unwrap() as u64
        )
    }
    pub fn eclipse_to_mushi_no_trade_ceil(&self, es_amount: u64) -> Result<u64>{
        Ok(
            ((es_amount as u128).checked_mul(self.global_state.token_supply as u128).unwrap())
            .checked_add(self.get_backing(0)? as u128 - 1).unwrap()
            .checked_div(self.get_backing(0)? as u128).unwrap() as u64
        )
    }
    pub fn eclipse_to_mushi_no_trade(&self, es_amount: u64) -> Result<u64>{
        Ok(
            ((es_amount as u128).checked_mul(self.global_state.token_supply as u128).unwrap())
            .checked_div(self.get_backing(0)? as u128).unwrap() as u64
        )
    }
    pub fn safety_check(&mut self, es_amount: u64, plus:bool) -> Result<()> {
        let mut backing = self.get_backing(0)?;
        if plus {
            backing = backing.checked_add(es_amount).unwrap();
        } else {
            backing = backing.checked_sub(es_amount).unwrap();
        }

        let new_price: u64 = (backing as u128).checked_mul(LAMPORTS_PER_ECLIPSE as u128).unwrap()
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

    pub fn safety_check_borrow(&mut self, es_amount: u64, plus:bool, mushi_amount: u64) -> Result<()> {
        let mut backing = self.get_backing(0)?;
        if plus {
            backing = backing.checked_add(es_amount).unwrap();
        } else {
            backing = backing.checked_sub(es_amount).unwrap();
        }

        let new_price: u64 = (backing as u128).checked_mul(LAMPORTS_PER_ECLIPSE as u128).unwrap()
                                            .checked_div(self.global_state.token_supply as u128).unwrap() as u64;
        let _total_collateral = self.token_vault.amount + mushi_amount;

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
    
    pub fn leverage_fee(&self, es_amount: u64, number_of_days: u64) -> Result<u64> {
        let buy_fee_leverage = self.main_state.buy_fee_leverage;
        let mint_fee = es_amount.checked_mul(buy_fee_leverage).unwrap().checked_div(FEE_BASE_1000).unwrap();
        let interest = get_interest_fee(es_amount, number_of_days);
        Ok(mint_fee.checked_add(interest).unwrap())
    }
}

#[derive(Accounts)]
#[instruction(referral_address: Pubkey)]
pub struct ACommonExtReferral<'info> {
    pub common: ACommon<'info>, // Embed the existing ACommon struct
    
    #[account(
        mut,
    )]
    pub referral: SystemAccount<'info>,

    #[account(
        // init,
        // payer = common.user,
        // associated_token::mint = common.quote_mint,
        // associated_token::authority = referral,
        // associated_token::token_program = quote_token_program,

        mut,
        // init_if_needed,
        // payer = common.user,
        token::mint = common.quote_mint,
        token::authority = referral,
        token::token_program = quote_token_program,
    )]
    pub referral_quote_ata: Box<InterfaceAccount<'info, token_interface::TokenAccount>>,
    
    pub quote_token_program: Interface<'info, token_interface::TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
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

#[derive(Accounts)]
#[instruction(number_of_days: i64)]
pub struct ACommonExtLoan2<'info> {
    pub common: ACommon<'info>, // Embed the existing ACommon struct
    
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(
        mut,
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

#[derive(Accounts)]
pub struct ACommonExtSubLoan<'info> {
    pub common: ACommon<'info>, // Embed the existing ACommon struct
    
    #[account(
        mut,
        seeds = [
            b"daily-stats".as_ref(),
            get_date_string_from_timestamp(common.user_loan.end_date).as_bytes()
        ],
        bump
    )]
    pub daily_state_old_end_date: Box<Account<'info, DailyStats>>, 
}


#[derive(Accounts)]
#[instruction(number_of_days: i64)]
pub struct ACommonExtExtendLoan<'info> {
    pub common: ACommon<'info>, // Embed the existing ACommon struct
    
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(
        mut,
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
            get_date_string_from_timestamp(common.user_loan.end_date + (number_of_days) * SECONDS_IN_A_DAY).as_bytes()
        ],
        bump
    )]
    pub daily_state_new_end_date: Box<Account<'info, DailyStats>>, 
    pub system_program: Program<'info, System>,
}

use anchor_lang::prelude::*;

use crate::{
    constants::{
        SECONDS_IN_A_DAY, FEE_BASE_1000,
    },
    error::MushiProgramError,
};

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct DateEntry {
    pub date: u64,
    pub amount: u64,
}

#[account]
pub struct MainState {
    pub admin: Pubkey,
    pub token: Pubkey,
    pub fee_receiver: Pubkey,
    pub last_liquidation_date: i64,
    pub buy_fee: u64,
    pub sell_fee: u64,
    pub buy_fee_leverage: u64,
    pub token_supply: u64,
    pub total_borrowed: u64,
    pub total_collateral: u64,
    pub last_price: u64,
    pub sol_balance_in_vault: u64,
    pub borrowed_by_date: Vec<DateEntry>,
    pub collateral_by_date: Vec<DateEntry>,
}

impl MainState {
    pub const PREFIX_SEED: &'static [u8] = b"main_state";
    pub const MAX_SIZE: usize = std::mem::size_of::<Self>();

    pub fn calc_buy_amount(&self, sol_amount: u64) -> Result<u64> {
        let token_amount = sol_amount
            .checked_mul(self.token_supply)
            .unwrap()
            .checked_div(self.sol_balance_in_vault - sol_amount)
            .unwrap();
        return Ok(token_amount);
    }

    pub fn calc_sell_amount(&self, token_amount: u64) -> Result<u64> {
        let sol_amount = token_amount
            .checked_mul(self.get_backing().unwrap())
            .unwrap()
            .checked_div(self.token_supply)
            .unwrap();
        return Ok(sol_amount);
    }

    pub fn calc_buy_amount_lev(&self, value: u64, fee: u64) -> Result<u64> {
        let backing = self.get_backing().unwrap().checked_sub(fee).unwrap();
        let token_amount = value.checked_mul(self.token_supply).unwrap()
            .checked_add(backing - 1).unwrap()
            .checked_div(backing).unwrap();
        Ok(token_amount)
    }

    pub fn calc_buy_amount_no_trade_ceil(&self, value: u64) -> Result<u64> {
        let backing = self.get_backing().unwrap();
        let token_amount = value.checked_mul(self.token_supply).unwrap()
            .checked_add(backing - 1).unwrap()
            .checked_div(backing).unwrap();
        Ok(token_amount)
    }
    
    pub fn calc_buy_amount_no_trade(&self, value: u64) -> Result<u64> {
        let backing = self.get_backing().unwrap();
        let token_amount = value.checked_mul(self.token_supply).unwrap()
            .checked_div(backing).unwrap();
        Ok(token_amount)
    }
    
    pub fn get_midnight_timestamp(&self, date: u64) -> Result<u64> {
        let mid_night_timestamp = date - (date % (SECONDS_IN_A_DAY as u64));
        Ok(mid_night_timestamp + (SECONDS_IN_A_DAY as u64))
    }

    pub fn get_backing(&self) -> Result<u64> {
        let backing = self.sol_balance_in_vault + self.total_borrowed;
        Ok(backing)
    }

    pub fn update_borrowed(&mut self, date: u64, amount: u64) -> Result<()> {
        if let Some(entry) = self.borrowed_by_date
            .iter_mut()
            .find(|e| e.date == date) 
        {
            entry.amount = amount;
        } else {
            self.borrowed_by_date.push(DateEntry { date, amount });
        }
        Ok(())
    }

    pub fn update_collateral(&mut self, date: u64, amount: u64) -> Result<()> {
        if let Some(entry) = self.collateral_by_date
            .iter_mut()
            .find(|e| e.date == date) 
        {
            entry.amount = amount;
        } else {
            self.collateral_by_date.push(DateEntry { date, amount });
        }
        Ok(())
    }

    // Get borrowed amount for a date
    pub fn get_borrowed(&self, date: u64) -> u64 {
        self.borrowed_by_date
            .iter()
            .find(|e| e.date == date)
            .map(|e| e.amount)
            .unwrap_or(0)
    }

    // Get collateral amount for a date
    pub fn get_collateral(&self, date: u64) -> u64 {
        self.collateral_by_date
            .iter()
            .find(|e| e.date == date)
            .map(|e| e.amount)
            .unwrap_or(0)
    }

    pub fn add_loans_by_date(&mut self, borrowed: u64, collateral: u64, date: u64) -> Result<()> {
        // Update borrowed amount for the date
        if let Some(entry) = self.borrowed_by_date
            .iter_mut()
            .find(|e| e.date == date) 
        {
            entry.amount = entry.amount.checked_add(borrowed).unwrap();
        } else {
            self.borrowed_by_date.push(DateEntry { date, amount: borrowed });
        }

        // Update collateral amount for the date
        if let Some(entry) = self.collateral_by_date
            .iter_mut()
            .find(|e| e.date == date) 
        {
            entry.amount = entry.amount.checked_add(collateral).unwrap();
        } else {
            self.collateral_by_date.push(DateEntry { date, amount: collateral });
        }

        // Update total amounts
        self.total_borrowed = self.total_borrowed.checked_add(borrowed).unwrap();
        self.total_collateral = self.total_collateral.checked_add(collateral).unwrap();

        Ok(())
    }

    pub fn sub_loans_by_date(&mut self, borrowed: u64, collateral: u64, date: u64) -> Result<()> {
        // Update borrowed amount for the date
        if let Some(entry) = self.borrowed_by_date
            .iter_mut()
            .find(|e| e.date == date) 
        {
            entry.amount = entry.amount.checked_sub(borrowed).unwrap();
        } else {
            self.borrowed_by_date.push(DateEntry { date, amount: 0 });
        }

        // Update collateral amount for the date
        if let Some(entry) = self.collateral_by_date
            .iter_mut()
            .find(|e| e.date == date) 
        {
            entry.amount = entry.amount.checked_sub(collateral).unwrap();
        } else {
            self.collateral_by_date.push(DateEntry { date, amount: 0 });
        }

        // Update total amounts
        self.total_borrowed = self.total_borrowed.checked_sub(borrowed).unwrap();
        self.total_collateral = self.total_collateral.checked_sub(collateral).unwrap();

        Ok(())  
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct Loan {
    pub collateral: u64,
    pub borrowed: u64,
    pub end_date: u64,
    pub number_of_days: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct LoanEntry {
    pub loan: Loan,
    pub address: Pubkey,
}

#[account]
pub struct LoanState {
    pub loans: Vec<LoanEntry>,
}

impl LoanState {
    pub const PREFIX_SEED: &'static [u8] = b"loan_state";
    pub const MAX_SIZE: usize = std::mem::size_of::<Self>();

    pub fn set_loan(&mut self, address: Pubkey, collateral: u64, borrowed: u64, end_date: u64, number_of_days: u64) {
        let loan = Loan {
            collateral,
            borrowed,
            end_date,
            number_of_days,
        };
        
        let loan_entry = LoanEntry {
            loan,
            address,
        };

        // If loan exists for this address, update it
        if let Some(existing_entry) = self.loans.iter_mut().find(|entry| entry.address == address) {
            existing_entry.loan = loan_entry.loan;
        } else {
            // If no loan exists, add new entry
            self.loans.push(loan_entry);
        }
    }

    pub fn get_loan_by_address(&self, address: Pubkey) -> (u64, u64, u64, u64) {
        let current_time = Clock::get().unwrap().unix_timestamp as u64;
        
        if let Some(loan_entry) = self.loans.iter().find(|entry| entry.address == address) {
            if loan_entry.loan.end_date >= current_time {
                return (
                    loan_entry.loan.collateral,
                    loan_entry.loan.borrowed,
                    loan_entry.loan.end_date,
                    loan_entry.loan.number_of_days
                );
            }
        }
        
        (0, 0, 0, 0)
    }

    pub fn is_loan_expired(&self, address: Pubkey) -> bool {
        let current_time = Clock::get().unwrap().unix_timestamp as u64;
        
        if let Some(loan_entry) = self.loans.iter().find(|entry| entry.address == address) {
            return loan_entry.loan.end_date < current_time;
        }
        
        true // If no loan exists, consider it expired
    }

    pub fn delete_loan(&mut self, address: Pubkey) {
        if let Some(pos) = self.loans.iter().position(|entry| entry.address == address) {
            self.loans.remove(pos);
        }
    }

    pub fn leverage_fee(&self, sol_amount: u64, number_of_days: u64, buy_fee_leverage: u64) -> Result<u64> {
        // Calculate mint fee: (sonic * buy_fee_leverage) / FEE_BASE_1000
        let mint_fee = sol_amount
            .checked_mul(buy_fee_leverage)
            .unwrap()
            .checked_div(FEE_BASE_1000)
            .unwrap();

        // Calculate interest fee
        let interest = self.get_interest_fee(sol_amount, number_of_days)?;

        // Return total fee: mint_fee + interest
        Ok(mint_fee.checked_add(interest).unwrap())
    }

    pub fn get_interest_fee(&self, amount: u64, number_of_days: u64) -> Result<u64> {
        // Calculate interest rate: (0.039e18 * numberOfDays / 365) + 0.001e18
        let daily_rate: u64 = 39_000_000_000_000_000; // 0.039e18
        let base_rate: u64 = 1_000_000_000_000_000;   // 0.001e18
        let days_in_year: u64 = 365;
        
        let interest_rate = daily_rate
            .checked_mul(number_of_days)
            .unwrap()
            .checked_div(days_in_year)
            .unwrap()
            .checked_add(base_rate)
            .unwrap();

        // Calculate final interest: (amount * interest_rate) / 1e18
        let interest = amount
            .checked_mul(interest_rate)
            .unwrap()
            .checked_div(1_000_000_000_000_000_000) // 1e18
            .unwrap();

        Ok(interest)
    }
}
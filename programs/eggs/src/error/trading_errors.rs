use anchor_lang::prelude::*;

#[error_code]
pub enum TradingError {
    #[msg("Operation requires trading to be initialized")]
    TradingNotInitialized,
    #[msg("Fee address not set")]
    FeeAddressNotSet,
    #[msg("Invalid fee amount")]
    InvalidFeeAmount,
    #[msg("Value must be above minimum")]
    BelowMinimumValue,
    #[msg("Fee calculation error")]
    FeeCalculationError,
    #[msg("Price cannot decrease")]
    PriceDecrease,
    #[msg("Invalid parameter")]
    InvalidParameter,
    #[msg("Insufficient funds")]
    InsufficientFunds,
    #[msg("Max supply exceeded")]
    MaxSupplyExceeded,
    #[msg("Contract balance insufficient")]
    ContractBalanceInsufficient,
} 
 
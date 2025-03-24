use anchor_lang::prelude::*;

#[error_code]
pub enum EggsError {
    #[msg("Operation requires trading to be initialized")]
    TradingNotInitialized,
    #[msg("Fee address not set")]
    FeeAddressNotSet,
    #[msg("Invalid fee amount")]
    InvalidFeeAmount,
    #[msg("Value must be above minimum")]
    BelowMinimumValue,
    #[msg("Loan has expired")]
    LoanExpired,
    #[msg("User already has an active loan")]
    UserHasActiveLoan,
    #[msg("Insufficient collateral")]
    InsufficientCollateral,
    #[msg("Repay amount too large")]
    RepayAmountTooLarge,
    #[msg("Incorrect repayment amount")]
    IncorrectRepaymentAmount,
    #[msg("Fee calculation error")]
    FeeCalculationError,
    #[msg("Loan too long")]
    LoanTooLong,
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
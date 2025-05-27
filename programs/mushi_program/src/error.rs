use anchor_lang::prelude::error_code;

#[error_code]
pub enum MushiProgramError {

    #[msg("method caller is not authorised")]
    UnAuthorised,

    #[msg("invalid input")]
    InvalidInput,

    #[msg("amount is too small")]
    TooSmallInputAmount,

    #[msg("buy fee must be greater than FEES_BUY and be less than 2.5%")]
    InvalidBuyFee,

    #[msg("sell fee must be greater than FEES_SELL and be less than 2.5%")]
    InvalidSellFee,

    #[msg("leverage buy fee must be less 2.5%")]
    InvalidBuyFeeLeverage,

    #[msg("The mushi balance of the contract must be greater than or equal to the collateral")]
    SafetyCheckCollateralFailed,
    
    #[msg("The price of mushi cannot decrease")]
    SafetyCheckPriceFailed,

    #[msg("invalid number of days")]
    InvalidNumberOfDays,

    #[msg("invalid fee amount")]
    InvalidFeeAmount,

    #[msg("invalid fee receiver")]
    InvalidFeeReceiver,

    #[msg("invalid sol amount")]
    InvalidSolAmount,
    
    #[msg("invalid eclipse amount")]
    InvalidEclipseAmount,

    #[msg("insufficient eclipse amount on vault")]
    InsufficientEclipseAmountOnVault,

    #[msg("insufficient mushi amount on user ata")]
    InsufficientMushiAmountOnUserAta,

    #[msg("invalid loan amount")]
    InvalidLoanAmount,

    #[msg("not started")]
    NotStarted,

    #[msg("already started")]
    AlreadyStarted,

    #[msg("too small team fee")]
    TooSmallTeamFee,

    #[msg("loan expired")]
    LoanExpired,

    #[msg("invalid collateral amount")]
    InvalidCollateralAmount,

    #[msg("remove collateral failed")]
    RemoveCollateralFailed,
    #[msg("referral not found")]
    ReferralNotFound,

    #[msg("invalid referral account")]
    InvalidReferralAccount,

    #[msg("stake is not enabled")]
    StakeNotEnabled,

    #[msg("max supply exceeded")]
    MaxSupplyExceeded,
    
    #[msg("not enough daily stats accounts provided for liquidation")]
    NotEnoughDailyStatsAccounts,
    
    #[msg("invalid daily stats account provided")]
    InvalidDailyStatsAccount,

    #[msg("too small output amount")]
    TooSmallOutputAmount,
}

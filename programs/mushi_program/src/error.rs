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

    #[msg("safety check failed")]
    SafetyCheckFailed,

    #[msg("invalid number of days")]
    InvalidNumberOfDays,

    #[msg("invalid fee amount")]
    InvalidFeeAmount,

    #[msg("invalid sol amount")]
    InvalidSolAmount,
    
    #[msg("invalid loan amount")]
    InvalidLoanAmount,

    #[msg("not started")]
    NotStarted,

    #[msg("too small team fee")]
    TooSmallTeamFee,

    #[msg("loan expired")]
    LoanExpired,

    #[msg("invalid collateral amount")]
    InvalidCollateralAmount,

    #[msg("remove collateral failed")]
    RemoveCollateralFailed,
}

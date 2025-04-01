use anchor_lang::prelude::error_code;

#[error_code]
pub enum MushiProgramError {
    #[msg("method caller is not authorised")]
    UnAuthorised,
    #[msg("invalid input")]
    InvalidInput,
    #[msg("amount is too small")]
    TooSmallInputAmount,
    #[msg("invalid buy fee")]
    InvalidBuyFee,
    #[msg("invalid sell fee")]
    InvalidSellFee,
    #[msg("invalid buy fee leverage")]
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
}

use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("The amount is not present in the token account")]
    AmountNotPresentInAccount,
    #[msg("The deposit amount is too small")]
    DepositTooSmall
}

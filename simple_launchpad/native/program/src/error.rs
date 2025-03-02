//! Error types

use {
    num_derive::FromPrimitive,
    solana_program::{decode_error::DecodeError, program_error::ProgramError},
    thiserror::Error,
};

/// Errors that may be returned by the StakePool program.
#[derive(Clone, Debug, Eq, Error, FromPrimitive, PartialEq)]
pub enum LaunchTokenError {
    #[error("Less than lowest amount")]
    LessThanLowestAmount,
    #[error("Launch not start")]
    LaunchNotStart,
    #[error("Launch end")]
    LaunchEnd,
    #[error("Claim not start")]
    ClaimNotStart,
    #[error("No reserve to claim ")]
    NoReserveToClaim,
    #[error("Incorrect seller address")]
    IncorrectSellerAddress,
    #[error("Incorrect project")]
    IncorrectProject,
    #[error("Permission forbidden")]
    PermissionForbidden,
    #[error("Mint account not exist")]
    MintAccountNotExist,
}
impl From<LaunchTokenError> for ProgramError {
    fn from(e: LaunchTokenError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
impl<T> DecodeError<T> for LaunchTokenError {
    fn type_of() -> &'static str {
        "Project Error"
    }
}

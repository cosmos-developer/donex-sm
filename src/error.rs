use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},
    #[error("Address already registered")]
    AddressAlreadyLinked {},
    #[error("Social account already registered")]
    SocialAlreadyLinked {},
    #[error("Social info not found")]
    SocialInfoNotFound {},
    // Add any other custom errors you like here.
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.
}

use borsh::{BorshDeserialize, BorshSerialize};
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct Mint {
    pub amount: u64,
    pub bump: u8,
}

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct Claim {}

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct CreateClaimAccount {
    pub bump: u8,
}

use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use solana_program::pubkey::Pubkey;

#[repr(C)]
#[derive(Clone, Debug, Default, PartialEq, BorshDeserialize, BorshSerialize, BorshSchema)]
pub struct ProjectInfo {
    pub manager: Pubkey,
    pub token_price: u64,
    pub seller_account: Pubkey,
    pub launch_start_time: u64,
    pub launch_end_time: u64,
    pub claim_start_time: u64,
    pub token_program_id: Pubkey,
}

#[repr(C)]
#[derive(Clone, Debug, Default, PartialEq, BorshDeserialize, BorshSerialize, BorshSchema)]
pub struct MintInfo {
    pub buyer: Pubkey,
    pub project: Pubkey,
    pub amount: u64,
    pub is_claimed: bool,
}

impl MintInfo {
    pub const SIZE: usize = 32 + 32 + 8 + 1;

    pub const SEED_PREFIX: &'static str = "octu";

    // pub fn new(page_visits: u32, bump: u8) -> Self {
    //     PageVisits { page_visits, bump }
    // }

    // pub fn increment(&mut self) {
    //     self.page_visits += 1;
    // }
}

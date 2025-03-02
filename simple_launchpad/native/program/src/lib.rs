use solana_program::entrypoint as solana_entrypoint;

use crate::entrypoint::process_instruction;

pub mod entrypoint;
pub mod error;
pub mod instruction;
pub mod processor;
pub mod state;

solana_entrypoint!(process_instruction);

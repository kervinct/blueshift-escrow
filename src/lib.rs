#![no_std]
use pinocchio::{
    AccountView, Address, ProgramResult, entrypoint, error::ProgramError, nostd_panic_handler,
};

mod helpers;
mod instructions;
mod state;
pub use instructions::*;

entrypoint!(process_instruction);
nostd_panic_handler!();

pub const ID: Address =
    pinocchio::address::address!("22222222222222222222222222222222222222222222");

fn process_instruction(
    _program_id: &Address,
    accounts: &[AccountView],
    instruction_data: &[u8],
) -> ProgramResult {
    match instruction_data.split_first() {
        Some((Make::DISCRIMINATOR, data)) => Make::try_from((data, accounts))?.process(),
        Some((Take::DISCRIMINATOR, _)) => Take::try_from(accounts)?.process(),
        Some((Refund::DISCRIMINATOR, _)) => Refund::try_from(accounts)?.process(),
        _ => Err(ProgramError::InvalidInstructionData),
    }
}

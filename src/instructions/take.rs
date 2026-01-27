use pinocchio::{
    AccountView, Address, ProgramResult,
    cpi::{Seed, Signer},
    error::ProgramError,
};
use pinocchio_token::instructions::Transfer;

use crate::helpers::*;

pub struct TakeAccounts<'a> {
    pub taker: &'a AccountView,
    pub maker: &'a AccountView,
    pub escrow: &'a AccountView,
    pub mint_a: &'a AccountView,
    pub mint_b: &'a AccountView,
    pub vault: &'a AccountView,
    pub taker_ata_a: &'a AccountView,
    pub taker_ata_b: &'a AccountView,
    pub maker_ata_b: &'a AccountView,
    pub system_program: &'a AccountView,
    pub token_program: &'a AccountView,
}

impl<'a> TryFrom<&'a [AccountView]> for TakeAccounts<'a> {
    type Error = ProgramError;
    fn try_from(accounts: &'a [AccountView]) -> Result<Self, Self::Error> {
        let [
            taker,
            maker,
            escrow,
            mint_a,
            mint_b,
            vault,
            taker_ata_a,
            taker_ata_b,
            maker_ata_b,
            system_program,
            token_program,
            _,
        ] = accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };
        SignerAccount::check(taker)?;
        ProgramAccount::check(escrow)?;
        MintInterface::check(mint_a)?;
        MintInterface::check(mint_b)?;
        AssociatedTokenAccount::check(taker_ata_b, taker, mint_b, token_program)?;
        AssociatedTokenAccount::check(vault, escrow, mint_a, token_program)?;
        Ok(Self {
            taker,
            maker,
            escrow,
            mint_a,
            mint_b,
            taker_ata_a,
            taker_ata_b,
            maker_ata_b,
            vault,
            system_program,
            token_program,
        })
    }
}

pub struct Take<'a> {
    pub accounts: TakeAccounts<'a>,
}
impl<'a> TryFrom<&'a [AccountView]> for Take<'a> {
    type Error = ProgramError;
    fn try_from(accounts: &'a [AccountView]) -> Result<Self, Self::Error> {
        let accounts = TakeAccounts::try_from(accounts)?;
        AssociatedTokenAccount::init_if_needed(
            accounts.taker_ata_a,
            accounts.mint_a,
            accounts.taker,
            accounts.taker,
            accounts.system_program,
            accounts.token_program,
        )?;
        AssociatedTokenAccount::init_if_needed(
            accounts.maker_ata_b,
            accounts.mint_b,
            accounts.taker,
            accounts.maker,
            accounts.system_program,
            accounts.token_program,
        )?;
        Ok(Self { accounts })
    }
}

impl<'a> Take<'a> {
    pub const DISCRIMINATOR: &'a u8 = &1;
    pub fn process(&mut self) -> ProgramResult {
        let data = self.accounts.escrow.try_borrow()?;
        let escrow = crate::state::Escrow::load(&data)?;
        let escrow_key = Address::create_program_address(
            &[
                b"escrow",
                self.accounts.maker.address().as_ref(),
                &escrow.seed.to_le_bytes(),
                &escrow.bump,
            ],
            &crate::ID,
        )?;
        if escrow_key.ne(self.accounts.escrow.address()) {
            return Err(ProgramError::InvalidAccountOwner);
        }

        let seed_binding = escrow.seed.to_le_bytes();
        let bump_binding = escrow.bump;
        let escrow_seeds = [
            Seed::from(b"escrow"),
            Seed::from(self.accounts.maker.address().as_ref()),
            Seed::from(seed_binding.as_ref()),
            Seed::from(bump_binding.as_ref()),
        ];
        let signer = Signer::from(&escrow_seeds);
        let amount =
            pinocchio_token::state::TokenAccount::from_account_view(self.accounts.vault)?.amount();

        Transfer {
            from: self.accounts.vault,
            to: self.accounts.taker_ata_a,
            authority: self.accounts.escrow,
            amount,
        }
        .invoke_signed(core::slice::from_ref(&signer))?;
        pinocchio_token::instructions::CloseAccount {
            account: self.accounts.vault,
            destination: self.accounts.maker,
            authority: self.accounts.escrow,
        }
        .invoke_signed(core::slice::from_ref(&signer))?;
        Transfer {
            from: self.accounts.taker_ata_b,
            to: self.accounts.maker_ata_b,
            authority: self.accounts.taker,
            amount: escrow.receive,
        }
        .invoke()?;

        drop(data);
        ProgramAccount::close(self.accounts.escrow, self.accounts.taker)?;
        Ok(())
    }
}

use crate::helpers::*;
use pinocchio::{
    AccountView, ProgramResult,
    cpi::{Seed, Signer},
    error::ProgramError,
};
use pinocchio_system::create_account_with_minimum_balance_signed;
use pinocchio_token::instructions::Transfer;

pub struct MakeAccounts<'a> {
    pub maker: &'a AccountView,
    pub escrow: &'a AccountView,
    pub mint_a: &'a AccountView,
    pub mint_b: &'a AccountView,
    pub maker_ata_a: &'a AccountView,
    pub vault: &'a AccountView,
    pub system_program: &'a AccountView,
    pub token_program: &'a AccountView,
}
impl<'a> TryFrom<&'a [AccountView]> for MakeAccounts<'a> {
    type Error = ProgramError;
    fn try_from(accounts: &'a [AccountView]) -> Result<Self, Self::Error> {
        let [
            maker,
            escrow,
            mint_a,
            mint_b,
            maker_ata_a,
            vault,
            system_program,
            token_program,
            _,
        ] = accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };
        if !maker.is_signer() {
            return Err(ProgramError::IllegalOwner);
        }

        MintAccount::check(mint_a)?;
        MintAccount::check(mint_b)?;
        AssociatedTokenAccount::check(maker_ata_a, maker, mint_a, token_program)?;

        let (vault_key, _) = solana_address::Address::find_program_address(
            &[
                escrow.address().as_ref(),
                pinocchio_token::ID.as_ref(),
                mint_a.address().as_ref(),
            ],
            &pinocchio_associated_token_account::ID,
        );
        if vault.address().ne(&vault_key) {
            return Err(ProgramError::InvalidAccountOwner);
        }
        if !vault.is_data_empty() {
            return Err(ProgramError::AccountAlreadyInitialized);
        }

        Ok(Self {
            maker,
            escrow,
            mint_a,
            mint_b,
            maker_ata_a,
            vault,
            system_program,
            token_program,
        })
    }
}

pub struct MakeInstructionData {
    pub seed: u64,
    pub receive: u64,
    pub amount: u64,
}
impl<'a> TryFrom<&'a [u8]> for MakeInstructionData {
    type Error = ProgramError;

    fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
        if data.len() != size_of::<u64>() * 3 {
            return Err(ProgramError::InvalidInstructionData);
        }
        let seed = u64::from_le_bytes(data[0..8].try_into().unwrap());
        let receive = u64::from_le_bytes(data[8..16].try_into().unwrap());
        let amount = u64::from_le_bytes(data[16..24].try_into().unwrap());
        if amount == 0 {
            return Err(ProgramError::InvalidInstructionData);
        }
        Ok(Self {
            seed,
            receive,
            amount,
        })
    }
}

pub struct Make<'a> {
    pub accounts: MakeAccounts<'a>,
    pub instruction_data: MakeInstructionData,
    pub bump: u8,
}
impl<'a> TryFrom<(&'a [u8], &'a [AccountView])> for Make<'a> {
    type Error = ProgramError;
    fn try_from((data, accounts): (&'a [u8], &'a [AccountView])) -> Result<Self, Self::Error> {
        let accounts = MakeAccounts::try_from(accounts)?;
        let instruction_data = MakeInstructionData::try_from(data)?;
        let (_, bump) = solana_address::Address::find_program_address(
            &[
                b"escrow",
                accounts.maker.address().as_ref(),
                &instruction_data.seed.to_le_bytes(),
            ],
            &crate::ID,
        );
        let seed_binding = instruction_data.seed.to_le_bytes();
        let bump_binding = [bump];
        let escrow_seeds = [
            Seed::from(b"escrow"),
            Seed::from(accounts.maker.address().as_ref()),
            Seed::from(&seed_binding),
            Seed::from(&bump_binding),
        ];
        let signers = [Signer::from(&escrow_seeds)];
        create_account_with_minimum_balance_signed(
            accounts.escrow,
            crate::state::Escrow::LEN,
            &crate::ID,
            accounts.maker,
            None,
            &signers,
        )?;
        AssociatedTokenAccount::init(
            accounts.vault,
            accounts.mint_a,
            accounts.maker,
            accounts.escrow,
            accounts.system_program,
            accounts.token_program,
        )?;
        Ok(Self {
            accounts,
            instruction_data,
            bump,
        })
    }
}

impl<'a> Make<'a> {
    pub const DISCRIMINATOR: &'a u8 = &0;
    pub fn process(&mut self) -> ProgramResult {
        let mut data = self.accounts.escrow.try_borrow_mut()?;
        let escrow = crate::state::Escrow::load_mut(data.as_mut())?;

        escrow.set_inner(
            self.instruction_data.seed,
            self.accounts.maker.address().clone(),
            self.accounts.mint_a.address().clone(),
            self.accounts.mint_b.address().clone(),
            self.instruction_data.receive,
            [self.bump],
        );
        Transfer {
            from: self.accounts.maker_ata_a,
            to: self.accounts.vault,
            authority: self.accounts.maker,
            amount: self.instruction_data.amount,
        }
        .invoke()?;
        Ok(())
    }
}

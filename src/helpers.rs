use pinocchio::{
    AccountView, Address, ProgramResult, cpi::Signer, error::ProgramError, sysvars::rent::Rent,
};
use pinocchio_system::instructions::CreateAccount;
use pinocchio_token::instructions::{InitializeAccount3, InitializeMint2};

pub trait AccountCheck {
    fn check(account: &AccountView) -> Result<(), ProgramError>;
}

pub struct SignerAccount;
impl AccountCheck for SignerAccount {
    fn check(account: &AccountView) -> Result<(), ProgramError> {
        if !account.is_signer() {
            return Err(ProgramError::MissingRequiredSignature);
        }
        Ok(())
    }
}

pub struct SystemAccount;
impl AccountCheck for SystemAccount {
    fn check(account: &AccountView) -> Result<(), ProgramError> {
        if !account.owned_by(&pinocchio_system::ID) {
            return Err(ProgramError::IllegalOwner);
        }
        Ok(())
    }
}

pub struct MintAccount;
impl AccountCheck for MintAccount {
    fn check(account: &AccountView) -> Result<(), ProgramError> {
        if !account.owned_by(&pinocchio_token::ID) {
            return Err(ProgramError::IllegalOwner);
        }
        if account.data_len() != pinocchio_token::state::Mint::LEN {
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(())
    }
}

pub trait MintInit {
    fn init(
        account: &AccountView,
        payer: &AccountView,
        rent: &AccountView,
        decimals: u8,
        mint_authority: &Address,
        freeze_authority: Option<&Address>,
    ) -> ProgramResult;
    fn init_if_needed(
        account: &AccountView,
        payer: &AccountView,
        rent: &AccountView,

        decimals: u8,
        mint_authority: &Address,
        freeze_authority: Option<&Address>,
    ) -> ProgramResult;
}

impl MintInit for MintAccount {
    fn init(
        account: &AccountView,
        payer: &AccountView,
        rent: &AccountView,
        decimals: u8,
        mint_authority: &Address,
        freeze_authority: Option<&Address>,
    ) -> ProgramResult {
        let lamports = Rent::from_account_view(rent)?
            .try_minimum_balance(pinocchio_token::state::Mint::LEN)?;
        CreateAccount {
            from: payer,
            to: account,
            lamports,
            space: pinocchio_token::state::Mint::LEN as u64,
            owner: &pinocchio_token::ID,
        }
        .invoke()?;
        InitializeMint2 {
            mint: account,
            decimals,
            mint_authority,
            freeze_authority,
        }
        .invoke()
    }
    fn init_if_needed(
        account: &AccountView,
        payer: &AccountView,
        rent: &AccountView,

        decimals: u8,
        mint_authority: &Address,
        freeze_authority: Option<&Address>,
    ) -> ProgramResult {
        match Self::check(account) {
            Ok(_) => Ok(()),
            Err(_) => Self::init(
                account,
                payer,
                rent,
                decimals,
                mint_authority,
                freeze_authority,
            ),
        }
    }
}

pub struct TokenAccount;
impl AccountCheck for TokenAccount {
    fn check(account: &AccountView) -> Result<(), ProgramError> {
        if !account.owned_by(&pinocchio_token::ID) {
            return Err(ProgramError::IllegalOwner);
        }
        if account
            .data_len()
            .ne(&pinocchio_token::state::TokenAccount::LEN)
        {
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(())
    }
}
pub trait AccountInit {
    fn init(
        account: &AccountView,
        mint: &AccountView,
        payer: &AccountView,
        rent: &AccountView,
        owner: &Address,
    ) -> ProgramResult;
    fn init_if_needed(
        account: &AccountView,
        mint: &AccountView,
        payer: &AccountView,
        rent: &AccountView,
        owner: &Address,
    ) -> ProgramResult;
}
impl AccountInit for TokenAccount {
    fn init(
        account: &AccountView,
        mint: &AccountView,
        payer: &AccountView,
        rent: &AccountView,
        owner: &Address,
    ) -> ProgramResult {
        let lamports = Rent::from_account_view(rent)?
            .try_minimum_balance(pinocchio_token::state::TokenAccount::LEN)?;
        CreateAccount {
            from: payer,
            to: account,
            lamports,
            space: pinocchio_token::state::TokenAccount::LEN as u64,
            owner: &pinocchio_token::ID,
        }
        .invoke()?;
        InitializeAccount3 {
            account,
            mint,
            owner,
        }
        .invoke()
    }
    fn init_if_needed(
        account: &AccountView,
        mint: &AccountView,
        payer: &AccountView,
        rent: &AccountView,
        owner: &Address,
    ) -> ProgramResult {
        match Self::check(account) {
            Ok(_) => Ok(()),
            Err(_) => Self::init(account, mint, payer, rent, owner),
        }
    }
}

pub const TOKEN_2022_PROGRAM_ID: [u8; 32] = [
    0x06, 0xdd, 0xf6, 0xe1, 0xee, 0x75, 0x8f, 0xde, 0x18, 0x42, 0x5d, 0xbc, 0xe4, 0x6c, 0xcd, 0xda,
    0xb6, 0x1a, 0xfc, 0x4d, 0x83, 0xb9, 0x0d, 0x27, 0xfe, 0xbd, 0xf9, 0x28, 0xd8, 0xa1, 0x8b, 0xfc,
];
const TOKEN_2022_ACCOUNT_DISCRIMINATOR_OFFSET: usize = 165;
pub const TOKEN2022_MINT_DISCRIMINATOR: u8 = 0x01;
pub const TOKEN_2022_TOKEN_ACCOUNT_DISCRIMINATOR: u8 = 0x02;
pub struct Mint2022Account;
impl AccountCheck for Mint2022Account {
    fn check(account: &AccountView) -> Result<(), ProgramError> {
        if !account.owned_by(&TOKEN_2022_PROGRAM_ID.into()) {
            return Err(ProgramError::IllegalOwner);
        }
        let data = account.try_borrow()?;
        if data.len().ne(&pinocchio_token::state::Mint::LEN) {
            if data.len().le(&TOKEN_2022_ACCOUNT_DISCRIMINATOR_OFFSET) {
                return Err(ProgramError::InvalidAccountData);
            }
            if data[TOKEN_2022_ACCOUNT_DISCRIMINATOR_OFFSET].ne(&TOKEN2022_MINT_DISCRIMINATOR) {
                return Err(ProgramError::InvalidAccountData);
            }
        }
        Ok(())
    }
}
impl MintInit for Mint2022Account {
    fn init(
        account: &AccountView,
        payer: &AccountView,
        rent: &AccountView,
        decimals: u8,
        mint_authority: &Address,
        freeze_authority: Option<&Address>,
    ) -> ProgramResult {
        let lamports = Rent::from_account_view(rent)?
            .try_minimum_balance(pinocchio_token::state::Mint::LEN)?;
        CreateAccount {
            from: payer,
            to: account,
            lamports,
            space: pinocchio_token::state::Mint::LEN as u64,
            owner: &TOKEN_2022_PROGRAM_ID.into(),
        }
        .invoke()?;
        InitializeMint2 {
            mint: account,
            decimals,
            mint_authority,
            freeze_authority,
        }
        .invoke()
    }
    fn init_if_needed(
        account: &AccountView,
        payer: &AccountView,
        rent: &AccountView,
        decimals: u8,
        mint_authority: &Address,
        freeze_authority: Option<&Address>,
    ) -> ProgramResult {
        match Self::check(account) {
            Ok(_) => Ok(()),
            Err(_) => Self::init(
                account,
                payer,
                rent,
                decimals,
                mint_authority,
                freeze_authority,
            ),
        }
    }
}
pub struct TokenAccount2022Account;
impl AccountCheck for TokenAccount2022Account {
    fn check(account: &AccountView) -> Result<(), ProgramError> {
        if !account.owned_by(&TOKEN_2022_PROGRAM_ID.into()) {
            return Err(ProgramError::IllegalOwner);
        }
        let data = account.try_borrow()?;
        if data.len().ne(&pinocchio_token::state::TokenAccount::LEN) {
            if data.len().le(&TOKEN_2022_ACCOUNT_DISCRIMINATOR_OFFSET) {
                return Err(ProgramError::InvalidAccountData);
            }
            if data[TOKEN_2022_ACCOUNT_DISCRIMINATOR_OFFSET]
                .ne(&TOKEN_2022_TOKEN_ACCOUNT_DISCRIMINATOR)
            {
                return Err(ProgramError::InvalidAccountData);
            }
        }
        Ok(())
    }
}
impl AccountInit for TokenAccount2022Account {
    fn init_if_needed(
        account: &AccountView,
        mint: &AccountView,
        payer: &AccountView,
        rent: &AccountView,
        owner: &Address,
    ) -> ProgramResult {
        match Self::check(account) {
            Ok(_) => Ok(()),
            Err(_) => Self::init(account, mint, payer, rent, owner),
        }
    }
    fn init(
        account: &AccountView,
        mint: &AccountView,
        payer: &AccountView,
        rent: &AccountView,
        owner: &Address,
    ) -> ProgramResult {
        let lamports = Rent::from_account_view(rent)?
            .try_minimum_balance(pinocchio_token::state::TokenAccount::LEN)?;
        CreateAccount {
            from: payer,
            to: account,
            lamports,
            space: pinocchio_token::state::TokenAccount::LEN as u64,
            owner: &TOKEN_2022_PROGRAM_ID.into(),
        }
        .invoke()?;
        InitializeAccount3 {
            account,
            mint,
            owner,
        }
        .invoke()
    }
}

pub struct MintInterface;
impl AccountCheck for MintInterface {
    fn check(account: &AccountView) -> Result<(), ProgramError> {
        let is_token_2022 = account.owned_by(&TOKEN_2022_PROGRAM_ID.into());
        let is_spl_token = account.owned_by(&pinocchio_token::ID);
        if !is_token_2022 && !is_spl_token {
            return Err(ProgramError::IllegalOwner);
        }

        let data = account.try_borrow()?;
        if is_spl_token {
            if data.len().ne(&pinocchio_token::state::Mint::LEN) {
                return Err(ProgramError::InvalidAccountData);
            }
        } else if is_token_2022 {
            if data.len().le(&TOKEN_2022_ACCOUNT_DISCRIMINATOR_OFFSET) {
                return Err(ProgramError::InvalidAccountData);
            }
            if data[TOKEN_2022_ACCOUNT_DISCRIMINATOR_OFFSET].ne(&TOKEN2022_MINT_DISCRIMINATOR) {
                return Err(ProgramError::InvalidAccountData);
            }
        }
        Ok(())
    }
}
pub struct TokenAccountInterface;
impl AccountCheck for TokenAccountInterface {
    fn check(account: &AccountView) -> Result<(), ProgramError> {
        let is_owned_by_token_2022 = account.owned_by(&TOKEN_2022_PROGRAM_ID.into());
        let is_owned_by_spl_token = account.owned_by(&pinocchio_token::ID);
        if !is_owned_by_spl_token && !is_owned_by_token_2022 {
            return Err(ProgramError::IllegalOwner);
        }
        let data = account.try_borrow()?;

        if is_owned_by_spl_token {
            if data.len().ne(&pinocchio_token::state::TokenAccount::LEN) {
                return Err(ProgramError::InvalidAccountData);
            }
        } else if is_owned_by_token_2022 {
            if data.len().le(&TOKEN_2022_ACCOUNT_DISCRIMINATOR_OFFSET) {
                return Err(ProgramError::InvalidAccountData);
            }
            if data[TOKEN_2022_ACCOUNT_DISCRIMINATOR_OFFSET]
                .ne(&TOKEN_2022_TOKEN_ACCOUNT_DISCRIMINATOR)
            {
                return Err(ProgramError::InvalidAccountData);
            }
        }
        Ok(())
    }
}

pub trait AssociatedTokenAccountCheck {
    fn check(
        account: &AccountView,
        authority: &AccountView,
        mint: &AccountView,
        token_program: &AccountView,
    ) -> Result<(), ProgramError>;
}
pub struct AssociatedTokenAccount;
impl AssociatedTokenAccountCheck for AssociatedTokenAccount {
    fn check(
        account: &AccountView,
        authority: &AccountView,
        mint: &AccountView,
        token_program: &AccountView,
    ) -> Result<(), ProgramError> {
        TokenAccount::check(account)?;
        if Address::find_program_address(
            &[
                authority.address().as_ref(),
                token_program.address().as_ref(),
                mint.address().as_ref(),
            ],
            &pinocchio_associated_token_account::ID,
        )
        .0
        .ne(account.address())
        {
            return Err(ProgramError::InvalidArgument);
        }
        Ok(())
    }
}
pub trait AssociatedTokenAccountInit {
    fn init(
        account: &AccountView,
        mint: &AccountView,
        payer: &AccountView,
        owner: &AccountView,
        system_program: &AccountView,
        token_program: &AccountView,
    ) -> ProgramResult;
    fn init_if_needed(
        account: &AccountView,
        mint: &AccountView,
        payer: &AccountView,
        owner: &AccountView,
        system_program: &AccountView,
        token_program: &AccountView,
    ) -> ProgramResult;
    fn init_signed(
        account: &AccountView,
        mint: &AccountView,
        payer: &AccountView,
        owner: &AccountView,
        system_program: &AccountView,
        token_program: &AccountView,
        signer: &[Signer],
    ) -> ProgramResult;
    fn init_if_needed_signed(
        account: &AccountView,
        mint: &AccountView,
        payer: &AccountView,
        owner: &AccountView,
        system_program: &AccountView,
        token_program: &AccountView,
        signer: &[Signer],
    ) -> ProgramResult;
}
impl AssociatedTokenAccountInit for AssociatedTokenAccount {
    fn init(
        account: &AccountView,
        mint: &AccountView,
        payer: &AccountView,
        owner: &AccountView,
        system_program: &AccountView,
        token_program: &AccountView,
    ) -> ProgramResult {
        pinocchio_associated_token_account::instructions::Create {
            funding_account: payer,
            account,
            wallet: owner,
            mint,
            system_program,
            token_program,
        }
        .invoke()
    }
    fn init_if_needed(
        account: &AccountView,
        mint: &AccountView,
        payer: &AccountView,
        owner: &AccountView,
        system_program: &AccountView,
        token_program: &AccountView,
    ) -> ProgramResult {
        match Self::check(account, payer, mint, token_program) {
            Ok(_) => Ok(()),
            Err(_) => Self::init(account, mint, payer, owner, system_program, token_program),
        }
    }
    fn init_signed(
        account: &AccountView,
        mint: &AccountView,
        payer: &AccountView,
        owner: &AccountView,
        system_program: &AccountView,
        token_program: &AccountView,
        signer: &[Signer],
    ) -> ProgramResult {
        pinocchio_associated_token_account::instructions::Create {
            funding_account: payer,
            account,
            wallet: owner,
            mint,
            system_program,
            token_program,
        }
        .invoke_signed(signer)
    }
    fn init_if_needed_signed(
        account: &AccountView,
        mint: &AccountView,
        payer: &AccountView,
        owner: &AccountView,
        system_program: &AccountView,
        token_program: &AccountView,
        signer: &[Signer],
    ) -> ProgramResult {
        match Self::check(account, payer, mint, token_program) {
            Ok(_) => Ok(()),
            Err(_) => Self::init_signed(
                account,
                mint,
                payer,
                owner,
                system_program,
                token_program,
                signer,
            ),
        }
    }
}

pub struct ProgramAccount;
impl AccountCheck for ProgramAccount {
    fn check(account: &AccountView) -> Result<(), ProgramError> {
        if !account.owned_by(&crate::ID) {
            return Err(ProgramError::InvalidAccountOwner);
        }
        if account.data_len().ne(&crate::state::Escrow::LEN) {
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(())
    }
}
pub trait ProgramAccountInit {
    fn init<'a, T: Sized>(
        payer: &AccountView,
        account: &AccountView,
        rent: &AccountView,
        signer: &[Signer],
        space: usize,
    ) -> ProgramResult;
}
impl ProgramAccountInit for ProgramAccount {
    fn init<'a, T: Sized>(
        payer: &AccountView,
        account: &AccountView,
        rent: &AccountView,
        signer: &[Signer],
        space: usize,
    ) -> ProgramResult {
        let lamports = Rent::from_account_view(rent)?.try_minimum_balance(space)?;
        CreateAccount {
            from: payer,
            to: account,
            lamports,
            space: space as u64,
            owner: &crate::ID,
        }
        .invoke_signed(signer)?;
        Ok(())
    }
}
pub trait AccountClose {
    fn close(account: &AccountView, destination: &AccountView) -> ProgramResult;
}
impl AccountClose for ProgramAccount {
    fn close(account: &AccountView, destination: &AccountView) -> ProgramResult {
        {
            let mut data = account.try_borrow_mut()?;
            data[0] = 0xff;
        }
        let dst_curr_lamports = destination.lamports();
        destination.set_lamports(dst_curr_lamports + account.lamports());
        account.resize(1)?;
        account.close()
    }
}

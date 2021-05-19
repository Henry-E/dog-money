use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, TokenAccount, Transfer, MintTo};
use anchor_lang::solana_program::program_option::COption;

#[program]
pub mod dog_money {
    use super::*;
    pub fn initialize_user(ctx: Context<InitializeUser>, amount: u64, nonce: u8) -> ProgramResult {
        let user_data = &mut ctx.accounts.user_data;
        user_data.first_deposit = ctx.accounts.clock.unix_timestamp;

        // Transfer USDC from user to vault
        let cpi_accounts = Transfer {
            from: ctx.accounts.user_usdc.to_account_info(),
            to: ctx.accounts.program_vault.to_account_info(),
            authority: ctx.accounts.authority.clone(),
        };
        let cpi_program = ctx.accounts.token_program.clone();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, amount)?;

        // Mint 1,0000x dog money to user account
        let dog_money_amount = amount.checked_mul(1000).unwrap();
        let seeds = &[ctx.accounts.usdc_mint.to_account_info().key.as_ref(), &[nonce], ];
        let signer = &[&seeds[..]];
        let cpi_accounts = MintTo {
            mint: ctx.accounts.dog_money_mint.to_account_info(),
            to: ctx.accounts.user_dog_money.to_account_info(),
            authority: ctx.accounts.program_signer.clone()
        };
        let cpi_program = ctx.accounts.token_program.clone();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        token::mint_to(cpi_ctx, dog_money_amount)?;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeUser<'info> {
    program_signer: AccountInfo<'info>,
    #[account(associated = authority, with = usdc_mint)]
    user_data: ProgramAccount<'info, UserData>,
    #[account(signer)]
    authority: AccountInfo<'info>,
    usdc_mint: CpiAccount<'info, Mint>,
    #[account(mut, "user_usdc.owner == *authority.key")]
    user_usdc: CpiAccount<'info, TokenAccount>,
    #[account(mut)]
    program_vault: CpiAccount<'info, TokenAccount>,
    #[account(mut,
    "dog_money_mint.mint_authority == COption::Some(*program_signer.key)")]
    dog_money_mint: CpiAccount<'info, Mint>,
    #[account(mut, "user_dog_money.owner == *authority.key")]
    user_dog_money: CpiAccount<'info, TokenAccount>,
    // We already know its address and that it's executable
    #[account(executable, "token_program.key == &token::ID")]
    token_program: AccountInfo<'info>,
    rent: Sysvar<'info, Rent>,
    system_program: AccountInfo<'info>,
    clock: Sysvar<'info, Clock>,
}


#[associated]
pub struct UserData {
    pub first_deposit: i64,
}
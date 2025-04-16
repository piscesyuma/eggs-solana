use anchor_lang::prelude::*;

use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_interface;
use mushi_stake_vault::ixs::StakeInput;
use mushi_stake_vault::program::MushiStakeVault;

use mushi_stake_vault::state::MainState as StakeVaultMainState;
use mushi_stake_vault::cpi::{accounts::Stake as VaultStakeCpi};
use mushi_stake_vault::state::VAULT_OWNER_SEED;

use crate::error::MushiProgramError;
use crate::state::{GlobalStats, MainState};

pub fn stake(ctx: Context<Stake>, amount: u64) -> Result<()> {
    let cpi_ctx = CpiContext::new(
        ctx.accounts.stake_vault_program.to_account_info(),
        VaultStakeCpi {
            user: ctx.accounts.user.to_account_info(),
            main_state: ctx.accounts.mushi_stake_vault.to_account_info(),
            user_mushi_token_ata: ctx.accounts.user_mushi_token_ata.to_account_info(),
            user_eclipse_token_ata: ctx.accounts.user_eclipse_token_ata.to_account_info(),
            user_stake_token_ata: ctx.accounts.user_stake_token_ata.to_account_info(),
            mushi_token_vault: ctx.accounts.mushi_token_vault.to_account_info(),
            mushi_token_mint: ctx.accounts.mushi_token_mint.to_account_info(),
            eclipse_token_vault: ctx.accounts.eclipse_token_vault.to_account_info(),
            eclipse_token_mint: ctx.accounts.eclipse_token_mint.to_account_info(),
            stake_token_mint: ctx.accounts.stake_token_mint.to_account_info(),
            token_vault_owner: ctx.accounts.token_vault_owner.to_account_info(),
            associated_token_program: ctx.accounts.associated_token_program.to_account_info(),
            token_program: ctx.accounts.token_program.to_account_info(),
            token2022_program: ctx.accounts.token2022_program.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
        },
    );

    let res = mushi_stake_vault::cpi::stake(cpi_ctx, StakeInput { amount })?;

    Ok(())
}

#[derive(Accounts)]
pub struct Stake<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut)]
    pub mushi_stake_vault: Account<'info, StakeVaultMainState>,
    #[account(    
        mut,
        seeds = [GlobalStats::PREFIX_SEED],
        bump,
    )]
    pub global_state: Box<Account<'info, GlobalStats>>,
    #[account(    
        mut,
        seeds = [MainState::PREFIX_SEED],
        bump,
    )]
    pub main_state: Box<Account<'info, MainState>>,
    #[account(
        mut,
        token::mint = mushi_token_mint,
        token::authority = user,
        token::token_program = token_program,
    )]
    
    pub user_mushi_token_ata: Box<InterfaceAccount<'info, token_interface::TokenAccount>>,
    #[account(
        mut,
        token::mint = eclipse_token_mint,
        token::authority = user,
        token::token_program = token2022_program,
    )]
    pub user_eclipse_token_ata: Box<InterfaceAccount<'info, token_interface::TokenAccount>>,
    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = stake_token_mint,
        associated_token::authority = user,
        associated_token::token_program = token_program,
        // mut,
        // token::mint = stake_token_mint,
        // token::authority = user,
        // token::token_program = token_program,
    )]
    pub user_stake_token_ata: Box<InterfaceAccount<'info, token_interface::TokenAccount>>,
    #[account(
        mut,
        associated_token::mint = mushi_token_mint,
        associated_token::authority = token_vault_owner,
        associated_token::token_program = token_program,
    )]
    pub mushi_token_vault: Box<InterfaceAccount<'info, token_interface::TokenAccount>>,
    #[account(
        mut,
        mint::token_program = token_program,
        address = global_state.base_token,
    )]
    pub mushi_token_mint: InterfaceAccount<'info, token_interface::Mint>,
    #[account(
        mut,
        token::mint = eclipse_token_mint,
        token::authority = token_vault_owner,
        token::token_program = token2022_program,
    )]
    pub eclipse_token_vault: Box<InterfaceAccount<'info, token_interface::TokenAccount>>,
    #[account(
        mut,
        mint::token_program = token2022_program,
        address = main_state.quote_token,
    )]
    pub eclipse_token_mint: InterfaceAccount<'info, token_interface::Mint>,
    #[account(
        mut,
        mint::token_program = token_program,
        address = main_state.stake_token,
    )]
    pub stake_token_mint: InterfaceAccount<'info, token_interface::Mint>,
    #[account(
        mut,
        // seeds = [VAULT_OWNER_SEED],
        // bump,
    )]
    pub token_vault_owner: SystemAccount<'info>,

    pub stake_vault_program: Program<'info, MushiStakeVault>,
    pub token_program: Interface<'info, token_interface::TokenInterface>,
    pub token2022_program: Interface<'info, token_interface::TokenInterface>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

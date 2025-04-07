use anchor_lang::{
    prelude::*,
    solana_program::program::{invoke, invoke_signed},
};
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Mint, TokenAccount},
};
use mpl_token_metadata::{
    instructions::{CreateMetadataAccountV3, CreateMetadataAccountV3InstructionArgs},
    types::{Creator, DataV2},
};

use crate::{
    constants::{LAMPORTS_PER_SOL, MIN, SECONDS_IN_A_DAY, VAULT_SEED},
    error::MushiProgramError,
    program::MushiProgram,
    utils::{burn_tokens, mint_to_tokens_by_main_state, transfer_sol},
    MainState, GlobalStats,
};

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct StartInput {
    pub sol_amount: u64,
    pub token_name: String,
    pub token_symbol: String,
    pub token_uri: String,
}

pub fn start(ctx: Context<AStart>, input: StartInput) -> Result<()> {
    let main_state = &mut ctx.accounts.main_state;
    let global_state = &mut ctx.accounts.global_state;
    let mint = ctx.accounts.token.to_account_info();
    let admin = ctx.accounts.admin.to_account_info();
    //checks
    let team_mint_amount = input.sol_amount * MIN;
    require!(team_mint_amount >= LAMPORTS_PER_SOL, MushiProgramError::InvalidInput);

    let token_vault = ctx.accounts.token_vault.to_account_info();
    let token_program = ctx.accounts.token_program.to_account_info();
    // mint tokens
    mint_to_tokens_by_main_state(
        mint.to_account_info(),
        main_state.to_account_info(),
        token_vault.to_account_info(),
        token_program.to_account_info(),
        team_mint_amount,
        *ctx.bumps.get("main_state").unwrap(),
    )?;

    // burn tokens
    burn_tokens(
        token_vault.to_account_info(),
        mint.to_account_info(),
        ctx.accounts.token_vault_owner.to_account_info(),
        token_program.to_account_info(),
        1 * LAMPORTS_PER_SOL,
        Some(&[&[VAULT_SEED, &[*ctx.bumps.get("token_vault_owner").unwrap()]]]),
    )?;
    global_state.token_supply = team_mint_amount;
    global_state.started = true;
    global_state.token = mint.key();
    global_state.total_borrowed = 0;
    global_state.total_collateral = 0;
    global_state.last_price = 0;
    
    msg!(&mint.key().to_string());
    // transfer sol to vault owner
    let system_program = ctx.accounts.system_program.to_account_info();
    transfer_sol(
        ctx.accounts.admin.to_account_info(),
        ctx.accounts.token_vault_owner.to_account_info(), 
        system_program.to_account_info(), 
        input.sol_amount, 
        None)?;
    // set token metadata
    let set_metadata_ix = CreateMetadataAccountV3 {
        metadata: ctx.accounts.token_metadata_account.key(),
        mint: mint.key(),
        mint_authority: main_state.key(),
        payer: ctx.accounts.admin.key(),
        rent: Some(ctx.accounts.sysvar_rent.key()),
        system_program: ctx.accounts.system_program.key(),
        update_authority: (main_state.key(), true),
    }
    .instruction(CreateMetadataAccountV3InstructionArgs {
        data: DataV2 {
            name: input.token_name,
            symbol: input.token_symbol,
            uri: input.token_uri,
            creators: Some(vec![Creator {
                address: main_state.key(),
                share: 100,
                verified: true,
            }]),
            seller_fee_basis_points: 100,
            collection: None,
            uses: None,
        },
        is_mutable: false,
        collection_details: None,
    });
    invoke_signed(
        &set_metadata_ix,
        &[
            main_state.to_account_info(),
            admin.clone(),
            mint.clone(),
            ctx.accounts.token_metadata_account.to_account_info(),
            ctx.accounts.mpl_program.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
            ctx.accounts.sysvar_rent.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
        ],
        &[&[
            MainState::PREFIX_SEED,
            &[*ctx.bumps.get("main_state").unwrap()],
        ]],
    )?;
    Ok(())
}

#[derive(Accounts)]
pub struct AStart<'info> {
    #[account(
        mut,
        address=main_state.admin @MushiProgramError::UnAuthorised
    )]
    pub admin: Signer<'info>,
    #[account(
        mut,
        seeds=[MainState::PREFIX_SEED],
        bump,
    )]
    pub main_state: Account<'info, MainState>,
    #[account(
        mut,
        seeds=[GlobalStats::PREFIX_SEED],
        bump,
    )]
    pub global_state: Account<'info, GlobalStats>,
    #[account(
        init,
        payer = admin,
        signer,
        mint::decimals = 6,
        mint::authority = main_state,
        mint::freeze_authority=main_state,
    )]
    pub token: Account<'info, Mint>,
    ///CHECK:
    #[account(
        mut,
        seeds = [b"metadata", mpl_program.key.as_ref(), token.key().as_ref()],
        seeds::program = mpl_program,
        bump,
    )]
    pub token_metadata_account: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [VAULT_SEED],
        bump,
    )]
    pub token_vault_owner: SystemAccount<'info>,
    #[account(
        init,
        payer = admin,
        associated_token::mint = token,
        associated_token::authority = token_vault_owner,
    )]
    pub token_vault: Box<Account<'info, TokenAccount>>,
    ///CHECK:
    pub sysvar_rent: AccountInfo<'info>,
    ///CHECK:
    pub mpl_program: AccountInfo<'info>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, token::Token>,
    pub system_program: Program<'info, System>,
}

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
    constants::{INITIAL_BURN_TOKEN_AMOUNT, LAMPORTS_PER_SOL, MIN, MIN_INITIALIZE_TOKEN_AMOUNT, SECONDS_IN_A_DAY, VAULT_SEED},
    error::MushiProgramError,
    program::MushiProgram,
    utils::{burn_tokens, mint_to_tokens_by_main_state, trasnfer_sol},
    MainState,
};

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct InitializeInput {
    pub sol_amount: u64,
    pub fee_receiver: Pubkey,
    pub token_name: String,
    pub token_symbol: String,
    pub token_uri: String,
}

pub fn initialize(ctx: Context<AInitialize>, input: InitializeInput) -> Result<()> {
    let admin = ctx.accounts.admin.to_account_info();
    let state = &mut ctx.accounts.main_state;
    let mint = ctx.accounts.token.to_account_info();
    //checks
    let token_amount = input.sol_amount * LAMPORTS_PER_SOL * MIN;
    if token_amount < LAMPORTS_PER_SOL {
        return Err(MushiProgramError::InvalidInput.into());
    }

    // set state
    state.admin = admin.key();
    state.fee_receiver = input.fee_receiver;
    state.token = mint.key();
    let current_time = Clock::get().unwrap().unix_timestamp;
    state.last_liquidation_date = current_time - (current_time % SECONDS_IN_A_DAY) + SECONDS_IN_A_DAY;
    state.sell_fee = 975;
    state.buy_fee = 975;
    state.buy_fee_leverage = 10;

    let vault_token = ctx.accounts.token_vault.to_account_info();
    let token_program = ctx.accounts.token_program.to_account_info();
    // mint tokens
    mint_to_tokens_by_main_state(
        mint.to_account_info(),
        state.to_account_info(),
        vault_token.to_account_info(),
        token_program.to_account_info(),
        token_amount,
        *ctx.bumps.get("main_state").unwrap(),
    )?;

    // burn tokens
    burn_tokens(
        vault_token.to_account_info(),
        mint.to_account_info(),
        ctx.accounts.token_vault_owner.to_account_info(),
        token_program.to_account_info(),
        1 * LAMPORTS_PER_SOL,
        Some(&[&[VAULT_SEED, &[*ctx.bumps.get("token_vault_owner").unwrap()]]]),
    )?;
    state.token_supply = token_amount;
    // transfer sol to vault owner
    let system_program = ctx.accounts.system_program.to_account_info();
    trasnfer_sol(
        ctx.accounts.admin.to_account_info(),
        ctx.accounts.token_vault_owner.to_account_info(), 
        system_program.to_account_info(), 
        input.sol_amount, 
        None)?;
    state.sol_balance_in_vault = input.sol_amount;
    // set token metadata
    let set_metadata_ix = CreateMetadataAccountV3 {
        metadata: ctx.accounts.token_metadata_account.key(),
        mint: mint.key(),
        mint_authority: state.key(),
        payer: admin.key(),
        rent: Some(ctx.accounts.sysvar_rent.key()),
        system_program: ctx.accounts.system_program.key(),
        update_authority: (state.key(), true),
    }
    .instruction(CreateMetadataAccountV3InstructionArgs {
        data: DataV2 {
            name: input.token_name,
            symbol: input.token_symbol,
            uri: input.token_uri,
            creators: Some(vec![Creator {
                address: state.key(),
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
            state.to_account_info(),
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
pub struct AInitialize<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        init,
        payer = admin,
        seeds = [MainState::PREFIX_SEED],
        bump,
        space =  8 + MainState::MAX_SIZE,
    )]
    pub main_state: Box<Account<'info, MainState>>,

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

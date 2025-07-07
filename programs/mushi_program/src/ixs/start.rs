use anchor_lang::{
    prelude::*,
    solana_program::program::{invoke, invoke_signed},
    solana_program::sysvar::rent::Rent,
};
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Mint, TokenAccount}, token_interface,
};
use mpl_token_metadata::{
    instructions::{CreateMetadataAccountV3, CreateMetadataAccountV3InstructionArgs},
    types::{Creator, DataV2},
};

use crate::{
    constants::{LAMPORTS_PER_ECLIPSE, MAX_SUPPLY, MIN_INITIALIZE_RATIO, SECONDS_IN_A_DAY, VAULT_SEED}, error::MushiProgramError, program::MushiProgram, utils::{burn_tokens, mint_to_tokens_by_main_state, transfer_tokens_checked}, GlobalStats, MainState
};

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct StartInput {
    pub es_amount: u64,
    pub token_name: String,
    pub token_symbol: String,
    pub token_uri: String,
}

pub fn start(ctx: Context<AStart>, input: StartInput) -> Result<()> {
    let main_state = &mut ctx.accounts.main_state;
    let global_state = &mut ctx.accounts.global_state;
    let mushi_mint = ctx.accounts.base_token.to_account_info();
    let admin = ctx.accounts.admin.to_account_info();
    //checks

    require!(!main_state.started, MushiProgramError::AlreadyStarted);
    let team_mint_amount = input.es_amount * MIN_INITIALIZE_RATIO;
    require!(team_mint_amount >= LAMPORTS_PER_ECLIPSE, MushiProgramError::InvalidInput);
    
    
    let token_vault = ctx.accounts.token_vault.to_account_info();
    let mushi_token_program = ctx.accounts.base_token_program.to_account_info();
    
    require!(global_state.token_supply + team_mint_amount <= MAX_SUPPLY, MushiProgramError::MaxSupplyExceeded);
    // mint tokens
    mint_to_tokens_by_main_state(
        mushi_mint.to_account_info(),
        main_state.to_account_info(),
        token_vault.to_account_info(),
        mushi_token_program.to_account_info(),
        team_mint_amount,
        *ctx.bumps.get("main_state").unwrap(),
    )?;

    // burn tokens
    burn_tokens(
        token_vault.to_account_info(),
        mushi_mint.to_account_info(),
        ctx.accounts.token_vault_owner.to_account_info(),
        mushi_token_program.to_account_info(),
        1 * LAMPORTS_PER_ECLIPSE,
        Some(&[&[VAULT_SEED, &[*ctx.bumps.get("token_vault_owner").unwrap()]]]),
    )?;

    main_state.started = true;

    global_state.token_supply = team_mint_amount;
    global_state.base_token = mushi_mint.key();
    global_state.total_borrowed = 0;
    global_state.total_collateral = 0;
    global_state.last_price = 0;
    
    msg!(&mushi_mint.key().to_string());

    let quote_mint = ctx.accounts.quote_mint.to_account_info();
    let quote_token_program = ctx.accounts.quote_token_program.to_account_info();
    let decimals = ctx.accounts.quote_mint.decimals;

    transfer_tokens_checked(
        ctx.accounts.admin_quote_ata.to_account_info(),
        ctx.accounts.quote_vault.to_account_info(),
        ctx.accounts.admin.to_account_info(),
        quote_mint.clone(),
        quote_token_program.clone(),
        input.es_amount, 
        decimals,
        None,
    )?;
    
    // set token metadata
    let set_metadata_ix = CreateMetadataAccountV3 {
        metadata: ctx.accounts.token_metadata_account.key(),
        mint: mushi_mint.key(),
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
            mushi_mint.clone(),
            ctx.accounts.token_metadata_account.to_account_info(),
            ctx.accounts.mpl_program.to_account_info(),
            ctx.accounts.base_token_program.to_account_info(),
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
        mut,
        address = main_state.quote_token,
    )]
    pub quote_mint: Box<InterfaceAccount<'info, token_interface::Mint>>,

    #[account(
        mut,
        token::mint = quote_mint,
        token::token_program = quote_token_program,
    )]
    pub admin_quote_ata: Box<InterfaceAccount<'info, token_interface::TokenAccount>>,

    #[account(
        init,
        payer = admin,
        signer,
        mint::decimals = 6,
        mint::authority = main_state,
        mint::freeze_authority=main_state,
        mint::token_program = base_token_program,
    )]
    pub base_token: Box<InterfaceAccount<'info, token_interface::Mint>>,

    ///CHECK:
    #[account(
        mut,
        seeds = [b"metadata", mpl_program.key.as_ref(), base_token.key().as_ref()],
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
        associated_token::mint = base_token,
        associated_token::authority = token_vault_owner,
        associated_token::token_program = base_token_program,
    )]
    pub token_vault: Box<InterfaceAccount<'info, token_interface::TokenAccount>>,

    #[account(
        init,
        payer = admin,
        associated_token::mint = quote_mint,
        associated_token::authority = token_vault_owner,
        associated_token::token_program = quote_token_program,
    )]
    pub quote_vault: Box<InterfaceAccount<'info, token_interface::TokenAccount>>,
    
    #[account(
        mut,
        address=main_state.fee_receiver,
    )]
    pub fee_receiver:SystemAccount<'info>,
    
    #[account(
        init,
        payer = admin,
        associated_token::mint = quote_mint,
        associated_token::authority = fee_receiver,
        associated_token::token_program = quote_token_program,

        // init_if_needed,
        // payer = admin,
        // token::mint = quote_mint,
        // token::authority = fee_receiver,
        // token::token_program = quote_token_program,
    )]
    pub fee_receiver_quote_ata: Box<InterfaceAccount<'info, token_interface::TokenAccount>>,

    ///CHECK:
    pub sysvar_rent: Sysvar<'info, Rent>,
    ///CHECK:
    pub mpl_program: AccountInfo<'info>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub base_token_program: Interface<'info, token_interface::TokenInterface>,
    pub quote_token_program: Interface<'info, token_interface::TokenInterface>,
    pub system_program: Program<'info, System>,
}

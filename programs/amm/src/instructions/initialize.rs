use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_interface::{Mint, TokenInterface,TokenAccount};

use crate::Config;

#[derive(Accounts)]
#[instruction(seed:u64)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub initializer:Signer<'info>,
    
    pub mint_a:InterfaceAccount<'info, Mint>,
    pub mint_b:InterfaceAccount<'info, Mint>,

    #[account(
        init,
        payer = initializer,
        mint::decimals = 6,
        mint::authority = config,
        seeds=[b"lp",config.key().as_ref()],
        bump
    )]
    pub mint_lp: InterfaceAccount<'info, Mint>,

    #[account(
        init,
        payer=initializer,
        associated_token::mint = mint_a,
        associated_token::authority = initializer,
        associated_token::token_program = token_program,
    )]
    pub vault_a:InterfaceAccount<'info, TokenAccount>,

    #[account(
        init,
        payer=initializer,
        associated_token::mint = mint_b,
        associated_token::authority = initializer,
        associated_token::token_program = token_program,
    )]
    pub vault_b:InterfaceAccount<'info, TokenAccount>,

    #[account(
        init,
        payer=initializer,
        space=8+Config::INIT_SPACE,
        seeds=[b"config",seed.to_le_bytes().as_ref()],
        bump
    )]
    pub config:Account<'info,Config>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> Initialize<'info>{
    fn initialize(&mut self,seed:u64,config_bump:u8,lp_bump:u8)->Result<()>{
        self.config.set_inner(
            Config {
                mint_a: self.mint_a.key(),
                mint_b:self.mint_b.key(),
                seed:seed,
                total_lp_issued:0,
                config_bump:config_bump,
                lp_bump:lp_bump,
                locked:false,
                authority:self.initializer.key()
            }
        );
        Ok(())
    }
}

pub fn handler(ctx: Context<Initialize>,seed:u64) -> Result<()> {
    ctx.accounts.initialize(seed, ctx.bumps.config, ctx.bumps.mint_lp)?;
    Ok(())
}

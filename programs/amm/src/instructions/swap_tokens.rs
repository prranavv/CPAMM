use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_interface::{self, Mint, TokenAccount, TokenInterface};
use anchor_spl::token_2022::{MintTo, TransferChecked};
use fixed::types::I64F64;

use crate::Config;
use crate::error::ErrorCode;
#[derive(Accounts)]
pub struct SwapTokens<'info>{
    pub user: Signer<'info>,
    pub mint_a:InterfaceAccount<'info, Mint>,
    pub mint_b:InterfaceAccount<'info, Mint>,
    #[account(
        has_one = mint_a,
        has_one = mint_b,
        seeds=[b"config",config.seed.to_le_bytes().as_ref()],
        bump = config.config_bump
    )]
    pub config:Account<'info,Config>,
    #[account(
        mint::decimals = 6,
        mint::authority = config,
        seeds=[b"lp",config.key().as_ref()],
        bump = config.lp_bump
    )]
    pub mint_lp: InterfaceAccount<'info, Mint>,

    #[account(
        associated_token::mint = mint_a,
        associated_token::authority = config.authority,
        associated_token::token_program = token_program,
    )]
    pub vault_a:InterfaceAccount<'info, TokenAccount>,

    #[account(
        associated_token::mint = mint_b,
        associated_token::authority = config.authority,
        associated_token::token_program = token_program,
    )]
    pub vault_b:InterfaceAccount<'info, TokenAccount>,

    #[account(
        associated_token::mint = mint_a,
        associated_token::authority = user,
        associated_token::token_program = token_program,
    )]
    pub mint_a_account: InterfaceAccount<'info, TokenAccount>,
    
    #[account(
        associated_token::mint = mint_b,
        associated_token::authority = user,
        associated_token::token_program = token_program,
    )]
    pub mint_b_account: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> SwapTokens<'info>{
    fn swap_tokens(&mut self,swap_a:bool,amount:u64)->Result<()>{
        if swap_a{
            require!(amount<=self.mint_a_account.amount,ErrorCode::AmountNotPresentInAccount)
            
        }else{
            require!(amount<=self.mint_b_account.amount,ErrorCode::AmountNotPresentInAccount);
        }
        let constant = I64F64::from_num(self.vault_a.amount)
                            .checked_mul(I64F64::from_num(I64F64::from_num(self.vault_b.amount)))
                            .unwrap();
        let output = if swap_a{

            let o = constant.checked_div(
                                                I64F64::from_num(self.vault_a.amount)
                                                    .checked_add(I64F64::from_num(amount)).unwrap()
                                                ).unwrap()
                                                .to_num::<u64>();
            o-self.vault_b.amount
        }else{
            let o = constant.checked_div(
                                                I64F64::from_num(self.vault_b.amount)
                                                    .checked_add(I64F64::from_num(amount)).unwrap()
                                                ).unwrap()
                                                .to_num::<u64>();
            o-self.vault_a.amount
        };

        if swap_a{
            let decimals = self.mint_a.decimals;
            let cpi_accounts = TransferChecked{
                mint: self.mint_a.to_account_info(),
                from:self.mint_a_account.to_account_info(),
                to:self.vault_a.to_account_info(),
                authority:self.user.to_account_info()
            };

            let cpi_program = self.token_program.to_account_info();
            let cpi_context = CpiContext::new(*cpi_program.key, cpi_accounts);
            token_interface::transfer_checked(cpi_context, amount, decimals).unwrap();

            let decimals = self.mint_b.decimals;
            let signer_seeds: &[&[&[u8]]] = &[&[b"config",&self.config.seed.to_le_bytes(),&[self.config.config_bump]]];
            let cpi_accounts = TransferChecked{
                mint:self.mint_b.to_account_info(),
                from:self.vault_b.to_account_info(),
                to:self.mint_b_account.to_account_info(),
                authority:self.config.to_account_info()
            };

            let cpi_program = self.token_program.to_account_info();
            let cpi_context = CpiContext::new(*cpi_program.key, cpi_accounts).with_signer(signer_seeds);
            token_interface::transfer_checked(cpi_context, output, decimals).unwrap();

        }else {
            let decimals = self.mint_b.decimals;
            let cpi_accounts = TransferChecked{
                mint: self.mint_b.to_account_info(),
                from:self.mint_b_account.to_account_info(),
                to:self.vault_b.to_account_info(),
                authority:self.user.to_account_info()
            };
            let signer_seeds: &[&[&[u8]]] = &[&[b"config",&self.config.seed.to_le_bytes(),&[self.config.config_bump]]];

            let cpi_program = self.token_program.to_account_info();
            let cpi_context = CpiContext::new(*cpi_program.key, cpi_accounts);
            token_interface::transfer_checked(cpi_context, amount, decimals).unwrap();

            let decimals = self.mint_a.decimals;
            let cpi_accounts = TransferChecked{
                mint:self.mint_a.to_account_info(),
                from:self.vault_a.to_account_info(),
                to:self.mint_a_account.to_account_info(),
                authority:self.config.to_account_info()
            };

            let cpi_program = self.token_program.to_account_info();
            let cpi_context = CpiContext::new(*cpi_program.key, cpi_accounts).with_signer(signer_seeds);
            token_interface::transfer_checked(cpi_context, output, decimals).unwrap();
        }
        Ok(())
    }
}

pub fn handler(ctx:Context<SwapTokens>,swap_a:bool,amount:u64)->Result<()>{
    ctx.accounts.swap_tokens(swap_a, amount)
}
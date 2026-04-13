use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_interface::{self, Mint, TokenAccount, TokenInterface};
use anchor_spl::token_2022::{Burn, MintTo, TransferChecked};
use fixed::types::I64F64;
use crate::{Config, MINIMUM_LIQUIDITY};
use crate::error::ErrorCode;
#[derive(Accounts)]
pub struct WithdrawLiquidity<'info>{
    pub initializer:Signer<'info>,
    
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

    #[account(
        mut,
        associated_token::mint = mint_lp,
        associated_token::authority = user,
        associated_token::token_program = token_program,
    )]
    pub mint_lp_account:InterfaceAccount<'info, TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl <'info> WithdrawLiquidity<'info>{
    fn withdraw_liquidity(&mut self,lp_token_number:u64)->Result<()>{
        require!(lp_token_number<=self.mint_lp_account.amount,ErrorCode::AmountNotPresentInAccount);

        let ratio = I64F64::from_num(lp_token_number).checked_div(I64F64::from_num(self.config.total_lp_issued+MINIMUM_LIQUIDITY)).unwrap();
        let mint_a_amt = I64F64::from_num(self.vault_a.amount).checked_mul(ratio).unwrap().to_num::<u64>();
        let mint_b_amt = I64F64::from_num(self.vault_b.amount).checked_mul(ratio).unwrap().to_num::<u64>();
        let bump = self.config.lp_bump;
        let key=self.config.key();
        let signer_seeds: &[&[&[u8]]] = &[&[b"lp",&[bump],key.as_ref()]];

        let decimals = self.mint_lp.decimals;
        let cpi_accounts = TransferChecked{
            mint: self.mint_lp.to_account_info(),
            from:self.vault_a.to_account_info(),
            to:self.mint_a_account.to_account_info(),
            authority:self.initializer.to_account_info()
        };

        let cpi_program = self.token_program.to_account_info();
        let cpi_context = CpiContext::new(*cpi_program.key, cpi_accounts).with_signer(signer_seeds);
        token_interface::transfer_checked(cpi_context, mint_a_amt, decimals).unwrap();

        let decimals = self.mint_lp.decimals;
        let cpi_accounts = TransferChecked{
            mint: self.mint_lp.to_account_info(),
            from:self.vault_b.to_account_info(),
            to:self.mint_b_account.to_account_info(),
            authority:self.initializer.to_account_info()
        };

        let cpi_program = self.token_program.to_account_info();
        let cpi_context = CpiContext::new(*cpi_program.key, cpi_accounts).with_signer(signer_seeds);
        token_interface::transfer_checked(cpi_context, mint_b_amt, decimals).unwrap();

        let cpi_accounts = Burn{
            mint:self.mint_lp.to_account_info(),
            from:self.mint_lp_account.to_account_info(),
            authority:self.initializer.to_account_info()
        };
        let cpi_context = CpiContext::new(*cpi_program.key, cpi_accounts);
        token_interface::burn(cpi_context, lp_token_number)?;

        self.config.total_lp_issued=self.config.total_lp_issued.checked_sub(lp_token_number).unwrap();
        Ok(())
    }
}

pub fn handler(ctx:Context<WithdrawLiquidity>,lp_amount:u64)->Result<()>{
    ctx.accounts.withdraw_liquidity(lp_amount)    
}
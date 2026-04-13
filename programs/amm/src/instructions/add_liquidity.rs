use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_interface::{self, Mint, TokenAccount, TokenInterface};
use anchor_spl::token_2022::{MintTo, TransferChecked};

use crate::{Config, MINIMUM_LIQUIDITY};
use crate::error::ErrorCode;

use fixed::types::I64F64;
#[derive(Accounts)]
pub struct AddLiquidity<'info>{
    #[account(mut)]
    pub user:Signer<'info>,
    
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
        init,
        payer=user,
        associated_token::mint = mint_lp,
        associated_token::authority = user,
        associated_token::token_program = token_program,
    )]
    pub mint_lp_account:InterfaceAccount<'info, TokenAccount>,
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

impl<'info> AddLiquidity<'info>{
    fn add_liquidity(&mut self,mint_a_amount:u64,mint_b_amount:u64)->Result<()>{
        require!(mint_a_amount<=self.mint_a_account.amount,ErrorCode::AmountNotPresentInAccount);
        require!(mint_b_amount<=self.mint_b_account.amount,ErrorCode::AmountNotPresentInAccount);
        let mut mint_a_amount = mint_a_amount;
        let mut mint_b_amount = mint_b_amount;
        let if_first_deposit = self.vault_a.amount==0 && self.vault_b.amount==0;
        (mint_a_amount,mint_b_amount) = if if_first_deposit{
            (mint_a_amount,mint_b_amount)
        }else{
            if self.vault_a.amount>self.vault_b.amount{
                (   
                    mint_a_amount,
                    I64F64::from_num(self.vault_a.amount)
                    .checked_div(I64F64::from_num(self.vault_b.amount)).unwrap()
                    .checked_mul(I64F64::from_num(mint_b_amount)).unwrap()
                    .to_num::<u64>(),
                )
            }else{
                (
                    mint_a_amount,
                    I64F64::from_num(self.vault_b.amount)
                    .checked_div(I64F64::from_num(self.vault_a.amount)).unwrap()
                    .checked_mul(I64F64::from_num(mint_a_amount)).unwrap()
                    .to_num::<u64>(),
                )
            }
        };

        let mut liquidity: u64; 
        
        if if_first_deposit{
            liquidity=I64F64::from_num(mint_a_amount)
                                    .checked_mul(I64F64::from_num(mint_b_amount)).unwrap()
                                    .sqrt()
                                    .to_num::<u64>();
            if liquidity<MINIMUM_LIQUIDITY{
                return err!(ErrorCode::DepositTooSmall)
            }
            liquidity-=MINIMUM_LIQUIDITY;
        }else{
            liquidity = std::cmp::min(
                I64F64::from_num(mint_a_amount)
                    .checked_mul(I64F64::from_num(self.config.total_lp_issued)).unwrap()
                    .checked_div(I64F64::from_num(self.vault_a.amount)).unwrap()
                    .to_num::<u64>(),
                I64F64::from_num(mint_b_amount)
                    .checked_mul(I64F64::from_num(self.config.total_lp_issued)).unwrap()
                    .checked_div(I64F64::from_num(self.vault_b.amount)).unwrap()
                    .to_num::<u64>(),
            );

            
        }
        
        let decimals = self.mint_a.decimals;
        let cpi_accounts = TransferChecked{
            mint:self.mint_a.to_account_info(),
            from:self.mint_a_account.to_account_info(),
            to:self.vault_a.to_account_info(),
            authority:self.user.to_account_info()
        };
        let cpi_program = self.token_program.to_account_info();
        let cpi_context = CpiContext::new(*cpi_program.key, cpi_accounts);
        token_interface::transfer_checked(cpi_context, mint_a_amount, decimals).unwrap();

        let decimals = self.mint_b.decimals;
        let cpi_accounts = TransferChecked{
            mint:self.mint_b.to_account_info(),
            from:self.mint_b_account.to_account_info(),
            to:self.vault_b.to_account_info(),
            authority:self.user.to_account_info()
        };
        let cpi_program = self.token_program.to_account_info();
        let cpi_context = CpiContext::new(*cpi_program.key, cpi_accounts);
        token_interface::transfer_checked(cpi_context, mint_a_amount, decimals).unwrap();

        //mint liquidity
        let bump = self.config.lp_bump;
        let key=self.config.key();
        let signer_seeds: &[&[&[u8]]] = &[&[b"lp",&[bump],key.as_ref()]];
        let cpi_accounts = MintTo{
            mint:self.mint_lp.to_account_info(),
            to:self.mint_lp_account.to_account_info(),
            authority:self.user.to_account_info()
        };

        let cpi_program_id = self.token_program.key();
        let cpi_context = CpiContext::new(cpi_program_id,cpi_accounts).with_signer(signer_seeds);
        token_interface::mint_to(cpi_context, liquidity).unwrap();

        self.config.total_lp_issued=self.config.total_lp_issued.checked_add(liquidity).unwrap();
        Ok(())
    }
}
//4000USD 20SOL
//20 SOL 4000USD
pub fn handler(ctx:Context<AddLiquidity>,mint_amount_a:u64,mint_amount_b:u64)->Result<()>{
    ctx.accounts.add_liquidity(mint_amount_a, mint_amount_b)?;
    Ok(())
}
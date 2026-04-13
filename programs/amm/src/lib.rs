pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("BJCsLEjLbSsNGUndJGBoNnk77LSRjC1gDiM693wPHAkb");

#[program]
pub mod amm {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>,seed:u64) -> Result<()> {
        initialize::handler(ctx,seed)
    }

    pub fn add_liquidity(ctx:Context<AddLiquidity>,mint_amount_a:u64,mint_amount_b:u64)->Result<()>{
        add_liquidity::handler(ctx, mint_amount_a, mint_amount_b)
    }

    pub fn swap_tokens(ctx:Context<SwapTokens>,swap_a:bool,amount:u64)->Result<()>{
        swap_tokens::handler(ctx, swap_a, amount)
    }

    pub fn withdraw_liquidity(ctx:Context<WithdrawLiquidity>,lp_amount:u64)->Result<()>{
        withdraw_liquidity::handler(ctx, lp_amount)
    }
}

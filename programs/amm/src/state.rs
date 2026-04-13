use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Config{
    pub mint_a:Pubkey,
    pub mint_b:Pubkey,
    pub locked:bool,
    pub seed:u64,
    pub total_lp_issued:u64,
    pub config_bump:u8,
    pub lp_bump:u8,
    pub authority:Pubkey
}
# CPAMM — Constant Product Automated Market Maker

A Solana-based Constant Product AMM built with the Anchor framework. This program enables permissionless token swaps and liquidity provision using the classic **x × y = k** invariant.

## Overview

This AMM allows users to create token pools, provide liquidity for any SPL token pair, and swap between them at prices determined algorithmically by the constant product formula. Liquidity providers receive LP tokens representing their share of the pool.

**Program ID:** `BJCsLEjLbSsNGUndJGBoNnk77LSRjC1gDiM693wPHAkb`

## How It Works

The constant product formula ensures that the product of the two token reserves always remains equal before and after a swap (excluding fees):

```
reserve_a × reserve_b = k
```

When a user swaps token A for token B, the amount of B they receive is calculated such that the product `k` is preserved. This creates a price curve where larger trades relative to the pool size result in more slippage.

## Instructions

### `initialize(seed: u64)`

Creates a new liquidity pool for a token pair. Sets up the config account, LP token mint, and vault token accounts for both tokens. The `seed` parameter allows multiple pools to exist for the same token pair.

### `add_liquidity(mint_amount_a: u64, mint_amount_b: u64)`

Deposits tokens into the pool and mints LP tokens to the provider.

- **First deposit:** LP tokens minted = `√(amount_a × amount_b) - MINIMUM_LIQUIDITY`. The minimum liquidity (1000 units) is permanently locked to prevent the pool from being fully drained.
- **Subsequent deposits:** The amounts are adjusted to maintain the current pool ratio. LP tokens are minted proportionally to the existing supply.

### `swap_tokens(swap_a: bool, amount: u64)`

Swaps one token for the other using the constant product formula.

- `swap_a = true` — swap token A for token B
- `swap_a = false` — swap token B for token A

The output amount is determined by: `output = k / (reserve_in + amount_in) - reserve_out`

### `withdraw_liquidity(lp_amount: u64)`

Burns LP tokens and returns a proportional share of both pool tokens to the user. The withdrawal ratio is: `lp_amount / total_lp_issued`.

## Account Structure

### Config

Stores pool state and parameters:

| Field            | Type     | Description                              |
|------------------|----------|------------------------------------------|
| `mint_a`         | `Pubkey` | Mint address of token A                  |
| `mint_b`         | `Pubkey` | Mint address of token B                  |
| `locked`         | `bool`   | Whether the pool is locked               |
| `seed`           | `u64`    | Unique seed for PDA derivation           |
| `total_lp_issued`| `u64`    | Total LP tokens minted                   |
| `config_bump`    | `u8`     | PDA bump for the config account          |
| `lp_bump`        | `u8`     | PDA bump for the LP mint                 |
| `authority`      | `Pubkey` | Pool authority (initializer)             |

## PDA Seeds

- **Config:** `["config", seed.to_le_bytes()]`
- **LP Mint:** `["lp", config.key()]`

## Tech Stack

- **Solana** — runtime
- **Anchor** — framework
- **SPL Token / Token-2022** — token standard support
- **fixed** (`I64F64`) — fixed-point arithmetic for precise ratio calculations

## Building

```bash
anchor build
```

## Deploying

```bash
anchor deploy
```

## Testing

```bash
anchor test
```

## License

MIT
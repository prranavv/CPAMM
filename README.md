<h1 align="center">CPAMM</h1>
<p align="center"><strong>Constant Product Automated Market Maker on Solana</strong></p>

<p align="center">
  <img src="https://img.shields.io/badge/Rust-1.75+-DEA584?style=flat-square&logo=rust" />
  <img src="https://img.shields.io/badge/Solana-3.0-9945FF?style=flat-square&logo=solana" />
  <img src="https://img.shields.io/badge/Anchor-1.0.0-blue?style=flat-square" />
  <img src="https://img.shields.io/badge/SPL--Token-✓-00D18C?style=flat-square" />
  <img src="https://img.shields.io/badge/Token--2022-✓-00D18C?style=flat-square" />
  <img src="https://img.shields.io/badge/License-MIT-green?style=flat-square" />
</p>

<p align="center">
  A permissionless AMM built with the Anchor framework that enables token swaps and liquidity provision for any SPL token pair using the classic <strong>x × y = k</strong> invariant. Liquidity providers receive LP tokens representing their proportional share of the pool.
</p>

---

## The Problem

Decentralized token trading on Solana requires infrastructure that is trustless, permissionless, and composable. Traditional order-book exchanges need active market makers and suffer from low liquidity for long-tail token pairs. Users need a way to swap between any two SPL tokens instantly without relying on centralized intermediaries or waiting for counterparties.

## The Solution

CPAMM implements the **constant product formula** — the same model pioneered by Uniswap — natively on Solana. Anyone can create a pool for any token pair, provide liquidity to earn LP tokens, and swap tokens at algorithmically determined prices. No order books, no intermediaries, no gatekeepers.

```
reserve_a × reserve_b = k
```

When a user swaps token A for token B, the output amount is calculated such that `k` is preserved. Larger trades relative to pool size result in more slippage — this is by design, as it protects liquidity providers from being drained.

**Program ID:** `BJCsLEjLbSsNGUndJGBoNnk77LSRjC1gDiM693wPHAkb`

---

## Features

- **Permissionless pool creation** — anyone can create a liquidity pool for any SPL token pair using a unique seed parameter, allowing multiple pools per pair
- **PDA-custodied vaults** — pool reserves are held in associated token accounts owned by the config PDA, ensuring only the program can move funds
- **Constant product swaps** — token prices determined algorithmically by the `x × y = k` invariant
- **LP token minting** — liquidity providers receive LP tokens proportional to their share of the pool, redeemable for underlying assets at any time
- **Minimum liquidity lock** — first deposit permanently locks 1000 LP token units to prevent the pool from being fully drained (à la Uniswap V2)
- **Proportional withdrawals** — burn LP tokens to receive your exact share of both pool reserves
- **Ratio-preserving deposits** — subsequent deposits are automatically adjusted to maintain the current pool ratio
- **Fixed-point arithmetic** — uses `I64F64` fixed-point math for precise ratio calculations, avoiding floating-point rounding errors
- **Token-2022 support** — compatible with both SPL Token and Token-2022 token standards via `token_interface`


## Instructions

| Instruction | Parameters | Description |
|---|---|---|
| **`initialize`** | `seed: u64` | Creates a new liquidity pool — sets up the config PDA, LP token mint (PDA), and vault ATAs owned by the config PDA. The seed allows multiple pools for the same pair. |
| **`add_liquidity`** | `mint_amount_a: u64, mint_amount_b: u64` | Deposits tokens into the PDA-owned vaults and mints LP tokens. First deposit: `√(a × b) - MINIMUM_LIQUIDITY`. Subsequent deposits are ratio-adjusted. |
| **`swap_tokens`** | `swap_a: bool, amount: u64` | Swaps one token for the other. `swap_a = true` swaps A→B, `false` swaps B→A. Output: `k / (reserve_in + amount_in) - reserve_out`. No swap fee is applied. The config PDA signs outbound transfers. |
| **`withdraw_liquidity`** | `lp_amount: u64` | Burns LP tokens and returns a proportional share of both pool tokens. Ratio: `lp_amount / (total_lp_issued + MINIMUM_LIQUIDITY)`. The config PDA signs outbound transfers. |

---

## Account Structure

### Config (PDA)

| Field | Type | Description |
|---|---|---|
| `mint_a` | `Pubkey` | Mint address of token A |
| `mint_b` | `Pubkey` | Mint address of token B |
| `locked` | `bool` | Whether the pool is locked |
| `seed` | `u64` | Unique seed for PDA derivation |
| `total_lp_issued` | `u64` | Total LP tokens minted (excluding minimum liquidity) |
| `config_bump` | `u8` | PDA bump for the config account |
| `lp_bump` | `u8` | PDA bump for the LP mint |
| `authority` | `Pubkey` | Pool authority (initializer) |

### PDA Seeds

| Account | Seeds |
|---|---|
| **Config** | `["config", seed.to_le_bytes()]` |
| **LP Mint** | `["lp", config.key()]` |

### Vaults

The token vaults are **associated token accounts owned by the config PDA**. Only the program can move funds in or out by signing with the config PDA's seeds (`["config", seed.to_le_bytes(), config_bump]`). This ensures trustless custody — no external wallet can drain the pool.

---

## How the Math Works

```
Constant Product Invariant:
  reserve_a × reserve_b = k

Swap Output (no fee):
  output = reserve_out - (k / (reserve_in + amount_in))

First LP Deposit:
  lp_tokens = √(amount_a × amount_b) - MINIMUM_LIQUIDITY (1000)

Subsequent LP Deposits:
  Amounts are ratio-adjusted first:
    if reserve_a > reserve_b:
      mint_b_amount = (reserve_a / reserve_b) × mint_b_amount
    else:
      mint_b_amount = (reserve_b / reserve_a) × mint_a_amount

  lp_tokens = min(
    amount_a × total_lp / reserve_a,
    amount_b × total_lp / reserve_b
  )

Withdrawal:
  amount_a_out = reserve_a × lp_amount / (total_lp + MINIMUM_LIQUIDITY)
  amount_b_out = reserve_b × lp_amount / (total_lp + MINIMUM_LIQUIDITY)
```

All arithmetic uses `I64F64` fixed-point types from the `fixed` crate (64-bit integer + 64-bit fractional precision).

---

## Quick Start

### Prerequisites

- Rust 1.75+
- Solana CLI 2.0+
- Anchor CLI 1.0+
- Node.js 18+ (for tests)

### 1. Clone and build

```bash
git clone https://github.com/prranavv/cpamm.git
cd cpamm

anchor build
```

### 2. Deploy

```bash
anchor deploy
```

### 3. Test

```bash
anchor test
```

---

## Tech Stack

| Component | Technology | Purpose |
|---|---|---|
| **Runtime** | Solana | High-throughput L1 blockchain |
| **Framework** | Anchor 1.0 | Solana program development framework with IDL generation |
| **Token Standard** | SPL Token / Token-2022 | Fungible token operations via `token_interface` |
| **Arithmetic** | `fixed` crate (`I64F64`) | Fixed-point math for precise ratio calculations |
| **Language** | Rust | On-chain program logic |
| **Testing** | LiteSVM + Solana SDK | Local validator simulation for integration tests |

---

## How the Math Works

```
Constant Product Invariant:
  reserve_a × reserve_b = k

Swap Output:
  output = reserve_out - (k / (reserve_in + amount_in))
         = reserve_out × amount_in / (reserve_in + amount_in)

First LP Deposit:
  lp_tokens = √(amount_a × amount_b) - MINIMUM_LIQUIDITY (1000)

Subsequent LP Deposits:
  lp_tokens = min(
    amount_a × total_lp / reserve_a,
    amount_b × total_lp / reserve_b
  )

Withdrawal:
  amount_a_out = reserve_a × lp_amount / total_lp
  amount_b_out = reserve_b × lp_amount / total_lp
```

---

## FAQ's

**"Why the constant product formula instead of a concentrated liquidity model?"**
> The constant product formula is the simplest and most battle-tested AMM design. It provides liquidity across the entire price range without requiring active position management from LPs. For a foundational AMM implementation, `x × y = k` offers the best tradeoff between simplicity, security, and composability.

**"What is the minimum liquidity lock and why does it exist?"**
> The first liquidity provider permanently locks 1000 LP token units (the `MINIMUM_LIQUIDITY` constant). This prevents an attacker from creating a pool, withdrawing all liquidity to zero, and then manipulating the price by depositing a tiny amount at an extreme ratio. It's the same approach Uniswap V2 uses.

**"Why fixed-point arithmetic instead of floating point?"**
> Solana's BPF runtime supports floating-point operations, but they introduce rounding errors that can be exploited in financial calculations. The `I64F64` type from the `fixed` crate provides 64 bits of integer precision and 64 bits of fractional precision — more than enough for token ratio calculations while being fully deterministic.

**"Can multiple pools exist for the same token pair?"**
> Yes. The `seed` parameter in `initialize` is part of the PDA derivation, so different seeds produce different config accounts. This allows competing pools for the same pair with potentially different liquidity depths.

**"Why use Token-2022 support?"**
> Token-2022 is Solana's next-generation token program with features like transfer fees, confidential transfers, and metadata extensions. Using `token_interface` instead of the legacy `token` module makes the AMM forward-compatible with both standards.

**"Why are there no swap fees?"**
> This is a learning implementation focused on the core constant product mechanics. In a production AMM, you'd deduct a fee (e.g., 0.3%) from the input amount before applying the invariant. The fee stays in the pool, growing `k` over time, which increases the value of each LP token — that's how LPs earn yield without any explicit fee distribution.

---

## Disclaimer

CPAMM is a learning and portfolio project demonstrating Solana program development with the Anchor framework. It has not been audited and is not intended for production use with real funds. See [Known Limitations](#known-limitations) for current issues. Always conduct a thorough security audit before deploying any DeFi protocol to mainnet.

---

## License

MIT — see [LICENSE](LICENSE) for details.

---

<p align="center">
  <sub>Built by <a href="https://github.com/prranavv">Pranav</a></sub>
</p>
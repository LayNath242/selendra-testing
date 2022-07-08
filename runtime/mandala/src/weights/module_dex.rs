// This file is part of Selendra.

// Copyright (C) 2021-2022 Selendra.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

//! Autogenerated weights for module_dex
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-03-16, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 1024

// Executed Command:
// target/production/selendra
// benchmark
// --chain=dev
// --steps=50
// --repeat=20
// --pallet=*
// --extrinsic=*
// --execution=wasm
// --wasm-execution=compiled
// --heap-pages=4096
// --template=./templates/runtime-weight-template.hbs
// --output=./runtime/mandala/src/weights/

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for module_dex.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> module_dex::WeightInfo for WeightInfo<T> {
	// Storage: Dex TradingPairStatuses (r:1 w:1)
	fn enable_trading_pair() -> Weight {
		(12_499_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: Dex TradingPairStatuses (r:1 w:1)
	fn disable_trading_pair() -> Weight {
		(12_488_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: Dex TradingPairStatuses (r:1 w:1)
	// Storage: Tokens TotalIssuance (r:1 w:0)
	// Storage: Dex ProvisioningPool (r:1 w:0)
	fn list_provisioning() -> Weight {
		(19_296_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(3 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: Dex TradingPairStatuses (r:1 w:1)
	fn update_provisioning_parameters() -> Weight {
		(6_492_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: Dex TradingPairStatuses (r:1 w:1)
	// Storage: Tokens Accounts (r:1 w:1)
	// Storage: Tokens TotalIssuance (r:1 w:1)
	// Storage: System Account (r:1 w:1)
	// Storage: Dex LiquidityPool (r:1 w:1)
	// Storage: Dex InitialShareExchangeRates (r:0 w:1)
	fn end_provisioning() -> Weight {
		(46_726_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(5 as Weight))
			.saturating_add(T::DbWeight::get().writes(6 as Weight))
	}
	// Storage: Dex TradingPairStatuses (r:1 w:1)
	// Storage: Dex ProvisioningPool (r:1 w:1)
	// Storage: System Account (r:1 w:1)
	// Storage: Tokens Accounts (r:2 w:2)
	fn add_provision() -> Weight {
		(60_123_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(5 as Weight))
			.saturating_add(T::DbWeight::get().writes(5 as Weight))
	}
	// Storage: Dex TradingPairStatuses (r:1 w:0)
	// Storage: Dex ProvisioningPool (r:2 w:1)
	// Storage: Dex InitialShareExchangeRates (r:1 w:1)
	// Storage: Tokens Accounts (r:2 w:2)
	// Storage: System Account (r:1 w:1)
	fn claim_dex_share() -> Weight {
		(56_055_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(7 as Weight))
			.saturating_add(T::DbWeight::get().writes(5 as Weight))
	}
	// Storage: Dex TradingPairStatuses (r:1 w:0)
	// Storage: Dex LiquidityPool (r:1 w:1)
	// Storage: Tokens TotalIssuance (r:1 w:1)
	// Storage: System Account (r:1 w:1)
	// Storage: Tokens Accounts (r:3 w:3)
	// Storage: EvmAccounts EvmAddresses (r:1 w:0)
	// Storage: EVM Accounts (r:1 w:1)
	// Storage: EvmAccounts Accounts (r:0 w:1)
	fn add_liquidity() -> Weight {
		(87_579_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(9 as Weight))
			.saturating_add(T::DbWeight::get().writes(8 as Weight))
	}
	// Storage: Dex TradingPairStatuses (r:1 w:0)
	// Storage: Dex LiquidityPool (r:1 w:1)
	// Storage: Tokens TotalIssuance (r:1 w:1)
	// Storage: System Account (r:1 w:1)
	// Storage: Tokens Accounts (r:4 w:4)
	// Storage: EvmAccounts EvmAddresses (r:1 w:0)
	// Storage: EVM Accounts (r:1 w:1)
	// Storage: Rewards PoolInfos (r:1 w:1)
	// Storage: Rewards SharesAndWithdrawnRewards (r:1 w:1)
	// Storage: EvmAccounts Accounts (r:0 w:1)
	fn add_liquidity_and_stake() -> Weight {
		(118_499_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(12 as Weight))
			.saturating_add(T::DbWeight::get().writes(11 as Weight))
	}
	// Storage: Dex LiquidityPool (r:1 w:1)
	// Storage: Tokens TotalIssuance (r:1 w:1)
	// Storage: Tokens Accounts (r:3 w:3)
	// Storage: System Account (r:1 w:1)
	fn remove_liquidity() -> Weight {
		(75_102_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(6 as Weight))
			.saturating_add(T::DbWeight::get().writes(6 as Weight))
	}
	// Storage: Dex LiquidityPool (r:1 w:1)
	// Storage: Tokens TotalIssuance (r:1 w:1)
	// Storage: Rewards SharesAndWithdrawnRewards (r:1 w:1)
	// Storage: Tokens Accounts (r:4 w:4)
	// Storage: System Account (r:2 w:1)
	// Storage: Rewards PoolInfos (r:1 w:1)
	// Storage: EvmAccounts EvmAddresses (r:1 w:0)
	// Storage: EVM Accounts (r:1 w:1)
	// Storage: EvmAccounts Accounts (r:0 w:1)
	fn remove_liquidity_by_unstake() -> Weight {
		(127_690_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(12 as Weight))
			.saturating_add(T::DbWeight::get().writes(11 as Weight))
	}
	// Storage: Dex TradingPairStatuses (r:1 w:0)
	// Storage: Dex LiquidityPool (r:1 w:1)
	// Storage: System Account (r:1 w:1)
	// Storage: Tokens Accounts (r:2 w:2)
	fn swap_with_exact_supply(u: u32, ) -> Weight {
		(43_500_000 as Weight)
			// Standard Error: 140_000
			.saturating_add((9_293_000 as Weight).saturating_mul(u as Weight))
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().reads((2 as Weight).saturating_mul(u as Weight)))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
			.saturating_add(T::DbWeight::get().writes((1 as Weight).saturating_mul(u as Weight)))
	}
	// Storage: Dex TradingPairStatuses (r:1 w:0)
	// Storage: Dex LiquidityPool (r:1 w:1)
	// Storage: System Account (r:1 w:1)
	// Storage: Tokens Accounts (r:2 w:2)
	fn swap_with_exact_target(u: u32, ) -> Weight {
		(42_673_000 as Weight)
			// Standard Error: 139_000
			.saturating_add((9_633_000 as Weight).saturating_mul(u as Weight))
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().reads((2 as Weight).saturating_mul(u as Weight)))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
			.saturating_add(T::DbWeight::get().writes((1 as Weight).saturating_mul(u as Weight)))
	}
	fn refund_provision() -> Weight {
		(105_716_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(7 as Weight))
			.saturating_add(T::DbWeight::get().writes(5 as Weight))
	}
	fn abort_provisioning() -> Weight {
		(78_617_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(5 as Weight))
			.saturating_add(T::DbWeight::get().writes(6 as Weight))
	}
}

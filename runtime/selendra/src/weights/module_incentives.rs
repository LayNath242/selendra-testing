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

//! Autogenerated weights for module_incentives
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
// --output=./runtime/selendra/src/weights/

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for module_incentives.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> module_incentives::WeightInfo for WeightInfo<T> {
	// Storage: EmergencyShutdown IsShutdown (r:1 w:0)
	// Storage: Rewards PoolInfos (r:1 w:0)
	// Storage: Incentives IncentiveRewardAmounts (r:2 w:0)
	// Storage: System Account (r:2 w:0)
	fn on_initialize(c: u32, ) -> Weight {
		(7_499_000 as Weight)
			// Standard Error: 183_000
			.saturating_add((14_908_000 as Weight).saturating_mul(c as Weight))
			.saturating_add(T::DbWeight::get().reads(3 as Weight))
			.saturating_add(T::DbWeight::get().reads((3 as Weight).saturating_mul(c as Weight)))
	}
	// Storage: Tokens Accounts (r:2 w:2)
	// Storage: EvmAccounts EvmAddresses (r:1 w:0)
	// Storage: EVM Accounts (r:1 w:1)
	// Storage: System Account (r:1 w:1)
	// Storage: Rewards PoolInfos (r:1 w:1)
	// Storage: Rewards SharesAndWithdrawnRewards (r:1 w:1)
	// Storage: EvmAccounts Accounts (r:0 w:1)
	fn deposit_dex_share() -> Weight {
		(54_104_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(7 as Weight))
			.saturating_add(T::DbWeight::get().writes(7 as Weight))
	}
	// Storage: Rewards SharesAndWithdrawnRewards (r:1 w:1)
	// Storage: Tokens Accounts (r:2 w:2)
	// Storage: System Account (r:1 w:0)
	// Storage: Rewards PoolInfos (r:1 w:1)
	fn withdraw_dex_share() -> Weight {
		(50_968_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(5 as Weight))
			.saturating_add(T::DbWeight::get().writes(4 as Weight))
	}
	// Storage: Rewards SharesAndWithdrawnRewards (r:1 w:1)
	// Storage: Rewards PoolInfos (r:1 w:1)
	// Storage: Incentives PendingMultiRewards (r:1 w:1)
	// Storage: Incentives ClaimRewardDeductionRates (r:1 w:0)
	// Storage: System Account (r:1 w:1)
	fn claim_rewards() -> Weight {
		(53_272_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(5 as Weight))
			.saturating_add(T::DbWeight::get().writes(4 as Weight))
	}
	// Storage: Incentives IncentiveRewardAmounts (r:1 w:1)
	fn update_incentive_rewards(c: u32, ) -> Weight {
		(3_452_000 as Weight)
			// Standard Error: 151_000
			.saturating_add((6_112_000 as Weight).saturating_mul(c as Weight))
			.saturating_add(T::DbWeight::get().reads((1 as Weight).saturating_mul(c as Weight)))
			.saturating_add(T::DbWeight::get().writes((1 as Weight).saturating_mul(c as Weight)))
	}
	// Storage: Incentives DexSavingRewardRates (r:1 w:1)
	fn update_dex_saving_rewards(c: u32, ) -> Weight {
		(1_721_000 as Weight)
			// Standard Error: 56_000
			.saturating_add((1_471_000 as Weight).saturating_mul(c as Weight))
			.saturating_add(T::DbWeight::get().reads((1 as Weight).saturating_mul(c as Weight)))
			.saturating_add(T::DbWeight::get().writes((1 as Weight).saturating_mul(c as Weight)))
	}
	// Storage: Incentives ClaimRewardDeductionRates (r:1 w:1)
	fn update_claim_reward_deduction_rates(c: u32, ) -> Weight {
		(2_574_000 as Weight)
			// Standard Error: 215_000
			.saturating_add((2_071_000 as Weight).saturating_mul(c as Weight))
			.saturating_add(T::DbWeight::get().reads((1 as Weight).saturating_mul(c as Weight)))
			.saturating_add(T::DbWeight::get().writes((1 as Weight).saturating_mul(c as Weight)))
	}
}
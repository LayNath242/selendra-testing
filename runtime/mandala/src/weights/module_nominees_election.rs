// This file is part of Acala.

// Copyright (C) 2020-2022 Acala Foundation.
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

//! Autogenerated weights for module_nominees_election
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-03-16, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 1024

// Executed Command:
// target/production/acala
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

/// Weight functions for module_nominees_election.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> module_nominees_election::WeightInfo for WeightInfo<T> {
	// Storage: NomineesElection Ledger (r:1 w:1)
	// Storage: Tokens Accounts (r:1 w:1)
	// Storage: NomineesElection Nominations (r:1 w:0)
	// Storage: Tokens Locks (r:1 w:1)
	fn bond() -> Weight {
		(22_433_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(4 as Weight))
			.saturating_add(T::DbWeight::get().writes(3 as Weight))
	}
	// Storage: NomineesElection Ledger (r:1 w:1)
	// Storage: NomineesElection CurrentEra (r:1 w:0)
	// Storage: NomineesElection Nominations (r:1 w:0)
	// Storage: Tokens Locks (r:1 w:1)
	// Storage: Tokens Accounts (r:1 w:1)
	fn unbond() -> Weight {
		(19_916_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(5 as Weight))
			.saturating_add(T::DbWeight::get().writes(3 as Weight))
	}
	// Storage: NomineesElection Ledger (r:1 w:1)
	// Storage: NomineesElection Nominations (r:1 w:0)
	// Storage: Tokens Locks (r:1 w:1)
	// Storage: Tokens Accounts (r:1 w:1)
	fn rebond(c: u32, ) -> Weight {
		(26_507_000 as Weight)
			// Standard Error: 19_000
			.saturating_add((114_000 as Weight).saturating_mul(c as Weight))
			.saturating_add(T::DbWeight::get().reads(4 as Weight))
			.saturating_add(T::DbWeight::get().writes(3 as Weight))
	}
	// Storage: NomineesElection Ledger (r:1 w:1)
	// Storage: NomineesElection CurrentEra (r:1 w:0)
	// Storage: Tokens Locks (r:1 w:1)
	// Storage: Tokens Accounts (r:1 w:1)
	fn withdraw_unbonded(_c: u32, ) -> Weight {
		(23_750_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(4 as Weight))
			.saturating_add(T::DbWeight::get().writes(3 as Weight))
	}
	// Storage: NomineesElection Ledger (r:1 w:0)
	// Storage: NomineesElection Nominations (r:1 w:1)
	// Storage: NomineesElection Votes (r:1 w:1)
	fn nominate(c: u32, ) -> Weight {
		(3_493_000 as Weight)
			// Standard Error: 269_000
			.saturating_add((5_074_000 as Weight).saturating_mul(c as Weight))
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().reads((1 as Weight).saturating_mul(c as Weight)))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
			.saturating_add(T::DbWeight::get().writes((1 as Weight).saturating_mul(c as Weight)))
	}
	// Storage: NomineesElection Nominations (r:1 w:1)
	// Storage: NomineesElection Ledger (r:1 w:0)
	// Storage: NomineesElection Votes (r:1 w:1)
	fn chill(c: u32, ) -> Weight {
		(9_980_000 as Weight)
			// Standard Error: 112_000
			.saturating_add((3_142_000 as Weight).saturating_mul(c as Weight))
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().reads((1 as Weight).saturating_mul(c as Weight)))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
			.saturating_add(T::DbWeight::get().writes((1 as Weight).saturating_mul(c as Weight)))
	}
}
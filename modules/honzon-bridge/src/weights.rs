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

//! Autogenerated weights for module_honzon_bridge
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2021-11-24, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("karura-dev"), DB CACHE: 128

// Executed Command:
// target/release/selendra
// benchmark
// --chain=karura-dev
// --steps=50
// --repeat=20
// --pallet=module-homa-lite
// --extrinsic=*
// --execution=wasm
// --wasm-execution=compiled
// --heap-pages=4096
// --template=./templates/module-weight-template.hbs
// --output=./modules/homa-lite/src/weights.rs


#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(clippy::unnecessary_cast)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weight functions needed for module_homa_lite.
pub trait WeightInfo {
	fn to_bridged() -> Weight;
	fn from_bridged() -> Weight;
}

/// Weights for module_homa_lite using the Selendra node and recommended hardware.
pub struct SelendraWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SelendraWeight<T> {
	fn to_bridged() -> Weight {
		(8_000_000 as Weight)
	}
	fn from_bridged() -> Weight {
		(8_000_000 as Weight)
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	fn to_bridged() -> Weight {
		(8_000_000 as Weight)
	}
	fn from_bridged() -> Weight {
		(8_000_000 as Weight)
	}
}

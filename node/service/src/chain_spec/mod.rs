// This file is part of Selendra.

// Copyright (C) 2020-2022 Selendra.
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

pub mod selendra;
pub use selendra::ChainSpec;

use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
use sp_consensus_babe::AuthorityId as BabeId;
use sp_finality_grandpa::AuthorityId as GrandpaId;

use sc_chain_spec::ChainSpecExtension;
use sp_core::{sr25519, Pair, Public, H160};
use sp_runtime::traits::IdentifyAccount;

use std::str::FromStr;
use coins_bip39::{English, Mnemonic, Wordlist};
use k256::{
	ecdsa::{SigningKey, VerifyingKey},
	EncodedPoint as K256PublicKey,
};
use tiny_keccak::{Hasher, Keccak};
use elliptic_curve::sec1::ToEncodedPoint;

use acala_primitives::{currency::TokenInfo, AccountId, AccountPublic, Balance};

const TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";
const DEFAULT_PROTOCOL_ID: &str = "sel";

/// Helper function to generate a crypto pair from seed
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

/// Helper function to generate an account ID from seed
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
	AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Helper function to generate stash, controller and session key from seed
pub fn authority_keys_from_seed(
	seed: &str,
) -> (AccountId, AccountId, GrandpaId, BabeId, ImOnlineId, AuthorityDiscoveryId) {
	(
		get_account_id_from_seed::<sr25519::Public>(&format!("{}//stash", seed)),
		get_account_id_from_seed::<sr25519::Public>(seed),
		get_from_seed::<GrandpaId>(seed),
		get_from_seed::<BabeId>(seed),
		get_from_seed::<ImOnlineId>(seed),
		get_from_seed::<AuthorityDiscoveryId>(seed),
	)
}

fn testnet_accounts() -> Vec<AccountId> {
	vec![
		get_account_id_from_seed::<sr25519::Public>("Alice"),
		get_account_id_from_seed::<sr25519::Public>("Bob"),
		get_account_id_from_seed::<sr25519::Public>("Charlie"),
		get_account_id_from_seed::<sr25519::Public>("Dave"),
		get_account_id_from_seed::<sr25519::Public>("Eve"),
		get_account_id_from_seed::<sr25519::Public>("Ferdie"),
		get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
		get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
		get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
		get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
		get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
		get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
	]
}

// Child key at derivation path: m/44'/60'/0'/0/{index}
const DEFAULT_DERIVATION_PATH_PREFIX: &str = "m/44'/60'/0'/0/";

fn generate_evm_address<W: Wordlist>(phrase: &str, index: u32) -> H160 {
	let derivation_path =
		coins_bip32::path::DerivationPath::from_str(&format!("{}{}", DEFAULT_DERIVATION_PATH_PREFIX, index))
			.expect("should parse the default derivation path");
	let mnemonic = Mnemonic::<W>::new_from_phrase(phrase).unwrap();

	let derived_priv_key = mnemonic.derive_key(&derivation_path, None).unwrap();
	let key: &SigningKey = derived_priv_key.as_ref();
	let secret_key: SigningKey = SigningKey::from_bytes(&key.to_bytes()).unwrap();
	let verify_key: VerifyingKey = secret_key.verifying_key();

	let point: &K256PublicKey = &verify_key.to_encoded_point(false);
	let public_key = point.to_bytes();

	let hash = keccak256(&public_key[1..]);
	H160::from_slice(&hash[12..])
}

fn keccak256<S>(bytes: S) -> [u8; 32]
where
	S: AsRef<[u8]>,
{
	let mut output = [0u8; 32];
	let mut hasher = Keccak::v256();
	hasher.update(bytes.as_ref());
	hasher.finalize(&mut output);
	output
}

fn get_evm_accounts(mnemonic: Option<&str>) -> Vec<H160> {
	let phrase = mnemonic.unwrap_or("fox sight canyon orphan hotel grow hedgehog build bless august weather swarm");
	let mut evm_accounts = Vec::new();
	for index in 0..10u32 {
		let addr = generate_evm_address::<English>(phrase, index);
		evm_accounts.push(addr);
		log::info!("index: {:?}, evm address: {:?}", index, addr);
	}
	evm_accounts
}

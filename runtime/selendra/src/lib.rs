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

//! The Dev runtime. This can be compiled with `#[no_std]`, ready for Wasm.

#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit.
#![recursion_limit = "512"]
#![allow(clippy::unnecessary_mut_passed)]
#![allow(clippy::or_fun_call)]
#![allow(clippy::from_over_into)]
#![allow(clippy::upper_case_acronyms)]

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

use codec::{Decode, DecodeLimit, Encode};
use frame_support::pallet_prelude::InvalidTransaction;
pub use frame_support::{
	construct_runtime, log, parameter_types,
	traits::{
		ConstBool, ConstU128, ConstU16, ConstU32, Contains, ContainsLengthBound,
		Currency as PalletCurrency, EnsureOrigin, EqualPrivilegeOnly, Everything, Get, Imbalance,
		InstanceFilter, IsSubType, IsType, KeyOwnerProofSystem, LockIdentifier, Nothing,
		OnRuntimeUpgrade, OnUnbalanced, Randomness, SortedMembers, U128CurrencyToVote,
		WithdrawReasons,
	},
	weights::{
		constants::{BlockExecutionWeight, RocksDbWeight, WEIGHT_PER_SECOND},
		DispatchClass, IdentityFee, Weight,
	},
	PalletId, RuntimeDebug, StorageValue,
};
use frame_system::{EnsureRoot, RawOrigin};
use module_asset_registry::AssetIdMaps;
use module_cdp_engine::CollateralCurrencyIds;
use module_currencies::BasicCurrencyAdapter;
use module_evm::{runner::RunnerExtended, CallInfo, CreateInfo};
use module_evm_accounts::EvmAddressMapping;
use module_support::{AssetIdMapping, DispatchableTask, PoolId};
use module_transaction_payment::TargetedFeeAdjustment;

use orml_traits::{
	create_median_value_data_provider, parameter_type_with_key, DataFeeder, DataProviderExtended,
	GetByKey,
};
use pallet_grandpa::{
	fg_primitives, AuthorityId as GrandpaId, AuthorityList as GrandpaAuthorityList,
};
use pallet_session::historical::{self as pallet_session_historical};
#[cfg(feature = "std")]
pub use pallet_staking::StakerStatus;
use pallet_transaction_payment::{FeeDetails, RuntimeDispatchInfo};
use primitives::{
	evm::{AccessListItem, EthereumTransactionMessage},
	unchecked_extrinsic::SelendraUncheckedExtrinsic,
};
use sp_api::impl_runtime_apis;
use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
use sp_core::{crypto::KeyTypeId, OpaqueMetadata, H160};
use sp_runtime::{
	create_runtime_str, generic, impl_opaque_keys,
	traits::{
		AccountIdConversion, BadOrigin, BlakeTwo256, Block as BlockT, Bounded, Convert, NumberFor,
		SaturatedConversion, StaticLookup, Verify,
	},
	transaction_validity::{TransactionSource, TransactionValidity},
	ApplyExtrinsicResult, DispatchResult, FixedPointNumber,
};
use sp_std::prelude::*;

#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;

pub use pallet_timestamp::Call as TimestampCall;
#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;
pub use sp_runtime::{Perbill, Percent, Permill, Perquintill};

pub use authority::AuthorityConfigImpl;
pub use constants::{fee::*, time::*};
pub use primitives::{
	currency::AssetIds,
	evm::{BlockLimits, EstimateResourcesRequest},
	AccountId, AccountIndex, Address, Amount, AuctionId, AuthoritysOriginId, Balance, BlockNumber,
	CurrencyId, DataProviderId, EraIndex, Hash, Lease, Moment, Multiplier, Nonce,
	ReserveIdentifier, Share, Signature, TokenSymbol, TradingPair,
};
pub use runtime_common::{
	cent, dollar, microcent, millicent, AllPrecompiles, BlockHashCount, CouncilInstance,
	CouncilMembershipInstance, EnsureRootOrAllCouncil, EnsureRootOrAllTechnicalCommittee,
	EnsureRootOrHalfCouncil, EnsureRootOrHalfFinancialCouncil, EnsureRootOrOneCouncil,
	EnsureRootOrOneThirdsTechnicalCommittee, EnsureRootOrThreeFourthsCouncil,
	EnsureRootOrTwoThirdsCouncil, EnsureRootOrTwoThirdsTechnicalCommittee, ExchangeRate,
	ExistentialDepositsTimesOneHundred, FinancialCouncilInstance,
	FinancialCouncilMembershipInstance, GasToWeight, MaxTipsOfPriority,
	OffchainSolutionWeightLimit, OperationalFeeMultiplier, OperatorMembershipInstanceSelendra,
	Price, ProxyType, Rate, Ratio, RuntimeBlockLength, RuntimeBlockWeights, SystemContractsFilter,
	TechnicalCommitteeInstance, TechnicalMembershipInstance, TimeStampedPrice, TipPerWeightStep,
	DAI, DOT, KSM, KUSD, LSEL, RENBTC, SEL,
};

mod authority;
pub mod constants;
mod voter_bags;
mod weights;

// runtime config
mod config;

use crate::config::{
	consensus_config::EpochDuration,
	dex_config::TradingPathLimit,
	evm_config::{StorageDepositPerByte, TxFeePerGas},
	funan_config::MaxSwapSlippageCompareToOracle,
};
#[cfg(test)]
use config::evm_config::NewContractExtraBytes;

/// This runtime version.
#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
	spec_name: create_runtime_str!("selendra"),
	impl_name: create_runtime_str!("selendra"),
	authoring_version: 1,
	spec_version: 101,
	impl_version: 0,
	apis: RUNTIME_API_VERSIONS,
	transaction_version: 1,
	state_version: 0,
};

/// The version infromation used to identify this runtime when compiled
/// natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
	NativeVersion { runtime_version: VERSION, can_author_with: Default::default() }
}

/// The BABE epoch configuration at genesis.
pub const BABE_GENESIS_EPOCH_CONFIG: sp_consensus_babe::BabeEpochConfiguration =
	sp_consensus_babe::BabeEpochConfiguration {
		c: PRIMARY_PROBABILITY,
		allowed_slots: sp_consensus_babe::AllowedSlots::PrimaryAndSecondaryPlainSlots,
	};

impl_opaque_keys! {
	pub struct SessionKeys {
		pub babe: Babe,
		pub grandpa: Grandpa,
		pub im_online: ImOnline,
		pub authority_discovery: AuthorityDiscovery,
	}
}

// Pallet accounts of runtime
parameter_types! {
	pub const TreasuryPalletId: PalletId = PalletId(*b"sel/trsy");
	pub const LoansPalletId: PalletId = PalletId(*b"sel/loan");
	pub const DEXPalletId: PalletId = PalletId(*b"sel/dexm");
	pub const CDPTreasuryPalletId: PalletId = PalletId(*b"sel/cdpt");
	pub const FunanTreasuryPalletId: PalletId = PalletId(*b"sel/hztr");
	pub const IncentivesPalletId: PalletId = PalletId(*b"sel/inct");
	pub const CollatorPotId: PalletId = PalletId(*b"sel/cpot");
	// Treasury reserve
	pub const TreasuryReservePalletId: PalletId = PalletId(*b"sel/reve");
	pub const PhragmenElectionPalletId: LockIdentifier = *b"sel/phre";
	pub const NftPalletId: PalletId = PalletId(*b"sel/aNFT");
	pub UnreleasedNativeVaultAccountId: AccountId = PalletId(*b"sel/urls").into_account_truncating();
	// This Pallet is only used to payment fee pool, it's not added to whitelist by design.
	// because transaction payment pallet will ensure the accounts always have enough ED.
	pub const TransactionPaymentPalletId: PalletId = PalletId(*b"sel/fees");
	pub const StableAssetPalletId: PalletId = PalletId(*b"nuts/sta");
}

pub fn get_all_module_accounts() -> Vec<AccountId> {
	vec![
		TreasuryPalletId::get().into_account_truncating(),
		LoansPalletId::get().into_account_truncating(),
		DEXPalletId::get().into_account_truncating(),
		CDPTreasuryPalletId::get().into_account_truncating(),
		FunanTreasuryPalletId::get().into_account_truncating(),
		IncentivesPalletId::get().into_account_truncating(),
		TreasuryReservePalletId::get().into_account_truncating(),
		CollatorPotId::get().into_account_truncating(),
		UnreleasedNativeVaultAccountId::get(),
		StableAssetPalletId::get().into_account_truncating(),
	]
}

parameter_types! {
	pub const Version: RuntimeVersion = VERSION;
	pub const SS58Prefix: u8 = 42; // Ss58AddressFormat::SubstrateAccount
}

pub struct BaseCallFilter;
impl Contains<Call> for BaseCallFilter {
	fn contains(call: &Call) -> bool {
		!module_transaction_pause::PausedTransactionFilter::<Runtime>::contains(call) &&
			!matches!(call, Call::Democracy(pallet_democracy::Call::propose { .. }),)
	}
}

impl frame_system::Config for Runtime {
	type AccountId = AccountId;
	type Call = Call;
	type Lookup = (Indices, EvmAccounts);
	type Index = Nonce;
	type BlockNumber = BlockNumber;
	type Hash = Hash;
	type Hashing = BlakeTwo256;
	type Header = generic::Header<BlockNumber, BlakeTwo256>;
	type Event = Event;
	type Origin = Origin;
	type BlockHashCount = BlockHashCount;
	type BlockWeights = RuntimeBlockWeights;
	type BlockLength = RuntimeBlockLength;
	type Version = Version;
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount =
		(module_evm::CallKillAccount<Runtime>, module_evm_accounts::CallKillAccount<Runtime>);
	type DbWeight = RocksDbWeight;
	type BaseCallFilter = BaseCallFilter;
	type SystemWeightInfo = ();
	type SS58Prefix = SS58Prefix;
	type OnSetCode = ();
	type MaxConsumers = ConstU32<16>;
}

parameter_types! {
	pub const MinimumPeriod: u64 = SLOT_DURATION / 2;
}

impl pallet_timestamp::Config for Runtime {
	/// A timestamp: milliseconds since the unix epoch.
	type Moment = Moment;
	type OnTimestampSet = ();
	// type OnTimestampSet = Babe;
	type MinimumPeriod = MinimumPeriod;
	type WeightInfo = ();
}

parameter_types! {
	pub const MaxReserves: u32 = ReserveIdentifier::Count as u32;
	pub NativeTokenExistentialDeposit: Balance = 10 * cent(SEL);
	// For weight estimation, we assume that the most locks on an individual account will be 50.
	// This number may need to be adjusted in the future if this assumption no longer holds true.
	pub const MaxLocks: u32 = 50;
}

impl pallet_balances::Config for Runtime {
	type Balance = Balance;
	type DustRemoval = Treasury;
	type Event = Event;
	type ExistentialDeposit = NativeTokenExistentialDeposit;
	type AccountStore = frame_system::Pallet<Runtime>;
	type MaxLocks = MaxLocks;
	type MaxReserves = MaxReserves;
	type ReserveIdentifier = ReserveIdentifier;
	type WeightInfo = ();
}

parameter_types! {
	pub TransactionByteFee: Balance = 10 * millicent(SEL);
	pub const TargetBlockFullness: Perquintill = Perquintill::from_percent(25);
	pub AdjustmentVariable: Multiplier = Multiplier::saturating_from_rational(1, 100_000);
	pub MinimumMultiplier: Multiplier = Multiplier::saturating_from_rational(1, 1_000_000_000u128);
}

impl pallet_sudo::Config for Runtime {
	type Event = Event;
	type Call = Call;
}

parameter_types! {
	pub const ProposalBond: Permill = Permill::from_percent(5);
	pub ProposalBondMinimum: Balance = dollar(SEL);
	pub ProposalBondMaximum: Balance = 5 * dollar(SEL);
	pub const SpendPeriod: BlockNumber = DAYS;
	pub const Burn: Permill = Permill::from_percent(0);
	pub const TipCountdown: BlockNumber = DAYS;
	pub const TipFindersFee: Percent = Percent::from_percent(10);
	pub TipReportDepositBase: Balance = dollar(SEL);
	pub const SevenDays: BlockNumber = 7 * DAYS;
	pub const ZeroDay: BlockNumber = 0;
	pub const OneDay: BlockNumber = DAYS;
	pub BountyDepositBase: Balance = dollar(SEL);
	pub const BountyDepositPayoutDelay: BlockNumber = DAYS;
	pub const BountyUpdatePeriod: BlockNumber = 14 * DAYS;
	pub const CuratorDepositMultiplier: Permill = Permill::from_percent(50);
	pub CuratorDepositMin: Balance = dollar(SEL);
	pub CuratorDepositMax: Balance = 100 * dollar(SEL);
	pub BountyValueMinimum: Balance = 5 * dollar(SEL);
	pub DataDepositPerByte: Balance = cent(SEL);
	pub const MaximumReasonLength: u32 = 16384;
	pub const MaxApprovals: u32 = 100;
}

impl pallet_treasury::Config for Runtime {
	type PalletId = TreasuryPalletId;
	type Currency = Balances;
	type ApproveOrigin = EnsureRootOrHalfCouncil;
	type RejectOrigin = EnsureRootOrHalfCouncil;
	type Event = Event;
	type OnSlash = ();
	type ProposalBond = ProposalBond;
	type ProposalBondMinimum = ProposalBondMinimum;
	type ProposalBondMaximum = ProposalBondMaximum;
	type SpendPeriod = SpendPeriod;
	type Burn = Burn;
	type BurnDestination = ();
	type SpendFunds = Bounties;
	type WeightInfo = ();
	type MaxApprovals = MaxApprovals;
}

impl pallet_bounties::Config for Runtime {
	type Event = Event;
	type BountyDepositBase = BountyDepositBase;
	type BountyDepositPayoutDelay = BountyDepositPayoutDelay;
	type BountyUpdatePeriod = BountyUpdatePeriod;
	type BountyValueMinimum = BountyValueMinimum;
	type CuratorDepositMultiplier = CuratorDepositMultiplier;
	type CuratorDepositMin = CuratorDepositMin;
	type CuratorDepositMax = CuratorDepositMax;
	type DataDepositPerByte = DataDepositPerByte;
	type MaximumReasonLength = MaximumReasonLength;
	type WeightInfo = ();
	type ChildBountyManager = ();
}

impl pallet_tips::Config for Runtime {
	type Event = Event;
	type DataDepositPerByte = DataDepositPerByte;
	type MaximumReasonLength = MaximumReasonLength;
	type Tippers = PhragmenElection;
	type TipCountdown = TipCountdown;
	type TipFindersFee = TipFindersFee;
	type TipReportDepositBase = TipReportDepositBase;
	type WeightInfo = ();
}

impl orml_auction::Config for Runtime {
	type Event = Event;
	type Balance = Balance;
	type AuctionId = AuctionId;
	type Handler = AuctionManager;
	type WeightInfo = weights::orml_auction::WeightInfo<Runtime>;
}

parameter_types! {
	pub const MinimumCount: u32 = 1;
	pub const ExpiresIn: Moment = 1000 * 60 * 60; // 1 hours
	pub RootOperatorAccountId: AccountId = AccountId::from([0xffu8; 32]);
}

type SelendraDataProvider = orml_oracle::Instance1;
impl orml_oracle::Config<SelendraDataProvider> for Runtime {
	type Event = Event;
	type OnNewData = ();
	type CombineData =
		orml_oracle::DefaultCombineData<Runtime, MinimumCount, ExpiresIn, SelendraDataProvider>;
	type Time = Timestamp;
	type OracleKey = CurrencyId;
	type OracleValue = Price;
	type RootOperatorAccountId = RootOperatorAccountId;
	type Members = OperatorMembershipSelendra;
	type MaxHasDispatchedSize = ConstU32<40>;
	type WeightInfo = weights::orml_oracle::WeightInfo<Runtime>;
}

create_median_value_data_provider!(
	AggregatedDataProvider,
	CurrencyId,
	Price,
	TimeStampedPrice,
	[SelendraOracle]
);
// Aggregated data provider cannot feed.
impl DataFeeder<CurrencyId, Price, AccountId> for AggregatedDataProvider {
	fn feed_value(_: AccountId, _: CurrencyId, _: Price) -> DispatchResult {
		Err("Not supported".into())
	}
}

pub struct DustRemovalWhitelist;
impl Contains<AccountId> for DustRemovalWhitelist {
	fn contains(a: &AccountId) -> bool {
		get_all_module_accounts().contains(a)
	}
}

parameter_type_with_key! {
	pub ExistentialDeposits: |currency_id: CurrencyId| -> Balance {
		match currency_id {
			CurrencyId::Token(symbol) => match symbol {
				TokenSymbol::SEL => cent(*currency_id),
				TokenSymbol::KUSD => cent(*currency_id),
				TokenSymbol::LSEL |
				TokenSymbol::DOT |
				TokenSymbol::KSM |
				TokenSymbol::DAI |
				TokenSymbol::RENBTC => Balance::max_value() // unsupported
			},
			CurrencyId::DexShare(dex_share_0, _) => {
				let currency_id_0: CurrencyId = (*dex_share_0).into();

				// initial dex share amount is calculated based on currency_id_0,
				// use the ED of currency_id_0 as the ED of lp token.
				if currency_id_0 == GetNativeCurrencyId::get() {
					NativeTokenExistentialDeposit::get()
				} else if let CurrencyId::Erc20(address) = currency_id_0 {
					// LP token with erc20
					AssetIdMaps::<Runtime>::get_asset_metadata(AssetIds::Erc20(address)).
						map_or(Balance::max_value(), |metatata| metatata.minimal_balance)
				} else {
					Self::get(&currency_id_0)
				}
			},
			CurrencyId::Erc20(_) => Balance::max_value(), // not handled by orml-tokens
			CurrencyId::StableAssetPoolToken(stable_asset_id) => {
				AssetIdMaps::<Runtime>::get_asset_metadata(AssetIds::StableAssetId(*stable_asset_id)).
					map_or(Balance::max_value(), |metatata| metatata.minimal_balance)
			},
			CurrencyId::ForeignAsset(foreign_asset_id) => {
				AssetIdMaps::<Runtime>::get_asset_metadata(AssetIds::ForeignAssetId(*foreign_asset_id)).
					map_or(Balance::max_value(), |metatata| metatata.minimal_balance)
			},
		}
	};
}

parameter_types! {
	pub TreasuryAccount: AccountId = TreasuryPalletId::get().into_account_truncating();
}

impl orml_tokens::Config for Runtime {
	type Event = Event;
	type Balance = Balance;
	type Amount = Amount;
	type CurrencyId = CurrencyId;
	type WeightInfo = weights::orml_tokens::WeightInfo<Runtime>;
	type ExistentialDeposits = ExistentialDeposits;
	type OnDust = orml_tokens::TransferDust<Runtime, TreasuryAccount>;
	type MaxLocks = MaxLocks;
	type MaxReserves = MaxReserves;
	type ReserveIdentifier = ReserveIdentifier;
	type DustRemovalWhitelist = DustRemovalWhitelist;
	type OnNewTokenAccount = ();
	type OnKilledTokenAccount = ();
}

parameter_types! {
	pub const GetNativeCurrencyId: CurrencyId = SEL;
	pub const GetStableCurrencyId: CurrencyId = KUSD;
	pub Erc20HoldingAccount: H160 = primitives::evm::ERC20_HOLDING_ACCOUNT;
}

impl module_currencies::Config for Runtime {
	type Event = Event;
	type MultiCurrency = Tokens;
	type NativeCurrency = BasicCurrencyAdapter<Runtime, Balances, Amount, BlockNumber>;
	type GetNativeCurrencyId = GetNativeCurrencyId;
	type Erc20HoldingAccount = Erc20HoldingAccount;
	type WeightInfo = weights::module_currencies::WeightInfo<Runtime>;
	type AddressMapping = EvmAddressMapping<Runtime>;
	type EVMBridge = module_evm_bridge::EVMBridge<Runtime>;
	type GasToWeight = GasToWeight;
	type SweepOrigin = EnsureRootOrOneCouncil;
	type OnDust = module_currencies::TransferDust<Runtime, TreasuryAccount>;
}

pub struct EnsureRootOrTreasury;
impl EnsureOrigin<Origin> for EnsureRootOrTreasury {
	type Success = AccountId;

	fn try_origin(o: Origin) -> Result<Self::Success, Origin> {
		Into::<Result<RawOrigin<AccountId>, Origin>>::into(o).and_then(|o| match o {
			RawOrigin::Root => Ok(TreasuryPalletId::get().into_account_truncating()),
			RawOrigin::Signed(caller) => {
				if caller == TreasuryPalletId::get().into_account_truncating() {
					Ok(caller)
				} else {
					Err(Origin::from(Some(caller)))
				}
			},
			r => Err(Origin::from(r)),
		})
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn successful_origin() -> Origin {
		let zero_account_id =
			AccountId::decode(&mut sp_runtime::traits::TrailingZeroInput::zeroes())
				.expect("infinite length input; no invalid inputs for type; qed");
		Origin::from(RawOrigin::Signed(zero_account_id))
	}
}

parameter_types! {
	pub MaximumSchedulerWeight: Weight = Perbill::from_percent(10) * RuntimeBlockWeights::get().max_block;
	// Retry a scheduled item every 25 blocks (5 minute) until the preimage exists.
	pub const NoPreimagePostponement: Option<u32> = Some(5 * MINUTES);
}

impl pallet_scheduler::Config for Runtime {
	type Event = Event;
	type Origin = Origin;
	type PalletsOrigin = OriginCaller;
	type Call = Call;
	type MaximumWeight = MaximumSchedulerWeight;
	type ScheduleOrigin = EnsureRoot<AccountId>;
	type MaxScheduledPerBlock = ConstU32<50>;
	type WeightInfo = ();
	type OriginPrivilegeCmp = EqualPrivilegeOnly;
	type PreimageProvider = Preimage;
	type NoPreimagePostponement = NoPreimagePostponement;
}

parameter_types! {
	pub PreimageBaseDeposit: Balance = deposit(2, 64);
	pub PreimageByteDeposit: Balance = deposit(0, 1);
}

impl pallet_preimage::Config for Runtime {
	type WeightInfo = ();
	type Event = Event;
	type Currency = Balances;
	type ManagerOrigin = EnsureRoot<AccountId>;
	// Max size 4MB allowed: 4096 * 1024
	type MaxSize = ConstU32<4_194_304>;
	type BaseDeposit = PreimageBaseDeposit;
	type ByteDeposit = PreimageByteDeposit;
}

impl<LocalCall> frame_system::offchain::CreateSignedTransaction<LocalCall> for Runtime
where
	Call: From<LocalCall>,
{
	fn create_transaction<C: frame_system::offchain::AppCrypto<Self::Public, Self::Signature>>(
		call: Call,
		public: <Signature as sp_runtime::traits::Verify>::Signer,
		account: AccountId,
		nonce: Nonce,
	) -> Option<(Call, <UncheckedExtrinsic as sp_runtime::traits::Extrinsic>::SignaturePayload)> {
		// take the biggest period possible.
		let period =
			BlockHashCount::get().checked_next_power_of_two().map(|c| c / 2).unwrap_or(2) as u64;
		let current_block = System::block_number()
			.saturated_into::<u64>()
			// The `System::block_number` is initialized with `n+1`,
			// so the actual block number is `n`.
			.saturating_sub(1);
		let tip = 0;
		let extra: SignedExtra = (
			frame_system::CheckNonZeroSender::<Runtime>::new(),
			frame_system::CheckSpecVersion::<Runtime>::new(),
			frame_system::CheckTxVersion::<Runtime>::new(),
			frame_system::CheckGenesis::<Runtime>::new(),
			frame_system::CheckMortality::<Runtime>::from(generic::Era::mortal(
				period,
				current_block,
			)),
			runtime_common::CheckNonce::<Runtime>::from(nonce),
			frame_system::CheckWeight::<Runtime>::new(),
			module_transaction_payment::ChargeTransactionPayment::<Runtime>::from(tip),
			module_evm::SetEvmOrigin::<Runtime>::new(),
		);
		let raw_payload = SignedPayload::new(call, extra)
			.map_err(|e| {
				log::warn!("Unable to create signed payload: {:?}", e);
			})
			.ok()?;
		let signature = raw_payload.using_encoded(|payload| C::sign(payload, public))?;
		let address = Indices::unlookup(account);
		let (call, extra, _) = raw_payload.deconstruct();
		Some((call, (address, signature, extra)))
	}
}

impl frame_system::offchain::SigningTypes for Runtime {
	type Public = <Signature as sp_runtime::traits::Verify>::Signer;
	type Signature = Signature;
}

impl<C> frame_system::offchain::SendTransactionTypes<C> for Runtime
where
	Call: From<C>,
{
	type OverarchingCall = Call;
	type Extrinsic = UncheckedExtrinsic;
}

impl module_transaction_pause::Config for Runtime {
	type Event = Event;
	type UpdateOrigin = EnsureRootOrThreeFourthsCouncil;
	type WeightInfo = weights::module_transaction_pause::WeightInfo<Runtime>;
}

parameter_types! {
	pub DefaultFeeTokens: Vec<CurrencyId> = vec![KUSD, DOT, LSEL];
	pub const CustomFeeSurplus: Percent = Percent::from_percent(50);
	pub const AlternativeFeeSurplus: Percent = Percent::from_percent(25);
}

impl module_transaction_payment::Config for Runtime {
	type Event = Event;
	type Call = Call;
	type NativeCurrencyId = GetNativeCurrencyId;
	type Currency = Balances;
	type MultiCurrency = Currencies;
	type OnTransactionPayment = ();
	type AlternativeFeeSwapDeposit = NativeTokenExistentialDeposit;
	type OperationalFeeMultiplier = OperationalFeeMultiplier;
	type TipPerWeightStep = TipPerWeightStep;
	type MaxTipsOfPriority = MaxTipsOfPriority;
	type WeightToFee = WeightToFee;
	type TransactionByteFee = TransactionByteFee;
	type FeeMultiplierUpdate =
		TargetedFeeAdjustment<Self, TargetBlockFullness, AdjustmentVariable, MinimumMultiplier>;
	type DEX = Dex;
	type MaxSwapSlippageCompareToOracle = MaxSwapSlippageCompareToOracle;
	type TradingPathLimit = TradingPathLimit;
	type PriceSource = module_prices::RealTimePriceProvider<Runtime>;
	type WeightInfo = weights::module_transaction_payment::WeightInfo<Runtime>;
	type PalletId = TransactionPaymentPalletId;
	type TreasuryAccount = TreasuryAccount;
	type UpdateOrigin = EnsureRootOrHalfCouncil;
	type CustomFeeSurplus = CustomFeeSurplus;
	type AlternativeFeeSurplus = AlternativeFeeSurplus;
	type DefaultFeeTokens = DefaultFeeTokens;
}

impl module_asset_registry::Config for Runtime {
	type Event = Event;
	type Currency = Balances;
	type EVMBridge = module_evm_bridge::EVMBridge<Runtime>;
	type RegisterOrigin = EnsureRootOrHalfCouncil;
	type WeightInfo = weights::module_asset_registry::WeightInfo<Runtime>;
}

impl orml_rewards::Config for Runtime {
	type Share = Balance;
	type Balance = Balance;
	type PoolId = PoolId;
	type CurrencyId = CurrencyId;
	type Handler = Incentives;
}

parameter_types! {
	pub const AccumulatePeriod: BlockNumber = MINUTES;
	pub const EarnShareBooster: Permill = Permill::from_percent(30);
}

impl module_incentives::Config for Runtime {
	type Event = Event;
	type RewardsSource = UnreleasedNativeVaultAccountId;
	type StableCurrencyId = GetStableCurrencyId;
	type NativeCurrencyId = GetNativeCurrencyId;
	type EarnShareBooster = EarnShareBooster;
	type AccumulatePeriod = AccumulatePeriod;
	type UpdateOrigin = EnsureRootOrThreeFourthsCouncil;
	type CDPTreasury = CdpTreasury;
	type Currency = Currencies;
	type DEX = Dex;
	type EmergencyShutdown = EmergencyShutdown;
	type PalletId = IncentivesPalletId;
	type WeightInfo = weights::module_incentives::WeightInfo<Runtime>;
}

parameter_types! {
	pub const GetLiquidCurrencyId: CurrencyId = LSEL;
	pub const GetStakingCurrencyId: CurrencyId = DOT;
}

parameter_types! {
	pub CreateClassDeposit: Balance = 20 * dollar(SEL);
	pub CreateTokenDeposit: Balance = 2 * dollar(SEL);
}

impl module_nft::Config for Runtime {
	type Event = Event;
	type Currency = Balances;
	type CreateClassDeposit = CreateClassDeposit;
	type CreateTokenDeposit = CreateTokenDeposit;
	type DataDepositPerByte = DataDepositPerByte;
	type PalletId = NftPalletId;
	type MaxAttributesBytes = ConstU32<2048>;
	type WeightInfo = weights::module_nft::WeightInfo<Runtime>;
}

impl orml_nft::Config for Runtime {
	type ClassId = u32;
	type TokenId = u64;
	type ClassData = module_nft::ClassData<Balance>;
	type TokenData = module_nft::TokenData<Balance>;
	type MaxClassMetadata = ConstU32<1024>;
	type MaxTokenMetadata = ConstU32<1024>;
}

pub struct EnsurePoolAssetId;
impl module_stable_asset::traits::ValidateAssetId<CurrencyId> for EnsurePoolAssetId {
	fn validate(currency_id: CurrencyId) -> bool {
		matches!(currency_id, CurrencyId::StableAssetPoolToken(_))
	}
}

pub struct ConvertBalanceSelendra;
impl orml_tokens::ConvertBalance<Balance, Balance> for ConvertBalanceSelendra {
	type AssetId = CurrencyId;

	fn convert_balance(balance: Balance, asset_id: CurrencyId) -> Balance {
		match asset_id {
			LSEL => ExchangeRate::saturating_from_rational(1, 10)
				.checked_mul_int(balance)
				.unwrap_or(Bounded::max_value()),
			_ => balance,
		}
	}

	fn convert_balance_back(balance: Balance, asset_id: CurrencyId) -> Balance {
		match asset_id {
			LSEL => ExchangeRate::saturating_from_rational(10, 1)
				.checked_mul_int(balance)
				.unwrap_or(Bounded::max_value()),
			_ => balance,
		}
	}
}

pub struct IsLiquidToken;
impl Contains<CurrencyId> for IsLiquidToken {
	fn contains(currency_id: &CurrencyId) -> bool {
		matches!(currency_id, CurrencyId::Token(TokenSymbol::LSEL))
	}
}

type RebaseTokens = orml_tokens::Combiner<
	AccountId,
	IsLiquidToken,
	orml_tokens::Mapper<
		AccountId,
		Currencies,
		ConvertBalanceSelendra,
		Balance,
		GetLiquidCurrencyId,
	>,
	Currencies,
>;

pub type RebasedStableAsset = module_support::RebasedStableAsset<
	StableAsset,
	ConvertBalanceSelendra,
	module_aggregated_dex::RebasedStableAssetErrorConvertor<Runtime>,
>;

impl module_stable_asset::Config for Runtime {
	type Event = Event;
	type AssetId = CurrencyId;
	type Balance = Balance;
	type Assets = RebaseTokens;
	type PalletId = StableAssetPalletId;

	type AtLeast64BitUnsigned = u128;
	type FeePrecision = ConstU128<10_000_000_000>; // 10 decimals
	type APrecision = ConstU128<100>; // 2 decimals
	type PoolAssetLimit = ConstU32<5>;
	type SwapExactOverAmount = ConstU128<100>;
	type WeightInfo = weights::module_stable_asset::WeightInfo<Runtime>;
	type ListingOrigin = EnsureRootOrHalfCouncil;
	type EnsurePoolAssetId = EnsurePoolAssetId;
}

#[derive(Clone, Encode, Decode, PartialEq, Eq, RuntimeDebug)]
pub struct ConvertEthereumTx;

impl
	Convert<
		(Call, SignedExtra),
		Result<(EthereumTransactionMessage, SignedExtra), InvalidTransaction>,
	> for ConvertEthereumTx
{
	fn convert(
		(call, mut extra): (Call, SignedExtra),
	) -> Result<(EthereumTransactionMessage, SignedExtra), InvalidTransaction> {
		match call {
			Call::EVM(module_evm::Call::eth_call {
				action,
				input,
				value,
				gas_limit,
				storage_limit,
				access_list,
				valid_until,
			}) => {
				if System::block_number() > valid_until {
					return Err(InvalidTransaction::Stale)
				}

				let (_, _, _, _, mortality, check_nonce, _, charge, ..) = extra.clone();

				if mortality != frame_system::CheckEra::from(sp_runtime::generic::Era::Immortal) {
					// require immortal
					return Err(InvalidTransaction::BadProof)
				}

				let nonce = check_nonce.nonce;
				let tip = charge.0;

				extra.5.mark_as_ethereum_tx(valid_until);

				Ok((
					EthereumTransactionMessage {
						chain_id: EVM::chain_id(),
						genesis: System::block_hash(0),
						nonce,
						tip,
						gas_limit,
						storage_limit,
						action,
						value,
						input,
						valid_until,
						access_list,
					},
					extra,
				))
			},
			_ => Err(InvalidTransaction::BadProof),
		}
	}
}

#[derive(Clone, Encode, Decode, PartialEq, Eq, RuntimeDebug)]
pub struct PayerSignatureVerification;

impl Convert<(Call, SignedExtra), Result<(), InvalidTransaction>> for PayerSignatureVerification {
	fn convert((call, extra): (Call, SignedExtra)) -> Result<(), InvalidTransaction> {
		if let Call::TransactionPayment(module_transaction_payment::Call::with_fee_paid_by {
			call,
			payer_addr,
			payer_sig,
		}) = call
		{
			let payer_account: [u8; 32] = payer_addr
				.encode()
				.as_slice()
				.try_into()
				.map_err(|_| InvalidTransaction::BadSigner)?;
			// payer signature is aim at inner call of `with_fee_paid_by` call.
			let raw_payload =
				SignedPayload::new(*call, extra).map_err(|_| InvalidTransaction::BadSigner)?;
			if !raw_payload
				.using_encoded(|payload| payer_sig.verify(payload, &payer_account.into()))
			{
				return Err(InvalidTransaction::BadProof)
			}
		}
		Ok(())
	}
}

/// Block header type as expected by this runtime.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;
/// A Block signed with a Justification
pub type SignedBlock = generic::SignedBlock<Block>;
/// BlockId type as expected by this runtime.
pub type BlockId = generic::BlockId<Block>;
/// The SignedExtension to the basic transaction logic.
pub type SignedExtra = (
	frame_system::CheckNonZeroSender<Runtime>,
	frame_system::CheckSpecVersion<Runtime>,
	frame_system::CheckTxVersion<Runtime>,
	frame_system::CheckGenesis<Runtime>,
	frame_system::CheckEra<Runtime>,
	runtime_common::CheckNonce<Runtime>,
	frame_system::CheckWeight<Runtime>,
	module_transaction_payment::ChargeTransactionPayment<Runtime>,
	module_evm::SetEvmOrigin<Runtime>,
);
/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic = SelendraUncheckedExtrinsic<
	Call,
	SignedExtra,
	ConvertEthereumTx,
	StorageDepositPerByte,
	TxFeePerGas,
	PayerSignatureVerification,
>;
/// The payload being signed in transactions.
pub type SignedPayload = generic::SignedPayload<Call, SignedExtra>;
/// Extrinsic type that has already been checked.
pub type CheckedExtrinsic = generic::CheckedExtrinsic<AccountId, Call, SignedExtra>;
/// Executive: handles dispatch to the various modules.
pub type Executive = frame_executive::Executive<
	Runtime,
	Block,
	frame_system::ChainContext<Runtime>,
	Runtime,
	AllPalletsWithSystem,
	(),
>;

construct_runtime!(
	pub enum Runtime where
		Block = Block,
		NodeBlock = primitives::Block,
		UncheckedExtrinsic = UncheckedExtrinsic
	{
		// Core
		System: frame_system = 0,
		Timestamp: pallet_timestamp = 1,
		Scheduler: pallet_scheduler = 2,
		TransactionPause: module_transaction_pause = 3,
		Preimage: pallet_preimage = 4,

		// Tokens & Related
		Balances: pallet_balances = 10,
		Tokens: orml_tokens exclude_parts { Call } = 11,
		Currencies: module_currencies = 12,
		TransactionPayment: module_transaction_payment = 14,

		// Treasury
		Treasury: pallet_treasury = 20,
		Bounties: pallet_bounties = 21,
		Tips: pallet_tips = 22,

		// Utility
		Utility: pallet_utility = 30,
		Multisig: pallet_multisig = 31,
		Recovery: pallet_recovery = 32,
		Proxy: pallet_proxy = 33,
		IdleScheduler: module_idle_scheduler = 34,
		Indices: pallet_indices = 39,

		// Consensus
		// Authorship must be before session in order to note author in the correct session and era
		// for im-online and staking.
		Authorship: pallet_authorship = 40,
		Babe: pallet_babe = 41,
		Staking: pallet_staking = 42,
		Offences: pallet_offences = 43,
		Historical: pallet_session_historical::{Pallet} = 44,
		Session: pallet_session = 45,
		Grandpa: pallet_grandpa = 46,
		ImOnline: pallet_im_online = 47,
		AuthorityDiscovery: pallet_authority_discovery = 48,
		// placed behind indices to maintain it.
		ElectionProviderMultiPhase: pallet_election_provider_multi_phase = 49,
		VoterList: pallet_bags_list = 50,
		NominationPools: pallet_nomination_pools = 51,

		// Governance
		Authority: orml_authority = 60,
		Council: pallet_collective::<Instance1> = 61,
		CouncilMembership: pallet_membership::<Instance1> = 62,
		FinancialCouncil: pallet_collective::<Instance2> = 63,
		FinancialCouncilMembership: pallet_membership::<Instance2> = 64,
		TechnicalCommittee: pallet_collective::<Instance4> = 65,
		TechnicalMembership: pallet_membership::<Instance4> = 66,
		PhragmenElection: pallet_elections_phragmen = 67,
		Democracy: pallet_democracy = 68,

		// Oracle
		//
		// NOTE: OperatorMembership must be placed after Oracle or else will have race condition on initialization
		SelendraOracle: orml_oracle::<Instance1> = 80,
		OperatorMembershipSelendra: pallet_membership::<Instance5> = 82,

		// Selendra Core
		Auction: orml_auction = 100,
		Rewards: orml_rewards = 101,
		OrmlNFT: orml_nft exclude_parts { Call } = 102,

		// Dex
		Prices: module_prices = 110,
		Dex: module_dex = 111,
		DexOracle: module_dex_oracle = 112,
		AggregatedDex: module_aggregated_dex = 113,

		// Funan
		AuctionManager: module_auction_manager = 120,
		Loans: module_loans = 121,
		Funan: module_funan = 122,
		CdpTreasury: module_cdp_treasury = 123,
		CdpEngine: module_cdp_engine = 124,
		EmergencyShutdown: module_emergency_shutdown = 125,

		// Selendra Other
		Incentives: module_incentives = 140,
		NFT: module_nft = 141,
		AssetRegistry: module_asset_registry = 142,

		// Smart contracts
		EVM: module_evm = 180,
		EVMBridge: module_evm_bridge exclude_parts { Call } = 181,
		EvmAccounts: module_evm_accounts = 182,

		// Stable asset
		StableAsset: module_stable_asset = 200,

		// Dev
		Sudo: pallet_sudo = 255,
	}
);

#[cfg(feature = "runtime-benchmarks")]
#[macro_use]
extern crate frame_benchmarking;

#[cfg(feature = "runtime-benchmarks")]
mod benches {
	define_benchmarks!(
		[frame_benchmarking, BaselineBench::<Runtime>]
		[pallet_babe, Babe]
		[pallet_bags_list, VoterList]
		[pallet_balances, Balances]
		[pallet_bounties, Bounties]
		[pallet_collective, Council]
		[pallet_democracy, Democracy]
		[pallet_election_provider_multi_phase, ElectionProviderMultiPhase]
		[pallet_election_provider_support_benchmarking, EPSBench::<Runtime>]
		[pallet_elections_phragmen, PhragmenElection]
		[pallet_grandpa, Grandpa]
		[pallet_im_online, ImOnline]
		[pallet_multisig, Multisig]
		[pallet_nomination_pools, NominationPoolsBench::<Runtime>]
		[pallet_offences, OffencesBench::<Runtime>]
		[pallet_preimage, Preimage]
		[pallet_proxy, Proxy]
		[pallet_scheduler, Scheduler]
		[pallet_session, SessionBench::<Runtime>]
		[pallet_staking, Staking]
		[frame_system, SystemBench::<Runtime>]
		[pallet_timestamp, Timestamp]
		[pallet_tips, Tips]
		[pallet_treasury, Treasury]
		[pallet_utility, Utility]
	);
}

impl_runtime_apis! {
	impl sp_api::Core<Block> for Runtime {
		fn version() -> RuntimeVersion {
			VERSION
		}

		fn execute_block(block: Block) {
			Executive::execute_block(block)
		}

		fn initialize_block(header: &<Block as BlockT>::Header) {
			Executive::initialize_block(header)
		}
	}

	impl sp_api::Metadata<Block> for Runtime {
		fn metadata() -> OpaqueMetadata {
			OpaqueMetadata::new(Runtime::metadata().into())
		}
	}

	impl sp_block_builder::BlockBuilder<Block> for Runtime {
		fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyExtrinsicResult {
			Executive::apply_extrinsic(extrinsic)
		}

		fn finalize_block() -> <Block as BlockT>::Header {
			Executive::finalize_block()
		}

		fn inherent_extrinsics(data: sp_inherents::InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
			data.create_extrinsics()
		}

		fn check_inherents(
			block: Block,
			data: sp_inherents::InherentData,
		) -> sp_inherents::CheckInherentsResult {
			data.check_extrinsics(&block)
		}
	}

	impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
		fn validate_transaction(
			source: TransactionSource,
			tx: <Block as BlockT>::Extrinsic,
			block_hash: <Block as BlockT>::Hash,
		) -> TransactionValidity {
			Executive::validate_transaction(source, tx, block_hash)
		}
	}

	impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
		fn offchain_worker(header: &<Block as BlockT>::Header) {
			Executive::offchain_worker(header)
		}
	}

	impl fg_primitives::GrandpaApi<Block> for Runtime {
		fn grandpa_authorities() -> GrandpaAuthorityList {
			Grandpa::grandpa_authorities()
		}

		fn current_set_id() -> fg_primitives::SetId {
			Grandpa::current_set_id()
		}

		fn submit_report_equivocation_unsigned_extrinsic(
			equivocation_proof: fg_primitives::EquivocationProof<
				<Block as BlockT>::Hash,
				NumberFor<Block>,
			>,
			key_owner_proof: fg_primitives::OpaqueKeyOwnershipProof,
		) -> Option<()> {
			let key_owner_proof = key_owner_proof.decode()?;

			Grandpa::submit_unsigned_equivocation_report(
				equivocation_proof,
				key_owner_proof,
			)
		}

		fn generate_key_ownership_proof(
			_set_id: fg_primitives::SetId,
			authority_id: GrandpaId,
		) -> Option<fg_primitives::OpaqueKeyOwnershipProof> {
			use codec::Encode;

			Historical::prove((fg_primitives::KEY_TYPE, authority_id))
				.map(|p| p.encode())
				.map(fg_primitives::OpaqueKeyOwnershipProof::new)
		}
	}

	impl sp_consensus_babe::BabeApi<Block> for Runtime {
		fn configuration() -> sp_consensus_babe::BabeGenesisConfiguration {
			// The choice of `c` parameter (where `1 - c` represents the
			// probability of a slot being empty), is done in accordance to the
			// slot duration and expected target block time, for safely
			// resisting network delays of maximum two seconds.
			// <https://research.web3.foundation/en/latest/polkadot/BABE/Babe/#6-practical-results>
			sp_consensus_babe::BabeGenesisConfiguration {
				slot_duration: Babe::slot_duration(),
				epoch_length: EpochDuration::get(),
				c: BABE_GENESIS_EPOCH_CONFIG.c,
				genesis_authorities: Babe::authorities().to_vec(),
				randomness: Babe::randomness(),
				allowed_slots: BABE_GENESIS_EPOCH_CONFIG.allowed_slots,
			}
		}

		fn current_epoch_start() -> sp_consensus_babe::Slot {
			Babe::current_epoch_start()
		}

		fn current_epoch() -> sp_consensus_babe::Epoch {
			Babe::current_epoch()
		}

		fn next_epoch() -> sp_consensus_babe::Epoch {
			Babe::next_epoch()
		}

		fn generate_key_ownership_proof(
			_slot: sp_consensus_babe::Slot,
			authority_id: sp_consensus_babe::AuthorityId,
		) -> Option<sp_consensus_babe::OpaqueKeyOwnershipProof> {
			use codec::Encode;

			Historical::prove((sp_consensus_babe::KEY_TYPE, authority_id))
				.map(|p| p.encode())
				.map(sp_consensus_babe::OpaqueKeyOwnershipProof::new)
		}

		fn submit_report_equivocation_unsigned_extrinsic(
			equivocation_proof: sp_consensus_babe::EquivocationProof<<Block as BlockT>::Header>,
			key_owner_proof: sp_consensus_babe::OpaqueKeyOwnershipProof,
		) -> Option<()> {
			let key_owner_proof = key_owner_proof.decode()?;

			Babe::submit_unsigned_equivocation_report(
				equivocation_proof,
				key_owner_proof,
			)
		}
	}

	impl sp_authority_discovery::AuthorityDiscoveryApi<Block> for Runtime {
		fn authorities() -> Vec<AuthorityDiscoveryId> {
			AuthorityDiscovery::authorities()
		}
	}

	impl sp_session::SessionKeys<Block> for Runtime {
		fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
			SessionKeys::generate(seed)
		}

		fn decode_session_keys(
			encoded: Vec<u8>,
		) -> Option<Vec<(Vec<u8>, KeyTypeId)>> {
			SessionKeys::decode_into_raw_public_keys(&encoded)
		}
	}

	impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Nonce> for Runtime {
		fn account_nonce(account: AccountId) -> Nonce {
			System::account_nonce(account)
		}
	}

	impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<
		Block,
		Balance,
	> for Runtime {
		fn query_info(uxt: <Block as BlockT>::Extrinsic, len: u32) -> RuntimeDispatchInfo<Balance> {
			TransactionPayment::query_info(uxt, len)
		}

		fn query_fee_details(uxt: <Block as BlockT>::Extrinsic, len: u32) -> FeeDetails<Balance> {
			TransactionPayment::query_fee_details(uxt, len)
		}
	}

	impl orml_oracle_rpc_runtime_api::OracleApi<
		Block,
		DataProviderId,
		CurrencyId,
		TimeStampedPrice,
	> for Runtime {
		fn get_value(provider_id: DataProviderId ,key: CurrencyId) -> Option<TimeStampedPrice> {
			match provider_id {
				DataProviderId::Selendra => SelendraOracle::get_no_op(&key),
				DataProviderId::Aggregated => <AggregatedDataProvider as DataProviderExtended<_, _>>::get_no_op(&key)
			}
		}

		fn get_all_values(provider_id: DataProviderId) -> Vec<(CurrencyId, Option<TimeStampedPrice>)> {
			match provider_id {
				DataProviderId::Selendra => SelendraOracle::get_all_values(),
				DataProviderId::Aggregated => <AggregatedDataProvider as DataProviderExtended<_, _>>::get_all_values()
			}
		}
	}

	impl orml_tokens_rpc_runtime_api::TokensApi<
		Block,
		CurrencyId,
		Balance,
	> for Runtime {
		fn query_existential_deposit(key: CurrencyId) -> Balance {
			if key == GetNativeCurrencyId::get() {
				NativeTokenExistentialDeposit::get()
			} else {
				ExistentialDeposits::get(&key)
			}
		}
	}

	impl module_evm_rpc_runtime_api::EVMRuntimeRPCApi<Block, Balance> for Runtime {
		fn block_limits() -> BlockLimits {
			BlockLimits {
				max_gas_limit: runtime_common::EvmLimits::<Runtime>::max_gas_limit(),
				max_storage_limit: runtime_common::EvmLimits::<Runtime>::max_storage_limit(),
			}
		}

		fn call(
			from: H160,
			to: H160,
			data: Vec<u8>,
			value: Balance,
			gas_limit: u64,
			storage_limit: u32,
			access_list: Option<Vec<AccessListItem>>,
			_estimate: bool,
		) -> Result<CallInfo, sp_runtime::DispatchError> {
			<Runtime as module_evm::Config>::Runner::rpc_call(
				from,
				from,
				to,
				data,
				value,
				gas_limit,
				storage_limit,
				access_list.unwrap_or_default().into_iter().map(|v| (v.address, v.storage_keys)).collect(),
				<Runtime as module_evm::Config>::config(),
			)
		}

		fn create(
			from: H160,
			data: Vec<u8>,
			value: Balance,
			gas_limit: u64,
			storage_limit: u32,
			access_list: Option<Vec<AccessListItem>>,
			_estimate: bool,
		) -> Result<CreateInfo, sp_runtime::DispatchError> {
			<Runtime as module_evm::Config>::Runner::rpc_create(
				from,
				data,
				value,
				gas_limit,
				storage_limit,
				access_list.unwrap_or_default().into_iter().map(|v| (v.address, v.storage_keys)).collect(),
				<Runtime as module_evm::Config>::config(),
			)
		}

		fn get_estimate_resources_request(extrinsic: Vec<u8>) -> Result<EstimateResourcesRequest, sp_runtime::DispatchError> {
			let utx = UncheckedExtrinsic::decode_all_with_depth_limit(sp_api::MAX_EXTRINSIC_DEPTH, &mut &*extrinsic)
				.map_err(|_| sp_runtime::DispatchError::Other("Invalid parameter extrinsic, decode failed"))?;

			let request = match utx.0.function {
				Call::EVM(module_evm::Call::call{target, input, value, gas_limit, storage_limit, access_list}) => {
					Some(EstimateResourcesRequest {
						from: None,
						to: Some(target),
						gas_limit: Some(gas_limit),
						storage_limit: Some(storage_limit),
						value: Some(value),
						data: Some(input),
						access_list: Some(access_list)
					})
				}
				Call::EVM(module_evm::Call::create{input, value, gas_limit, storage_limit, access_list}) => {
					Some(EstimateResourcesRequest {
						from: None,
						to: None,
						gas_limit: Some(gas_limit),
						storage_limit: Some(storage_limit),
						value: Some(value),
						data: Some(input),
						access_list: Some(access_list)
					})
				}
				_ => None,
			};

			request.ok_or(sp_runtime::DispatchError::Other("Invalid parameter extrinsic, not evm Call"))
		}
	}

	#[cfg(feature = "try-runtime")]
	impl frame_try_runtime::TryRuntime<Block> for Runtime {
		fn on_runtime_upgrade() -> (Weight, Weight) {
			// NOTE: intentional unwrap: we don't want to propagate the error backwards, and want to
			// have a backtrace here. If any of the pre/post migration checks fail, we shall stop
			// right here and right now.
			let weight = Executive::try_runtime_upgrade().unwrap();
			(weight, RuntimeBlockWeights::get().max_block)
		}

		fn execute_block_no_check(block: Block) -> Weight {
			Executive::execute_block_no_check(block)
		}
	}

	#[cfg(feature = "runtime-benchmarks")]
	impl frame_benchmarking::Benchmark<Block> for Runtime {
		fn benchmark_metadata(extra: bool) -> (
			Vec<frame_benchmarking::BenchmarkList>,
			Vec<frame_support::traits::StorageInfo>,
		) {
			use frame_benchmarking::{baseline, Benchmarking, BenchmarkList};
			use frame_support::traits::StorageInfoTrait;

			// Trying to add benchmarks directly to the Session Pallet caused cyclic dependency
			// issues. To get around that, we separated the Session benchmarks into its own crate,
			// which is why we need these two lines below.
			use pallet_session_benchmarking::Pallet as SessionBench;
			use pallet_offences_benchmarking::Pallet as OffencesBench;
			use pallet_election_provider_support_benchmarking::Pallet as EPSBench;
			use frame_system_benchmarking::Pallet as SystemBench;
			use baseline::Pallet as BaselineBench;
			use pallet_nomination_pools_benchmarking::Pallet as NominationPoolsBench;

			let mut list = Vec::<BenchmarkList>::new();
			list_benchmarks!(list, extra);

			let storage_info = AllPalletsWithSystem::storage_info();

			(list, storage_info)
		}

		fn dispatch_benchmark(
			config: frame_benchmarking::BenchmarkConfig
		) -> Result<Vec<frame_benchmarking::BenchmarkBatch>, sp_runtime::RuntimeString> {
			use frame_benchmarking::{baseline, Benchmarking, BenchmarkBatch,  TrackedStorageKey};

			// Trying to add benchmarks directly to the Session Pallet caused cyclic dependency
			// issues. To get around that, we separated the Session benchmarks into its own crate,
			// which is why we need these two lines below.
			use pallet_session_benchmarking::Pallet as SessionBench;
			use pallet_offences_benchmarking::Pallet as OffencesBench;
			use pallet_election_provider_support_benchmarking::Pallet as EPSBench;
			use frame_system_benchmarking::Pallet as SystemBench;
			use baseline::Pallet as BaselineBench;
			use pallet_nomination_pools_benchmarking::Pallet as NominationPoolsBench;

			impl pallet_session_benchmarking::Config for Runtime {}
			impl pallet_offences_benchmarking::Config for Runtime {}
			impl pallet_election_provider_support_benchmarking::Config for Runtime {}
			impl frame_system_benchmarking::Config for Runtime {}
			impl baseline::Config for Runtime {}
			impl pallet_nomination_pools_benchmarking::Config for Runtime {}

			let whitelist: Vec<TrackedStorageKey> = vec![
				// Block Number
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef702a5c1b19ab7a04f536c519aca4983ac").to_vec().into(),
				// Total Issuance
				hex_literal::hex!("c2261276cc9d1f8598ea4b6a74b15c2f57c875e4cff74148e4628f264b974c80").to_vec().into(),
				// Execution Phase
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef7ff553b5a9862a516939d82b3d3d8661a").to_vec().into(),
				// Event Count
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef70a98fdbe9ce6c55837576c60c7af3850").to_vec().into(),
				// System Events
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef780d41e5e16056765bc8461851072c9d7").to_vec().into(),
				// System BlockWeight
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef734abf5cb34d6244378cddbf18e849d96").to_vec().into(),
				// Treasury Account
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef7b99d880ec681799c0cf30e8886371da95ecffd7b6c0f78751baa9d281e0bfa3a6d6f646c70792f74727372790000000000000000000000000000000000000000").to_vec().into(),
			];

			let mut batches = Vec::<BenchmarkBatch>::new();
			let params = (&config, &whitelist);
			add_benchmarks!(params, batches);
			Ok(batches)
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use frame_support::dispatch::DispatchInfo;
	use frame_system::offchain::CreateSignedTransaction;
	use module_support::AddressMapping;
	use sp_runtime::traits::SignedExtension;

	#[test]
	fn validate_transaction_submitter_bounds() {
		fn is_submit_signed_transaction<T>()
		where
			T: CreateSignedTransaction<Call>,
		{
		}

		is_submit_signed_transaction::<Runtime>();
	}

	#[test]
	fn ensure_can_create_contract() {
		// Ensure that the `ExistentialDeposit` for creating the contract >= account `ExistentialDeposit`.
		// Otherwise, the creation of the contract account will fail because it is less than
		// ExistentialDeposit.
		assert!(
			Balance::from(NewContractExtraBytes::get()).saturating_mul(
				<StorageDepositPerByte as frame_support::traits::Get<Balance>>::get() /
					10u128.saturating_pow(6)
			) >= NativeTokenExistentialDeposit::get()
		);
	}

	#[test]
	fn check_call_size() {
		assert!(
			core::mem::size_of::<Call>() <= 280,
			"size of Call is more than 280 bytes: some calls have too big arguments, use Box to \
			reduce the size of Call.
			If the limit is too strong, maybe consider increasing the limit",
		);
	}

	#[test]
	fn convert_tx_check_evm_nonce() {
		sp_io::TestExternalities::new_empty().execute_with(|| {
			let alice: AccountId = sp_runtime::AccountId32::from([8; 32]);
			System::inc_account_nonce(&alice); // system::account.nonce = 1

			let address = EvmAddressMapping::<Runtime>::get_evm_address(&alice)
				.unwrap_or_else(|| EvmAddressMapping::<Runtime>::get_default_evm_address(&alice));

			// set evm nonce to 3
			module_evm::Accounts::<Runtime>::insert(
				&address,
				module_evm::AccountInfo { nonce: 3, contract_info: None },
			);

			let call = Call::EVM(module_evm::Call::eth_call {
				action: module_evm::TransactionAction::Create,
				input: vec![0x01],
				value: 0,
				gas_limit: 21_000,
				storage_limit: 1_000,
				valid_until: 30,
				access_list: vec![],
			});

			let extra: SignedExtra = (
				frame_system::CheckNonZeroSender::<Runtime>::new(),
				frame_system::CheckSpecVersion::<Runtime>::new(),
				frame_system::CheckTxVersion::<Runtime>::new(),
				frame_system::CheckGenesis::<Runtime>::new(),
				frame_system::CheckEra::<Runtime>::from(generic::Era::Immortal),
				runtime_common::CheckNonce::<Runtime>::from(3),
				frame_system::CheckWeight::<Runtime>::new(),
				module_transaction_payment::ChargeTransactionPayment::<Runtime>::from(0),
				module_evm::SetEvmOrigin::<Runtime>::new(),
			);

			let mut expected_extra = extra.clone();
			expected_extra.5.mark_as_ethereum_tx(30);

			assert_eq!(
				ConvertEthereumTx::convert((call.clone(), extra.clone())).unwrap(),
				(
					EthereumTransactionMessage {
						nonce: 3, // evm::account.nonce
						tip: 0,
						gas_limit: 21_000,
						storage_limit: 1_000,
						action: module_evm::TransactionAction::Create,
						value: 0,
						input: vec![0x01],
						chain_id: 0,
						genesis: sp_core::H256::default(),
						valid_until: 30,
						access_list: vec![],
					},
					expected_extra.clone()
				)
			);

			let info = DispatchInfo::default();

			// valid tx in future
			assert_eq!(
				extra.5.validate(&alice, &call, &info, 0),
				Ok(sp_runtime::transaction_validity::ValidTransaction {
					priority: 0,
					requires: vec![Encode::encode(&(alice.clone(), 2u32))],
					provides: vec![Encode::encode(&(alice.clone(), 3u32))],
					longevity: sp_runtime::transaction_validity::TransactionLongevity::MAX,
					propagate: true,
				})
			);
			// valid evm tx
			assert_eq!(
				expected_extra.5.validate(&alice, &call, &info, 0),
				Ok(sp_runtime::transaction_validity::ValidTransaction {
					priority: 0,
					requires: vec![],
					provides: vec![Encode::encode(&(address, 3u32))],
					longevity: 30,
					propagate: true,
				})
			);

			// valid evm tx in future
			expected_extra.5.nonce = 4;
			assert_eq!(
				expected_extra.5.validate(&alice, &call, &info, 0),
				Ok(sp_runtime::transaction_validity::ValidTransaction {
					priority: 0,
					requires: vec![Encode::encode(&(address, 3u32))],
					provides: vec![Encode::encode(&(address, 4u32))],
					longevity: 30,
					propagate: true,
				})
			);
		});
	}

	fn new_test_ext() -> sp_io::TestExternalities {
		let t = frame_system::GenesisConfig::default().build_storage::<Runtime>().unwrap();
		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| System::set_block_number(1));
		ext
	}

	#[test]
	fn payer_signature_verify() {
		use sp_core::Pair;

		let extra: SignedExtra = (
			frame_system::CheckNonZeroSender::<Runtime>::new(),
			frame_system::CheckSpecVersion::<Runtime>::new(),
			frame_system::CheckTxVersion::<Runtime>::new(),
			frame_system::CheckGenesis::<Runtime>::new(),
			frame_system::CheckEra::<Runtime>::from(generic::Era::Immortal),
			runtime_common::CheckNonce::<Runtime>::from(0),
			frame_system::CheckWeight::<Runtime>::new(),
			module_transaction_payment::ChargeTransactionPayment::<Runtime>::from(0),
			module_evm::SetEvmOrigin::<Runtime>::new(),
		);

		// correct payer signature
		new_test_ext().execute_with(|| {
			let payer = sp_keyring::AccountKeyring::Charlie;

			let call = Call::Balances(pallet_balances::Call::transfer {
				dest: sp_runtime::MultiAddress::Id(sp_keyring::AccountKeyring::Bob.to_account_id()),
				value: 100,
			});

			let raw_payload = SignedPayload::new(call.clone(), extra.clone()).unwrap();
			let payer_signature = raw_payload.using_encoded(|payload| payer.pair().sign(payload));

			let fee_call =
				Call::TransactionPayment(module_transaction_payment::Call::with_fee_paid_by {
					call: Box::new(call),
					payer_addr: payer.to_account_id(),
					payer_sig: sp_runtime::MultiSignature::Sr25519(payer_signature),
				});
			assert!(PayerSignatureVerification::convert((fee_call, extra.clone())).is_ok());
		});

		// wrong payer signature
		new_test_ext().execute_with(|| {
			let hacker = sp_keyring::AccountKeyring::Dave;

			let call = Call::Balances(pallet_balances::Call::transfer {
				dest: sp_runtime::MultiAddress::Id(sp_keyring::AccountKeyring::Bob.to_account_id()),
				value: 100,
			});
			let hacker_call = Call::Balances(pallet_balances::Call::transfer {
				dest: sp_runtime::MultiAddress::Id(
					sp_keyring::AccountKeyring::Dave.to_account_id(),
				),
				value: 100,
			});

			let raw_payload = SignedPayload::new(hacker_call.clone(), extra.clone()).unwrap();
			let payer_signature = raw_payload.using_encoded(|payload| hacker.pair().sign(payload));

			let fee_call =
				Call::TransactionPayment(module_transaction_payment::Call::with_fee_paid_by {
					call: Box::new(call),
					payer_addr: hacker.to_account_id(),
					payer_sig: sp_runtime::MultiSignature::Sr25519(payer_signature),
				});
			assert!(PayerSignatureVerification::convert((fee_call, extra)).is_err());
		});
	}
}

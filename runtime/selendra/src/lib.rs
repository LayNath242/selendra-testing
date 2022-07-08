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
		ConstBool, ConstU128, ConstU16, ConstU32, Contains, ContainsLengthBound, Currency as PalletCurrency,
		EnsureOrigin, EqualPrivilegeOnly, Everything, Get, Imbalance, InstanceFilter, IsSubType, IsType,
		KeyOwnerProofSystem, LockIdentifier, Nothing, OnRuntimeUpgrade, OnUnbalanced, Randomness, SortedMembers,
		U128CurrencyToVote, WithdrawReasons,
	},
	weights::{
		constants::{BlockExecutionWeight, RocksDbWeight, WEIGHT_PER_SECOND},
		DispatchClass, IdentityFee, Weight,
	},
	PalletId, RuntimeDebug, StorageValue,
};
use frame_system::{EnsureRoot, RawOrigin};
use module_asset_registry::{AssetIdMaps, EvmErc20InfoMapping};
use module_cdp_engine::CollateralCurrencyIds;
use module_currencies::BasicCurrencyAdapter;
use module_evm::{runner::RunnerExtended, CallInfo, CreateInfo};
use module_evm_accounts::EvmAddressMapping;
use module_support::{AssetIdMapping, DispatchableTask, PoolId};
use module_transaction_payment::TargetedFeeAdjustment;

use orml_tokens::CurrencyAdapter;
use orml_traits::{
	create_median_value_data_provider, parameter_type_with_key, DataFeeder, DataProviderExtended, GetByKey,
};
use pallet_transaction_payment::{FeeDetails, RuntimeDispatchInfo};
use primitives::{
	evm::{AccessListItem, EthereumTransactionMessage},
	unchecked_extrinsic::SelendraUncheckedExtrinsic,
};
#[cfg(feature = "std")]
pub use pallet_staking::StakerStatus;
use sp_api::impl_runtime_apis;
use sp_core::{crypto::KeyTypeId, OpaqueMetadata, H160};
use sp_runtime::{
	create_runtime_str, generic, impl_opaque_keys,
	traits::{
		AccountIdConversion, BadOrigin, BlakeTwo256, Block as BlockT, Convert, SaturatedConversion, StaticLookup,
		Verify, Bounded, NumberFor
	},
	transaction_validity::{TransactionSource, TransactionValidity},
	ApplyExtrinsicResult, DispatchResult, FixedPointNumber,
};
use sp_std::prelude::*;
use pallet_session::historical::{self as pallet_session_historical};
use pallet_grandpa::{
	fg_primitives, AuthorityId as GrandpaId, AuthorityList as GrandpaAuthorityList,
};
use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;

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
	AccountId, AccountIndex, Address, Amount, AuctionId, AuthoritysOriginIdMadala, Balance, BlockNumber, CurrencyId,
	DataProviderId, EraIndex, Hash, Lease, Moment, Multiplier, Nonce, ReserveIdentifier, Share, Signature, TokenSymbol,
	TradingPair,
};
pub use runtime_common::{
	cent, dollar, microcent, millicent, AllPrecompiles, EnsureRootOrAllCouncil,
	EnsureRootOrAllTechnicalCommittee, EnsureRootOrHalfFinancialCouncil, EnsureRootOrHalfCouncil,
	EnsureRootOrOneCouncil, EnsureRootOrOneThirdsTechnicalCommittee,
	EnsureRootOrThreeFourthsCouncil, EnsureRootOrTwoThirdsCouncil,
	EnsureRootOrTwoThirdsTechnicalCommittee, ExchangeRate, ExistentialDepositsTimesOneHundred,
	FinancialCouncilInstance, FinancialCouncilMembershipInstance, GasToWeight, CouncilInstance,
	CouncilMembershipInstance, MaxTipsOfPriority, BlockHashCount,
	OffchainSolutionWeightLimit, OperationalFeeMultiplier, OperatorMembershipInstanceSelendra, Price, ProxyType, Rate,
	Ratio, RuntimeBlockLength, RuntimeBlockWeights, SystemContractsFilter, TechnicalCommitteeInstance,
	TechnicalCommitteeMembershipInstance, TimeStampedPrice, TipPerWeightStep, ACA, AUSD, DOT, LACA, KSM, RENBTC,
};

/// Import the stable_asset pallet.
pub use module_stable_asset;

mod authority;
mod benchmarking;
pub mod constants;
/// Weights for pallets used in the runtime.
mod weights;
mod voter_bags;

// runtime config
mod config;

pub use config::consensus_config::{EpochDuration, MaxNominations};
#[cfg(test)]
use config::evm_config::NewContractExtraBytes;
use config::evm_config::{StorageDepositPerByte, TxFeePerGas};

/// This runtime version.
#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
	spec_name: create_runtime_str!("selendra"),
	impl_name: create_runtime_str!("selendra"),
	authoring_version: 1,
	spec_version: 1001,
	impl_version: 0,
	apis: RUNTIME_API_VERSIONS,
	transaction_version: 1,
	state_version: 0,
};

/// The version infromation used to identify this runtime when compiled
/// natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
	NativeVersion {
		runtime_version: VERSION,
		can_author_with: Default::default(),
	}
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
	pub const TreasuryPalletId: PalletId = PalletId(*b"aca/trsy");
	pub const LoansPalletId: PalletId = PalletId(*b"aca/loan");
	pub const DEXPalletId: PalletId = PalletId(*b"aca/dexm");
	pub const CDPTreasuryPalletId: PalletId = PalletId(*b"aca/cdpt");
	pub const HonzonTreasuryPalletId: PalletId = PalletId(*b"aca/hztr");
	pub const IncentivesPalletId: PalletId = PalletId(*b"aca/inct");
	pub const CollatorPotId: PalletId = PalletId(*b"aca/cpot");
	// Treasury reserve
	pub const TreasuryReservePalletId: PalletId = PalletId(*b"aca/reve");
	pub const PhragmenElectionPalletId: LockIdentifier = *b"aca/phre";
	pub const NftPalletId: PalletId = PalletId(*b"aca/aNFT");
	pub UnreleasedNativeVaultAccountId: AccountId = PalletId(*b"aca/urls").into_account_truncating();
	// This Pallet is only used to payment fee pool, it's not added to whitelist by design.
	// because transaction payment pallet will ensure the accounts always have enough ED.
	pub const TransactionPaymentPalletId: PalletId = PalletId(*b"aca/fees");
	pub const StableAssetPalletId: PalletId = PalletId(*b"nuts/sta");
	// lock identifier for earning module
	pub const EarningLockIdentifier: LockIdentifier = *b"aca/earn";
}

pub fn get_all_module_accounts() -> Vec<AccountId> {
	vec![
		TreasuryPalletId::get().into_account_truncating(),
		LoansPalletId::get().into_account_truncating(),
		DEXPalletId::get().into_account_truncating(),
		CDPTreasuryPalletId::get().into_account_truncating(),
		HonzonTreasuryPalletId::get().into_account_truncating(),
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
		!module_transaction_pause::PausedTransactionFilter::<Runtime>::contains(call)
			&& !matches!(call, Call::Democracy(pallet_democracy::Call::propose { .. }),)
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
	type OnKilledAccount = (
		module_evm::CallKillAccount<Runtime>,
		module_evm_accounts::CallKillAccount<Runtime>,
	);
	type DbWeight = RocksDbWeight;
	type BaseCallFilter = BaseCallFilter;
	type SystemWeightInfo = ();
	type SS58Prefix = SS58Prefix;
	type OnSetCode = ();
	type MaxConsumers = ConstU32<16>;
}

parameter_types! {
	pub const DisabledValidatorsThreshold: Perbill = Perbill::from_percent(33);
	pub const SessionDuration: BlockNumber = DAYS; // used in SessionManagerConfig of genesis
}

parameter_types! {
	pub const CollatorKickThreshold: Permill = Permill::from_percent(50);
	// Ensure that can create the author(`ExistentialDeposit`) with dev mode.
	pub MinRewardDistributeAmount: Balance = NativeTokenExistentialDeposit::get();
}

parameter_types! {
	pub IndexDeposit: Balance = dollar(ACA);
}

impl pallet_indices::Config for Runtime {
	type AccountIndex = AccountIndex;
	type Event = Event;
	type Currency = Balances;
	type Deposit = IndexDeposit;
	type WeightInfo = ();
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
	pub NativeTokenExistentialDeposit: Balance = 10 * cent(ACA);
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
	pub TransactionByteFee: Balance = 10 * millicent(ACA);
	pub const TargetBlockFullness: Perquintill = Perquintill::from_percent(25);
	pub AdjustmentVariable: Multiplier = Multiplier::saturating_from_rational(1, 100_000);
	pub MinimumMultiplier: Multiplier = Multiplier::saturating_from_rational(1, 1_000_000_000u128);
}

impl pallet_sudo::Config for Runtime {
	type Event = Event;
	type Call = Call;
}

parameter_types! {
	pub const CouncilMotionDuration: BlockNumber = 7 * DAYS;
	pub const CouncilDefaultMaxProposals: u32 = 100;
	pub const CouncilDefaultMaxMembers: u32 = 100;
}

impl pallet_collective::Config<CouncilInstance> for Runtime {
	type Origin = Origin;
	type Proposal = Call;
	type Event = Event;
	type MotionDuration = CouncilMotionDuration;
	type MaxProposals = CouncilDefaultMaxProposals;
	type MaxMembers = CouncilDefaultMaxMembers;
	type DefaultVote = pallet_collective::PrimeDefaultVote;
	type WeightInfo = ();
}

impl pallet_membership::Config<CouncilMembershipInstance> for Runtime {
	type Event = Event;
	type AddOrigin = EnsureRootOrThreeFourthsCouncil;
	type RemoveOrigin = EnsureRootOrThreeFourthsCouncil;
	type SwapOrigin = EnsureRootOrThreeFourthsCouncil;
	type ResetOrigin = EnsureRootOrThreeFourthsCouncil;
	type PrimeOrigin = EnsureRootOrThreeFourthsCouncil;
	type MembershipInitialized = Council;
	type MembershipChanged = Council;
	type MaxMembers = CouncilDefaultMaxMembers;
	type WeightInfo = ();
}

parameter_types! {
	pub const FinancialCouncilMotionDuration: BlockNumber = 7 * DAYS;
}

impl pallet_collective::Config<FinancialCouncilInstance> for Runtime {
	type Origin = Origin;
	type Proposal = Call;
	type Event = Event;
	type MotionDuration = FinancialCouncilMotionDuration;
	type MaxProposals = CouncilDefaultMaxProposals;
	type MaxMembers = CouncilDefaultMaxMembers;
	type DefaultVote = pallet_collective::PrimeDefaultVote;
	type WeightInfo = ();
}

impl pallet_membership::Config<FinancialCouncilMembershipInstance> for Runtime {
	type Event = Event;
	type AddOrigin = EnsureRootOrTwoThirdsCouncil;
	type RemoveOrigin = EnsureRootOrTwoThirdsCouncil;
	type SwapOrigin = EnsureRootOrTwoThirdsCouncil;
	type ResetOrigin = EnsureRootOrTwoThirdsCouncil;
	type PrimeOrigin = EnsureRootOrTwoThirdsCouncil;
	type MembershipInitialized = FinancialCouncil;
	type MembershipChanged = FinancialCouncil;
	type MaxMembers = CouncilDefaultMaxMembers;
	type WeightInfo = ();
}

parameter_types! {
	pub const TechnicalCommitteeMotionDuration: BlockNumber = 7 * DAYS;
}

impl pallet_collective::Config<TechnicalCommitteeInstance> for Runtime {
	type Origin = Origin;
	type Proposal = Call;
	type Event = Event;
	type MotionDuration = TechnicalCommitteeMotionDuration;
	type MaxProposals = CouncilDefaultMaxProposals;
	type MaxMembers = CouncilDefaultMaxMembers;
	type DefaultVote = pallet_collective::PrimeDefaultVote;
	type WeightInfo = ();
}

impl pallet_membership::Config<TechnicalCommitteeMembershipInstance> for Runtime {
	type Event = Event;
	type AddOrigin = EnsureRootOrTwoThirdsCouncil;
	type RemoveOrigin = EnsureRootOrTwoThirdsCouncil;
	type SwapOrigin = EnsureRootOrTwoThirdsCouncil;
	type ResetOrigin = EnsureRootOrTwoThirdsCouncil;
	type PrimeOrigin = EnsureRootOrTwoThirdsCouncil;
	type MembershipInitialized = TechnicalCommittee;
	type MembershipChanged = TechnicalCommittee;
	type MaxMembers = CouncilDefaultMaxMembers;
	type WeightInfo = ();
}

impl pallet_membership::Config<OperatorMembershipInstanceSelendra> for Runtime {
	type Event = Event;
	type AddOrigin = EnsureRootOrTwoThirdsCouncil;
	type RemoveOrigin = EnsureRootOrTwoThirdsCouncil;
	type SwapOrigin = EnsureRootOrTwoThirdsCouncil;
	type ResetOrigin = EnsureRootOrTwoThirdsCouncil;
	type PrimeOrigin = EnsureRootOrTwoThirdsCouncil;
	type MembershipInitialized = ();
	type MembershipChanged = SelendraOracle;
	type MaxMembers = ConstU32<50>;
	type WeightInfo = ();
}

impl pallet_utility::Config for Runtime {
	type Event = Event;
	type Call = Call;
	type PalletsOrigin = OriginCaller;
	type WeightInfo = ();
}

parameter_types! {
	pub MultisigDepositBase: Balance = 500 * millicent(ACA);
	pub MultisigDepositFactor: Balance = 100 * millicent(ACA);
}

impl pallet_multisig::Config for Runtime {
	type Event = Event;
	type Call = Call;
	type Currency = Balances;
	type DepositBase = MultisigDepositBase;
	type DepositFactor = MultisigDepositFactor;
	type MaxSignatories = ConstU16<100>;
	type WeightInfo = ();
}

pub struct CouncilProvider;
impl SortedMembers<AccountId> for CouncilProvider {
	fn sorted_members() -> Vec<AccountId> {
		Council::members()
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn add(_: &AccountId) {
		todo!()
	}
}

impl ContainsLengthBound for CouncilProvider {
	fn max_len() -> usize {
		100
	}
	fn min_len() -> usize {
		0
	}
}

parameter_types! {
	pub const ProposalBond: Permill = Permill::from_percent(5);
	pub ProposalBondMinimum: Balance = dollar(ACA);
	pub ProposalBondMaximum: Balance = 5 * dollar(ACA);
	pub const SpendPeriod: BlockNumber = DAYS;
	pub const Burn: Permill = Permill::from_percent(0);
	pub const TipCountdown: BlockNumber = DAYS;
	pub const TipFindersFee: Percent = Percent::from_percent(10);
	pub TipReportDepositBase: Balance = dollar(ACA);
	pub const SevenDays: BlockNumber = 7 * DAYS;
	pub const ZeroDay: BlockNumber = 0;
	pub const OneDay: BlockNumber = DAYS;
	pub BountyDepositBase: Balance = dollar(ACA);
	pub const BountyDepositPayoutDelay: BlockNumber = DAYS;
	pub const BountyUpdatePeriod: BlockNumber = 14 * DAYS;
	pub const CuratorDepositMultiplier: Permill = Permill::from_percent(50);
	pub CuratorDepositMin: Balance = dollar(ACA);
	pub CuratorDepositMax: Balance = 100 * dollar(ACA);
	pub BountyValueMinimum: Balance = 5 * dollar(ACA);
	pub DataDepositPerByte: Balance = cent(ACA);
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
	type Tippers = CouncilProvider;
	type TipCountdown = TipCountdown;
	type TipFindersFee = TipFindersFee;
	type TipReportDepositBase = TipReportDepositBase;
	type WeightInfo = ();
}

parameter_types! {
	pub ConfigDepositBase: Balance = 10 * cent(ACA);
	pub FriendDepositFactor: Balance = cent(ACA);
	pub RecoveryDeposit: Balance = 10 * cent(ACA);
}

impl pallet_recovery::Config for Runtime {
	type Event = Event;
	type Call = Call;
	type Currency = Balances;
	type ConfigDepositBase = ConfigDepositBase;
	type FriendDepositFactor = FriendDepositFactor;
	type MaxFriends = ConstU32<9>;
	type RecoveryDeposit = RecoveryDeposit;
	type WeightInfo = ();
}

parameter_types! {
	pub const LaunchPeriod: BlockNumber = 2 * HOURS;
	pub const VotingPeriod: BlockNumber = HOURS;
	pub const FastTrackVotingPeriod: BlockNumber = HOURS;
	pub MinimumDeposit: Balance = 100 * cent(ACA);
	pub const EnactmentPeriod: BlockNumber = MINUTES;
	pub const CooloffPeriod: BlockNumber = MINUTES;
}

impl pallet_democracy::Config for Runtime {
	type Proposal = Call;
	type Event = Event;
	type Currency = Balances;
	type EnactmentPeriod = EnactmentPeriod;
	type LaunchPeriod = LaunchPeriod;
	type VotingPeriod = VotingPeriod;
	type VoteLockingPeriod = EnactmentPeriod; // Same as EnactmentPeriod
	type MinimumDeposit = MinimumDeposit;
	/// A straight majority of the council can decide what their next motion is.
	type ExternalOrigin = EnsureRootOrHalfCouncil;
	/// A majority can have the next scheduled referendum be a straight majority-carries vote.
	type ExternalMajorityOrigin = EnsureRootOrHalfCouncil;
	/// A unanimous council can have the next scheduled referendum be a straight default-carries
	/// (NTB) vote.
	type ExternalDefaultOrigin = EnsureRootOrAllCouncil;
	/// Two thirds of the technical committee can have an ExternalMajority/ExternalDefault vote
	/// be tabled immediately and with a shorter voting/enactment period.
	type FastTrackOrigin = EnsureRootOrTwoThirdsTechnicalCommittee;
	type InstantOrigin = EnsureRootOrAllTechnicalCommittee;
	type InstantAllowed = ConstBool<true>;
	type FastTrackVotingPeriod = FastTrackVotingPeriod;
	// To cancel a proposal which has been passed, 2/3 of the council must agree to it.
	type CancellationOrigin = EnsureRootOrTwoThirdsCouncil;
	type BlacklistOrigin = EnsureRoot<AccountId>;
	// To cancel a proposal before it has been passed, the technical committee must be unanimous or
	// Root must agree.
	type CancelProposalOrigin = EnsureRootOrAllTechnicalCommittee;
	// Any single technical committee member may veto a coming council proposal, however they can
	// only do it once and it lasts only for the cooloff period.
	type VetoOrigin = pallet_collective::EnsureMember<AccountId, TechnicalCommitteeInstance>;
	type CooloffPeriod = CooloffPeriod;
	type PreimageByteDeposit = PreimageByteDeposit;
	type OperationalPreimageOrigin = pallet_collective::EnsureMember<AccountId, CouncilInstance>;
	type Slash = Treasury;
	type Scheduler = Scheduler;
	type PalletsOrigin = OriginCaller;
	type MaxVotes = ConstU32<100>;
	//TODO: might need to weight for Selendra
	type WeightInfo = pallet_democracy::weights::SubstrateWeight<Runtime>;
	type MaxProposals = CouncilDefaultMaxProposals;
}

impl orml_auction::Config for Runtime {
	type Event = Event;
	type Balance = Balance;
	type AuctionId = AuctionId;
	type Handler = AuctionManager;
	type WeightInfo = weights::orml_auction::WeightInfo<Runtime>;
}

impl orml_authority::Config for Runtime {
	type Event = Event;
	type Origin = Origin;
	type PalletsOrigin = OriginCaller;
	type Call = Call;
	type Scheduler = Scheduler;
	type AsOriginId = AuthoritysOriginIdMadala;
	type AuthorityConfig = AuthorityConfigImpl;
	type WeightInfo = weights::orml_authority::WeightInfo<Runtime>;
}

parameter_types! {
	pub CandidacyBond: Balance = 10 * dollar(ACA);
	pub VotingBondBase: Balance = 2 * dollar(ACA);
	pub VotingBondFactor: Balance = dollar(ACA);
	pub const TermDuration: BlockNumber = 7 * DAYS;
}

impl pallet_elections_phragmen::Config for Runtime {
	type PalletId = PhragmenElectionPalletId;
	type Event = Event;
	type Currency = CurrencyAdapter<Runtime, GetLiquidCurrencyId>;
	type CurrencyToVote = U128CurrencyToVote;
	type ChangeMembers = Council;
	type InitializeMembers = Council;
	type CandidacyBond = CandidacyBond;
	type VotingBondBase = VotingBondBase;
	type VotingBondFactor = VotingBondFactor;
	type TermDuration = TermDuration;
	type DesiredMembers = ConstU32<13>;
	type DesiredRunnersUp = ConstU32<7>;
	type LoserCandidate = ();
	type KickedMember = ();
	type WeightInfo = ();
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
	type CombineData = orml_oracle::DefaultCombineData<Runtime, MinimumCount, ExpiresIn, SelendraDataProvider>;
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
				TokenSymbol::ACA => cent(*currency_id),
				TokenSymbol::AUSD => cent(*currency_id),
				TokenSymbol::DOT |
				TokenSymbol::LACA |
				TokenSymbol::KUSD |
				TokenSymbol::KSM |
				TokenSymbol::LKSM |
				TokenSymbol::RENBTC |
				TokenSymbol::CASH => Balance::max_value() // unsupported
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
	pub StableCurrencyFixedPrice: Price = Price::saturating_from_rational(1, 1);
	pub RewardRatePerRelaychainBlock: Rate = Rate::saturating_from_rational(2_492, 100_000_000_000u128);	// 14% annual staking reward rate of Polkadot
}

parameter_type_with_key! {
	pub PricingPegged: |_currency_id: CurrencyId| -> Option<CurrencyId> {
		None
	};
}

impl module_prices::Config for Runtime {
	type Event = Event;
	type Source = AggregatedDataProvider;
	type GetStableCurrencyId = GetStableCurrencyId;
	type StableCurrencyFixedPrice = StableCurrencyFixedPrice;
	type LockOrigin = EnsureRootOrTwoThirdsCouncil;
	type DEX = Dex;
	type Currency = Currencies;
	type Erc20InfoMapping = EvmErc20InfoMapping<Runtime>;
	type PricingPegged = PricingPegged;
	type WeightInfo = weights::module_prices::WeightInfo<Runtime>;
}

parameter_types! {
	pub const GetNativeCurrencyId: CurrencyId = ACA;
	pub const GetStableCurrencyId: CurrencyId = AUSD;
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
			}
			r => Err(Origin::from(r)),
		})
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn successful_origin() -> Origin {
		let zero_account_id = AccountId::decode(&mut sp_runtime::traits::TrailingZeroInput::zeroes())
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

parameter_types! {
	pub MinimumIncrementSize: Rate = Rate::saturating_from_rational(2, 100);
	pub const AuctionTimeToClose: BlockNumber = 15 * MINUTES;
	pub const AuctionDurationSoftCap: BlockNumber = 2 * HOURS;
}

impl module_auction_manager::Config for Runtime {
	type Event = Event;
	type Currency = Currencies;
	type Auction = Auction;
	type MinimumIncrementSize = MinimumIncrementSize;
	type AuctionTimeToClose = AuctionTimeToClose;
	type AuctionDurationSoftCap = AuctionDurationSoftCap;
	type GetStableCurrencyId = GetStableCurrencyId;
	type CDPTreasury = CdpTreasury;
	type PriceSource = module_prices::PriorityLockedPriceProvider<Runtime>;
	type UnsignedPriority = runtime_common::AuctionManagerUnsignedPriority;
	type EmergencyShutdown = EmergencyShutdown;
	type WeightInfo = weights::module_auction_manager::WeightInfo<Runtime>;
}

impl module_loans::Config for Runtime {
	type Event = Event;
	type Currency = Currencies;
	type RiskManager = CdpEngine;
	type CDPTreasury = CdpTreasury;
	type PalletId = LoansPalletId;
	type OnUpdateLoan = module_incentives::OnUpdateLoan<Runtime>;
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
	) -> Option<(
		Call,
		<UncheckedExtrinsic as sp_runtime::traits::Extrinsic>::SignaturePayload,
	)> {
		// take the biggest period possible.
		let period = BlockHashCount::get()
			.checked_next_power_of_two()
			.map(|c| c / 2)
			.unwrap_or(2) as u64;
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

parameter_types! {
	pub DefaultLiquidationRatio: Ratio = Ratio::saturating_from_rational(110, 100);
	pub DefaultDebitExchangeRate: ExchangeRate = ExchangeRate::saturating_from_rational(1, 10);
	pub DefaultLiquidationPenalty: Rate = Rate::saturating_from_rational(5, 100);
	pub MinimumDebitValue: Balance = dollar(AUSD);
	pub MaxSwapSlippageCompareToOracle: Ratio = Ratio::saturating_from_rational(15, 100);
}

impl module_cdp_engine::Config for Runtime {
	type Event = Event;
	type PriceSource = module_prices::PriorityLockedPriceProvider<Runtime>;
	type DefaultLiquidationRatio = DefaultLiquidationRatio;
	type DefaultDebitExchangeRate = DefaultDebitExchangeRate;
	type DefaultLiquidationPenalty = DefaultLiquidationPenalty;
	type MinimumDebitValue = MinimumDebitValue;
	type MinimumCollateralAmount =
		ExistentialDepositsTimesOneHundred<GetNativeCurrencyId, NativeTokenExistentialDeposit, ExistentialDeposits>;
	type GetStableCurrencyId = GetStableCurrencyId;
	type CDPTreasury = CdpTreasury;
	type UpdateOrigin = EnsureRootOrHalfFinancialCouncil;
	type MaxSwapSlippageCompareToOracle = MaxSwapSlippageCompareToOracle;
	type UnsignedPriority = runtime_common::CdpEngineUnsignedPriority;
	type EmergencyShutdown = EmergencyShutdown;
	type UnixTime = Timestamp;
	type Currency = Currencies;
	type DEX = Dex;
	type Swap = SelendraSwap;
	type WeightInfo = weights::module_cdp_engine::WeightInfo<Runtime>;
}

parameter_types! {
	pub DepositPerAuthorization: Balance = dollar(ACA);
}

impl module_honzon::Config for Runtime {
	type Event = Event;
	type Currency = Balances;
	type DepositPerAuthorization = DepositPerAuthorization;
	type CollateralCurrencyIds = CollateralCurrencyIds<Runtime>;
	type WeightInfo = weights::module_honzon::WeightInfo<Runtime>;
}

impl module_emergency_shutdown::Config for Runtime {
	type Event = Event;
	type CollateralCurrencyIds = CollateralCurrencyIds<Runtime>;
	type PriceSource = Prices;
	type CDPTreasury = CdpTreasury;
	type AuctionManagerHandler = AuctionManager;
	type ShutdownOrigin = EnsureRootOrHalfCouncil;
	type WeightInfo = weights::module_emergency_shutdown::WeightInfo<Runtime>;
}

parameter_types! {
	pub const GetExchangeFee: (u32, u32) = (1, 1000);	// 0.1%
	pub const ExtendedProvisioningBlocks: BlockNumber = 2 * DAYS;
	pub const TradingPathLimit: u32 = 4;
	pub AlternativeSwapPathJointList: Vec<Vec<CurrencyId>> = vec![
		vec![ACA],
		vec![LACA],
	];
}

impl module_dex::Config for Runtime {
	type Event = Event;
	type Currency = Currencies;
	type GetExchangeFee = GetExchangeFee;
	type TradingPathLimit = TradingPathLimit;
	type PalletId = DEXPalletId;
	type Erc20InfoMapping = EvmErc20InfoMapping<Runtime>;
	type DEXIncentives = Incentives;
	type WeightInfo = weights::module_dex::WeightInfo<Runtime>;
	type ListingOrigin = EnsureRootOrHalfCouncil;
	type ExtendedProvisioningBlocks = ExtendedProvisioningBlocks;
	type OnLiquidityPoolUpdated = ();
}

impl module_aggregated_dex::Config for Runtime {
	type DEX = Dex;
	type StableAsset = RebasedStableAsset;
	type GovernanceOrigin = EnsureRootOrHalfCouncil;
	type DexSwapJointList = AlternativeSwapPathJointList;
	type SwapPathLimit = ConstU32<3>;
	type WeightInfo = ();
}

pub type RebasedStableAsset = module_support::RebasedStableAsset<
	StableAsset,
	ConvertBalanceSelendra,
	module_aggregated_dex::RebasedStableAssetErrorConvertor<Runtime>,
>;

pub type SelendraSwap = module_aggregated_dex::AggregatedSwap<Runtime>;

impl module_dex_oracle::Config for Runtime {
	type DEX = Dex;
	type Time = Timestamp;
	type UpdateOrigin = EnsureRootOrHalfCouncil;
	type WeightInfo = weights::module_dex_oracle::WeightInfo<Runtime>;
}

parameter_types! {
	pub HonzonTreasuryAccount: AccountId = HonzonTreasuryPalletId::get().into_account_truncating();
}

impl module_cdp_treasury::Config for Runtime {
	type Event = Event;
	type Currency = Currencies;
	type GetStableCurrencyId = GetStableCurrencyId;
	type AuctionManagerHandler = AuctionManager;
	type UpdateOrigin = EnsureRootOrHalfFinancialCouncil;
	type DEX = Dex;
	type Swap = SelendraSwap;
	type MaxAuctionsCount = ConstU32<50>;
	type PalletId = CDPTreasuryPalletId;
	type TreasuryAccount = HonzonTreasuryAccount;
	type WeightInfo = weights::module_cdp_treasury::WeightInfo<Runtime>;
	type StableAsset = RebasedStableAsset;
}

impl module_transaction_pause::Config for Runtime {
	type Event = Event;
	type UpdateOrigin = EnsureRootOrThreeFourthsCouncil;
	type WeightInfo = weights::module_transaction_pause::WeightInfo<Runtime>;
}

parameter_types! {
	pub DefaultFeeTokens: Vec<CurrencyId> = vec![AUSD, DOT, LACA];
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
	type FeeMultiplierUpdate = TargetedFeeAdjustment<Self, TargetBlockFullness, AdjustmentVariable, MinimumMultiplier>;
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
	pub const GetLiquidCurrencyId: CurrencyId = LACA;
	pub const GetStakingCurrencyId: CurrencyId = DOT;
}

parameter_types! {
	pub CreateClassDeposit: Balance = 20 * dollar(ACA);
	pub CreateTokenDeposit: Balance = 2 * dollar(ACA);
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

parameter_types! {
	// One storage item; key size 32, value size 8; .
	pub ProxyDepositBase: Balance = deposit(1, 8);
	// Additional storage item size of 33 bytes.
	pub ProxyDepositFactor: Balance = deposit(0, 33);
	pub AnnouncementDepositBase: Balance = deposit(1, 8);
	pub AnnouncementDepositFactor: Balance = deposit(0, 66);
}

impl InstanceFilter<Call> for ProxyType {
	fn filter(&self, c: &Call) -> bool {
		match self {
			// Always allowed Call::Utility no matter type.
			// Only transactions allowed by Proxy.filter can be executed,
			// otherwise `BadOrigin` will be returned in Call::Utility.
			_ if matches!(c, Call::Utility(..)) => true,
			ProxyType::Any => true,
			ProxyType::CancelProxy => matches!(c, Call::Proxy(pallet_proxy::Call::reject_announcement { .. })),
			ProxyType::Governance => {
				matches!(
					c,
					Call::Authority(..)
						| Call::Democracy(..) | Call::PhragmenElection(..)
						| Call::Council(..)
						| Call::FinancialCouncil(..)
						| Call::TechnicalCommittee(..)
						| Call::Treasury(..) | Call::Bounties(..)
						| Call::Tips(..)
				)
			}
			ProxyType::Auction => {
				matches!(c, Call::Auction(orml_auction::Call::bid { .. }))
			}
			ProxyType::Swap => {
				matches!(
					c,
					Call::Dex(module_dex::Call::swap_with_exact_supply { .. })
						| Call::Dex(module_dex::Call::swap_with_exact_target { .. })
				)
			}
			ProxyType::Loan => {
				matches!(
					c,
					Call::Honzon(module_honzon::Call::adjust_loan { .. })
						| Call::Honzon(module_honzon::Call::close_loan_has_debit_by_dex { .. })
						| Call::Honzon(module_honzon::Call::adjust_loan_by_debit_value { .. })
						| Call::Honzon(module_honzon::Call::transfer_debit { .. })
				)
			}
			ProxyType::DexLiquidity => {
				matches!(
					c,
					Call::Dex(module_dex::Call::add_liquidity { .. })
						| Call::Dex(module_dex::Call::remove_liquidity { .. })
				)
			}
			ProxyType::StableAssetSwap => {
				matches!(c, Call::StableAsset(module_stable_asset::Call::swap { .. }))
			}
			ProxyType::StableAssetLiquidity => {
				matches!(
					c,
					Call::StableAsset(module_stable_asset::Call::mint { .. })
						| Call::StableAsset(module_stable_asset::Call::redeem_proportion { .. })
						| Call::StableAsset(module_stable_asset::Call::redeem_single { .. })
						| Call::StableAsset(module_stable_asset::Call::redeem_multi { .. })
				)
			}
		}
	}
	fn is_superset(&self, o: &Self) -> bool {
		match (self, o) {
			(x, y) if x == y => true,
			(ProxyType::Any, _) => true,
			(_, ProxyType::Any) => false,
			_ => false,
		}
	}
}

impl pallet_proxy::Config for Runtime {
	type Event = Event;
	type Call = Call;
	type Currency = Balances;
	type ProxyType = ProxyType;
	type ProxyDepositBase = ProxyDepositBase;
	type ProxyDepositFactor = ProxyDepositFactor;
	type MaxProxies = ConstU32<32>;
	type WeightInfo = ();
	type MaxPending = ConstU32<32>;
	type CallHasher = BlakeTwo256;
	type AnnouncementDepositBase = AnnouncementDepositBase;
	type AnnouncementDepositFactor = AnnouncementDepositFactor;
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
			LACA => ExchangeRate::saturating_from_rational(1, 10)
				.checked_mul_int(balance)
				.unwrap_or(Bounded::max_value()),
			_ => balance,
		}
	}

	fn convert_balance_back(balance: Balance, asset_id: CurrencyId) -> Balance {
		match asset_id {
			LACA => ExchangeRate::saturating_from_rational(10, 1)
				.checked_mul_int(balance)
				.unwrap_or(Bounded::max_value()),
			_ => balance,
		}
	}
}

pub struct IsLiquidToken;
impl Contains<CurrencyId> for IsLiquidToken {
	fn contains(currency_id: &CurrencyId) -> bool {
		matches!(currency_id, CurrencyId::Token(TokenSymbol::LACA))
	}
}

type RebaseTokens = orml_tokens::Combiner<
	AccountId,
	IsLiquidToken,
	orml_tokens::Mapper<AccountId, Currencies, ConvertBalanceSelendra, Balance, GetLiquidCurrencyId>,
	Currencies,
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

impl Convert<(Call, SignedExtra), Result<(EthereumTransactionMessage, SignedExtra), InvalidTransaction>>
	for ConvertEthereumTx
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
					return Err(InvalidTransaction::Stale);
				}

				let (_, _, _, _, mortality, check_nonce, _, charge, ..) = extra.clone();

				if mortality != frame_system::CheckEra::from(sp_runtime::generic::Era::Immortal) {
					// require immortal
					return Err(InvalidTransaction::BadProof);
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
			}
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
			let raw_payload = SignedPayload::new(*call, extra).map_err(|_| InvalidTransaction::BadSigner)?;
			if !raw_payload.using_encoded(|payload| payer_sig.verify(payload, &payer_account.into())) {
				return Err(InvalidTransaction::BadProof);
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
pub type Executive =
	frame_executive::Executive<Runtime, Block, frame_system::ChainContext<Runtime>, Runtime, AllPalletsWithSystem, ()>;

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

		// Governance
		Council: pallet_collective::<Instance1> = 50,
		CouncilMembership: pallet_membership::<Instance1> = 51,
		FinancialCouncil: pallet_collective::<Instance2> = 52,
		FinancialCouncilMembership: pallet_membership::<Instance2> = 53,
		TechnicalCommittee: pallet_collective::<Instance4> = 56,
		TechnicalCommitteeMembership: pallet_membership::<Instance4> = 57,

		// norminator
		VoterList: pallet_bags_list = 60,
		NominationPools: pallet_nomination_pools = 61,

		Authority: orml_authority = 70,
		PhragmenElection: pallet_elections_phragmen = 71,
		Democracy: pallet_democracy = 72,

		// Oracle
		//
		// NOTE: OperatorMembership must be placed after Oracle or else will have race condition on initialization
		SelendraOracle: orml_oracle::<Instance1> = 80,
		OperatorMembershipSelendra: pallet_membership::<Instance5> = 82,

		// ORML Core
		Auction: orml_auction = 100,
		Rewards: orml_rewards = 101,
		OrmlNFT: orml_nft exclude_parts { Call } = 102,

		// Selendra Core
		Prices: module_prices = 110,
		Dex: module_dex = 111,
		DexOracle: module_dex_oracle = 112,
		AggregatedDex: module_aggregated_dex = 113,

		// Honzon
		AuctionManager: module_auction_manager = 120,
		Loans: module_loans = 121,
		Honzon: module_honzon = 122,
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
extern crate orml_benchmarking;

#[cfg(feature = "runtime-benchmarks")]
mod benches {
	define_benchmarks!(
		[module_dex, benchmarking::dex]
		[module_dex_oracle, benchmarking::dex_oracle]
		[module_asset_registry, benchmarking::asset_registry]
		[module_auction_manager, benchmarking::auction_manager]
		[module_cdp_engine, benchmarking::cdp_engine]
		[module_earning, benchmarking::earning]
		[module_emergency_shutdown, benchmarking::emergency_shutdown]
		[module_evm, benchmarking::evm]
		[module_honzon, benchmarking::honzon]
		[module_cdp_treasury, benchmarking::cdp_treasury]
		[module_transaction_pause, benchmarking::transaction_pause]
		[module_transaction_payment, benchmarking::transaction_payment]
		[module_incentives, benchmarking::incentives]
		[module_prices, benchmarking::prices]
		[module_evm_accounts, benchmarking::evm_accounts]
		[module_currencies, benchmarking::currencies]
		[orml_tokens, benchmarking::tokens]
		[orml_auction, benchmarking::auction]
		[orml_authority, benchmarking::authority]
		[orml_oracle, benchmarking::oracle]
		[module_stable_asset, benchmarking::module_stable_asset]
		[module_idle_scheduler, benchmarking::idle_scheduler]
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

	// benchmarks for selendra modules
	#[cfg(feature = "runtime-benchmarks")]
	impl frame_benchmarking::Benchmark<Block> for Runtime {
		fn benchmark_metadata(extra: bool) -> (
			Vec<frame_benchmarking::BenchmarkList>,
			Vec<frame_support::traits::StorageInfo>,
		) {
			use frame_benchmarking::{list_benchmark as frame_list_benchmark, Benchmarking, BenchmarkList};
			use frame_support::traits::StorageInfoTrait;

			use module_nft::benchmarking::Pallet as NftBench;

			let mut list = Vec::<BenchmarkList>::new();

			frame_list_benchmark!(list, extra, module_nft, NftBench::<Runtime>);
			list_benchmarks!(list, extra);

			let storage_info = AllPalletsWithSystem::storage_info();

			return (list, storage_info)
		}

		fn dispatch_benchmark(
			config: frame_benchmarking::BenchmarkConfig
		) -> Result<Vec<frame_benchmarking::BenchmarkBatch>, sp_runtime::RuntimeString> {
			use frame_benchmarking::{Benchmarking, BenchmarkBatch, add_benchmark as frame_add_benchmark, TrackedStorageKey};
			use module_nft::benchmarking::Pallet as NftBench;

			let whitelist: Vec<TrackedStorageKey> = vec![
				// Block Number
				// frame_system::Number::<Runtime>::hashed_key().to_vec(),
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef702a5c1b19ab7a04f536c519aca4983ac").to_vec().into(),
				// Total Issuance
				hex_literal::hex!("c2261276cc9d1f8598ea4b6a74b15c2f57c875e4cff74148e4628f264b974c80").to_vec().into(),
				// Execution Phase
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef7ff553b5a9862a516939d82b3d3d8661a").to_vec().into(),
				// Event Count
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef70a98fdbe9ce6c55837576c60c7af3850").to_vec().into(),
				// System Events
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef780d41e5e16056765bc8461851072c9d7").to_vec().into(),
				// Caller 0 Account
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef7b99d880ec681799c0cf30e8886371da946c154ffd9992e395af90b5b13cc6f295c77033fce8a9045824a6690bbf99c6db269502f0a8d1d2a008542d5690a0749").to_vec().into(),
				// Treasury Account
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef7b99d880ec681799c0cf30e8886371da95ecffd7b6c0f78751baa9d281e0bfa3a6d6f646c70792f74727372790000000000000000000000000000000000000000").to_vec().into(),
			];
			let mut batches = Vec::<BenchmarkBatch>::new();
			let params = (&config, &whitelist);

			frame_add_benchmark!(params, batches, module_nft, NftBench::<Runtime>);
			add_benchmarks!(params, batches);

			if batches.is_empty() { return Err("Benchmark not found for this module.".into()) }
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
				<StorageDepositPerByte as frame_support::traits::Get<Balance>>::get() / 10u128.saturating_pow(6)
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
				module_evm::AccountInfo {
					nonce: 3,
					contract_info: None,
				},
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
		let t = frame_system::GenesisConfig::default()
			.build_storage::<Runtime>()
			.unwrap();
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

			let fee_call = Call::TransactionPayment(module_transaction_payment::Call::with_fee_paid_by {
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
				dest: sp_runtime::MultiAddress::Id(sp_keyring::AccountKeyring::Dave.to_account_id()),
				value: 100,
			});

			let raw_payload = SignedPayload::new(hacker_call.clone(), extra.clone()).unwrap();
			let payer_signature = raw_payload.using_encoded(|payload| hacker.pair().sign(payload));

			let fee_call = Call::TransactionPayment(module_transaction_payment::Call::with_fee_paid_by {
				call: Box::new(call),
				payer_addr: hacker.to_account_id(),
				payer_sig: sp_runtime::MultiSignature::Sr25519(payer_signature),
			});
			assert!(PayerSignatureVerification::convert((fee_call, extra)).is_err());
		});
	}
}
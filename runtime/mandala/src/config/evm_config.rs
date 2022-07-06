
use crate::{
	dollar, parameter_types, weights, AllPrecompiles, Babe, Balances, Currencies, DispatchableTask,
	EnsureRootOrHalfCouncil, EnsureRootOrTwoThirdsTechnicalCommittee, Event, GasToWeight,
	IdleScheduler, Runtime, RuntimeBlockWeights, RuntimeDebug, TreasuryAccount, Weight,
	EVM, H160, ACA,
};

use codec::{Decode, Encode};
use scale_info::TypeInfo;

use module_evm::{EvmChainId, EvmTask};
use module_evm_accounts::EvmAddressMapping;

use primitives::{define_combined_task, task::TaskResult, Balance};

impl module_evm_accounts::Config for Runtime {
	type Event = Event;
	type Currency = Balances;
	type AddressMapping = EvmAddressMapping<Runtime>;
	type TransferAll = Currencies;
	type ChainId = EvmChainId<Runtime>;
	type WeightInfo = weights::module_evm_accounts::WeightInfo<Runtime>;
}


parameter_types! {
	pub NetworkContractSource: H160 = H160::from_low_u64_be(0);
	pub PrecompilesValue: AllPrecompiles<Runtime> = AllPrecompiles::<_>::mandala();
}

#[cfg(feature = "with-ethereum-compatibility")]
parameter_types! {
	pub const NewContractExtraBytes: u32 = 0;
	pub const DeveloperDeposit: Balance = 0;
	pub const PublicationFee: Balance = 0;
}

#[cfg(not(feature = "with-ethereum-compatibility"))]
parameter_types! {
	pub const NewContractExtraBytes: u32 = 10_000;
	pub DeveloperDeposit: Balance = dollar(ACA);
	pub PublicationFee: Balance = dollar(ACA);
}

#[derive(Clone, Encode, Decode, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct StorageDepositPerByte;
impl<I: From<Balance>> frame_support::traits::Get<I> for StorageDepositPerByte {
	fn get() -> I {
		#[cfg(not(feature = "with-ethereum-compatibility"))]
		// NOTE: ACA decimals is 12, convert to 18.
		// 10 * millicent(ACA) * 10^6
		return I::from(100_000_000_000_000);
		#[cfg(feature = "with-ethereum-compatibility")]
		return I::from(0);
	}
}

#[derive(Clone, Encode, Decode, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct TxFeePerGas;
impl<I: From<Balance>> frame_support::traits::Get<I> for TxFeePerGas {
	fn get() -> I {
		// NOTE: 200 GWei
		// ensure suffix is 0x0000
		I::from(200u128.saturating_mul(10u128.saturating_pow(9)) & !0xffff)
	}
}

#[cfg(feature = "with-ethereum-compatibility")]
static LONDON_CONFIG: module_evm_utility::evm::Config = module_evm_utility::evm::Config::london();

impl module_evm::Config for Runtime {
	type AddressMapping = EvmAddressMapping<Runtime>;
	type Currency = Balances;
	type TransferAll = Currencies;
	type NewContractExtraBytes = NewContractExtraBytes;
	type StorageDepositPerByte = StorageDepositPerByte;
	type TxFeePerGas = TxFeePerGas;
	type Event = Event;
	type PrecompilesType = AllPrecompiles<Self>;
	type PrecompilesValue = PrecompilesValue;
	type GasToWeight = GasToWeight;
	type ChargeTransactionPayment = module_transaction_payment::ChargeTransactionPayment<Runtime>;
	type NetworkContractOrigin = EnsureRootOrTwoThirdsTechnicalCommittee;
	type NetworkContractSource = NetworkContractSource;
	type DeveloperDeposit = DeveloperDeposit;
	type PublicationFee = PublicationFee;
	type TreasuryAccount = TreasuryAccount;
	type FreePublicationOrigin = EnsureRootOrHalfCouncil;
	type Runner = module_evm::runner::stack::Runner<Self>;
	type FindAuthor = pallet_session::FindAccountFromAuthorIndex<Self, Babe>;
	type Task = ScheduledTasks;
	type IdleScheduler = IdleScheduler;
	type WeightInfo = weights::module_evm::WeightInfo<Runtime>;

	#[cfg(feature = "with-ethereum-compatibility")]
	fn config() -> &'static module_evm_utility::evm::Config {
		&LONDON_CONFIG
	}
}

impl module_evm_bridge::Config for Runtime {
	type EVM = EVM;
}

define_combined_task! {
	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
	pub enum ScheduledTasks {
		EvmTask(EvmTask<Runtime>),
	}
}

parameter_types!(
	// At least 2% of max block weight should remain before idle tasks are dispatched.
	pub MinimumWeightRemainInBlock: Weight = RuntimeBlockWeights::get().max_block / 50;
);

impl module_idle_scheduler::Config for Runtime {
	type Event = Event;
	type WeightInfo = ();
	type Task = ScheduledTasks;
	type MinimumWeightRemainInBlock = MinimumWeightRemainInBlock;
}


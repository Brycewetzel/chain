/*
 * This file is part of the Nodle Chain distributed at https://github.com/NodleCode/chain
 * Copyright (C) 2020  Nodle International
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 */

#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

use cumulus_primitives_core::relay_chain::Balance as RelayChainBalance;
use frame_support::{
    construct_runtime, parameter_types,
    traits::{Get, Randomness},
    weights::{
        constants::{BlockExecutionWeight, ExtrinsicBaseWeight, RocksDbWeight, WEIGHT_PER_SECOND},
        DispatchClass, IdentityFee, Weight,
    },
};
use frame_system::{
    limits::{BlockLength, BlockWeights},
    EnsureRoot,
};
use nodle_chain_primitives::{
    AccountId, AccountIndex, Amount, Balance, BlockNumber, CertificateId, CurrencyId, Hash, Index,
    Moment, Signature,
};
use orml_currencies::BasicCurrencyAdapter;
use orml_traits::parameter_type_with_key;
use orml_xcm_support::{
    CurrencyIdConverter, IsConcreteWithGeneralKey, MultiCurrencyAdapter, XcmHandler as XcmHandlerT,
};
use pallet_transaction_payment::{FeeDetails, Multiplier, TargetedFeeAdjustment};
use pallet_transaction_payment_rpc_runtime_api::RuntimeDispatchInfo;
use polkadot_parachain::primitives::Sibling;
use sp_core::{
    u32_trait::{_1, _2},
    OpaqueMetadata,
};
#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;
use sp_runtime::{
    create_runtime_str, generic, impl_opaque_keys,
    traits::{AccountIdConversion, BlakeTwo256, Block as BlockT, Convert, StaticLookup, Zero},
    transaction_validity::{TransactionSource, TransactionValidity},
    ApplyExtrinsicResult, DispatchResult, FixedPointNumber, ModuleId, Perbill, Perquintill,
};
use sp_std::prelude::*;
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;
use static_assertions::const_assert;
use xcm::v0::{Junction, MultiLocation, NetworkId, Xcm};
use xcm_builder::{
    AccountId32Aliases, LocationInverter, ParentIsDefault, RelayChainAsNative,
    SiblingParachainAsNative, SiblingParachainConvertsVia, SignedAccountId32AsNative,
    SovereignSignedViaLocation,
};
use xcm_executor::{traits::NativeAsset, Config, XcmExecutor};

pub mod constants;
mod implementations;

use implementations::{DealWithFees, ProxyType};

impl_opaque_keys! {
    pub struct SessionKeys {}
}

/// This runtime version.
/// This should not be thought of as classic Semver (major/minor/tiny).
/// This triplet have different semantics and mis-interpretation could cause problems.
/// In particular: bug fixes should result in an increment of `spec_version` and possibly `authoring_version`,
/// absolutely not `impl_version` since they change the semantics of the runtime.
pub const VERSION: RuntimeVersion = RuntimeVersion {
    spec_name: create_runtime_str!("nodle-chain"),
    impl_name: create_runtime_str!("nodle-chain"),

    /// `authoring_version` is the version of the authorship interface. An authoring node
    /// will not attempt to author blocks unless this is equal to its native runtime.
    authoring_version: 1,

    /// Version of the runtime specification. A full-node will not attempt to use its native
    /// runtime in substitute for the on-chain Wasm runtime unless all of `spec_name`,
    /// `spec_version` and `authoring_version` are the same between Wasm and native.
    spec_version: 44,

    /// Version of the implementation of the specification. Nodes are free to ignore this; it
    /// serves only as an indication that the code is different; as long as the other two versions
    /// are the same then while the actual code may be different, it is nonetheless required to
    /// do the same thing.
    /// Non-consensus-breaking optimizations are about the only changes that could be made which
    /// would result in only the `impl_version` changing.
    impl_version: 0,

    /// Used for hardware wallets. This typically happens when `SignedExtra` changes.
    transaction_version: 3,

    apis: RUNTIME_API_VERSIONS,
};

/// The version infromation used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
    NativeVersion {
        runtime_version: VERSION,
        can_author_with: Default::default(),
    }
}

/// We assume that ~10% of the block weight is consumed by `on_initalize` handlers.
/// This is used to limit the maximal weight of a single extrinsic.
const AVERAGE_ON_INITIALIZE_RATIO: Perbill = Perbill::from_percent(10);
/// We allow `Normal` extrinsics to fill up the block up to 75%, the rest can be used
/// by  Operational  extrinsics.
const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);
/// We allow for 2 seconds of compute with a 6 second average block time.
const MAXIMUM_BLOCK_WEIGHT: Weight = 2 * WEIGHT_PER_SECOND;

parameter_types! {
    pub const BlockHashCount: BlockNumber = 2400;
    pub const Version: RuntimeVersion = VERSION;
    pub RuntimeBlockLength: BlockLength =
        BlockLength::max_with_normal_ratio(5 * 1024 * 1024, NORMAL_DISPATCH_RATIO);
    pub RuntimeBlockWeights: BlockWeights = BlockWeights::builder()
        .base_block(BlockExecutionWeight::get())
        .for_class(DispatchClass::all(), |weights| {
            weights.base_extrinsic = ExtrinsicBaseWeight::get();
        })
        .for_class(DispatchClass::Normal, |weights| {
            weights.max_total = Some(NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT);
        })
        .for_class(DispatchClass::Operational, |weights| {
            weights.max_total = Some(MAXIMUM_BLOCK_WEIGHT);
            // Operational transactions have some extra reserved space, so that they
            // are included even if block reached `MAXIMUM_BLOCK_WEIGHT`.
            weights.reserved = Some(
                MAXIMUM_BLOCK_WEIGHT - NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT
            );
        })
        .avg_block_initialization(AVERAGE_ON_INITIALIZE_RATIO)
        .build_or_panic();
    // https://github.com/paritytech/substrate/blob/74a50abd6cbaad1253daf3585d5cdaa4592e9184/primitives/core/src/crypto.rs#L517
    pub const SS58Prefix: u8 = 37;
}

const_assert!(NORMAL_DISPATCH_RATIO.deconstruct() >= AVERAGE_ON_INITIALIZE_RATIO.deconstruct());

impl frame_system::Config for Runtime {
    type BaseCallFilter = ();
    type BlockWeights = RuntimeBlockWeights;
    type BlockLength = RuntimeBlockLength;
    type DbWeight = RocksDbWeight;
    type Origin = Origin;
    type Call = Call;
    type Index = Index;
    type BlockNumber = BlockNumber;
    type Hash = Hash;
    type Hashing = BlakeTwo256;
    type AccountId = AccountId;
    type Lookup = Indices;
    type Header = generic::Header<BlockNumber, BlakeTwo256>;
    type Event = Event;
    type BlockHashCount = BlockHashCount;
    type Version = Version;
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<Balance>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = frame_system::weights::SubstrateWeight<Runtime>;
    type SS58Prefix = SS58Prefix;
}

parameter_types! {
    pub const IndexDeposit: Balance = 1 * constants::DOLLARS;
}

impl pallet_indices::Config for Runtime {
    type AccountIndex = AccountIndex;
    type Currency = Balances;
    type Deposit = IndexDeposit;
    type Event = Event;
    type WeightInfo = ();
}

parameter_types! {
    pub const ExistentialDeposit: Balance = 1 * constants::MILLICENTS;
    // For weight estimation, we assume that the most locks on an individual account will be 50.
    // This number may need to be adjusted in the future if this assumption no longer holds true.
    pub const MaxLocks: u32 = 50;
}

impl pallet_balances::Config for Runtime {
    type Balance = Balance;
    type Event = Event;
    type DustRemoval = CompanyReserve;
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type MaxLocks = MaxLocks;
    type WeightInfo = ();
}

parameter_types! {
    pub const TransactionByteFee: Balance = 10 * constants::MILLICENTS;
    // For a sane configuration, this should always be less than `AvailableBlockRatio`.
    // Fees raises after a fullness of 25%
    pub const TargetBlockFullness: Perquintill = constants::TARGET_BLOCK_FULLNESS;
    pub AdjustmentVariable: Multiplier = Multiplier::saturating_from_rational(1, 100_000);
    pub MinimumMultiplier: Multiplier = Multiplier::saturating_from_rational(1, 1_000_000_000u128);
}

impl pallet_transaction_payment::Config for Runtime {
    type OnChargeTransaction = pallet_transaction_payment::CurrencyAdapter<Balances, DealWithFees>;
    type TransactionByteFee = TransactionByteFee;
    type WeightToFee = IdentityFee<Balance>;
    type FeeMultiplierUpdate =
        TargetedFeeAdjustment<Self, TargetBlockFullness, AdjustmentVariable, MinimumMultiplier>;
}

impl pallet_grants::Config for Runtime {
    type Event = Event;
    type Currency = Balances;
    type CancelOrigin =
        pallet_collective::EnsureProportionMoreThan<_1, _2, AccountId, FinancialCollective>;
}

parameter_types! {
    pub const MinimumPeriod: u64 = constants::SLOT_DURATION / 2;
}

impl pallet_timestamp::Config for Runtime {
    type Moment = Moment;
    type OnTimestampSet = ();
    type MinimumPeriod = MinimumPeriod;
    type WeightInfo = ();
}

// Shared parameters with all collectives / committees
parameter_types! {
    pub const MotionDuration: BlockNumber = 2 * constants::DAYS;
    pub const MaxProposals: u32 = 100;
    pub const MaxMembers: u32 = 50;
}

// --- Technical committee

impl pallet_membership::Config<pallet_membership::Instance1> for Runtime {
    type Event = Event;
    type AddOrigin = pallet_collective::EnsureProportionMoreThan<_1, _2, AccountId, RootCollective>;
    type RemoveOrigin =
        pallet_collective::EnsureProportionMoreThan<_1, _2, AccountId, RootCollective>;
    type SwapOrigin =
        pallet_collective::EnsureProportionMoreThan<_1, _2, AccountId, RootCollective>;
    type ResetOrigin =
        pallet_collective::EnsureProportionMoreThan<_1, _2, AccountId, RootCollective>;
    type PrimeOrigin =
        pallet_collective::EnsureProportionMoreThan<_1, _2, AccountId, RootCollective>;
    type MembershipInitialized = TechnicalCommittee;
    type MembershipChanged = TechnicalCommittee;
}

type TechnicalCollective = pallet_collective::Instance2;
impl pallet_collective::Config<TechnicalCollective> for Runtime {
    type Origin = Origin;
    type Proposal = Call;
    type Event = Event;
    type MotionDuration = MotionDuration;
    type MaxProposals = MaxProposals;
    type WeightInfo = ();
    type MaxMembers = MaxMembers;
    type DefaultVote = pallet_collective::PrimeDefaultVote;
}

// --- Financial committee

impl pallet_membership::Config<pallet_membership::Instance3> for Runtime {
    type Event = Event;
    type AddOrigin = pallet_collective::EnsureProportionMoreThan<_1, _2, AccountId, RootCollective>;
    type RemoveOrigin =
        pallet_collective::EnsureProportionMoreThan<_1, _2, AccountId, RootCollective>;
    type SwapOrigin =
        pallet_collective::EnsureProportionMoreThan<_1, _2, AccountId, RootCollective>;
    type ResetOrigin =
        pallet_collective::EnsureProportionMoreThan<_1, _2, AccountId, RootCollective>;
    type PrimeOrigin =
        pallet_collective::EnsureProportionMoreThan<_1, _2, AccountId, RootCollective>;
    type MembershipInitialized = FinancialCommittee;
    type MembershipChanged = FinancialCommittee;
}

type FinancialCollective = pallet_collective::Instance3;
impl pallet_collective::Config<FinancialCollective> for Runtime {
    type Origin = Origin;
    type Proposal = Call;
    type Event = Event;
    type MotionDuration = MotionDuration;
    type MaxProposals = MaxProposals;
    type WeightInfo = ();
    type MaxMembers = MaxMembers;
    type DefaultVote = pallet_collective::PrimeDefaultVote;
}

// --- Root committee

impl pallet_membership::Config<pallet_membership::Instance4> for Runtime {
    type Event = Event;
    type AddOrigin = pallet_collective::EnsureProportionMoreThan<_1, _2, AccountId, RootCollective>;
    type RemoveOrigin =
        pallet_collective::EnsureProportionMoreThan<_1, _2, AccountId, RootCollective>;
    type SwapOrigin =
        pallet_collective::EnsureProportionMoreThan<_1, _2, AccountId, RootCollective>;
    type ResetOrigin =
        pallet_collective::EnsureProportionMoreThan<_1, _2, AccountId, RootCollective>;
    type PrimeOrigin =
        pallet_collective::EnsureProportionMoreThan<_1, _2, AccountId, RootCollective>;
    type MembershipInitialized = RootCommittee;
    type MembershipChanged = RootCommittee;
}

type RootCollective = pallet_collective::Instance4;
impl pallet_collective::Config<RootCollective> for Runtime {
    type Origin = Origin;
    type Proposal = Call;
    type Event = Event;
    type MotionDuration = MotionDuration;
    type MaxProposals = MaxProposals;
    type WeightInfo = ();
    type MaxMembers = MaxMembers;
    type DefaultVote = pallet_collective::PrimeDefaultVote;
}

impl pallet_mandate::Config for Runtime {
    type Event = Event;
    type Call = Call;
    type ExternalOrigin =
        pallet_collective::EnsureProportionAtLeast<_1, _2, AccountId, RootCollective>;
}

parameter_types! {
    pub MaximumSchedulerWeight: Weight = Perbill::from_percent(80) *
        RuntimeBlockWeights::get().max_block;
    pub const MaxScheduledPerBlock: u32 = 50;
}

impl pallet_scheduler::Config for Runtime {
    type Event = Event;
    type Origin = Origin;
    type PalletsOrigin = OriginCaller;
    type Call = Call;
    type MaximumWeight = MaximumSchedulerWeight;
    type ScheduleOrigin = frame_system::EnsureRoot<AccountId>;
    type MaxScheduledPerBlock = MaxScheduledPerBlock;
    type WeightInfo = ();
}

parameter_types! {
    pub const AmendmentDelay: BlockNumber = 2 * constants::DAYS;
}

impl pallet_amendments::Config for Runtime {
    type Event = Event;
    type Amendment = Call;
    type Scheduler = Scheduler;
    type SubmissionOrigin =
        pallet_collective::EnsureProportionMoreThan<_1, _2, AccountId, TechnicalCollective>;
    type VetoOrigin =
        pallet_collective::EnsureProportionMoreThan<_1, _2, AccountId, RootCollective>;
    type Delay = AmendmentDelay;
    type PalletsOrigin = OriginCaller;
}

parameter_types! {
    pub const CompanyReserveModuleId: ModuleId = ModuleId(*b"py/resrv"); // 5EYCAe5ijiYfha9GzQDgPVtUCYDY9B8ZgcyiANL2L34crMoR
    pub CompanyReserveAccount: AccountId = CompanyReserveModuleId::get().into_account();
}

impl pallet_reserve::Config<pallet_reserve::Instance1> for Runtime {
    type Event = Event;
    type Currency = pallet_balances::Pallet<Runtime>;
    type ExternalOrigin =
        pallet_collective::EnsureProportionMoreThan<_1, _2, AccountId, FinancialCollective>;
    type Call = Call;
    type ModuleId = CompanyReserveModuleId;
}

parameter_types! {
    pub const InternationalReserveModuleId: ModuleId = ModuleId(*b"py/rvint"); // 5EYCAe5ijiYfi6GQAEPSHYDwvw4CkyGtPTS52BjLh42GygSv
}

impl pallet_reserve::Config<pallet_reserve::Instance2> for Runtime {
    type Event = Event;
    type Currency = pallet_balances::Pallet<Runtime>;
    type ExternalOrigin =
        pallet_collective::EnsureProportionMoreThan<_1, _2, AccountId, FinancialCollective>;
    type Call = Call;
    type ModuleId = InternationalReserveModuleId;
}

parameter_types! {
    pub const UsaReserveModuleId: ModuleId = ModuleId(*b"py/rvusa"); // 5EYCAe5ijiYfi6MEfWpZC3nJ38KFZ9EQSFpsj9mgYgTtVNri
}

impl pallet_reserve::Config<pallet_reserve::Instance3> for Runtime {
    type Event = Event;
    type Currency = pallet_balances::Pallet<Runtime>;
    type ExternalOrigin =
        pallet_collective::EnsureProportionMoreThan<_1, _2, AccountId, FinancialCollective>;
    type Call = Call;
    type ModuleId = UsaReserveModuleId;
}

parameter_types! {
    pub const BasicDeposit: Balance = 10 * constants::DOLLARS;       // 258 bytes on-chain
    pub const FieldDeposit: Balance = 250 * constants::CENTS;        // 66 bytes on-chain
    pub const SubAccountDeposit: Balance = 2 * constants::DOLLARS;   // 53 bytes on-chain
    pub const MaxSubAccounts: u32 = 100;
    pub const MaxAdditionalFields: u32 = 100;
    pub const MaxRegistrars: u32 = 20;
}

impl pallet_identity::Config for Runtime {
    type Event = Event;
    type Currency = Balances;
    type BasicDeposit = BasicDeposit;
    type FieldDeposit = FieldDeposit;
    type SubAccountDeposit = SubAccountDeposit;
    type MaxSubAccounts = MaxSubAccounts;
    type MaxAdditionalFields = MaxAdditionalFields;
    type Slashed = CompanyReserve;
    type ForceOrigin =
        pallet_collective::EnsureProportionMoreThan<_1, _2, AccountId, TechnicalCollective>;
    type RegistrarOrigin =
        pallet_collective::EnsureProportionMoreThan<_1, _2, AccountId, TechnicalCollective>;
    type MaxRegistrars = MaxRegistrars;
    type WeightInfo = ();
}

parameter_types! {
    pub const ConfigDepositBase: Balance = 5 * constants::DOLLARS;
    pub const FriendDepositFactor: Balance = 50 * constants::CENTS;
    pub const MaxFriends: u16 = 9;
    pub const RecoveryDeposit: Balance = 5 * constants::DOLLARS;
}

impl pallet_recovery::Config for Runtime {
    type Event = Event;
    type Call = Call;
    type Currency = Balances;
    type ConfigDepositBase = ConfigDepositBase;
    type FriendDepositFactor = FriendDepositFactor;
    type MaxFriends = MaxFriends;
    type RecoveryDeposit = RecoveryDeposit;
}

impl pallet_utility::Config for Runtime {
    type Event = Event;
    type Call = Call;
    type WeightInfo = ();
}

parameter_types! {
    // One storage item; key size 32, value size 8; .
    pub const ProxyDepositBase: Balance = constants::deposit(1, 8);
    // Additional storage item size of 33 bytes.
    pub const ProxyDepositFactor: Balance = constants::deposit(0, 33);
    pub const MaxProxies: u16 = 32;
    pub const AnnouncementDepositBase: Balance = constants::deposit(1, 8);
    pub const AnnouncementDepositFactor: Balance = constants::deposit(0, 66);
    pub const MaxPending: u16 = 32;
}

impl pallet_proxy::Config for Runtime {
    type Event = Event;
    type Call = Call;
    type Currency = Balances;
    type ProxyType = ProxyType;
    type ProxyDepositBase = ProxyDepositBase;
    type ProxyDepositFactor = ProxyDepositFactor;
    type MaxProxies = MaxProxies;
    type WeightInfo = ();
    type MaxPending = MaxPending;
    type CallHasher = BlakeTwo256;
    type AnnouncementDepositBase = AnnouncementDepositBase;
    type AnnouncementDepositFactor = AnnouncementDepositFactor;
}

parameter_types! {
    // One storage item; key size is 32; value is size 4+4+16+32 bytes = 56 bytes.
    pub const DepositBase: Balance = constants::deposit(1, 88);
    // Additional storage item size of 32 bytes.
    pub const DepositFactor: Balance = constants::deposit(0, 32);
    pub const MaxSignatories: u16 = 100;
}

impl pallet_multisig::Config for Runtime {
    type Event = Event;
    type Call = Call;
    type Currency = Balances;
    type DepositBase = DepositBase;
    type DepositFactor = DepositFactor;
    type MaxSignatories = MaxSignatories;
    type WeightInfo = ();
}

parameter_types! {
    // TCR economics
    pub const MinimumApplicationAmount: Balance = 5 * constants::NODL;
    pub const MinimumCounterAmount: Balance = 10 * constants::NODL;
    // Challenging is considerably more expensive as it would lead to the removal of the member
    pub const MinimumChallengeAmount: Balance = 100 * constants::NODL;
    // If you lose you loose 1/3 of your bid
    pub const LoosersSlash: Perbill = Perbill::from_percent(33);

    // TCR ops
    // We use 3 days to account for different time zones and weekends
    pub const FinalizeApplicationPeriod: BlockNumber = 3 * constants::DAYS;
    // 7 days was chosen to provide enough for a complete review but still manageable
    pub const FinalizeChallengePeriod: BlockNumber = 7 * constants::DAYS;
}

impl pallet_tcr::Config<pallet_tcr::Instance1> for Runtime {
    type Event = Event;
    type Currency = Balances;
    type MinimumApplicationAmount = MinimumApplicationAmount;
    type MinimumCounterAmount = MinimumCounterAmount;
    type MinimumChallengeAmount = MinimumChallengeAmount;
    type LoosersSlash = LoosersSlash;
    type FinalizeApplicationPeriod = FinalizeApplicationPeriod;
    type FinalizeChallengePeriod = FinalizeChallengePeriod;
    type ChangeMembers = PkiRootOfTrust;
}

parameter_types! {
    // Total onboarding cost: 10 NODL + fees (with TCR application)
    pub const SlotBookingCost: Balance = 10 * constants::NODL;
    // Doesn't need to be as expensive
    pub const SlotRenewingCost: Balance = 1 * constants::NODL;
    // One year validity, unless revoked or renewed
    pub const SlotValidity: BlockNumber = 365 * constants::DAYS;
}

impl pallet_root_of_trust::Config for Runtime {
    type Event = Event;
    type Currency = Balances;
    type CertificateId = CertificateId;
    type SlotBookingCost = SlotBookingCost;
    type SlotRenewingCost = SlotRenewingCost;
    type SlotValidity = SlotValidity;
    type FundsCollector = CompanyReserve;
}

impl pallet_emergency_shutdown::Config for Runtime {
    type Event = Event;
    type ShutdownOrigin =
        pallet_collective::EnsureProportionMoreThan<_1, _2, AccountId, RootCollective>;
}

parameter_types! {
    pub const ProtocolFee: Perbill = Perbill::from_percent(20);
    pub const MaximumCoinsEverAllocated: Balance = 1_259_995_654_473_120_000_000;
}

impl pallet_allocations::Config for Runtime {
    type Event = Event;
    type Currency = Balances;
    type ProtocolFee = ProtocolFee;
    type ProtocolFeeReceiver = CompanyReserve;
    type MaximumCoinsEverAllocated = MaximumCoinsEverAllocated;
    type ExistentialDeposit = <Runtime as pallet_balances::Config>::ExistentialDeposit;
}

impl pallet_membership::Config<pallet_membership::Instance5> for Runtime {
    type Event = Event;
    type AddOrigin =
        pallet_collective::EnsureProportionMoreThan<_1, _2, AccountId, TechnicalCollective>;
    type RemoveOrigin =
        pallet_collective::EnsureProportionMoreThan<_1, _2, AccountId, TechnicalCollective>;
    type SwapOrigin =
        pallet_collective::EnsureProportionMoreThan<_1, _2, AccountId, TechnicalCollective>;
    type ResetOrigin =
        pallet_collective::EnsureProportionMoreThan<_1, _2, AccountId, TechnicalCollective>;
    type PrimeOrigin =
        pallet_collective::EnsureProportionMoreThan<_1, _2, AccountId, TechnicalCollective>;
    type MembershipInitialized = Allocations;
    type MembershipChanged = Allocations;
}

impl parachain_info::Config for Runtime {}

impl cumulus_pallet_parachain_system::Config for Runtime {
    type Event = Event;
    type OnValidationData = ();
    type SelfParaId = parachain_info::Module<Runtime>;
    type DownwardMessageHandlers = ();
    type HrmpMessageHandlers = ();
}

parameter_types! {
    pub const PolkadotNetworkId: NetworkId = NetworkId::Polkadot;
}

pub struct AccountId32Convert;
impl Convert<AccountId, [u8; 32]> for AccountId32Convert {
    fn convert(account_id: AccountId) -> [u8; 32] {
        account_id.into()
    }
}

parameter_types! {
    pub NodleNetwork: NetworkId = NetworkId::Named("nodle".into());
    pub RelayChainOrigin: Origin = cumulus_pallet_xcm_handler::Origin::Relay.into();
    pub Ancestry: MultiLocation = MultiLocation::X1(Junction::Parachain {
        id: ParachainInfo::get().into(),
    });
    pub const RelayChainCurrencyId: CurrencyId = CurrencyId::DOT;
}

pub type LocationConverter = (
    ParentIsDefault<AccountId>,
    SiblingParachainConvertsVia<Sibling, AccountId>,
    AccountId32Aliases<NodleNetwork, AccountId>,
);

pub type LocalAssetTransactor = MultiCurrencyAdapter<
    Currencies,
    UnknownTokens,
    IsConcreteWithGeneralKey<CurrencyId, RelayToNative>,
    LocationConverter,
    AccountId,
    CurrencyIdConverter<CurrencyId, RelayChainCurrencyId>,
    CurrencyId,
>;

pub type LocalOriginConverter = (
    SovereignSignedViaLocation<LocationConverter, Origin>,
    RelayChainAsNative<RelayChainOrigin, Origin>,
    SiblingParachainAsNative<cumulus_pallet_xcm_handler::Origin, Origin>,
    SignedAccountId32AsNative<NodleNetwork, Origin>,
);

pub struct XcmConfig;
impl Config for XcmConfig {
    type Call = Call;
    type XcmSender = XcmHandler;
    type AssetTransactor = LocalAssetTransactor;
    type OriginConverter = LocalOriginConverter;
    type IsReserve = NativeAsset;
    type IsTeleporter = ();
    type LocationInverter = LocationInverter<Ancestry>;
}

impl cumulus_pallet_xcm_handler::Config for Runtime {
    type Event = Event;
    type XcmExecutor = XcmExecutor<XcmConfig>;
    type UpwardMessageSender = ParachainSystem;
    type HrmpMessageSender = ParachainSystem;
    type SendXcmOrigin = EnsureRoot<AccountId>;
    type AccountIdConverter = LocationConverter;
}

pub struct RelayToNative;
impl Convert<RelayChainBalance, Balance> for RelayToNative {
    fn convert(val: u128) -> Balance {
        // both native and relay have 12 decimals
        val
    }
}

pub struct NativeToRelay;
impl Convert<Balance, RelayChainBalance> for NativeToRelay {
    fn convert(val: u128) -> Balance {
        // both native and relay have 12 decimals
        val
    }
}

pub struct HandleXcm;
impl XcmHandlerT<AccountId> for HandleXcm {
    fn execute_xcm(origin: AccountId, xcm: Xcm) -> DispatchResult {
        XcmHandler::execute_xcm(origin, xcm)
    }
}

impl orml_xtokens::Config for Runtime {
    type Event = Event;
    type Balance = Balance;
    type ToRelayChainBalance = NativeToRelay;
    type AccountId32Convert = AccountId32Convert;
    type RelayChainNetworkId = PolkadotNetworkId;
    type ParaId = ParachainInfo;
    type XcmHandler = HandleXcm;
}

parameter_type_with_key! {
    pub ExistentialDeposits: |_currency_id: CurrencyId| -> Balance {
        Zero::zero()
    };
}

impl orml_tokens::Config for Runtime {
    type Event = Event;
    type Balance = Balance;
    type Amount = Amount;
    type CurrencyId = CurrencyId;
    type WeightInfo = ();
    type ExistentialDeposits = ExistentialDeposits;
    type OnDust = orml_tokens::TransferDust<Runtime, CompanyReserveAccount>;
}

parameter_types! {
    pub const GetNodleTokenId: CurrencyId = CurrencyId::NODL;
}

pub type NodleToken = BasicCurrencyAdapter<Runtime, Balances, Amount, BlockNumber>;

impl orml_currencies::Config for Runtime {
    type Event = Event;
    type MultiCurrency = orml_tokens::Pallet<Runtime>;
    type NativeCurrency = NodleToken;
    type GetNativeCurrencyId = GetNodleTokenId;
    type WeightInfo = ();
}

impl orml_unknown_tokens::Config for Runtime {
    type Event = Event;
}

construct_runtime!(
    pub enum Runtime where
        Block = Block,
        NodeBlock = nodle_chain_primitives::Block,
        UncheckedExtrinsic = UncheckedExtrinsic
    {
        // System
        System: frame_system::{Pallet, Call, Storage, Config, Event<T>},
        Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent},
        Indices: pallet_indices::{Pallet, Call, Storage, Config<T>, Event<T>},
        Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
        TransactionPayment: pallet_transaction_payment::{Pallet, Storage},
        RandomnessCollectiveFlip: pallet_randomness_collective_flip::{Pallet, Call, Storage},

        // Governance
        TechnicalCommittee: pallet_collective::<Instance2>::{Pallet, Call, Storage, Origin<T>, Event<T>, Config<T>},
        TechnicalMembership: pallet_membership::<Instance1>::{Pallet, Call, Storage, Event<T>, Config<T>},
        FinancialCommittee: pallet_collective::<Instance3>::{Pallet, Call, Storage, Origin<T>, Event<T>, Config<T>},
        FinancialMembership: pallet_membership::<Instance3>::{Pallet, Call, Storage, Event<T>, Config<T>},
        RootCommittee: pallet_collective::<Instance4>::{Pallet, Call, Storage, Origin<T>, Event<T>, Config<T>},
        RootMembership: pallet_membership::<Instance4>::{Pallet, Call, Storage, Event<T>, Config<T>},
        Scheduler: pallet_scheduler::{Pallet, Call, Storage, Event<T>},
        Amendments: pallet_amendments::{Pallet, Call, Storage, Event<T>},
        Mandate: pallet_mandate::{Pallet, Call, Event},
        CompanyReserve: pallet_reserve::<Instance1>::{Pallet, Call, Storage, Config, Event<T>},
        InternationalReserve: pallet_reserve::<Instance2>::{Pallet, Call, Storage, Config, Event<T>},
        UsaReserve: pallet_reserve::<Instance3>::{Pallet, Call, Storage, Config, Event<T>},
        Grants: pallet_grants::{Pallet, Call, Storage, Config<T>, Event<T>},

        // Neat things
        Identity: pallet_identity::{Pallet, Call, Storage, Event<T>},
        Recovery: pallet_recovery::{Pallet, Call, Storage, Event<T>},
        Utility: pallet_utility::{Pallet, Call, Event},
        Proxy: pallet_proxy::{Pallet, Call, Storage, Event<T>},
        Multisig: pallet_multisig::{Pallet, Call, Storage, Event<T>},

        // Cumulus parachain
        ParachainInfo: parachain_info::{Pallet, Storage, Config},
        ParachainSystem: cumulus_pallet_parachain_system::{Pallet, Call, Storage, Inherent, Event},
        XcmHandler: cumulus_pallet_xcm_handler::{Pallet, Call, Event<T>, Origin},

        // Cross tokens support
        Currencies: orml_currencies::{Pallet, Call, Event<T>},
        Tokens: orml_tokens::{Pallet, Storage, Call, Event<T>, Config<T>},
        UnknownTokens: orml_unknown_tokens::{Pallet, Storage, Event},
        XTokens: orml_xtokens::{Pallet, Storage, Call, Event<T>},

        // Nodle Stack
        PkiTcr: pallet_tcr::<Instance1>::{Pallet, Call, Storage, Event<T>},
        PkiRootOfTrust: pallet_root_of_trust::{Pallet, Call, Storage, Event<T>},
        EmergencyShutdown: pallet_emergency_shutdown::{Pallet, Call, Event, Storage},
        Allocations: pallet_allocations::{Pallet, Call, Event<T>, Storage},
        AllocationsOracles: pallet_membership::<Instance5>::{Pallet, Call, Storage, Event<T>, Config<T>},
    }
);

/// The address format for describing accounts.
pub type Address = <Indices as StaticLookup>::Source;
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
    frame_system::CheckSpecVersion<Runtime>,
    frame_system::CheckTxVersion<Runtime>,
    frame_system::CheckGenesis<Runtime>,
    frame_system::CheckEra<Runtime>,
    frame_system::CheckNonce<Runtime>,
    frame_system::CheckWeight<Runtime>,
    pallet_transaction_payment::ChargeTransactionPayment<Runtime>,
);
/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic = generic::UncheckedExtrinsic<Address, Call, Signature, SignedExtra>;
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
    AllPallets,
>;

sp_api::impl_runtime_apis! {
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
            Runtime::metadata().into()
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

        fn random_seed() -> <Block as BlockT>::Hash {
            RandomnessCollectiveFlip::random_seed().0
        }
    }

    impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
        fn validate_transaction(source: TransactionSource, tx: <Block as BlockT>::Extrinsic) -> TransactionValidity {
            Executive::validate_transaction(source, tx)
        }
    }

    impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
        fn offchain_worker(header: &<Block as BlockT>::Header) {
            Executive::offchain_worker(header)
        }
    }

    impl sp_session::SessionKeys<Block> for Runtime {
        fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
            SessionKeys::generate(seed)
        }

        fn decode_session_keys(
            encoded: Vec<u8>,
        ) -> Option<Vec<(Vec<u8>, sp_core::crypto::KeyTypeId)>> {
            SessionKeys::decode_into_raw_public_keys(&encoded)
        }
    }

    impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Index> for Runtime {
        fn account_nonce(account: AccountId) -> Index {
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

    impl pallet_root_of_trust_runtime_api::RootOfTrustApi<Block, CertificateId> for Runtime {
        fn is_root_certificate_valid(cert: &CertificateId) -> bool {
            PkiRootOfTrust::is_root_certificate_valid(cert)
        }

        fn is_child_certificate_valid(root: &CertificateId, child: &CertificateId) -> bool {
            PkiRootOfTrust::is_child_certificate_valid(root, child)
        }
    }

    #[cfg(feature = "runtime-benchmarks")]
    impl frame_benchmarking::Benchmark<Block> for Runtime {
        fn dispatch_benchmark(
            config: frame_benchmarking::BenchmarkConfig,
        ) -> Result<Vec<frame_benchmarking::BenchmarkBatch>, sp_runtime::RuntimeString> {
            // We did not include the offences and sessions benchmarks as they are parity
            // specific and were causing some issues at compile time as they depend on the
            // presence of the staking and elections pallets.

            use frame_benchmarking::{Benchmarking, BenchmarkBatch, TrackedStorageKey, add_benchmark};
            use frame_system_benchmarking::Module as SystemBench;

            impl frame_system_benchmarking::Config for Runtime {}

            let whitelist: Vec<TrackedStorageKey> = vec![];
            let mut batches = Vec::<BenchmarkBatch>::new();
            let params = (&config, &whitelist);

            add_benchmark!(params, batches, frame_system, SystemBench::<Runtime>);
            add_benchmark!(params, batches, pallet_allocations, Allocations);
            add_benchmark!(params, batches, pallet_amendments, Amendments);
            add_benchmark!(params, batches, pallet_balances, Balances);
            add_benchmark!(params, batches, pallet_collective, TechnicalCommittee);
            add_benchmark!(params, batches, pallet_emergency_shutdown, EmergencyShutdown);
            add_benchmark!(params, batches, pallet_grants, Grants);
            add_benchmark!(params, batches, pallet_identity, Identity);
            add_benchmark!(params, batches, pallet_indices, Indices);
            add_benchmark!(params, batches, pallet_multisig, Multisig);
            add_benchmark!(params, batches, pallet_proxy, Proxy);
            add_benchmark!(params, batches, pallet_reserve, CompanyReserve);
            add_benchmark!(params, batches, pallet_root_of_trust, PkiRootOfTrust);
            add_benchmark!(params, batches, pallet_scheduler, Scheduler);
            add_benchmark!(params, batches, pallet_tcr, PkiTcr);
            add_benchmark!(params, batches, pallet_timestamp, Timestamp);
            add_benchmark!(params, batches, pallet_utility, Utility);

            // TODO: add benchs for ORML

            if batches.is_empty() { return Err("Benchmark not found for this pallet.".into()) }
            Ok(batches)
        }
    }
}

cumulus_pallet_parachain_system::register_validate_block!(Runtime, Executive);

use crate::{Module, Trait};
use super::*;
use sp_core::H256;
use frame_support::{
    impl_outer_event,
    impl_outer_origin, 
    parameter_types, weights::Weight, assert_ok, assert_noop,
                    traits::{OnFinalize, OnInitialize},
};
use sp_runtime::{
    traits::{BlakeTwo256, IdentityLookup}, testing::Header, Perbill,
};
use frame_system;
use balances;

impl_outer_origin! {
    pub enum Origin for Test {}
}

mod kitties_event {
    pub use crate::Event;
}

impl_outer_event! {
    pub enum TestEvent for Test {
        frame_system<T>, //??
        kitties_event<T>, //??
        balances<T>, //??
    }
}


#[derive(Clone, Eq, PartialEq)]
pub struct Test;
parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const MaximumBlockWeight: Weight = 1024;
    pub const MaximumBlockLength: u32 = 2 * 1024;
    pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
}
impl frame_system::Trait for Test {
    type BaseCallFilter = ();
    type Origin = Origin;
    type Call = ();
    type Index = u64;
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = TestEvent;
    type BlockHashCount = BlockHashCount;
    type MaximumBlockWeight = MaximumBlockWeight;
    type DbWeight = ();
    type BlockExecutionWeight = ();
    type ExtrinsicBaseWeight = ();
    type MaximumExtrinsicWeight = MaximumBlockWeight;
    type MaximumBlockLength = MaximumBlockLength;
    type AvailableBlockRatio = AvailableBlockRatio;
    type Version = ();
    type PalletInfo = ();
    type AccountData = balances::AccountData<u64>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
}
parameter_types! {
    pub const ExistentialDeposit: u64 = 1;
    pub const MaxLocks: u32 = 50;
}
pub type Balance = u64;
impl balances::Trait for Test {
    type MaxLocks = MaxLocks;
    /// The type for recording an account's balance.
    type Balance = Balance;
    /// The ubiquitous event type.
    type Event = TestEvent;
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = frame_system::Module<Test>;
    type WeightInfo = ();
}

type Randomness = pallet_randomness_collective_flip::Module<Test>;
parameter_types! {
    pub const NewKittyReserve: u64 = 5_000;
}
    impl Trait for Test {
    type Event = TestEvent;
    type Randomness = Randomness;
    type KittyIndex = u32;
    type NewKittyReserve = NewKittyReserve;
    type Currency = balances::Module<Self>;
}

pub type Kitties = Module<Test>;
pub type System = frame_system::Module<Test>;

fn run_to_block(n: u64) {
    while System::block_number() < n {
        Kitties::on_finalize(System::block_number());
        System::on_finalize(System::block_number());
        System::set_block_number(System::block_number() + 1);
        System::on_initialize(System::block_number());
        Kitties::on_initialize(System::block_number());
    }
}

pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap().into();
    balances::GenesisConfig::<Test> {
        balances: vec![(1, 5000000), (2, 51000000), (3, 5200000), (4, 53000000), (5, 54000000),(6, 50)],
    }
    .assimilate_storage(&mut t)
    .unwrap();

    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| System::set_block_number(1));
    ext
}
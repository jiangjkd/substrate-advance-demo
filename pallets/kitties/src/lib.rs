#![cfg_attr(not(feature = "std"), no_std)]


use codec::{Encode, Decode};
use frame_support::{Parameter,decl_module, decl_storage,decl_event, decl_error, traits::Get, ensure, StorageValue, StorageMap, traits::Randomness, sp_std::prelude::*};
use sp_io::hashing::blake2_128;
use frame_system::ensure_signed;
use sp_runtime::{DispatchError, traits::AtLeast32BitUnsigned};
use sp_runtime::traits::Bounded;
use sp_std::vec;
use frame_support::traits::Currency;
use frame_support::traits::ReservableCurrency;

#[derive(Encode, Decode, Debug, Clone)]
pub struct Kitty<T> {
    // 自身kittyid
    kitty_id: Option<T>,
    // 父母kitty 用元组表示
    parents_ids: (Option<T>, Option<T>),
    // 配偶kitty
    spouse_id: Option<T>,
    // DNA数据
    dna_data: [u8; 16],
}

type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::Balance;

pub trait Trait: frame_system::Trait {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
    type Randomness: Randomness<Self::Hash>;
    type KittyIndex: Parameter + AtLeast32BitUnsigned + Bounded + Default + Copy;
     // 创建kitty 的时候，需要质押的代币
    type NewKittyReserve: Get<BalanceOf<Self>>;
    // Currency 类型，用于质押等资产相关的操作
    type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
}


impl<T: std::cmp::PartialEq> Kitty<T>  {
    pub fn new() -> Self {
        Self {
            kitty_id: None, // kittyid
            parents_ids:(None, None), // 父母数据
            spouse_id: None, //配偶
            dna_data:[0; 16]
        }
    }
    // 设置kittyid
    pub fn set_kitty_id(&mut self, kitty_id: T) {
        self.kitty_id = Some(kitty_id);
    }
    // 设置dna
    pub fn set_dna_data(& mut self, dna_data: [u8; 16]) {
        self.dna_data = dna_data;
    }
    // 设置配偶
    pub fn set_spouse_id(&mut self, kitty_id: T) {
        self.spouse_id = Some(kitty_id);
    }
    // 设置父母
    pub fn set_parents_ids(&mut self, kitty_id1: Option<T>, kitty_id2: Option<T>) {
        self.parents_ids = (kitty_id1, kitty_id2);
    }
}


decl_storage! {
	trait Store for Module<T: Trait> as Kitties {
        // 用kittyIndex用作键,值为 kitty 的dna数据
		pub Kitties get(fn kitties): map hasher(blake2_128_concat) T::KittyIndex => Option<Kitty<T::KittyIndex>>;
        // 记录kitties的数量
		pub KittiesCount get(fn kitties_count): T::KittyIndex;
        // 记录每一只kitty的拥有者
		pub KittyOwner get(fn kitty_owner): map hasher(blake2_128_concat) T::KittyIndex => Option<T::AccountId>;
        // 记录某个账号拥有的所有kitty
        pub OwnedKitties get(fn owned_kitties): map hasher(blake2_128_concat) T::AccountId => vec::Vec<T::KittyIndex>;

        // 通过一个double map 可以从父母的任何一方的index映射到孩子的index. 使用Vec<KittyIndex>是因为同一对父母可能产生多个孩子
        // 在breed是更新
        pub KittiesChildren get(fn kitty_children): double_map hasher(blake2_128_concat) T::KittyIndex,  hasher(blake2_128_concat) T::KittyIndex => vec::Vec<T::KittyIndex>;

        // 可以通过 Kitties parents和kitties children 得到一个kittyIndex 到 bother(Vec<KittyIndex>)的一个映射关系
        // 在breed是更新
        pub KittiesBrother get(fn kitty_brother): map hasher(blake2_128_concat) T::KittyIndex => vec::Vec<T::KittyIndex>;
    }
}
decl_error! {
	pub enum Error for Module<T: Trait> {
		KittiesCountOverflow,
		InvalidKittyId,
		RequireDifferentParent,
        NotKittyOwner,
        MoneyNotEnough,
	}
}
decl_event!(
	pub enum Event<T> 
        where 
            AccountId = <T as frame_system::Trait>::AccountId,
            KittyIndex = <T as Trait>::KittyIndex {
		Created(AccountId, KittyIndex),
        Transfered(AccountId, AccountId, KittyIndex),
	}
);

decl_module! {
    
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		type Error = Error<T>;
        fn deposit_event() = default;

		// 创建kitty
		#[weight = 0]
		pub fn create(origin) {
            // 判断签名
			let sender = ensure_signed(origin)?;
			let kitty_id = Self::next_kitty_id()?;

			let dna = Self::random_value(&sender);

			// 创建新的kitty
			let mut new_kitty = Kitty::new();
            new_kitty.set_dna_data(dna);
            Self::insert_kitty(&sender, kitty_id, new_kitty);

            T::Currency::reserve(&sender, T::NewKittyReserve::get()).map_err(|_| Error::<T>::MoneyNotEnough)?;
            Self::deposit_event(RawEvent::Created(sender, kitty_id));
		}
        #[weight = 0]
        pub fn transfer(origin, to: T::AccountId, kitty_id: T::KittyIndex) {
            let sender = ensure_signed(origin)?;
            // 视频错误 没有校验kitty的所有者
            // 修正
            let account_id = Self::kitty_owner(kitty_id).ok_or(Error::<T>::InvalidKittyId)?;
            ensure!(account_id == sender.clone(),Error::<T>::NotKittyOwner);

            <KittyOwner<T>>::insert(kitty_id, to.clone());
            // 移除原来所有者的记录
            OwnedKitties::<T>::mutate(&sender, |val| val.retain(|&temp| temp != kitty_id));
            // 记录新的所有者的记录
            OwnedKitties::<T>::mutate(&to, |val| val.push(kitty_id));
            
            Self::deposit_event(RawEvent::Transfered(sender, to, kitty_id));


        }
		#[weight = 0]
		pub fn breed(origin, kitty_id_1: T::KittyIndex, kitty_id_2: T::KittyIndex) {
			let sender = ensure_signed(origin)?;
			let new_kitty_id = Self::do_breed(sender.clone(), kitty_id_1, kitty_id_2)?;
            Self::deposit_event(RawEvent::Created(sender, new_kitty_id));
		}
	}
}
fn combine_dna(dna1: u8, dna2: u8, selector: u8) -> u8 {
    (selector & dna1) | (!selector & dna2)
}
impl<T: Trait> Module<T> {
    
    fn next_kitty_id() -> sp_std::result::Result<T::KittyIndex, DispatchError> {
		let kitty_id = Self::kitties_count();
		if kitty_id == T::KittyIndex::max_value() {
			return Err(Error::<T>::KittiesCountOverflow.into());
		}
		Ok(kitty_id)
	}

	fn random_value(sender: &T::AccountId) -> [u8; 16] {
		let payload = (
            T::Randomness::random_seed(),
            &sender,
            <frame_system::Module<T>>::extrinsic_index(),
        );
        payload.using_encoded(blake2_128)
	}

	fn insert_kitty(owner: &T::AccountId, kitty_id: T::KittyIndex, kitty: Kitty<T::KittyIndex>) {
		<Kitties<T>>::insert(kitty_id, kitty);
        <KittiesCount<T>>::put(kitty_id + 1.into());
        // 设置kitty所有者
        <KittyOwner<T>>::insert(kitty_id, owner);
        if <OwnedKitties<T>>::contains_key(&owner) {
            // 已经存在 继续添加 
            <OwnedKitties<T>>::mutate(owner, |val| val.push(kitty_id));
        } else {
           // 不存大 创建新的
           <OwnedKitties<T>>::insert(owner, vec![kitty_id]);
        }
        
	}
    // 更新孩子信息
    fn update_kitties_children(
        children: T::KittyIndex,
        father: T::KittyIndex,
        mother: T::KittyIndex,
    ) {
        if <KittiesChildren<T>>::contains_key(father, mother) {
            let _ = <KittiesChildren<T>>::mutate(father, mother, |val| val.push(children));
        } else {
            // 如果不存在 重新插入一个新的
            <KittiesChildren<T>>::insert(father, mother, vec![children]);
        }
    }
    // 更新兄弟信息
    fn update_kitties_brother(new_kitty: &Kitty<T::KittyIndex>) {
        if let (Some(father), Some(mother)) = new_kitty.parents_ids {
            match new_kitty.kitty_id {
                Some(kitty_id) => {
                    if <KittiesChildren<T>>::contains_key(father, mother) {
                        let val: vec::Vec<T::KittyIndex> = <KittiesChildren<T>>::get(father, mother);
                        let reserve_val: vec::Vec<T::KittyIndex> =
                            val.into_iter().filter(|&val| val != kitty_id).collect();
                        <KittiesBrother<T>>::insert(kitty_id, reserve_val);
                    } else {
                        <KittiesBrother<T>>::insert(kitty_id, vec::Vec::<T::KittyIndex>::new());
                    }
                }
                None => {}
            }
            //let kitty_id: T::KittyIndex = new_kitty.kitty_id.unwrap();
            
        } 
    }

	fn do_breed(sender: T::AccountId, kitty_id_1: T::KittyIndex, kitty_id_2: T::KittyIndex) -> sp_std::result::Result<T::KittyIndex, DispatchError>  {
		let mut kitty1 = Self::kitties(kitty_id_1).ok_or(Error::<T>::InvalidKittyId)?;
		let mut kitty2 = Self::kitties(kitty_id_2).ok_or(Error::<T>::InvalidKittyId)?;

		ensure!(kitty_id_1 != kitty_id_2, Error::<T>::RequireDifferentParent);

		let new_kitty_id = Self::next_kitty_id()?;

		let kitty1_dna = kitty1.dna_data;
		let kitty2_dna = kitty2.dna_data;

		// 生成128位的随机值
		let selector = Self::random_value(&sender);
		let mut new_dna = [0u8; 16];

		// 生成新的kitty
		for i in 0..kitty1_dna.len() {
			new_dna[i] = combine_dna(kitty1_dna[i], kitty2_dna[i], selector[i]);
		}
        let mut new_kitty = Kitty::new();
        // 新kitty设置dna信息
        new_kitty.set_dna_data(new_dna);
        // 新kitty设置父母信息
        new_kitty.set_parents_ids(Some(kitty_id_1), Some(kitty_id_2));
		
        // 相互设置配偶信息
        kitty1.set_spouse_id(kitty_id_2);
        kitty2.set_spouse_id(kitty_id_1);

        // 更新double map 父母对应的孩子index
        Self::update_kitties_children(new_kitty_id, kitty_id_1, kitty_id_2);

        // 更新brother
        Self::update_kitties_brother(&new_kitty);

        T::Currency::reserve(&sender, T::NewKittyReserve::get()).map_err(|_| Error::<T>::MoneyNotEnough)?;
        // 添加kittyid->kitty映射
        Self::insert_kitty(&sender, new_kitty_id, new_kitty);
        

		Ok(new_kitty_id)
	}
}

#[cfg(test)]
mod tests {
    use super::*;
    use sp_core::H256;
    use frame_support::{impl_outer_origin, parameter_types, weights::Weight, assert_ok, assert_noop,
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
        type Event = ();
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
        type Event = ();
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
        type Event = ();
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
            balances: vec![(1, 5000000), (2, 51000000), (3, 5200000), (4, 53000000), (5, 54000000)],
        }
        .assimilate_storage(&mut t)
        .unwrap();

        let mut ext = sp_io::TestExternalities::new(t);
        ext.execute_with(|| System::set_block_number(1));
        ext
    }

    /// 创建kitty
    #[test]
    fn owned_kitties_can_append_values() {
        new_test_ext().execute_with(|| {
            run_to_block(10);
            assert_eq!(Kitties::create(Origin::signed(1)), Ok(()))
        })
    }

    // 转移kitty
    #[test]
    fn transfer_kitties() {
        new_test_ext().execute_with(|| {
            run_to_block(10);
            assert_ok!(Kitties::create(Origin::signed(1)));
            let id = Kitties::kitties_count();
            assert_ok!(Kitties::transfer(Origin::signed(1), 2 , id-1));
            assert_noop!(
                Kitties::transfer(Origin::signed(1), 2, id-1),
                Error::<Test>::NotKittyOwner
            );
        })
    }
}
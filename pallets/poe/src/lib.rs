#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// https://substrate.dev/docs/en/knowledgebase/runtime/frame

use frame_support::{decl_module, decl_storage, decl_event, decl_error, ensure, dispatch, traits::Get, sp_std::prelude::*};
use frame_system::{ self as system, ensure_signed };


#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/// Configure the pallet by specifying the parameters and types on which it depends.
//所有runtime类型和常量都放在这里。 
//如果此pallet依赖于其他特定的pallet，则应将依赖pallet的配置trait添加到继承的trait列表中。
pub trait Trait: system::Trait {
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    type MaxClaimLength: Get<u16>;
}

// The pallet's runtime storage items.
// Runtime存储允许在保证“类型安全“前提下使用Substrate存储数据库，因而可在块与块之间留存内容。
// https://substrate.dev/docs/en/knowledgebase/runtime/storage
decl_storage! {
    trait Store for Module<T: Trait> as PoeModule {
        Proofs get(fn proofs): map hasher(blake2_128_concat) Vec<u8> => (T::AccountId, T::BlockNumber);
    }
}


// Pallets use events to inform users when important changes are made.
// 事件是一种用于报告特定条件和情况发生的简单手段，用户、Dapp和区块链浏览器都可能对事件的感兴趣。没有它就很难发现。
// https://substrate.dev/docs/en/knowledgebase/runtime/events
decl_event!(
	pub enum Event<T> where AccountId = <T as system::Trait>::AccountId {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		ClaimCreated(AccountId, Vec<u8>),
        ClaimRevoked(AccountId, Vec<u8>),
	}
);

// Errors inform users that something went wrong.
decl_error! {
	pub enum Error for Module<T: Trait> {
		ProofAlreadyExist,
		ClaimNotExist,
        NotClaimOwner,
        ClaimExceedLength,
	}
}
// Dispatchable functions allows users to interact with the pallet and invoke state changes.
// These functions materialize as "extrinsics", which are often compared to transactions.
// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
// 定义了该pallet公开的可调用函数，并在区块执行时协调该pallet行为。
decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {

        type Error = Error<T>;

        fn deposit_event() = default;

        const MaxClaimLength: u16 = T::MaxClaimLength::get();

		/// Allow a user to claim ownership of an unclaimed proof.
        #[weight = 0]
        fn create_claim(origin, claim: Vec<u8>) -> dispatch::DispatchResult {

            let sender = ensure_signed(origin)?;
            // 判断存证的长度是否超过
            ensure!(claim.len() as u16 <= T::MaxClaimLength::get(), Error::<T>::ClaimExceedLength);

            // Verify that the specified proof has not already been claimed.
            ensure!(!Proofs::<T>::contains_key(&claim), Error::<T>::ProofAlreadyExist);

            // Get the block number from the FRAME System module.
            let current_block = <system::Module<T>>::block_number();

            // Store the proof with the sender and block number.
            Proofs::<T>::insert(&claim, (sender.clone(), current_block));

            // Emit an event that the claim was created.
            Self::deposit_event(RawEvent::ClaimCreated(sender, claim));
            Ok(())
        }
        /// Allow the owner to revoke their claim.
        #[weight = 0]
        fn revoke_claim(origin, claim: Vec<u8>)  -> dispatch::DispatchResult {
            
            let sender = ensure_signed(origin)?;

            // Verify that the specified proof has been claimed.
            ensure!(Proofs::<T>::contains_key(&claim), Error::<T>::ClaimNotExist);

            // Get owner of the claim.
            let (owner, _block_number) = Proofs::<T>::get(&claim);

            // Verify that sender of the current call is the claim owner.
            ensure!(sender == owner, Error::<T>::NotClaimOwner);

            // Remove claim from storage.
            Proofs::<T>::remove(&claim);

            // Emit an event that the claim was erased.
            Self::deposit_event(RawEvent::ClaimRevoked(sender, claim));
            Ok(())
        }
        #[weight = 0]
        pub fn transfer_claim(origin, claim: Vec<u8>, dest: T::AccountId) -> dispatch::DispatchResult {

            let sender = ensure_signed(origin)?;
            ensure!(Proofs::<T>::contains_key(&claim), Error::<T>::ClaimNotExist);
            let (owner, _block_number) = Proofs::<T>::get(&claim);
            ensure!(sender == owner, Error::<T>::NotClaimOwner);
            Proofs::<T>::insert(&claim, (dest, system::Module::<T>::block_number()));
            Ok(())
        }
	}
}

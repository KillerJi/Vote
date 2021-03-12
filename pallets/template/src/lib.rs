#![cfg_attr(not(feature = "std"), no_std)]
#![allow(non_snake_case, unused_must_use)]
/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://substrate.dev/docs/en/knowledgebase/runtime/frame>


use sp_runtime::{traits::{AtLeast32BitUnsigned,CheckedAdd, Bounded, One, Zero}};
use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*,traits::{Currency,ReservableCurrency}};
use frame_system::pallet_prelude::*;
use sp_std::{convert::TryInto,vec::Vec};

pub use pallet::*;

mod types;
pub use types::{VotingOptions,VotingInfo,VotingStatus,VotingNumber};
#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
		//type Balance: Parameter + Member + AtLeast32BitUnsigned + Default + Copy + MaybeSerializeDeserialize ;
		type ProposalIndex: Parameter + AtLeast32BitUnsigned + Bounded + Default + Copy;
		type Period: Get<Self::BlockNumber>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn ongoing_index)]
	pub type OngoingIndex<T: Config> = StorageValue<_, Vec<T::ProposalIndex>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn Finish_index)]
	pub type FinishIndex<T: Config> = StorageValue<_, Vec<T::ProposalIndex>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn vote_status)]
	pub type VoteStatus<T: Config> = StorageMap<_, Blake2_128Concat, T::ProposalIndex, VotingInfo<T::BlockNumber,BalanceOf<T>> , OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn next_index)]
	pub type NextIndex<T: Config>  = StorageValue<_, T::ProposalIndex, ValueQuery>;

	// #[pallet::storage]
	// #[pallet::getter(fn vote_test)]
	// pub type VoteTest<T: Config> = StorageValue<_, VotingInfo<T::BlockNumber,BalanceOf<T>>, ValueQuery>;

	#[pallet::genesis_config]
	#[derive(Default)]
	pub struct GenesisConfig;

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig {
	 	fn build(&self) {
			let xx: T::ProposalIndex = Zero::zero();
			NextIndex::<T>::put(xx);
		}
	}

	#[pallet::event]
	#[pallet::metadata(T::AccountId = "AccountId")]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		VoteHappend(T::AccountId, T::ProposalIndex, BalanceOf<T>, VotingOptions,VotingInfo<T::BlockNumber,BalanceOf<T>>),

		VoteCreated(T::AccountId, T::ProposalIndex, VotingInfo<T::BlockNumber,BalanceOf<T>>,),

	}

	#[pallet::error]
	pub enum Error<T> {
	
		VoteIndexMissing,

		VotingFinish,

		VotingOverFlow,

		OverFlow,

		ErrorEq,

		StatusErr,

		ProposalAlreadyExist,

	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(now: T::BlockNumber) -> Weight{
			let (reads,writes) = Self::end_finish(now).unwrap();
			T::DbWeight::get().reads_writes(reads, writes)
		}

	}

	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T:Config> Pallet<T> {
							
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn proposal(origin: OriginFor<T> ,finish:T::BlockNumber/* VoteInfo:VotingInfo<T::BlockNumber,T::Balance>*/) -> DispatchResultWithPostInfo{
			let who = ensure_signed(origin)?;

			let VoteIndex = NextIndex::<T>::get();

			NextIndex::<T>::put( VoteIndex.checked_add(&One::one()).ok_or(Error::<T>::VotingOverFlow)? );

			//ensure!(!VoteStatus::<T>::contains_key(VoteIndex), Error::<T>::ProposalAlreadyExist);

			let VoteInfo = VotingInfo::new(<frame_system::Module<T>>::block_number(),finish);

			// <VoteTest::<T>>::put(VoteInfo);

			VoteStatus::<T>::insert( VoteIndex, VoteInfo.clone() );

			Self::deposit_event(Event::VoteCreated(who,VoteIndex,VoteInfo));

			Ok(().into())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn vote(origin: OriginFor<T>, VoteIndex: T::ProposalIndex, VotingNumber: BalanceOf<T>, VoteKind: VotingOptions) -> DispatchResultWithPostInfo{
			let who = ensure_signed(origin)?;

			let mut VoteInfo = VoteStatus::<T>::get(VoteIndex).ok_or(Error::<T>::VoteIndexMissing)?;

			T::Currency::reserve(&who, VotingNumber)
			.map_err(|_| "locker can't afford to lock the amount requested")?;
			
			Self::ensure_ongoing(&VoteInfo.VoteStatus);
			if VoteKind == VotingOptions::A{

				VoteInfo.VoteNumber.A = VoteInfo.VoteNumber.A.checked_add(&VotingNumber).ok_or(Error::<T>::VotingOverFlow)?;
				
			}
			else{
				VoteInfo.VoteNumber.B = VoteInfo.VoteNumber.B.checked_add(&VotingNumber).ok_or(Error::<T>::VotingOverFlow)?;
			}
			
			VoteInfo.MajorityVote = Self::compare_voting_number(&VoteInfo).ok();

			VoteStatus::<T>::insert(VoteIndex, VoteInfo.clone() );

			Self::deposit_event(Event::VoteHappend(who,VoteIndex,VotingNumber,VoteKind,VoteInfo.clone()));

			Ok(().into())
		}

		

		
	}
}

impl<T: Config> Pallet<T> {
	fn ensure_ongoing(r: &VotingStatus ) -> Result<(), DispatchError>
	{
		match r{
			VotingStatus::Ongoing => Ok(()),
			_ => Err(Error::<T>::VotingFinish.into()),
		}

	}
	fn compare_voting_number(r: &VotingInfo<T::BlockNumber,BalanceOf<T>>) -> Result<VotingOptions, DispatchError>
	{
		if r.VoteNumber.A > r.VoteNumber.B
		{
			return Ok( VotingOptions::A ) ;
		}
		else if r.VoteNumber.A < r.VoteNumber.B{
			return Ok( VotingOptions::B ) ;
		}
		else
		{
			return Err(Error::<T>::ErrorEq.into())
		}
	}
	fn end_finish(now : T::BlockNumber) -> Result<(u64,u64), DispatchError>
	{
		let read = OngoingIndex::<T>::get();
		let reads : u64 = read.len().try_into().unwrap();
		let mut writes : u64 = 0;
		for i in OngoingIndex::<T>::get().iter(){
			VoteStatus::<T>::try_mutate(i ,|VoteInfor|{
				match VoteInfor{ 
					Some(r) => {
						if r.FinishedBlock == now {
							writes = writes.clone().checked_add(One::one()).ok_or(Error::<T>::OverFlow)?;
							(*r).VoteStatus = VotingStatus::Finished;
						}
						else{

						}
						Ok(())
					}
					None => {Err(Error::<T>::StatusErr)}//Err(Error::<T>::StatusErr.into())
				}
				
			});
		}
		Ok((reads,writes))
	}
}
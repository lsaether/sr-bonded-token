use rstd::prelude::*;
use parity_codec::Codec;
use support::{decl_module, decl_storage, decl_event, ensure, StorageValue, StorageMap, Parameter, dispatch::Result};
use system::{self, ensure_signed};
use runtime_primitives::traits::{CheckedSub, CheckedAdd, CheckedMul, CheckedDiv, Member, SimpleArithmetic, As};

/// The module's configuration trait.
pub trait Trait: system::Trait {
	type TokenBalance: Parameter + Member + SimpleArithmetic + Codec + Default + Copy + As<usize> + As<u64>;
	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

/// This module's storage items.
decl_storage! {
	trait Store for Module<T: Trait> as BondedFungibleToken {
		// Init get(is_init): bool;


		// Total Supply
		TotalSupply get(total_supply): T::TokenBalance;
		// Mapping of Accounts to Balances
		BalanceOf get(balance_of): map T::AccountId => T::TokenBalance;
		// Mapping of Accounts for `Account` to Allowance
		Allowance get(allowance): map (T::AccountId, T::AccountId) => T::TokenBalance;

		// Exponent
		Exponent get(exponent): u128;
		// Slope Numerator
		SlopeN get(slope_n): u128;
		// Slope Denominator
		SlopeD get(slope_d): u128;
	}
}

decl_module! {
	/// The module declaration.
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		// Initializing events
		// this is needed only if you are using events in your module
		fn deposit_event<T>() = default;

		pub fn transfer(origin, to: T::AccountId, value: T::TokenBalance) -> Result {
			let sender = ensure_signed(origin)?;
			Self::_transfer(sender, to, value)
		}

		pub fn approve(origin, spender: T::AccountId, value: T::TokenBalance) -> Result {
			let sender = ensure_signed(origin)?;
			// Make sure the approver/owner owns this token
			ensure!(<BalanceOf<T>>::exists(&sender), "Account does not own this token");

			// Get the current value of the allowance for this sender and spender
			// combination. If it doesn't exist then default 0 will be returned.
			let allowance = Self::allowance((sender.clone(), spender.clone()));

			// Add the value to the current allowance.
			// Uses `checked_add` which is Safe Math to avoid overflows.
			let updated_allowance = allowance.checked_add(&value).ok_or("overflow in calculating allowance")?;

			// Insert the new allowance value of this sender and spender combination.
			<Allowance<T>>::insert((sender.clone(), spender.clone()), updated_allowance);

			// Bubble up the Approval event.
			Self::deposit_event(RawEvent::Approval(sender, spender, value));
			Ok(())
		}

		pub fn transfer_from(_origin, from: T::AccountId, to: T::AccountId, value: T::TokenBalance) -> Result {
			ensure!(<Allowance<T>>::exists((from.clone(), to.clone())), "Allowance does not exist.");
			// This allowance works differently than in Ethereum.
			let allowance = Self::allowance((from.clone(), to.clone()));
			ensure!(allowance >= value, "Not enough allowance.");

			// Uses `checked_sub` to avoid underflows.
			let updated_allowance = allowance.checked_sub(&value).ok_or("Underflow in allowance calculation.")?;

			// Insert the new allowance value of this sender and spender combination.
			<Allowance<T>>::insert((from.clone(), to.clone()), updated_allowance);

			Self::deposit_event(RawEvent::Approval(from.clone(), to.clone(), value));
			Self::_transfer(from, to, value)
		}

		pub fn buy(_origin) -> Result {
			Ok(())
		}

		pub fn sell(_origin) -> Result {
			Ok(())
		}

		/// Test function to create some tokens.
		pub fn create_tokens(origin, amount: T::TokenBalance) -> Result {
			let sender = ensure_signed(origin)?;

			Self::_mint(sender, amount)?;
			Ok(())
		}
	}
}

decl_event!(
	/// An event in this module.
	pub enum Event<T> where AccountId = <T as system::Trait>::AccountId, TokenBalance = <T as self::Trait>::TokenBalance {
		// Event for transfer of tokens.
		Transfer(Option<AccountId>, Option<AccountId>, TokenBalance),
		// Event for approval.
		Approval(AccountId, AccountId, TokenBalance),
	}
);

/// All functions in the decl_module macro are part of the public interface of the module.
/// 
impl<T: Trait> Module<T> {
	/// Internal transfer function for ERC20 token.
	fn _transfer(from: T::AccountId, to: T::AccountId, value: T::TokenBalance) -> Result {
		ensure!(
			<BalanceOf<T>>::exists(from.clone()),
			"Account does not own any token."
		);

		let sender_balance = Self::balance_of(from.clone());
		ensure!(
			sender_balance >= value,
			"Not enough balance."
		);

		let updated_from_balance = sender_balance.checked_sub(&value).ok_or("Underflow in calculating balance.")?;
		let receiver_balance = Self::balance_of(to.clone());
		let updated_to_balance = receiver_balance.checked_add(&value).ok_or("Overflow in calculating balance.")?;

		// Insert the updated balances into storage.
		<BalanceOf<T>>::insert(from.clone(), updated_from_balance);
		<BalanceOf<T>>::insert(to.clone(), updated_to_balance);

		Self::deposit_event(RawEvent::Transfer(Some(from), Some(to), value));
		Ok(())
	}

	/// Internal mint function for ERC20 token.
	fn _mint(to: T::AccountId, amount: T::TokenBalance) -> Result {
		let balance = Self::balance_of(&to);

		let new_balance = match balance.checked_add(&amount) {
			Some(x) => x,
			None => return Err("Overflow while minting new tokens."),
		};

		let supply = Self::total_supply();
		
		let new_supply = match supply.checked_add(&amount) {
			Some(x) => x,
			None => return Err("Overflow while minting new tokens."),
		};

		<TotalSupply<T>>::put(new_supply);
		<BalanceOf<T>>::insert(to.clone(), new_balance);

		Self::deposit_event(RawEvent::Transfer(None, Some(to), amount));
		Ok(())
	}

	/// Internal burn function for Erc20 token.
	fn _burn(from: T::AccountId, amount: T::TokenBalance) -> Result {
		let balance = Self::balance_of(&from);

		let new_balance = match balance.checked_sub(&amount) {
			Some(x) => x,
			None => return Err("Underflow while burning tokens."),
		};

		let supply = Self::total_supply();

		let new_supply = match supply.checked_sub(&amount) {
			Some(x) => x,
			None => return Err("Underflow while burning tokens."),
		};

		<TotalSupply<T>>::put(new_supply);
		<BalanceOf<T>>::insert(from.clone(), new_balance);

		Self::deposit_event(RawEvent::Transfer(Some(from), None, amount));
		Ok(())
	}

	fn _calc_buy_price(tokens: T::TokenBalance) -> ::std::result::Result<u128, &'static str> {
		let supply = Self::total_supply();

		let new_supply = match supply.checked_add(&tokens) {
			Some(x) => x,
			None => return Err("Overflow while calculating buy price."),
		};

		return Self::_integral(new_supply);
	}

	fn _calc_sell_price(tokens: T::TokenBalance) -> ::std::result::Result<u128, &'static str> {
		let supply = Self::total_supply();

		let new_supply = match supply.checked_sub(&tokens) {
			Some(x) => x,
			None => return Err("Underflow while calculating sell price."),
		};

		return Self::_integral(new_supply)
	}

	fn _integral(to_x: T::TokenBalance) -> ::std::result::Result<u128, &'static str> {
		let nexp = match Self::exponent().checked_add(1) {
			Some(x) => x,
			None => return Err("Overflow when adding one to exponent."),
		};

		let slope = match Self::slope_n().checked_div(Self::slope_d()) {
			Some(x) => x,
			None => return Err("Underflow when attempting division."),
		};

		match (to_x ** &nexp).checked_mul(slope).unwrap().checked_div(nexp) {
			Some(x) => return Ok(x),
			None => return Err("Overflow when calculating integral."),
		}
	}
}

// tests for this module
// #[cfg(test)]
// mod tests {
// 	use super::*;

// 	use runtime_io::with_externalities;
// 	use primitives::{H256, Blake2Hasher};
// 	use support::{impl_outer_origin, assert_ok};
// 	use runtime_primitives::{
// 		BuildStorage,
// 		traits::{BlakeTwo256, IdentityLookup},
// 		testing::{Digest, DigestItem, Header}
// 	};

// 	impl_outer_origin! {
// 		pub enum Origin for Test {}
// 	}

// 	// For testing the module, we construct most of a mock runtime. This means
// 	// first constructing a configuration type (`Test`) which `impl`s each of the
// 	// configuration traits of modules we want to use.
// 	#[derive(Clone, Eq, PartialEq)]
// 	pub struct Test;
// 	impl system::Trait for Test {
// 		type Origin = Origin;
// 		type Index = u64;
// 		type BlockNumber = u64;
// 		type Hash = H256;
// 		type Hashing = BlakeTwo256;
// 		type Digest = Digest;
// 		type AccountId = u64;
// 		type Lookup = IdentityLookup<u64>;
// 		type Header = Header;
// 		type Event = ();
// 		type Log = DigestItem;
// 	}
// 	impl Trait for Test {
// 		type Event = ();
// 	}
// 	type BondedFungibleToken = Module<Test>;

// 	// This function basically just builds a genesis storage key/value store according to
// 	// our desired mockup.
// 	fn new_test_ext() -> runtime_io::TestExternalities<Blake2Hasher> {
// 		system::GenesisConfig::<Test>::default().build_storage().unwrap().0.into()
// 	}

// 	#[test]
// 	fn it_works_for_default_value() {
// 		with_externalities(&mut new_test_ext(), || {
// 			// Just a dummy test for the dummy funtion `do_something`
// 			// calling the `do_something` function with a value 42
// 			assert_ok!(BondedFungibleToken::do_something(Origin::signed(1), 42));
// 			// asserting that the stored value is equal to what we stored
// 			assert_eq!(BondedFungibleToken::something(), Some(42));
// 		});
// 	}
// }

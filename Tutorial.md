# Bonding Curves on Substrate (Tutorial)

![Image](https://images.unsplash.com/photo-1521210081147-93fea49570db?ixlib=rb-1.2.1&ixid=eyJhcHBfaWQiOjEyMDd9&auto=format&fit=crop&w=1189&q=80)

Recently, [I've been playing around with bonding curves](https://beta.convergent.cx).

For those who might not be familiar with the concept, bonding curves (sometimes also referred to as automated market makers) are mechanisms for the continuous generation and destruction of crypto-economic tokens.

You can think of them as a line on a chart in which the x-axis is the total supply and the y-axis is the price. As more tokens are minted or burned through buys and sells, the price will travel along this pre-determined path. Lately, they've been embraced by the community as a "better ICO" mechanism for token generation and have found use in DAOs through the idea of the continous organization.

For the past four months I've struggled to find an optmial implementation which is robust enough to support the multifactor uses of bonding curves on Ethereum while remaining gas efficient. Due to certain constraints of the EVM, this has been _really hard_.

## Enter Substrate

Substrate is the platform for blockchain innovators. It is a library written in the Rust programming language that aims to make it simple for developers to implement custom chain logic and smart contracts which can be deployed and updated in real-time through the use of runtime modules. One of the key selling points is that it allows for native code, compiled to WebAssembly, which should solve some of the headaches associated with working inside the EVM.

While there exists some great tutorials already like the [one]() from Gavin Wood at Web3 Summit and some examples of runtime modules exist on the [examples]() organization, the documentation around building on Substrate is still pretty sparse. Therefore this tutorial will be helpful to developers who might come from Ethereum development backgrounds, since that is where I come from. I will highlight some of the similiarities and differences for developing for Substrate runtimes. We will implement a bonding curve token, or `bonded token` for short.

## Setting up Environment / Installing Dependencies

The first step, like with any tutorial, is to ensure you have the correct dependencies installed. You will need the Rust programming language and the Substrate binaries. The easiest way to install Rust is using `rustup`, if you do not already have it you can get it by running this command: 

```shell
$ curl https://sh.rustup.rs -sSf | sh
```

Then making sure everything is up to date by running `rustup update`.

The developers of Substrate have also prepared a similar script which will download all the dependencies and install Substrate:

```shell
$ curl https://getsubstrate.io -sSf | bash
```

Follow the instructions on screen and refresh your environment by running `source ~/.cargo/env`. If the `~/.cargo/bin` directory is in your PATH (it should be), you will now have access to substrate binaries.

> Protip: If for some reason you do not have access to the substrate binaries as we move forward, try to quit your terminal and re-enter. Still if this does not work, you may have to pull them directly from [GitHub](https://github.com/paritytech/substrate-up) and put them in your PATH manually.

We will now create our project directories. In your working directory please run the following to create a new Substrate template (replace `lsaether` with your own handle/name):

```shell
$ substrate-node-new sr-bonded-token lsaether
```

After it compiles (may take a few minutes), also run:

```shell
$ substrate-ui-new bonded-token-ui
```

which will create the template ui we will use to interact with our bonded token.

## Creating our Module

Change into the `sr-bonded-token/runtime/src` directory and execute the `substrate-module-new` command to create a template module that we will modify.

```shell
$ cd ./sr-bonded-token/runtime/src
$ substrate-module-new bonded_token
```

This will create a file at `runtime/src/bonded_token.rs` which will have some boilerplate code for the module. Any time you create a module you will want to run this command to get started quickly. 

Our first steps is to add some code to the `runtime/src/lib.rs` file to make it aware of the new module we have just created. You will do this by adding the following:

You will declare the module by [inputting](https://github.com/lsaether/sr-bonded-token/blob/master/runtime/src/lib.rs#L50) 

```rust
mod bonded_token;
```

You will implement the trait for this runtime and define the types that we will [use](https://github.com/lsaether/sr-bonded-token/blob/master/runtime/src/lib.rs#L179)

```rust 
impl bonded_token::Trait for Runtime {
	/// The ubiquitous event type.
	type Event = Event;
	/// The type for recording an account's token balance.
	type TokenBalance = u128;
}
```

And you will put the module into the runtime by entering the following into the `construct_runtime!` module below the last module [declaration](https://github.com/lsaether/sr-bonded-token/blob/master/runtime/src/lib.rs#L202)

```rust
BondedToken: bonded_token::{Module, Call, Storage, Event<T>},
```

You can look over the [complete file](https://github.com/lsaether/sr-bonded-token/blob/master/runtime/src/lib.rs), as well as the rest of the code in the [GitHub repository](https://github.com/lsaether/sr-bonded-token).

## Defining the Bonded Token module

The first thing we need to do is to bring in the external modules we will be using in this file. Delete the current `use` statements at the top of `runtime/src/bonded_token.rs` and replace them with the following:

```rust
use rstd::prelude::*;
use parity_codec::Codec;
use support::{decl_module, decl_storage, decl_event, ensure, StorageValue, StorageMap, Parameter, dispatch::Result};
use {balances, system::{self, ensure_signed}};
use runtime_primitives::traits::{CheckedSub, CheckedAdd, Member, SimpleArithmetic, As};
```

In the module configuration trait we define a `TokenBalance` type which mirrors the pre-built `Balance` type and will be used to define an Account's balance of this Token. 

```rust
pub trait Trait: system::Trait {

}
```

The first thing we will do is to define the storage variable we will need. Since the `bonded_token` is a token, we will follow the ERC20 convention when defining the Token storage, but we will also need to add a few additional storage parameters to make this token into a bonded token.

In the `decl_storage!` macro, delete the template `Something` storage item define the following items:

```rust
decl_storage! {
	trait Store for Module<T: Trait> as bonded_token {
		/// Initializes this module with constructor parameters.
		Init get(is_init): bool;

		// Total Supply
		TotalSupply get(total_supply): u128;
		// Mapping of Accounts to Balances
		BalanceOf get(balance_of): map T::AccountId => u128;
		// Mapping of Accounts for `Account` to Allowance
		Allowance get(allowance): map (T::AccountId, T::AccountId) => u128;
	}
}
```

Notice that we use `u128` types, this is because the `Balance` trait can coerce into the `u128` type and the use of a generic type will make it easy for the front-end to make assumptions about the value.

Next we will move down to the `decl_module!` macro and again delete the templated `do_something` function. We will replace it with some standard ERC20 functions. These were cribbed from this example.

```rust
		pub fn transfer(origin, to: T::AccountId, value: u128) -> Result {
			let sender = ensure_signed(origin)?;
			Self::_transfer(sender, to, value)
		}

		pub fn approve(origin, spender: T::AccountId, value: u128) -> Result {
			let sender = ensure_signed(origin)?;
			// Make sure the approver/owner owns this token
			ensure!(<BalanceOf<T>>::exists(&sender), "Account does not own this token");

			// Get the current value of the allowance for this sender and spender
			// combination. If it doesn't exist then default 0 will be returned.
			let allowance = Self::allowance((sender.clone(), spender.clone()));

			// Add the value to the current allowance.
			// Uses `checked_add` which is Safe Math to avoid overflows.
			let updated_allowance = allowance.checked_add(value).ok_or("overflow in calculating allowance")?;

			// Insert the new allowance value of this sender and spender combination.
			<Allowance<T>>::insert((sender.clone(), spender.clone()), updated_allowance);

			// Bubble up the Approval event.
			Self::deposit_event(RawEvent::Approval(sender, spender, value));
			Ok(())
		}

		pub fn transfer_from(_origin, from: T::AccountId, to: T::AccountId, value: u128) -> Result {
			ensure!(<Allowance<T>>::exists((from.clone(), to.clone())), "Allowance does not exist.");
			// This allowance works differently than in Ethereum.
			let allowance = Self::allowance((from.clone(), to.clone()));
			ensure!(allowance >= value, "Not enough allowance.");

			// Uses `checked_sub` to avoid underflows.
			let updated_allowance = allowance.checked_sub(value).ok_or("Underflow in allowance calculation.")?;

			// Insert the new allowance value of this sender and spender combination.
			<Allowance<T>>::insert((from.clone(), to.clone()), updated_allowance);

			Self::deposit_event(RawEvent::Approval(from.clone(), to.clone(), value));
			Self::_transfer(from, to, value)
		} 
```


You might notice that we are using a `_transfer()` function which hasn't been declared yet. This is because we want to declare this function as internal and not public like the ones above. The way to declare an internal function in a Substrate runtime module is a little different. Underneath the `decl_event!` macro (which we will return to soon), create an `impl` block like so:

```rust
impl<T: Trait> Module<T> {
    /// All functions we declare inside of this `impl` block will
    /// we only accessible by this runtime if declared as a `fn` or
    /// to this and other runtimes if declared as `pub fn`. They will
    /// not be accessible to extrinsics.
}
```

Inside of the `impl` block we will place the internal `_transfer()` function so that it is only accessible to this runtime.

```rust
	/// Internal transfer function for ERC20 token.
	fn _transfer(from: T::AccountId, to: T::AccountId, value: u128) -> Result {
		ensure!(
			<BalanceOf<T>>::exists(from.clone()),
			"Account does not own any token."
		);

		let sender_balance = Self::balance_of(from.clone());
		ensure!(
			sender_balance >= value,
			"Not enough balance."
		);

		let updated_from_balance = sender_balance.checked_sub(value).ok_or("Underflow in calculating balance.")?;
		let receiver_balance = Self::balance_of(to.clone());
		let updated_to_balance = receiver_balance.checked_add(value).ok_or("Overflow in calculating balance.")?;

		// Insert the updated balances into storage.
		<BalanceOf<T>>::insert(from.clone(), updated_from_balance);
		<BalanceOf<T>>::insert(to.clone(), updated_to_balance);

		Self::deposit_event(RawEvent::Transfer(Some(from), Some(to), value));
		Ok(())
	}
```

Now we are still missing the `Transfer` and `Approval` events, and can add them to the `decl_event!` macro like so:

```rust
decl_event!(
	pub enum Event<T> where AccountId = <T as system::Trait>::AccountId {
		// Event for transfer of tokens.
		Transfer(Option<AccountId>, Option<AccountId>, u128),
		// Event for approval.
		Approval(AccountId, AccountId, u128),
	}
);
```

Whew! Okay, we've finished implementing the bare bones ERC20 functionality for our token. Next up, we will add `buy()`, `sell()` and `init()` functions to turn it into a bonded token.

## Adding the Bonding Curve functions

Bonded Tokens must be initialized with a polynomial which dictates the price path. In this case we will use a simple polynomial of the form `y = mx^n` and define `m` as `slope` and `n` as `exponent.` We define these items in the storage macro:

```rust
decl_storage! {
	trait Store for Module<T: Trait> as bonded_token {
        // ...

		// Exponent of the polynomial
		Exponent get(exponent): u128;
		// Slope of the polynomial
		Slope get(slope): u128;
    }
}
```

We will also need to keep track of how much value is kept by the token as the reserve, which will get paid back to users who sell their token. We can add a `Reserve` storage item too, this will be of type `T::Balance` since it a record of how much balance is kept.

```rust
decl_storage! {
	trait Store for Module<T: Trait> as bonded_token {
        // ...

		// Reserve held to incentive sells
		Reserve get(reserve): T::Balance;
    }
}
```

This also means we must bring in and use the `balances` module. At the top of the file put a new use statement:

```rust
use balances;
```

and in the `Trait` declaration add the balances Trait

```rust
// The trait you defined before, with the added `balanced::Trait`
pub trait Trait: system::Trait + balances::Trait {
	// ...
}
```

Now we will want to 

## The User Interface

TODO

![UI]('./ss.png)














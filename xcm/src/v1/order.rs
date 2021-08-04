// Copyright 2020 Parity Technologies (UK) Ltd.
// This file is part of Cumulus.

// Substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Substrate is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Cumulus.  If not, see <http://www.gnu.org/licenses/>.

//! Version 0 of the Cross-Consensus Message format data structures.

use super::{MultiAsset, MultiAssetFilter, MultiLocation, Xcm};
use alloc::vec::Vec;
use derivative::Derivative;
use parity_scale_codec::{self, Decode, Encode};

/// An instruction to be executed on some or all of the assets in holding, used by asset-related XCM messages.
#[derive(Derivative, Encode, Decode)]
#[derivative(Clone(bound = ""), Eq(bound = ""), PartialEq(bound = ""), Debug(bound = ""))]
#[codec(encode_bound())]
#[codec(decode_bound())]
pub enum Order<Call> {
	/// Do nothing. Not generally used.
	#[codec(index = 0)]
	Noop,

	/// Remove the asset(s) (`assets`) from holding and place equivalent assets under the ownership of `dest` within
	/// this consensus system.
	///
	/// - `assets`: The asset(s) to remove from holding.
	/// - `dest`: The new owner for the assets.
	///
	/// Errors:
	#[codec(index = 1)]
	DepositAsset { assets: MultiAssetFilter, max_assets: u32, beneficiary: MultiLocation },

	/// Remove the asset(s) (`assets`) from holding and place equivalent assets under the ownership of `dest` within
	/// this consensus system (i.e. its sovereign account).
	///
	/// Send an onward XCM message to `dest` of `ReserveAssetDeposited` with the given `effects`.
	///
	/// - `assets`: The asset(s) to remove from holding.
	/// - `dest`: The location whose sovereign account will own the assets and thus the effective beneficiary for the
	///   assets and the notification target for the reserve asset deposit message.
	/// - `effects`: The orders that should be contained in the `ReserveAssetDeposited` which is sent onwards to
	///   `dest`.
	///
	/// Errors:
	#[codec(index = 2)]
	DepositReserveAsset { assets: MultiAssetFilter, max_assets: u32, dest: MultiLocation, effects: Vec<Order<()>> },

	/// Remove the asset(s) (`give`) from holding and replace them with alternative assets.
	///
	/// The minimum amount of assets to be received into holding for the order not to fail may be stated.
	///
	/// - `give`: The asset(s) to remove from holding.
	/// - `receive`: The minimum amount of assets(s) which `give` should be exchanged for.
	///
	/// Errors:
	#[codec(index = 3)]
	ExchangeAsset { give: MultiAssetFilter, receive: Vec<MultiAsset> },

	/// Remove the asset(s) (`assets`) from holding and send a `WithdrawAsset` XCM message to a reserve location.
	///
	/// - `assets`: The asset(s) to remove from holding.
	/// - `reserve`: A valid location that acts as a reserve for all asset(s) in `assets`. The sovereign account
	///   of this consensus system *on the reserve location* will have appropriate assets withdrawn and `effects` will
	///   be executed on them. There will typically be only one valid location on any given asset/chain combination.
	/// - `effects`: The orders to execute on the assets once withdrawn *on the reserve location*.
	///
	/// Errors:
	#[codec(index = 4)]
	InitiateReserveWithdraw {
		assets: MultiAssetFilter,
		reserve: MultiLocation,
		effects: Vec<Order<()>>,
	},

	/// Remove the asset(s) (`assets`) from holding and send a `ReceiveTeleportedAsset` XCM message to a `destination`
	/// location.
	///
	/// - `assets`: The asset(s) to remove from holding.
	/// - `destination`: A valid location that has a bi-lateral teleportation arrangement.
	/// - `effects`: The orders to execute on the assets once arrived *on the destination location*.
	///
	/// NOTE: The `destination` location *MUST* respect this origin as a valid teleportation origin for all `assets`.
	/// If it does not, then the assets may be lost.
	///
	/// Errors:
	#[codec(index = 5)]
	InitiateTeleport { assets: MultiAssetFilter, dest: MultiLocation, effects: Vec<Order<()>> },

	/// Send a `Balances` XCM message with the `assets` value equal to the holding contents, or a portion thereof.
	///
	/// - `query_id`: An identifier that will be replicated into the returned XCM message.
	/// - `dest`: A valid destination for the returned XCM message. This may be limited to the current origin.
	/// - `assets`: A filter for the assets that should be reported back. The assets reported back will be, asset-
	///   wise, *the lesser of this value and the holding register*. No wildcards will be used when reporting assets
	///   back.
	///
	/// Errors:
	#[codec(index = 6)]
	QueryHolding {
		#[codec(compact)]
		query_id: u64,
		dest: MultiLocation,
		assets: MultiAssetFilter,
	},

	/// Pay for the execution of some XCM with up to `weight` picoseconds of execution time, paying for this with
	/// up to `fees` from the holding register.
	///
	/// - `fees`: The asset(s) to remove from holding to pay for fees.
	///
	/// Errors:
	#[codec(index = 7)]
	BuyExecution {
		fees: MultiAsset,
		weight: u64,
		debt: u64,
		halt_on_error: bool,
		xcm: Vec<Xcm<Call>>,
	},
}

pub mod opaque {
	pub type Order = super::Order<()>;
}

impl<Call> Order<Call> {
	pub fn into<C>(self) -> Order<C> {
		Order::from(self)
	}
	pub fn from<C>(order: Order<C>) -> Self {
		use Order::*;
		match order {
			Noop => Noop,
			DepositAsset { assets, max_assets, beneficiary } => DepositAsset { assets, max_assets, beneficiary },
			DepositReserveAsset { assets, max_assets, dest, effects } =>
				DepositReserveAsset { assets, max_assets, dest, effects },
			ExchangeAsset { give, receive } => ExchangeAsset { give, receive },
			InitiateReserveWithdraw { assets, reserve, effects } =>
				InitiateReserveWithdraw { assets, reserve, effects },
			InitiateTeleport { assets, dest, effects } =>
				InitiateTeleport { assets, dest, effects },
			QueryHolding { query_id, dest, assets } => QueryHolding { query_id, dest, assets },
			BuyExecution { fees, weight, debt, halt_on_error, xcm } => {
				let xcm = xcm.into_iter().map(Xcm::from).collect();
				BuyExecution { fees, weight, debt, halt_on_error, xcm }
			},
		}
	}
}

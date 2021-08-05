// Copyright 2021 Parity Technologies (UK) Ltd.
// This file is part of Polkadot.

// Polkadot is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Polkadot is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Polkadot.  If not, see <http://www.gnu.org/licenses/>.

use frame_support::{parameter_types, traits::All};
use xcm::v0::{Error as XcmError, MultiAsset, MultiLocation, Result as XcmResult, SendXcm, Xcm};
use xcm_builder::{
    AllowUnpaidExecutionFrom, LocationInverter, FixedWeightBounds, FixedRateOfConcreteFungible,
};
use xcm_executor::{Assets, traits::TransactAsset};

parameter_types! {
    pub Ancestry: MultiLocation = MultiLocation::Null;
	pub KsmPerSecond: (MultiLocation, u128) = (MultiLocation::Null, 1);
}

pub struct DoNothingRouter;
impl SendXcm for DoNothingRouter {
	fn send_xcm(_dest: MultiLocation, _msg: Xcm<()>) -> XcmResult {
		Ok(())
	}
}

pub type Barrier = AllowUnpaidExecutionFrom<All<MultiLocation>>;

pub struct DummyAssetTransactor;
impl TransactAsset for DummyAssetTransactor {
    fn deposit_asset(_what: &MultiAsset, _who: &MultiLocation) -> XcmResult {
        Ok(())
    }

    fn withdraw_asset(_what: &MultiAsset, _who: &MultiLocation) -> Result<Assets, XcmError> {
        Ok(Assets::default())
    }
}

pub struct XcmConfig;
impl xcm_executor::Config for XcmConfig {
	type Call = super::Call;
	type XcmSender = DoNothingRouter;
	type AssetTransactor = DummyAssetTransactor;
	type OriginConverter = pallet_xcm::XcmPassthrough<super::Origin>;
	type IsReserve = ();
	type IsTeleporter = ();
	type LocationInverter = LocationInverter<Ancestry>;
	type Barrier = Barrier;
	type Weigher = FixedWeightBounds<super::BaseXcmWeight, super::Call>;
	type Trader = FixedRateOfConcreteFungible<KsmPerSecond, ()>;
	type ResponseHandler = ();
}

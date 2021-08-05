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

use polkadot_test_client::{
    BlockBuilderExt, ClientBlockImportExt, DefaultTestClientBuilderExt, ExecutionStrategy,
    InitPolkadotBlockBuilder, TestClientBuilder, TestClientBuilderExt,
};
use polkadot_test_service::construct_extrinsic;
use sp_runtime::{generic::BlockId, traits::Block};
use sp_state_machine::InspectState;
use xcm::v0::{
    Error as XcmError,
    MultiAsset::*,
    MultiLocation::*,
    Order, Outcome,
    Xcm::*,
};
use xcm_executor::MAX_RECURSION_LIMIT;

#[test]
fn execute_within_recursion_limit() {
    let mut client = TestClientBuilder::new()
        .set_execution_strategy(ExecutionStrategy::AlwaysWasm)
        .build();

    let mut msg = WithdrawAsset {
        assets: vec![ConcreteFungible { id: Null, amount: 0 }],
        effects: vec![],
    };
    for _ in 0..MAX_RECURSION_LIMIT {
        msg = WithdrawAsset {
            assets: vec![ConcreteFungible { id: Null, amount: 0 }],
            effects: vec![Order::BuyExecution {
                fees: All,
                weight: 0,
                debt: 0,
                halt_on_error: true,
                // nest `msg` into itself on each iteration.
                xcm: vec![msg],
            }],
        };
    }

    let execute = construct_extrinsic(
        &client,
        polkadot_test_runtime::Call::Xcm(pallet_xcm::Call::execute(
            Box::new(msg.clone()),
            1_000_000_000,
        )),
        sp_keyring::Sr25519Keyring::Alice,
    );

    let mut block_builder = client.init_polkadot_block_builder();
    block_builder.push_polkadot_extrinsic(execute).expect("pushes extrinsic");

    let block = block_builder.build().expect("Finalizes the block").block;
    let block_hash = block.hash();

    futures::executor::block_on(client.import(sp_consensus::BlockOrigin::Own, block))
        .expect("imports the block");

    client
        .state_at(&BlockId::Hash(block_hash))
        .expect("state should exist")
        .inspect_state(|| {
            assert!(polkadot_test_runtime::System::events().iter().any(|r| matches!(
                r.event,
                polkadot_test_runtime::Event::Xcm(pallet_xcm::Event::Attempted(
                    Outcome::Complete(_)
                ),),
            )));
        });
}

#[test]
fn exceed_recursion_limit() {
    let mut client = TestClientBuilder::new()
        .set_execution_strategy(ExecutionStrategy::AlwaysWasm)
        .build();

    let mut msg = WithdrawAsset {
        assets: vec![ConcreteFungible { id: Null, amount: 0 }],
        effects: vec![],
    };
    for _ in 0..(MAX_RECURSION_LIMIT + 1) {
        msg = WithdrawAsset {
            assets: vec![ConcreteFungible { id: Null, amount: 0 }],
            effects: vec![Order::BuyExecution {
                fees: All,
                weight: 0,
                debt: 0,
                halt_on_error: true,
                // nest `msg` into itself on each iteration.
                xcm: vec![msg],
            }],
        };
    }

    let execute = construct_extrinsic(
        &client,
        polkadot_test_runtime::Call::Xcm(pallet_xcm::Call::execute(
            Box::new(msg.clone()),
            1_000_000_000,
        )),
        sp_keyring::Sr25519Keyring::Alice,
    );

    let mut block_builder = client.init_polkadot_block_builder();
    block_builder.push_polkadot_extrinsic(execute).expect("pushes extrinsic");

    let block = block_builder.build().expect("Finalizes the block").block;
    let block_hash = block.hash();

    futures::executor::block_on(client.import(sp_consensus::BlockOrigin::Own, block))
        .expect("imports the block");

    client
        .state_at(&BlockId::Hash(block_hash))
        .expect("state should exist")
        .inspect_state(|| {
            assert!(polkadot_test_runtime::System::events().iter().any(|r| matches!(
                r.event,
                polkadot_test_runtime::Event::Xcm(pallet_xcm::Event::Attempted(
                    Outcome::Incomplete(_, XcmError::RecursionLimitReached),
                )),
            )));
        });
}
/*
	Copyright 2021 Integritee AG and Supercomputing Systems AG

	Licensed under the Apache License, Version 2.0 (the "License");
	you may not use this file except in compliance with the License.
	You may obtain a copy of the License at

		http://www.apache.org/licenses/LICENSE-2.0

	Unless required by applicable law or agreed to in writing, software
	distributed under the License is distributed on an "AS IS" BASIS,
	WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
	See the License for the specific language governing permissions and
	limitations under the License.

*/

use crate::{
	trusted_command_utils::{
		get_accountid_from_str, get_identifiers, get_keystore_path, get_pair_from_str,
	},
	trusted_operation::perform_trusted_operation,
	Cli,
};
use codec::Decode;
use ita_stf::{Index, KeyPair, TrustedCall, TrustedGetter, TrustedOperation};
use log::*;
use my_node_runtime::Balance;
use sp_application_crypto::{ed25519, sr25519};
use sp_core::{crypto::Ss58Codec, sr25519 as sr25519_core, Pair};
use substrate_client_keystore::{KeystoreExt, LocalKeystore};

fn play_turn(cli: &Cli, trusted_args: &TrustedArgs, player: &str, column: u8) {
	let player = get_pair_from_str(trusted_args, arg_player);
	println!("player ss58 is {}", player.public().to_ss58check());
	println!("column choice is {:?}", column);

	println!("send trusted call play-turn from {} with column {:?}", player.public(), column);
	let (mrenclave, shard) = get_identifiers(trusted_args);
	let nonce = get_layer_two_nonce!(from, cli, trusted_args);

	let top: TrustedOperation = TrustedCall::connectfour_play_turn(
		sr25519_core::Public::from(player.public()).into(),
		column,
	)
	.sign(&KeyPair::Sr25519(player), nonce, &mrenclave, &shard)
	.into_trusted_operation(direct);

	let _ = perform_operation(cli, trusted_args, &top);
}

fn get_board(cli: &Cli, trusted_args: &TrustedArgs, account_id: &str, column: u8) {
	let player = get_pair_from_str(trusted_args, arg_player);

	let key_pair = sr25519_core::Pair::from(who.clone());

	let top: TrustedOperation =
		TrustedGetter::board(sr25519_core::Public::from(who.public()).into())
			.sign(&KeyPair::Sr25519(key_pair))
			.into();
	let res = perform_operation(cli, trusted_args, &top);
	debug!("received result for board");
	if let Some(v) = res {
		if let Ok(board) = crate::SgxBoardStruct::decode(&mut v.as_slice()) {
			println!("Last turn in block number: {}", board.last_turn);
			println!("Next player: {}", board.next_player);
			println!("Board state: {:?}", board.board_state);
			println!("Board:");
			for row in 0..6 {
				for column in 0..7 {
					print!(" {} ", board.board[column][row]);
				}
				println!()
			}
			println!("=====================");
			for column in 0..7 {
				print!(" {} ", column);
			}
			println!();
		} else {
			println!("could not decode board. maybe hasn't been set? {:x?}", v);
		}
	} else {
		println!("could not fetch board");
	};

	Ok(())
}

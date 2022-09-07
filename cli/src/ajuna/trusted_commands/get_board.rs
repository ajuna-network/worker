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

//! Query board state for account in keystore.

use crate::{
	trusted_command_utils::get_pair_from_str, trusted_commands::TrustedArgs,
	trusted_operation::perform_trusted_operation, Cli,
};
use codec::Decode;
use ita_stf::{KeyPair, SgxGameBoardStruct, TrustedGetter, TrustedOperation};
use log::*;
use sp_core::Pair;

#[derive(Parser)]
pub struct GetBoardCommand {
	/// Player's incognito AccountId in ss58check format
	player: String,
}

impl GetBoardCommand {
	pub(crate) fn run(&self, cli: &Cli, trusted_args: &TrustedArgs) {
		let player = get_pair_from_str(trusted_args, arg_player);
		let top: TrustedOperation = TrustedGetter::board(player.public().into())
			.sign(&KeyPair::Sr25519(player))
			.into();
		let res = perform_trusted_operation(cli, trusted_args, &top);
		debug!("received result for board");
		if let Some(v) = res {
			if let Ok(board) = SgxGameBoardStruct::decode(&mut v.as_slice()) {
				println!("Board: {:?}", board.state);
			} else {
				println!("could not decode board. maybe hasn't been set? {:x?}", v);
			}
		} else {
			println!("could not fetch board");
		};
	}
}

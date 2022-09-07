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

//! Dispute a board

use crate::{
	get_layer_two_nonce,
	trusted_command_utils::{get_identifiers, get_pair_from_str},
	trusted_commands::TrustedArgs,
	trusted_operation::perform_trusted_operation,
	Cli,
};
use ita_stf::{KeyPair, TrustedCall, TrustedOperation};
use sp_core::{crypto::Ss58Codec, Pair};
#[derive(Parser)]
pub struct DisputeCommand {
	/// Player's incognito AccountId in ss58check format
	player: String,
	/// The board id
	board_id: ita_stf::SgxBoardId,
}

impl DisputeCommand {
	pub(crate) fn run(&self, cli: &Cli, trusted_args: &TrustedArgs) {
		let player = get_pair_from_str(trusted_args, arg_player);
		println!("player ss58 is {}", player.public().to_ss58check());

		println!(
			"send trusted call dispute-game from {} for board {:?}",
			player.public(),
			board_id
		);
		let (mrenclave, shard) = get_identifiers(trusted_args);
		let nonce = get_layer_two_nonce!(player, cli, trusted_args);

		let top: TrustedOperation =
			TrustedCall::board_dispute_game(player.public().into(), *board_id)
				.sign(&KeyPair::Sr25519(player), nonce, &mrenclave, &shard)
				.into_trusted_operation(trusted_args.direct);

		let _ = perform_trusted_operation(cli, trusted_args, &top);
	}
}

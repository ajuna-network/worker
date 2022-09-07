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

//! Play a turn of a board game

use crate::{ajuna::TrustedArgs, trusted_command_utils::play_turn, Cli};
use ita_stf::{Coordinates, SgxGameTurn};

#[derive(Parser)]
pub struct DropBombCommand {
	/// Player's incognito AccountId in ss58check format
	player: String,
	// Column
	col: u8,
	// Row
	row: u8,
}

impl DropBombCommand {
	pub(crate) fn run(&self, cli: &Cli, trusted_args: &TrustedArgs) {
		play_turn(
			cli,
			trusted_args,
			player,
			SgxGameTurn::DropBomb(Coordinates { col: *col, row: *row }),
		)
	}
}

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

use crate::{
	trusted_command_utils::play_turn,
	Cli,
};

pub struct SideCommand(Side);
use std::str::FromStr;

impl FromStr for SideCommand {
	type Err = &'static str;
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		if s.eq("north") {
			return Ok(SideCommand(Side::North))
		}
		if s.eq("east") {
			return Ok(SideCommand(Side::East))
		}
		if s.eq("south") {
			return Ok(SideCommand(Side::South))
		}
		if s.eq("west") {
			return Ok(SideCommand(Side::West))
		}
		Err("Invalid side")
	}
}

#[derive(Parser)]
pub struct DropStoneCommand {
	/// Player's incognito AccountId in ss58check format
	player: String,
	side: SideCommand,
	n: u8,
}

impl DropStoneCommand {
	pub(crate) fn run(&self, cli: &Cli, trusted_args: &TrustedArgs) {
		play_turn(cli, trusted_args, player, SgxGameTurn::DropStone(((*side).0.clone(), *n))),
	}
}

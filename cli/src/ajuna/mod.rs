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
	ajuna::{
		commands::queue_game::QueueGameCommand,
		trusted_commands::{
			dispute::DisputeCommand, drop_bomb::DropBombCommand, drop_stone::DropStoneCommand,
			get_board::GetBoardCommand,
		},
	},
	trusted_commands::TrustedArgs,
	Cli,
};

mod commands;
mod trusted_commands;

#[derive(Subcommand)]
pub enum AjunaPublicCli {
	/// query parentchain balance for AccountId
	QueueGame(QueueGameCommand),
}

impl AjunaPublicCli {
	pub fn run(&self, cli: &Cli) {
		match self {
			AjunaPublicCli::QueueGame(cmd) => cmd.run(cli),
		}
	}
}

#[derive(Subcommand)]
pub enum AjunaTrustedCommands {
	DropBomb(DropBombCommand),
	DropStone(DropStoneCommand),
	GetBoard(GetBoardCommand),
	Dispute(DisputeCommand),
}

impl AjunaTrustedCommands {
	pub fn run(&self, cli: &Cli, trusted_args: &TrustedArgs) {
		match self {
			AjunaTrustedCommands::DropBomb(cmd) => cmd.run(cli, trusted_args),
			AjunaTrustedCommands::DropStone(cmd) => cmd.run(cli, trusted_args),
			AjunaTrustedCommands::GetBoard(cmd) => cmd.run(cli, trusted_args),
			AjunaTrustedCommands::Dispute(cmd) => cmd.run(cli, trusted_args),
		}
	}
}

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

use crate::{benchmark::BenchmarkCommands, Cli};

#[cfg(feature = "evm")]
use crate::evm::EvmCommands;
use crate::trusted_base_cli::TrustedBaseCli;

#[derive(Args)]
pub struct TrustedArgs {
	/// targeted worker MRENCLAVE
	#[clap(short, long)]
	pub(crate) mrenclave: String,

	/// shard identifier
	#[clap(short, long)]
	pub(crate) shard: Option<String>,

	/// signer for publicly observable extrinsic
	#[clap(short='a', long, default_value_t = String::from("//Alice"))]
	pub(crate) xt_signer: String,

	/// insert if direct invocation call is desired
	#[clap(short, long)]
	pub(crate) direct: bool,

	#[clap(subcommand)]
	pub(crate) command: TrustedCommands,
}

#[derive(Subcommand)]
pub enum TrustedCommands {
	#[clap(flatten)]
	BaseTrusted(TrustedBaseCli),

	#[cfg(feature = "evm")]
	#[clap(flatten)]
	EvmCommands(EvmCommands),

	#[clap(flatten)]
	Ajuna(AjunaCommands),

	/// Run Benchmark
	Benchmark(BenchmarkCommands),


}

	/// Play a turn of a board game
	DropBomb {
		/// Player's incognito AccountId in ss58check format
		player: String,
		// Column
		col: u8,
		// Row
		row: u8,
	},

	DropStone {
		player: String,
		side: SideCommand,
		n: u8,
	},

	/// Query board state for account in keystore
	GetBoard {
		/// Player's incognito AccountId in ss58check format
		player: String,
	},

	/// Dispute a board
	Dispute {
		/// Player's incognito AccountId in ss58check format
		player: String,
		/// The board id
		board_id: ita_stf::SgxBoardId,
	},
}

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

impl TrustedArgs {
	pub(crate) fn run(&self, cli: &Cli) {
		match &self.command {
			TrustedCommands::BaseTrusted(cmd) => cmd.run(cli, self),
			TrustedCommands::Benchmark(benchmark_commands) => benchmark_commands.run(cli, self),
			TrustedCommands::DropBomb { player, col, row } => play_turn(
				cli,
				trusted_args,
				player,
				SgxGameTurn::DropBomb(Coordinates { col: *col, row: *row }),
			),
			TrustedCommands::DropStone { player, side, n } =>
				play_turn(cli, trusted_args, player, SgxGameTurn::DropStone(((*side).0.clone(), *n))),
			TrustedCommands::GetBoard { player } => get_board(cli, trusted_args, player),
			TrustedCommands::Dispute { player, board_id } =>
				dispute_game(cli, trusted_args, player, board_id),
			#[cfg(feature = "evm")]
			TrustedCommands::EvmCommands(evm_commands) => evm_commands.run(cli, self),
		}
	}
}

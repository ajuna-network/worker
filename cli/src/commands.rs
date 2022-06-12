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

extern crate chrono;
use crate::{base_cli::BaseCli, trusted_commands::TrustedArgs, Cli};
use clap::Subcommand;

#[cfg(feature = "teeracle")]
use crate::exchange_oracle::ExchangeOracleSubCommand;

#[derive(Subcommand)]
pub enum Commands {
	#[clap(flatten)]
	Base(BaseCli),

	/// Sign up for a game, ready to be matched
	QueueGame {
		/// To be registered AccountId in ss58check format
		who: String,
	},

	/// trusted calls to worker enclave
	#[clap(after_help = "stf subcommands depend on the stf crate this has been built against")]
	Trusted(TrustedArgs),

	/// Subcommands for the exchange oracle.
	#[cfg(feature = "teeracle")]
	#[clap(subcommand)]
	ExchangeOracle(ExchangeOracleSubCommand),
}

pub fn match_command(cli: &Cli) {
	match &cli.command {
		Commands::Base(cmd) => cmd.run(cli),
		Commands::Trusted(cmd) => cmd.run(cli),
		#[cfg(feature = "teeracle")]
		Commands::ExchangeOracle(cmd) => cmd.run(cli),
		Commands::QueueGame { who } => queue_game(cli, who),
	};
}

fn queue_game(cli: &Cli, who: &str) {
	let who = get_pair_from_str(who);
	info!("Queueing player: {}", who.public().to_ss58check());
	let api = get_chain_api(cli).set_signer(sr25519_core::Pair::from(who));
	let xt: UncheckedExtrinsicV4<([u8; 2])> = compose_extrinsic!(api, REGISTRY, "queue");
	let tx_hash = api.send_extrinsic(xt.hex_encode(), XtStatus::InBlock).unwrap();
	println!("[+] Successfully registered player in game queue. Extrinsic Hash: {:?}\n", tx_hash);
}

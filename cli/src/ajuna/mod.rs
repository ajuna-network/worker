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
	base_cli::commands::{
		balance::BalanceCommand, faucet::FaucetCommand, listen::ListenCommand,
		shield_funds::ShieldFundsCommand, transfer::TransferCommand,
	},
	command_utils::*,
	Cli,
};
use base58::ToBase58;
use chrono::{DateTime, Utc};
use clap::Subcommand;
use itc_rpc_client::direct_client::DirectApi;
use itp_node_api::api_client::PalletTeerexApi;
use sp_application_crypto::{ed25519, sr25519};
use sp_core::{crypto::Ss58Codec, Pair};
use std::{
	path::PathBuf,
	time::{Duration, UNIX_EPOCH},
};
use substrate_api_client::Metadata;
use substrate_client_keystore::{KeystoreExt, LocalKeystore};

mod commands;

#[derive(Subcommand)]
pub enum AjunaCli {
	/// query parentchain balance for AccountId
	QueueGame(QueueGameCommand),
}

impl AjunaCli {
	pub fn run(&self, cli: &Cli) {
		match self {
			AjunaCli::QueueGame(cmd) => cmd.run(cli),
		}
	}
}


#[derive(Subcommand)]
pub enum AjunaTrustedCli {
	/// query parentchain balance for AccountId
	QueueGame(QueueGameCommand),
}

impl AjunaTrustedCli {
	pub fn run(&self, cli: &Cli) {
		match self {
			AjunaTrustedCli::QueueGame(cmd) => cmd.run(cli),
		}
	}
}

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

//! Queue a game on the parentchain:
//! Sign up for a game, ready to be matched

use crate::{
	command_utils::{get_chain_api, get_pair_from_str},
	Cli,
};
use itp_node_api::metadata::pallet_ajuna_game_registry::GAME_REGISTRY;
use log::*;
use sp_application_crypto::{Pair, Ss58Codec};
use sp_core::sr25519 as sr25519_core;
use substrate_api_client::{compose_extrinsic, UncheckedExtrinsicV4, XtStatus};

#[derive(Parser)]
pub struct QueueGameCommand {
	/// To be registered AccountId in ss58check format.
	account: String,
}

impl QueueGameCommand {
	pub(crate) fn run(&self, cli: &Cli) {
		let who = get_pair_from_str(&self.account);
		info!("Queueing player: {}", who.public().to_ss58check());
		let api = get_chain_api(cli).set_signer(sr25519_core::Pair::from(who));
		let xt: UncheckedExtrinsicV4<([u8; 2]), _> =
			compose_extrinsic!(api, GAME_REGISTRY, "queue");
		let tx_hash = api.send_extrinsic(xt.hex_encode(), XtStatus::InBlock).unwrap();
		println!(
			"[+] Successfully registered player in game queue. Extrinsic Hash: {:?}\n",
			tx_hash
		);
	}
}

use crate::{
	error::{Error, Result},
	traits::StfExecuteGames,
};
use ajuna_common::RunnerState;
use codec::Decode;
use ita_stf::{AccountId, ShardIdentifier, Stf, TrustedCall, TrustedCallSigned};
use itp_node_api::metadata::pallet_ajuna_runner::{GameId, RUNNER};
use itp_ocall_api::{EnclaveAttestationOCallApi, EnclaveOnChainOCallApi};
use itp_sgx_externalities::SgxExternalitiesTrait;
use itp_stf_state_handler::handle_state::HandleState;
use itp_storage::{storage_map_key, StorageHasher};
use itp_types::{OpaqueCall, H256};
use log::*;
use pallet_ajuna_gameregistry::Game;
use sp_core::ed25519;
use sp_runtime::traits::Block as ParentchainBlockTrait;
use std::{collections::BTreeSet, sync::Arc, vec::Vec};

pub struct StfGameExecutor<OCallApi, StateHandler> {
	state_handler: Arc<StateHandler>,
	ocall_api: Arc<OCallApi>,
}

impl<OCallApi, StateHandler> StfGameExecutor<OCallApi, StateHandler>
where
	OCallApi: EnclaveAttestationOCallApi + EnclaveOnChainOCallApi,
	StateHandler: HandleState<HashType = H256>,
	StateHandler::StateT: SgxExternalitiesTrait,
{
	pub fn new(state_handler: Arc<StateHandler>, ocall_api: Arc<OCallApi>) -> Self {
		Self { state_handler, ocall_api }
	}
}

impl<OCallApi, StateHandler> StfExecuteGames for StfGameExecutor<OCallApi, StateHandler>
where
	OCallApi: EnclaveAttestationOCallApi + EnclaveOnChainOCallApi,
	StateHandler: HandleState<HashType = H256>,
	StateHandler::StateT: SgxExternalitiesTrait,
{
	fn new_game<ParentchainBlock>(
		&self,
		game_id: GameId,
		shard: &ShardIdentifier,
		block: &ParentchainBlock,
	) -> Result<GameId>
	where
		ParentchainBlock: ParentchainBlockTrait<Hash = H256>,
	{
		// FIXME: We should take this from metadata.
		let runner_key = storage_map_key(RUNNER, "Runners", &game_id, &StorageHasher::Blake2_128);
		let game_entry: Option<RunnerState> =
			self.ocall_api.get_storage_verified(runner_key, block.header())?.into_tuple().1;

		match game_entry {
			Some(runner) => {
				let (state_lock, mut state) = self.state_handler.load_for_mutation(shard)?;
				let root = Stf::get_root(&mut state);
				let nonce = Stf::account_nonce(&mut state, &root);

				if let RunnerState::Accepted(mut runner_state) = runner {
					if let Ok(game) = Game::<AccountId>::decode(&mut runner_state) {
						if game.players.len() == 2 {
							let player_one = game.players[0].clone();
							let player_two = game.players[1].clone();

							let trusted_call = TrustedCallSigned::new(
								TrustedCall::board_new_game(
									root,
									game_id,
									BTreeSet::from([player_one, player_two]),
								),
								nonce,
								ed25519::Signature::from_raw([0u8; 64]).into(), //don't care about signature here
							);

							Stf::execute(
								&mut state,
								trusted_call,
								&mut Vec::<OpaqueCall>::new(),
								[0u8, 1u8],
							)
							.map_err::<Error, _>(|e| e.into())?;

							self.state_handler
								.write_after_mutation(state, state_lock, shard)
								.expect("write after mutation");
							// .map_err(|e| e.into());

							Ok(game_id)
						} else {
							error!("Game {} does not have 2 players", game_id);
							Ok(game_id)
						}
					} else {
						error!("Game {} failed decoding", game_id);
						Ok(game_id)
					}
				} else {
					error!("Game {} is not queued!", game_id);
					Ok(game_id)
				}
			},
			None => {
				error!("No game entry found for game {}", game_id);
				Ok(game_id)
			},
		}
	}

	fn finish_game(&self, game_id: GameId, shard: &ShardIdentifier) -> Result<GameId> {
		let (state_lock, mut state) = self.state_handler.load_for_mutation(shard)?;
		let root = Stf::get_root(&mut state);
		let nonce = Stf::account_nonce(&mut state, &root);
		let trusted_call = TrustedCallSigned::new(
			TrustedCall::board_finish_game(root, game_id),
			nonce,
			ed25519::Signature::from_raw([0u8; 64]).into(), //don't care about signature here
		);

		Stf::execute(&mut state, trusted_call, &mut Vec::<OpaqueCall>::new(), [0u8, 1u8])
			.map_err::<Error, _>(|e| e.into())?;

		self.state_handler
			.write_after_mutation(state, state_lock, shard)
			.expect("write after mutation");

		Ok(game_id)
	}
}

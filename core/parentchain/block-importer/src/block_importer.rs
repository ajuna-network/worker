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

//! Imports parentchain blocks and executes any indirect calls found in the extrinsics.

use log::*;
use pallet_ajuna_gameregistry::{game::GameEngine, Queue};
use sp_runtime::{
	generic::SignedBlock as SignedBlockG,
	traits::{Block as ParentchainBlockTrait, NumberFor},
};
use std::{marker::PhantomData, sync::Arc, vec, vec::Vec};

use ita_stf::ParentchainHeader;
use itc_parentchain_indirect_calls_executor::ExecuteIndirectCalls;
use itc_parentchain_light_client::{
	concurrent_access::ValidatorAccess, BlockNumberOps, LightClientState, Validator,
};
use itp_extrinsics_factory::CreateExtrinsics;
use itp_ocall_api::{EnclaveAttestationOCallApi, EnclaveOnChainOCallApi};
use itp_registry_storage::{RegistryStorage, RegistryStorageKeys};
use itp_settings::node::{
	ACK_GAME, GAME_REGISTRY_MODULE, PROCESSED_PARENTCHAIN_BLOCK, TEEREX_MODULE,
};
use itp_stf_executor::traits::{StfExecuteShieldFunds, StfExecuteTrustedCall, StfUpdateState};
use itp_stf_state_handler::query_shard_state::QueryShardState;
use itp_storage_verifier::GetStorageVerified;
use itp_types::{OpaqueCall, H256};

use crate::{
	beefy_merkle_tree::{merkle_root, Keccak256},
	error::Result,
	ImportParentchainBlocks,
};

/// Parentchain block import implementation.
pub struct ParentchainBlockImporter<
	ParentchainBlock,
	ValidatorAccessor,
	OCallApi,
	StfExecutor,
	ExtrinsicsFactory,
	IndirectCallsExecutor,
	StateHandler,
> where
	ParentchainBlock: ParentchainBlockTrait<Hash = H256>,
	NumberFor<ParentchainBlock>: BlockNumberOps,
	ValidatorAccessor: ValidatorAccess<ParentchainBlock>,
	OCallApi: EnclaveOnChainOCallApi + EnclaveAttestationOCallApi,
	StfExecutor: StfUpdateState + StfExecuteTrustedCall + StfExecuteShieldFunds,
	ExtrinsicsFactory: CreateExtrinsics,
	IndirectCallsExecutor: ExecuteIndirectCalls,
	StateHandler: QueryShardState,
{
	validator_accessor: Arc<ValidatorAccessor>,
	ocall_api: Arc<OCallApi>,
	stf_executor: Arc<StfExecutor>,
	extrinsics_factory: Arc<ExtrinsicsFactory>,
	indirect_calls_executor: Arc<IndirectCallsExecutor>,
	file_state_handler: Arc<StateHandler>,
	_phantom: PhantomData<ParentchainBlock>,
}

impl<
		ParentchainBlock,
		ValidatorAccessor,
		OCallApi,
		StfExecutor,
		ExtrinsicsFactory,
		IndirectCallsExecutor,
		StateHandler,
	>
	ParentchainBlockImporter<
		ParentchainBlock,
		ValidatorAccessor,
		OCallApi,
		StfExecutor,
		ExtrinsicsFactory,
		IndirectCallsExecutor,
		StateHandler,
	> where
	ParentchainBlock: ParentchainBlockTrait<Hash = H256, Header = ParentchainHeader>,
	NumberFor<ParentchainBlock>: BlockNumberOps,
	ValidatorAccessor: ValidatorAccess<ParentchainBlock>,
	OCallApi: EnclaveOnChainOCallApi + EnclaveAttestationOCallApi,
	StfExecutor: StfUpdateState + StfExecuteTrustedCall + StfExecuteShieldFunds,
	ExtrinsicsFactory: CreateExtrinsics,
	IndirectCallsExecutor: ExecuteIndirectCalls,
	StateHandler: QueryShardState,
{
	pub fn new(
		validator_accessor: Arc<ValidatorAccessor>,
		ocall_api: Arc<OCallApi>,
		stf_executor: Arc<StfExecutor>,
		extrinsics_factory: Arc<ExtrinsicsFactory>,
		indirect_calls_executor: Arc<IndirectCallsExecutor>,
		file_state_handler: Arc<StateHandler>,
	) -> Self {
		ParentchainBlockImporter {
			validator_accessor,
			ocall_api,
			stf_executor,
			extrinsics_factory,
			indirect_calls_executor,
			_phantom: Default::default(),
			file_state_handler,
		}
	}
}

impl<
		ParentchainBlock,
		ValidatorAccessor,
		OCallApi,
		StfExecutor,
		ExtrinsicsFactory,
		IndirectCallsExecutor,
		StateHandler,
	> ImportParentchainBlocks
	for ParentchainBlockImporter<
		ParentchainBlock,
		ValidatorAccessor,
		OCallApi,
		StfExecutor,
		ExtrinsicsFactory,
		IndirectCallsExecutor,
		StateHandler,
	> where
	ParentchainBlock: ParentchainBlockTrait<Hash = H256, Header = ParentchainHeader>,
	NumberFor<ParentchainBlock>: BlockNumberOps,
	ValidatorAccessor: ValidatorAccess<ParentchainBlock>,
	OCallApi: EnclaveOnChainOCallApi + EnclaveAttestationOCallApi + GetStorageVerified,
	StfExecutor: StfUpdateState + StfExecuteTrustedCall + StfExecuteShieldFunds,
	ExtrinsicsFactory: CreateExtrinsics,
	IndirectCallsExecutor: ExecuteIndirectCalls,
	StateHandler: QueryShardState,
{
	type SignedBlockType = SignedBlockG<ParentchainBlock>;

	fn import_parentchain_blocks(
		&self,
		blocks_to_import: Vec<Self::SignedBlockType>,
	) -> Result<()> {
		let mut calls = Vec::<OpaqueCall>::new();

		debug!("Import blocks to light-client!");
		for signed_block in blocks_to_import.into_iter() {
			let block = signed_block.block;
			let justifications = signed_block.justifications.clone();

			// Check if there are any extrinsics in the to-be-imported block that we sent and cached in the light-client before.
			// If so, remove them now from the cache.
			if let Err(e) = self.validator_accessor.execute_mut_on_validator(|v| {
				v.check_xt_inclusion(v.num_relays(), &block)?;

				v.submit_simple_header(v.num_relays(), block.header().clone(), justifications)
			}) {
				error!("[Validator] Header submission failed: {:?}", e);
				return Err(e.into())
			}

			// Perform state updates.
			if let Err(e) = self.stf_executor.update_states(block.header()) {
				error!("Error performing state updates upon block import");
				return Err(e.into())
			}

			// Execute indirect calls that were found in the extrinsics of the block,
			// incl. shielding and unshielding.
			match self.indirect_calls_executor.execute_indirect_calls_in_extrinsics(&block) {
				Ok((unshielding_call_confirmations, executed_shielding_calls)) => {
					// Include all unshielding confirmations that need to be executed on the parentchain.
					calls.extend(unshielding_call_confirmations.into_iter());
					// Include a processed parentchain block confirmation for each block.
					calls.push(create_processed_parentchain_block_call(
						block.hash(),
						executed_shielding_calls,
					));
				},
				Err(_) => error!("Error executing relevant extrinsics"),
			};

			// FIXME: Putting these blocks below in a separate function would be a little bit cleaner
			let maybe_queue: Option<Queue<H256>> = self
				.ocall_api
				.get_storage_verified(RegistryStorage::queue_game(), block.header())?
				.into_tuple()
				.1;
			match maybe_queue {
				Some(mut queue) => {
					//FIXME: if this would be a separate function, we could return here upon if queue.is_empty() check.
					if !queue.is_empty() {
						//FIXME: hardcoded, because currently hardcoded in the GameRegistry pallet.
						let game_engine = GameEngine::new(1u8, 1u8);
						let mut games = Vec::<H256>::new();
						while let Some(game) = queue.dequeue() {
							games.push(game)
						}
						//FIXME: we currently only take the first shard. How we handle sharding in general?
						let shard = self.file_state_handler.list_shards()?[0];
						let opaque_call = OpaqueCall::from_tuple(&(
							[GAME_REGISTRY_MODULE, ACK_GAME],
							&game_engine,
							games,
							shard,
						));
						let calls = vec![opaque_call];

						// Create extrinsic for acknowledge game.
						let ack_game_extrinsic =
							self.extrinsics_factory.create_extrinsics(calls.as_slice())?;

						// Sending the extrinsic requires mut access because the validator caches the sent extrinsics internally.
						self.validator_accessor.execute_mut_on_validator(|v| {
							v.send_extrinsics(self.ocall_api.as_ref(), ack_game_extrinsic)
						})?;
					}
				},
				None => {
					debug!("No game queued in GameRegistry pallet.");
				},
			}

			info!(
				"Successfully imported parentchain block (number: {}, hash: {})",
				block.header().number,
				block.header().hash()
			);
		}

		// Create extrinsics for all `unshielding` and `block processed` calls we've gathered.
		let parentchain_extrinsics = self.extrinsics_factory.create_extrinsics(calls.as_slice())?;

		// Sending the extrinsic requires mut access because the validator caches the sent extrinsics internally.
		self.validator_accessor.execute_mut_on_validator(|v| {
			v.send_extrinsics(self.ocall_api.as_ref(), parentchain_extrinsics)
		})?;

		Ok(())
	}
}

/// Creates a processed_parentchain_block extrinsic for a given parentchain block hash and the merkle executed extrinsics.
///
/// Calculates the merkle root of the extrinsics. In case no extrinsics are supplied, the root will be a hash filled with zeros.
fn create_processed_parentchain_block_call(block_hash: H256, extrinsics: Vec<H256>) -> OpaqueCall {
	let root: H256 = merkle_root::<Keccak256, _, _>(extrinsics).into();
	OpaqueCall::from_tuple(&([TEEREX_MODULE, PROCESSED_PARENTCHAIN_BLOCK], block_hash, root))
}

#[cfg(test)]
pub mod tests {
	use super::*;
	use codec::Encode;

	#[test]
	fn ensure_empty_extrinsic_vec_triggers_zero_filled_merkle_root() {
		// given
		let block_hash = H256::from([1; 32]);
		let extrinsics = Vec::new();
		let expected_call =
			([TEEREX_MODULE, PROCESSED_PARENTCHAIN_BLOCK], block_hash, H256::default()).encode();

		// when
		let call = create_processed_parentchain_block_call(block_hash, extrinsics);

		// then
		assert_eq!(call.0, expected_call);
	}

	#[test]
	fn ensure_non_empty_extrinsic_vec_triggers_non_zero_merkle_root() {
		// given
		let block_hash = H256::from([1; 32]);
		let extrinsics = vec![H256::from([4; 32]), H256::from([9; 32])];
		let zero_root_call =
			([TEEREX_MODULE, PROCESSED_PARENTCHAIN_BLOCK], block_hash, H256::default()).encode();

		// when
		let call = create_processed_parentchain_block_call(block_hash, extrinsics);

		// then
		assert_ne!(call.0, zero_root_call);
	}
}

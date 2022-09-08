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

use crate::{
	error::{Error, Result},
	ImportParentchainBlocks,
};
use ita_stf::ParentchainHeader;
use itc_parentchain_indirect_calls_executor::ExecuteIndirectCalls;
use itc_parentchain_light_client::{
	concurrent_access::ValidatorAccess, BlockNumberOps, ExtrinsicSender, LightClientState,
	Validator,
};
use itp_extrinsics_factory::CreateExtrinsics;
use itp_node_api::metadata::{
	pallet_ajuna_game_registry::{GameRegistryCallIndexes, GameRegistryStorageIndexes},
	pallet_ajuna_runner::GameId,
	provider::AccessNodeMetadata,
};
use itp_ocall_api::EnclaveOnChainOCallApi;
use itp_stf_executor::traits::StfUpdateState;
use itp_stf_state_handler::query_shard_state::QueryShardState;
use itp_types::{OpaqueCall, H256};
use log::*;
use sp_runtime::{
	generic::SignedBlock as SignedBlockG,
	traits::{Block as ParentchainBlockTrait, NumberFor},
};
use std::{format, marker::PhantomData, sync::Arc, vec::Vec};

/// Parentchain block import implementation.
pub struct ParentchainBlockImporter<
	ParentchainBlock,
	ValidatorAccessor,
	StfExecutor,
	ExtrinsicsFactory,
	IndirectCallsExecutor,
	OCallApi,
	StateHandler,
	NodeMetadataProvider,
> where
	ParentchainBlock: ParentchainBlockTrait<Hash = H256>,
	NumberFor<ParentchainBlock>: BlockNumberOps,
	ValidatorAccessor: ValidatorAccess<ParentchainBlock>,
	StfExecutor: StfUpdateState,
	ExtrinsicsFactory: CreateExtrinsics,
	IndirectCallsExecutor: ExecuteIndirectCalls,
	StateHandler: QueryShardState,
	OCallApi: EnclaveOnChainOCallApi,
	NodeMetadataProvider: AccessNodeMetadata,
	NodeMetadataProvider::MetadataType: GameRegistryCallIndexes,
{
	validator_accessor: Arc<ValidatorAccessor>,
	stf_executor: Arc<StfExecutor>,
	extrinsics_factory: Arc<ExtrinsicsFactory>,
	indirect_calls_executor: Arc<IndirectCallsExecutor>,
	file_state_handler: Arc<StateHandler>,
	ocall_api: Arc<OCallApi>,
	node_meta_data_provider: Arc<NodeMetadataProvider>,
	_phantom: PhantomData<ParentchainBlock>,
}

impl<
		ParentchainBlock,
		ValidatorAccessor,
		StfExecutor,
		ExtrinsicsFactory,
		IndirectCallsExecutor,
		StateHandler,
		OCallApi,
		NodeMetadataProvider,
	>
	ParentchainBlockImporter<
		ParentchainBlock,
		ValidatorAccessor,
		StfExecutor,
		ExtrinsicsFactory,
		IndirectCallsExecutor,
		StateHandler,
		OCallApi,
		NodeMetadataProvider,
	> where
	ParentchainBlock: ParentchainBlockTrait<Hash = H256, Header = ParentchainHeader>,
	NumberFor<ParentchainBlock>: BlockNumberOps,
	ValidatorAccessor: ValidatorAccess<ParentchainBlock>,
	StfExecutor: StfUpdateState,
	ExtrinsicsFactory: CreateExtrinsics,
	IndirectCallsExecutor: ExecuteIndirectCalls,
	StateHandler: QueryShardState,
	OCallApi: EnclaveOnChainOCallApi,
	NodeMetadataProvider: AccessNodeMetadata,
	NodeMetadataProvider::MetadataType: GameRegistryCallIndexes,
{
	pub fn new(
		validator_accessor: Arc<ValidatorAccessor>,
		stf_executor: Arc<StfExecutor>,
		extrinsics_factory: Arc<ExtrinsicsFactory>,
		indirect_calls_executor: Arc<IndirectCallsExecutor>,
		file_state_handler: Arc<StateHandler>,
		ocall_api: Arc<OCallApi>,
		node_meta_data_provider: Arc<NodeMetadataProvider>,
	) -> Self {
		ParentchainBlockImporter {
			validator_accessor,
			stf_executor,
			extrinsics_factory,
			indirect_calls_executor,
			file_state_handler,
			ocall_api,
			node_meta_data_provider,
			_phantom: Default::default(),
		}
	}
}

impl<
		ParentchainBlock,
		ValidatorAccessor,
		StfExecutor,
		ExtrinsicsFactory,
		IndirectCallsExecutor,
		StateHandler,
		OCallApi,
		NodeMetadataProvider,
	> ImportParentchainBlocks
	for ParentchainBlockImporter<
		ParentchainBlock,
		ValidatorAccessor,
		StfExecutor,
		ExtrinsicsFactory,
		IndirectCallsExecutor,
		StateHandler,
		OCallApi,
		NodeMetadataProvider,
	> where
	ParentchainBlock: ParentchainBlockTrait<Hash = H256, Header = ParentchainHeader>,
	NumberFor<ParentchainBlock>: BlockNumberOps,
	ValidatorAccessor: ValidatorAccess<ParentchainBlock>,
	StfExecutor: StfUpdateState,
	ExtrinsicsFactory: CreateExtrinsics,
	IndirectCallsExecutor: ExecuteIndirectCalls,
	StateHandler: QueryShardState,
	OCallApi: EnclaveOnChainOCallApi,
	NodeMetadataProvider: AccessNodeMetadata,
	NodeMetadataProvider::MetadataType: GameRegistryCallIndexes + GameRegistryStorageIndexes,
{
	type SignedBlockType = SignedBlockG<ParentchainBlock>;

	fn import_parentchain_blocks(
		&self,
		blocks_to_import: Vec<Self::SignedBlockType>,
	) -> Result<()> {
		let mut calls = Vec::<OpaqueCall>::new();

		debug!("Import blocks to light-client!");
		for signed_block in blocks_to_import.into_iter() {
			// Check if there are any extrinsics in the to-be-imported block that we sent and cached in the light-client before.
			// If so, remove them now from the cache.
			if let Err(e) = self.validator_accessor.execute_mut_on_validator(|v| {
				v.check_xt_inclusion(v.num_relays(), &signed_block.block)?;

				v.submit_block(v.num_relays(), &signed_block)
			}) {
				error!("[Validator] Header submission failed: {:?}", e);
				return Err(e.into())
			}

			let block = signed_block.block;
			// Perform state updates.
			if let Err(e) = self.stf_executor.update_states(block.header()) {
				error!("Error performing state updates upon block import");
				return Err(e.into())
			}

			// Execute indirect calls that were found in the extrinsics of the block,
			// incl. shielding and unshielding.
			match self.indirect_calls_executor.execute_indirect_calls_in_extrinsics(&block) {
				Ok(executed_shielding_calls) => {
					calls.push(executed_shielding_calls);
				},
				Err(_) => error!("Error executing relevant extrinsics"),
			};

			info!(
				"Successfully imported parentchain block (number: {}, hash: {})",
				block.header().number,
				block.header().hash()
			);

			// FIXME: Putting these blocks below in a separate function would be a little bit cleaner
			let queue_call =
				self.node_meta_data_provider.get_from_metadata(|m| m.queued_storage_map_key());
			let maybe_queued: Option<Vec<GameId>> = self
				.ocall_api
				.get_storage_verified(queue_call, block.header())
				.map_err(|e| Error::StorageVerified(format!("{:?}", e)))?
				.into_tuple()
				.1;

			match maybe_queued {
				Some(queued) => {
					if !queued.is_empty() {
						//FIXME: we currently only take the first shard. How we handle sharding in general?
						let shard = self.file_state_handler.list_shards().unwrap()[0];
						let ack_game_indexes = self
							.node_meta_data_provider
							.get_from_metadata(|m| m.ack_game_call_indexes());
						let ack_game_call =
							OpaqueCall::from_tuple(&(ack_game_indexes, queued, shard));

						calls.push(ack_game_call);
					}
				},
				None => {
					debug!("No game queued in GameRegistry pallet.");
				},
			}
		}

		// Create extrinsics for all `unshielding` and `block processed` calls we've gathered.
		let parentchain_extrinsics =
			self.extrinsics_factory.create_extrinsics(calls.as_slice(), None)?;

		// Sending the extrinsic requires mut access because the validator caches the sent extrinsics internally.
		self.validator_accessor
			.execute_mut_on_validator(|v| v.send_extrinsics(parentchain_extrinsics))?;
	}
}

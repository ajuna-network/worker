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
//! Implementation of the sidechain block importer struct.
//! Imports sidechain blocks and applies the accompanying state diff to its state.

// Reexport BlockImport trait which implements fn block_import()
pub use its_consensus_common::BlockImport;

use crate::{AuraVerifier, EnclaveOnChainOCallApi, SidechainBlockTrait};
use ita_stf::{
	hash::TrustedOperationOrHash, helpers::get_board_for, ParentchainHeader, SgxBoardStruct,
	TrustedCall, TrustedCallSigned,
};
use itc_parentchain_block_import_dispatcher::triggered_dispatcher::{
	PeekParentchainBlockImportQueue, TriggerParentchainBlockImport,
};
use itc_parentchain_light_client::{concurrent_access::ValidatorAccess, Validator};
use itp_enclave_metrics::EnclaveMetric;
use itp_extrinsics_factory::CreateExtrinsics;
use itp_ocall_api::{EnclaveMetricsOCallApi, EnclaveSidechainOCallApi};
use itp_settings::{
	node::{FINISH_GAME, GAME_REGISTRY_MODULE},
	sidechain::SLOT_DURATION,
};
use itp_sgx_crypto::StateCrypto;
use itp_stf_executor::ExecutedOperation;
use itp_stf_state_handler::handle_state::HandleState;
use itp_types::{OpaqueCall, H256};
use its_consensus_common::Error as ConsensusError;
use its_primitives::traits::{
	BlockData, Header as HeaderTrait, ShardIdentifierFor, SignedBlock as SignedBlockTrait,
};
use its_state::{SidechainDB, SidechainState};
use its_top_pool_executor::TopPoolCallOperator;
use its_validateer_fetch::ValidateerFetch;
use log::*;
use sgx_externalities::{SgxExternalities, SgxExternalitiesTrait};
use sp_core::Pair;
use sp_runtime::{
	generic::SignedBlock as SignedParentchainBlock, traits::Block as ParentchainBlockTrait,
};
use std::{marker::PhantomData, string::ToString, sync::Arc, vec::Vec};

/// Implements `BlockImport`.
#[derive(Clone)]
pub struct BlockImporter<
	Authority,
	ParentchainBlock,
	SignedSidechainBlock,
	OCallApi,
	SidechainState,
	StateHandler,
	StateKey,
	TopPoolExecutor,
	ParentchainBlockImporter,
	ExtrinsicsFactory,
	ValidatorAccessor,
> {
	state_handler: Arc<StateHandler>,
	state_key: StateKey,
	authority: Authority,
	top_pool_executor: Arc<TopPoolExecutor>,
	parentchain_block_importer: Arc<ParentchainBlockImporter>,
	ocall_api: Arc<OCallApi>,
	_phantom: PhantomData<(ParentchainBlock, SignedSidechainBlock, SidechainState)>,
	extrinsics_factory: Arc<ExtrinsicsFactory>,
	validator_accessor: Arc<ValidatorAccessor>,
}

impl<
		Authority,
		ParentchainBlock,
		SignedSidechainBlock,
		OCallApi,
		SidechainState,
		StateHandler,
		StateKey,
		TopPoolExecutor,
		ParentchainBlockImporter,
		ExtrinsicsFactory,
		ValidatorAccessor,
	>
	BlockImporter<
		Authority,
		ParentchainBlock,
		SignedSidechainBlock,
		OCallApi,
		SidechainState,
		StateHandler,
		StateKey,
		TopPoolExecutor,
		ParentchainBlockImporter,
		ExtrinsicsFactory,
		ValidatorAccessor,
	> where
	Authority: Pair,
	Authority::Public: std::fmt::Debug,
	ParentchainBlock: ParentchainBlockTrait<Hash = H256, Header = ParentchainHeader>,
	SignedSidechainBlock: SignedBlockTrait<Public = Authority::Public> + 'static,
	<<SignedSidechainBlock as SignedBlockTrait>::Block as SidechainBlockTrait>::HeaderType:
		HeaderTrait<ShardIdentifier = H256>,
	OCallApi: EnclaveSidechainOCallApi
		+ ValidateerFetch
		+ EnclaveMetricsOCallApi
		+ Send
		+ Sync
		+ EnclaveOnChainOCallApi,
	StateHandler: HandleState<StateT = SgxExternalities>,
	StateKey: StateCrypto + Copy,
	TopPoolExecutor:
		TopPoolCallOperator<ParentchainBlock, SignedSidechainBlock> + Send + Sync + 'static,
	ParentchainBlockImporter: TriggerParentchainBlockImport<SignedParentchainBlock<ParentchainBlock>>
		+ PeekParentchainBlockImportQueue<SignedParentchainBlock<ParentchainBlock>>
		+ Send
		+ Sync,
	ExtrinsicsFactory: CreateExtrinsics,
	ValidatorAccessor: ValidatorAccess<ParentchainBlock>,
{
	pub fn new(
		state_handler: Arc<StateHandler>,
		state_key: StateKey,
		authority: Authority,
		top_pool_executor: Arc<TopPoolExecutor>,
		parentchain_block_importer: Arc<ParentchainBlockImporter>,
		ocall_api: Arc<OCallApi>,
		extrinsics_factory: Arc<ExtrinsicsFactory>,
		validator_accessor: Arc<ValidatorAccessor>,
	) -> Self {
		Self {
			state_handler,
			state_key,
			authority,
			top_pool_executor,
			parentchain_block_importer,
			ocall_api,
			_phantom: Default::default(),
			extrinsics_factory,
			validator_accessor,
		}
	}

	pub(crate) fn update_top_pool(&self, sidechain_block: &SignedSidechainBlock::Block) {
		// FIXME: we should take the rpc author here directly #547.

		// Notify pool about imported block for status updates of the calls.
		self.top_pool_executor.on_block_imported(sidechain_block);

		// Remove calls from pool.
		let executed_operations = sidechain_block
			.block_data()
			.signed_top_hashes()
			.iter()
			.map(|hash| {
				// Only successfully executed operations are included in a block.
				ExecutedOperation::success(*hash, TrustedOperationOrHash::Hash(*hash), Vec::new())
			})
			.collect();

		let unremoved_calls = self
			.top_pool_executor
			.remove_calls_from_pool(&sidechain_block.header().shard_id(), executed_operations);

		for unremoved_call in unremoved_calls {
			error!(
				"Could not remove call {:?} from top pool",
				unremoved_call.trusted_operation_or_hash
			);
		}
	}

	pub(crate) fn block_author_is_self(&self, block_author: &SignedSidechainBlock::Public) -> bool {
		self.authority.public() == *block_author
	}

	fn get_calls_in_block(
		&self,
		sidechain_block: &SignedSidechainBlock::Block,
	) -> Result<Vec<TrustedCallSigned>, ConsensusError> {
		let shard = &sidechain_block.header().shard_id();
		let top_hashes = sidechain_block.block_data().signed_top_hashes();
		let calls = self
			.top_pool_executor
			.get_trusted_calls(shard)
			.map_err(|e| ConsensusError::Other(format!("{:?}", e).into()))?;

		Ok(calls
			.iter()
			.filter(|call| {
				top_hashes.contains(&self.top_pool_executor.get_trusted_call_hash(&call))
			})
			.map(|call| call.clone())
			.collect())
	}

	fn get_board_if_game_finished(
		&self,
		sidechain_block: &SignedSidechainBlock::Block,
		call: &TrustedCallSigned,
	) -> Result<Option<SgxBoardStruct>, ConsensusError> {
		let shard = &sidechain_block.header().shard_id();
		if let TrustedCall::connectfour_play_turn(account, _b) = &call.call {
			let mut state = self
				.state_handler
				.load(&shard)
				.map_err(|e| ConsensusError::Other(format!("{:?}", e).into()))?;
			if let Some(board) = state.execute_with(|| get_board_for(account.clone())) {
				error!("This is not implemented!");
				return Ok(Some(board))
			// if let BoardState::Finished(_) = board.board_state {
			// 	return Ok(Some(board))
			// }
			} else {
				error!("could not decode board. maybe hasn't been set?");
			}
		}
		Ok(None)
	}

	fn send_game_finished_extrinsic(
		&self,
		sidechain_block: &SignedSidechainBlock::Block,
		board: SgxBoardStruct,
	) -> Result<(), ConsensusError> {
		let shard = &sidechain_block.header().shard_id();
		// player 1 is red, player 2 is blue
		// the winner is not the next player
		let winner = match board.next_player {
			1 => board.blue.clone(),
			2 => board.red.clone(),
			_ =>
				return Err(ConsensusError::BadSidechainBlock(
					sidechain_block.hash(),
					"Unknown player, could not get the Winner.".to_string(),
				)),
		};

		let opaque_call =
			OpaqueCall::from_tuple(&([GAME_REGISTRY_MODULE, FINISH_GAME], shard, winner));

		let calls = vec![opaque_call];

		// Create extrinsic for finish game.
		let finish_game_extrinsic = self
			.extrinsics_factory
			.create_extrinsics(calls.as_slice())
			.map_err(|e| ConsensusError::Other(format!("{:?}", e).into()))?;

		// Sending the extrinsic requires mut access because the validator caches the sent extrinsics internally.
		self.validator_accessor
			.execute_mut_on_validator(|v| {
				v.send_extrinsics(self.ocall_api.as_ref(), finish_game_extrinsic)
			})
			.map_err(|e| ConsensusError::Other(format!("{:?}", e).into()))?;
		trace!("extrinsic finish game sent");
		Ok(())
	}
}

impl<
		Authority,
		ParentchainBlock,
		SignedSidechainBlock,
		OCallApi,
		StateHandler,
		StateKey,
		TopPoolExecutor,
		ParentchainBlockImporter,
		ExtrinsicsFactory,
		ValidatorAccessor,
	> BlockImport<ParentchainBlock, SignedSidechainBlock>
	for BlockImporter<
		Authority,
		ParentchainBlock,
		SignedSidechainBlock,
		OCallApi,
		SidechainDB<SignedSidechainBlock::Block, SgxExternalities>,
		StateHandler,
		StateKey,
		TopPoolExecutor,
		ParentchainBlockImporter,
		ExtrinsicsFactory,
		ValidatorAccessor,
	> where
	Authority: Pair,
	Authority::Public: std::fmt::Debug,
	ParentchainBlock: ParentchainBlockTrait<Hash = H256, Header = ParentchainHeader>,
	SignedSidechainBlock: SignedBlockTrait<Public = Authority::Public> + 'static,
	<<SignedSidechainBlock as SignedBlockTrait>::Block as SidechainBlockTrait>::HeaderType:
		HeaderTrait<ShardIdentifier = H256>,
	OCallApi: EnclaveSidechainOCallApi
		+ ValidateerFetch
		+ EnclaveOnChainOCallApi
		+ EnclaveMetricsOCallApi
		+ Send
		+ Sync
		+ itp_ocall_api::EnclaveOnChainOCallApi,
	StateHandler: HandleState<StateT = SgxExternalities>,
	StateKey: StateCrypto + Copy,
	TopPoolExecutor:
		TopPoolCallOperator<ParentchainBlock, SignedSidechainBlock> + Send + Sync + 'static,
	ParentchainBlockImporter: TriggerParentchainBlockImport<SignedParentchainBlock<ParentchainBlock>>
		+ PeekParentchainBlockImportQueue<SignedParentchainBlock<ParentchainBlock>>
		+ Send
		+ Sync,
	ExtrinsicsFactory: CreateExtrinsics,
	ValidatorAccessor: ValidatorAccess<ParentchainBlock>,
	SidechainDB<SignedSidechainBlock::Block, SgxExternalities>: SidechainState,
{
	type Verifier = AuraVerifier<
		Authority,
		ParentchainBlock,
		SignedSidechainBlock,
		SidechainDB<SignedSidechainBlock::Block, SgxExternalities>,
		OCallApi,
	>;
	type SidechainState = SidechainDB<SignedSidechainBlock::Block, SgxExternalities>;
	type StateCrypto = StateKey;
	type Context = OCallApi;

	fn verifier(&self, state: Self::SidechainState) -> Self::Verifier {
		AuraVerifier::<Authority, ParentchainBlock, _, _, _>::new(SLOT_DURATION, state)
	}

	fn apply_state_update<F>(
		&self,
		shard: &ShardIdentifierFor<SignedSidechainBlock>,
		mutating_function: F,
	) -> Result<(), ConsensusError>
	where
		F: FnOnce(Self::SidechainState) -> Result<Self::SidechainState, ConsensusError>,
	{
		let (write_lock, state) = self
			.state_handler
			.load_for_mutation(shard)
			.map_err(|e| ConsensusError::Other(format!("{:?}", e).into()))?;

		let updated_state = mutating_function(Self::SidechainState::new(state))?;

		self.state_handler
			.write_after_mutation(updated_state.ext, write_lock, shard)
			.map_err(|e| ConsensusError::Other(format!("{:?}", e).into()))?;

		Ok(())
	}

	fn verify_import<F>(
		&self,
		shard: &ShardIdentifierFor<SignedSidechainBlock>,
		verifying_function: F,
	) -> Result<SignedSidechainBlock, ConsensusError>
	where
		F: FnOnce(Self::SidechainState) -> Result<SignedSidechainBlock, ConsensusError>,
	{
		let state = self
			.state_handler
			.load(shard)
			.map_err(|e| ConsensusError::Other(format!("{:?}", e).into()))?;
		verifying_function(Self::SidechainState::new(state))
	}

	fn state_key(&self) -> Self::StateCrypto {
		self.state_key
	}

	fn get_context(&self) -> &Self::Context {
		&self.ocall_api
	}

	fn import_parentchain_block(
		&self,
		sidechain_block: &SignedSidechainBlock::Block,
		last_imported_parentchain_header: &ParentchainBlock::Header,
	) -> Result<ParentchainBlock::Header, ConsensusError> {
		// We trigger the import of parentchain blocks up until the last one we've seen in the
		// sidechain block that we're importing. This is done to prevent forks in the sidechain (#423)
		let maybe_latest_imported_block = self
			.parentchain_block_importer
			.import_until(|signed_parentchain_block| {
				signed_parentchain_block.block.hash()
					== sidechain_block.block_data().layer_one_head()
			})
			.map_err(|e| ConsensusError::Other(format!("{:?}", e).into()))?;

		Ok(maybe_latest_imported_block
			.map(|b| b.block.header().clone())
			.unwrap_or_else(|| last_imported_parentchain_header.clone()))
	}

	fn peek_parentchain_header(
		&self,
		sidechain_block: &SignedSidechainBlock::Block,
		last_imported_parentchain_header: &ParentchainBlock::Header,
	) -> Result<ParentchainBlock::Header, ConsensusError> {
		if sidechain_block.block_data().layer_one_head() == last_imported_parentchain_header.hash()
		{
			debug!("No queue peek necessary, sidechain block references latest imported parentchain block");
			return Ok(last_imported_parentchain_header.clone())
		}

		let parentchain_header_hash_to_peek = sidechain_block.block_data().layer_one_head();
		let maybe_signed_parentchain_block = self
			.parentchain_block_importer
			.peek(|parentchain_block| {
				parentchain_block.block.header().hash() == parentchain_header_hash_to_peek
			})
			.map_err(|e| ConsensusError::Other(format!("{:?}", e).into()))?;

		maybe_signed_parentchain_block
			.map(|signed_block| signed_block.block.header().clone())
			.ok_or_else(|| {
				ConsensusError::Other(
					format!(
						"Failed to find parentchain header in import queue (hash: {}) that is \
			associated with the current sidechain block that is to be imported (number: {}, hash: {})",
						sidechain_block.block_data().layer_one_head(),
						sidechain_block.header().block_number(),
						sidechain_block.hash()
					)
					.into(),
				)
			})
	}

	fn cleanup(&self, signed_sidechain_block: &SignedSidechainBlock) -> Result<(), ConsensusError> {
		let sidechain_block = signed_sidechain_block.block();

		for call in self.get_calls_in_block(sidechain_block)? {
			if let Some(board) = self.get_board_if_game_finished(sidechain_block, &call)? {
				self.send_game_finished_extrinsic(sidechain_block, board)?;
			}
		}

		// If the block has been proposed by this enclave, remove all successfully applied
		// trusted calls from the top pool.
		if self.block_author_is_self(sidechain_block.block_data().block_author()) {
			self.update_top_pool(sidechain_block)
		}

		// Send metric about sidechain block height (i.e. block number)
		let block_height_metric =
			EnclaveMetric::SetSidechainBlockHeight(sidechain_block.header().block_number());
		if let Err(e) = self.ocall_api.update_metric(block_height_metric) {
			warn!("Failed to update sidechain block height metric: {:?}", e);
		}

		Ok(())
	}
}

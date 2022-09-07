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
use crate::{error::Result, NodeMetadata};
use sp_core::storage::StorageKey;

pub type GameId = u32;
pub type AckGameFn = ([u8; 2], Vec<GameId>, ShardIdentifier);
pub type FinishGameFn = ([u8; 2], GameId, AccountId, ShardIdentifier);

pub type Enclave = EnclaveGen<AccountId>;

/// Pallet' name:
const GAME_REGISTRY: &str = "GameRegistry";

pub trait GameRegistryStorageIndexes {
	fn game_queues_storage_map_key(&self, index: u64) -> Result<StorageKey>;

	fn game_registry_storage_map_key(&self, index: u64) -> Result<StorageKey>;
}

impl GameRegistryStorageIndexes for NodeMetadata {
	fn game_queues_storage_map_key(&self, game_engine: GameEngine) -> Result<StorageKey> {
		self.storage_map_key(GAME_REGISTRY, "GameQueues", &game_engine)
	}

	fn game_registry_storage_map_key(&self, game_hash: Hash) -> Result<StorageKey> {
		self.storage_map_key(GAME_REGISTRY, "GameRegistry", &game_hash)
	}
}

pub trait GameRegistryCallIndexes {
	fn queue_call_indexes(&self) -> Result<[u8; 2]>;

	fn drop_game_call_indexes(&self) -> Result<[u8; 2]>;

	fn ack_game_call_indexes(&self) -> Result<[u8; 2]>;

	fn ready_game_call_indexes(&self) -> Result<[u8; 2]>;

	fn finish_game_call_indexes(&self) -> Result<[u8; 2]>;
}

impl GameRegistryCallIndexes for NodeMetadata {
	fn queue_call_indexes(&self) -> Result<[u8; 2]> {
		self.call_indexes(GAME_REGISTRY, "queue")
	}

	fn drop_game_call_indexes(&self) -> Result<[u8; 2]> {
		self.call_indexes(GAME_REGISTRY, "drop_game")
	}

	fn ack_game_call_indexes(&self) -> Result<[u8; 2]> {
		self.call_indexes(GAME_REGISTRY, "ack_game")
	}

	fn ready_game_call_indexes(&self) -> Result<[u8; 2]> {
		self.call_indexes(GAME_REGISTRY, "ready_game")
	}

	fn finish_game_call_indexes(&self) -> Result<[u8; 2]> {
		self.call_indexes(GAME_REGISTRY, "finish_game")
	}
}

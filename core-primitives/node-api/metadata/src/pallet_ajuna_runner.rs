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
use sp_core::{storage::StorageKey, H256};

pub type GameId = u32;

/// Pallet' name:
pub const RUNNER: &str = "Runner";

pub trait GameRegistryStorageIndexes {
	fn runner_storage_map_key(&self, runner_id: GameId) -> Result<StorageKey>;

	fn players_storage_map_key(&self, game_hash: H256) -> Result<StorageKey>;
}

impl GameRegistryStorageIndexes for NodeMetadata {
	fn runner_storage_map_key(&self, runner_id: GameId) -> Result<StorageKey> {
		self.storage_map_key(RUNNER, "Runners", &runner_id)
	}

	fn players_storage_map_key(&self, game_hash: H256) -> Result<StorageKey> {
		self.storage_map_key(RUNNER, "Players", &game_hash)
	}
}

pub trait GameRegistryCallIndexes {
	fn queue_call_indexes(&self) -> Result<[u8; 2]>;

	fn drop_game_call_indexes(&self) -> Result<[u8; 2]>;

	fn ack_game_call_indexes(&self) -> Result<[u8; 2]>;

	fn finish_game_call_indexes(&self) -> Result<[u8; 2]>;
}

impl GameRegistryCallIndexes for NodeMetadata {
	fn queue_call_indexes(&self) -> Result<[u8; 2]> {
		self.call_indexes(RUNNER, "queue")
	}

	fn drop_game_call_indexes(&self) -> Result<[u8; 2]> {
		self.call_indexes(RUNNER, "drop_game")
	}

	fn ack_game_call_indexes(&self) -> Result<[u8; 2]> {
		self.call_indexes(RUNNER, "ack_game")
	}

	fn finish_game_call_indexes(&self) -> Result<[u8; 2]> {
		self.call_indexes(RUNNER, "finish_game")
	}
}

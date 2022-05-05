#![cfg_attr(not(feature = "std"), no_std)]

use itp_storage::{storage_map_key, StorageHasher};
use itp_types::H256;
use sp_std::prelude::Vec;

pub const REGISTRY: &str = "GameRegistry";

pub struct RegistryStorage;

// Separate the prefix from the rest because in our case we changed the storage prefix due to
// the rebranding. With the below implementation of the `RegistryStorageKeys`, we could simply
// define another struct `OtherStorage`, implement `StoragePrefix` for it, and get the
// `RegistryStorageKeys` implementation for free.
pub trait StoragePrefix {
	fn prefix() -> &'static str;
}

impl StoragePrefix for RegistryStorage {
	fn prefix() -> &'static str {
		REGISTRY
	}
}

pub trait RegistryStorageKeys {
	fn queue_game() -> Vec<u8>;
	fn game_registry(game: H256) -> Vec<u8>;
}

impl<S: StoragePrefix> RegistryStorageKeys for S {
	fn queue_game() -> Vec<u8> {
		storage_map_key(Self::prefix(), "GameQueues", &StorageHasher::Identity)
	}

	fn game_registry(game: H256) -> Vec<u8> {
		storage_map_key(Self::prefix(), "GameRegistry", &game, &StorageHasher::Identity)
	}
}

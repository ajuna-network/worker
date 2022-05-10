#![cfg_attr(not(feature = "std"), no_std)]

// TODO this needs renaming to Runner as it will be reading the Runner storage
use itp_storage::{storage_map_key, StorageHasher};
// use itp_types::H256;
// use pallet_ajuna_gameregistry::game::GameEngine;
// use sp_std::prelude::Vec;

pub const RUNNER: &str = "Runner";

pub struct RunnerStorage;

// Separate the prefix from the rest because in our case we changed the storage prefix due to
// the rebranding. With the below implementation of the `RegistryStorageKeys`, we could simply
// define another struct `OtherStorage`, implement `StoragePrefix` for it, and get the
// `RegistryStorageKeys` implementation for free.
pub trait StoragePrefix {
	fn prefix() -> &'static str;
}

impl StoragePrefix for RunnerStorage {
	fn prefix() -> &'static str {
		RUNNER
	}
}

pub trait RunnerStorageKeys {
	// fn queue_game() -> Vec<u8>;
	// fn game_registry(game: H256) -> Vec<u8>;
	// TODO work out the return here, should be `RunnerState`
	fn runner(runner_id: u64) -> Option<u64>;
}

impl<S: StoragePrefix> RunnerStorageKeys for S {
	// fn queue_game() -> Vec<u8> {
	// 	// let game_engine = GameEngine::new(1u8, 1u8);
	// 	storage_map_key(Self::prefix(), "GameQueues", &game_engine, &StorageHasher::Identity)
	// }

	// fn game_registry(game: H256) -> Vec<u8> {
	// 	storage_map_key(Self::prefix(), "GameRegistry", &game, &StorageHasher::Identity)
	// }

	// TODO Type for RunnerId
	fn runner(runner_id: u64) -> Option<u64> {
		storage_map_key(Self::prefix(), "Runners", &runner_id, &StorageHasher::Blake2_128);
		None
	}
}

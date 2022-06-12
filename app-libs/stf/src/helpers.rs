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
	stf_sgx_primitives::types::*, AccountId, Index, SgxBoardId, SgxGameBoardStruct,
	SgxWinningBoard, StfError, StfResult, ENCLAVE_ACCOUNT_KEY, H256,
};
use codec::{Decode, Encode};
use itp_storage::{storage_double_map_key, storage_map_key, storage_value_key, StorageHasher};
use itp_utils::stringify::account_id_to_string;
use log::*;
use std::prelude::v1::*;

#[cfg(feature = "sgx")]
use crate::stf_sgx_primitives::types::{AccountData, AccountInfo};
#[cfg(feature = "std")]
use itp_types::{AccountData, AccountInfo};

pub fn get_storage_value<V: Decode>(
	storage_prefix: &'static str,
	storage_key_name: &'static str,
) -> Option<V> {
	let key = storage_value_key(storage_prefix, storage_key_name);
	get_storage_by_key_hash(key)
}

pub fn get_storage_map<K: Encode, V: Decode + Clone>(
	storage_prefix: &'static str,
	storage_key_name: &'static str,
	map_key: &K,
	hasher: &StorageHasher,
) -> Option<V> {
	let key = storage_map_key::<K>(storage_prefix, storage_key_name, map_key, hasher);
	get_storage_by_key_hash(key)
}

pub fn get_storage_double_map<K: Encode, Q: Encode, V: Decode + Clone>(
	storage_prefix: &'static str,
	storage_key_name: &'static str,
	first: &K,
	first_hasher: &StorageHasher,
	second: &Q,
	second_hasher: &StorageHasher,
) -> Option<V> {
	let key = storage_double_map_key::<K, Q>(
		storage_prefix,
		storage_key_name,
		first,
		first_hasher,
		second,
		second_hasher,
	);
	get_storage_by_key_hash(key)
}

/// Get value in storage.
pub fn get_storage_by_key_hash<V: Decode>(key: Vec<u8>) -> Option<V> {
	if let Some(value_encoded) = sp_io::storage::get(&key) {
		if let Ok(value) = Decode::decode(&mut value_encoded.as_slice()) {
			Some(value)
		} else {
			error!("could not decode state for key {:x?}", key);
			None
		}
	} else {
		info!("key not found in state {:x?}", key);
		None
	}
}

/// Get the AccountInfo key where the account is stored.
pub fn account_key_hash(account: &AccountId) -> Vec<u8> {
	storage_map_key("System", "Account", account, &StorageHasher::Blake2_128Concat)
}

pub fn enclave_signer_account() -> AccountId {
	get_storage_value("Sudo", ENCLAVE_ACCOUNT_KEY).expect("No enclave account")
}

/// Ensures an account is a registered enclave account.
pub fn ensure_enclave_signer_account(account: &AccountId) -> StfResult<()> {
	let expected_enclave_account = enclave_signer_account();
	if &expected_enclave_account == account {
		Ok(())
	} else {
		error!(
			"Expected enclave account {}, but found {}",
			account_id_to_string(&expected_enclave_account),
			account_id_to_string(account)
		);
		Err(StfError::RequireEnclaveSignerAccount)
	}
}

pub fn ensure_root(account: AccountId) -> StfResult<()> {
	if root() == account {
		Ok(())
	} else {
		Err(StfError::MissingPrivileges(account))
	}
}

pub fn get_board_for(who: AccountId) -> Option<SgxGameBoardStruct> {
	if let Some(board_id) = get_storage_map::<AccountId, SgxBoardId>(
		"AjunaBoard",
		"PlayerBoards",
		&who,
		&StorageHasher::Identity,
	) {
		if let Some(board) = get_storage_map::<SgxBoardId, SgxGameBoardStruct>(
			"AjunaBoard",
			"BoardStates",
			&board_id,
			&StorageHasher::Identity,
		) {
			Some(board)
		} else {
			debug!("could not read board");
			None
		}
	} else {
		debug!("could not read board id");
		None
	}
}

pub fn is_winner(who: AccountId) -> Option<SgxWinningBoard> {
	if let Some(board_id) = get_storage_map::<AccountId, SgxBoardId>(
		"AjunaBoard",
		"PlayerBoards",
		&who,
		&StorageHasher::Identity,
	) {
		if let Some(winner) = get_storage_map::<SgxBoardId, AccountId>(
			"AjunaBoard",
			"BoardWinners",
			&board_id,
			&StorageHasher::Identity,
		) {
			if who == winner {
				if let Some(_board) = get_storage_map::<SgxBoardId, SgxGameBoardStruct>(
					"AjunaBoard",
					"BoardStates",
					&board_id,
					&StorageHasher::Identity,
				) {
					return Some(SgxWinningBoard { winner, board_id })
				}
			}
		}
	} else {
		debug!("could not read board id");
	}

	None
}

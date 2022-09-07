#![cfg_attr(all(not(target_env = "sgx"), not(feature = "std")), no_std)]
#![cfg_attr(target_env = "sgx", feature(rustc_private))]

use codec::{Decode, Encode};
#[cfg(feature = "sgx")]
use sgx_tstd as std;
use sp_runtime::{
	generic::{Block as BlockG, Header as HeaderG, SignedBlock as SignedBlockG},
	traits::BlakeTwo256,
	OpaqueExtrinsic,
};
use std::vec::Vec;
use substrate_api_client::{
	AssetTip, AssetTipExtrinsicParams, AssetTipExtrinsicParamsBuilder, SubstrateDefaultSignedExtra,
	UncheckedExtrinsicV4,
};

use itp_storage::storage_entry::StorageEntry;
pub use rpc::*;
pub mod light_client_init_params;
pub mod rpc;

/// Substrate runtimes provide no string type. Hence, for arbitrary data of varying length the
/// `Vec<u8>` is used. In the polkadot-js the typedef `Text` is used to automatically
/// utf8 decode bytes into a string.
#[cfg(not(feature = "std"))]
pub type PalletString = Vec<u8>;

#[cfg(feature = "std")]
pub type PalletString = String;

pub use sp_core::{crypto::AccountId32 as AccountId, H256};
pub use substrate_api_client::{AccountData, AccountInfo};

pub type GameId = u32;
pub type AckGameFn = ([u8; 2], Vec<GameId>, ShardIdentifier);
pub type FinishGameFn = ([u8; 2], GameId, AccountId, ShardIdentifier);

pub type Enclave = EnclaveGen<AccountId>;

/// Configuration for the ExtrinsicParams.
/// PlainTipExtrinsicParams to construct a transaction for the default integritee node
// pub type ParentchainExtrinsicParams = PlainTipExtrinsicParams;
// pub type ParentchainExtrinsicParamsBuilder = PlainTipExtrinsicParamsBuilder;
/// To pay in asset fees use different ExtrinsicParams Config.
/// For asset payment in default substrate node :
pub type ParentchainExtrinsicParams = AssetTipExtrinsicParams;
pub type ParentchainExtrinsicParamsBuilder = AssetTipExtrinsicParamsBuilder;
pub type ParentchainUncheckedExtrinsic<Call> =
	UncheckedExtrinsicV4<Call, SubstrateDefaultSignedExtra<AssetTip>>;

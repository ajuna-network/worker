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
	error::Result,
	traits::{StateUpdateProposer, StfEnclaveSigning},
	BatchExecutionResult, ExecutedOperation,
};
use codec::Encode;
use ita_stf::{
	hash::{Hash, TrustedOperationOrHash},
	AccountId, KeyPair, ShardIdentifier, TrustedCall, TrustedCallSigned, TrustedOperation,
};
use itp_sgx_externalities::SgxExternalitiesTrait;
use itp_types::{Amount, GameId, OpaqueCall, H256};
use sp_core::Pair;
use sp_runtime::traits::Header as HeaderTrait;
use std::{marker::PhantomData, time::Duration};

/// Mock for the StfExecutor.
#[derive(Default)]
pub struct StfExecutorMock<StateType: SgxExternalitiesTrait + Encode> {
	_phantom: PhantomData<StateType>,
}

impl<StateType> StateUpdateProposer for StfExecutorMock<StateType>
where
	StateType: SgxExternalitiesTrait + Encode,
{
	type Externalities = StateType;

	fn propose_state_update<PH, F>(
		&self,
		trusted_calls: &[TrustedOperation],
		_header: &PH,
		_shard: &ShardIdentifier,
		_max_exec_duration: Duration,
		_prepare_state_function: F,
	) -> Result<BatchExecutionResult<Self::Externalities>>
	where
		PH: HeaderTrait<Hash = H256>,
		F: FnOnce(Self::Externalities) -> Self::Externalities,
	{
		todo!()
	}
}

impl StfExecuteShieldFunds for StfExecutorMock {
	fn execute_shield_funds(
		&self,
		_account: AccountId,
		_amount: Amount,
		_shard: &ShardIdentifier,
	) -> Result<H256> {
		todo!()
	}
}

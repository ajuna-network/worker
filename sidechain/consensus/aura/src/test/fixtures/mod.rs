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

pub mod types;

use itp_types::{AccountId, Enclave, Header};
use sp_runtime::traits::Header as HeaderTrait;
use std::time::Duration;

pub const SLOT_DURATION: Duration = Duration::from_millis(300);

pub fn validateer(account: AccountId) -> Enclave {
	Enclave::new(account, Default::default(), Default::default(), Default::default())
}

pub fn default_header() -> Header {
	Header::new(
		Default::default(),
		Default::default(),
		Default::default(),
		Default::default(),
		Default::default(),
	)
}

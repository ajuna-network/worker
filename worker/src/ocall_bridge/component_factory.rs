/*
    Copyright 2019 Supercomputing Systems AG
    Copyright (C) 2017-2019 Baidu, Inc. All Rights Reserved.

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

use crate::node_api_factory::CreateNodeApi;
use crate::ocall_bridge::bridge_api::{RemoteAttestationBridge, WorkerOnChainBridge};
use crate::ocall_bridge::remote_attestation_ocall::RemoteAttestationOCall;
use crate::ocall_bridge::worker_on_chain_ocall::WorkerOnChainOCall;
use crate::sync_block_gossiper::GossipBlocks;
use std::sync::Arc;

/// Factory trait (abstract factory) that creates instances
/// of all the components of the OCall Bridge
pub trait GetOCallBridgeComponents {
    /// remote attestation OCall API
    fn get_ra_api(&self) -> Arc<dyn RemoteAttestationBridge>;

    /// on chain OCall API
    fn get_oc_api(&self) -> Arc<dyn WorkerOnChainBridge>;
}

/// Concrete implementation, should be moved out of the OCall Bridge, into the worker
/// since the OCall bridge itself should not know any concrete types to ensure
/// our dependency graph is worker -> ocall bridge
pub struct OCallBridgeComponentFactory<N, B> {
    node_api_factory: Arc<N>,
    block_gossiper: Arc<B>,
}

impl<N, B> OCallBridgeComponentFactory<N, B> {
    pub fn new(node_api_factory: Arc<N>, block_gossiper: Arc<B>) -> Self {
        OCallBridgeComponentFactory {
            node_api_factory,
            block_gossiper,
        }
    }
}

impl<N, B> GetOCallBridgeComponents for OCallBridgeComponentFactory<N, B>
where
    N: CreateNodeApi + 'static,
    B: GossipBlocks + 'static,
{
    fn get_ra_api(&self) -> Arc<dyn RemoteAttestationBridge> {
        Arc::new(RemoteAttestationOCall {})
    }

    fn get_oc_api(&self) -> Arc<dyn WorkerOnChainBridge> {
        Arc::new(WorkerOnChainOCall::new(
            self.node_api_factory.clone(),
            self.block_gossiper.clone(),
        ))
    }
}
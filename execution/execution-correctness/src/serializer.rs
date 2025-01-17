// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::execution_correctness::ExecutionCorrectness;
use aptos_crypto::{ed25519::Ed25519PrivateKey, traits::SigningKey, HashValue};
use aptos_types::ledger_info::LedgerInfoWithSignatures;
use consensus_types::{block::Block, vote_proposal::VoteProposal};
use executor_types::{BlockExecutorTrait, Error, StateComputeResult};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Deserialize, Serialize)]
pub enum ExecutionCorrectnessInput {
    CommittedBlockId,
    Reset,
    ExecuteBlock(Box<(Block, HashValue)>),
    CommitBlocks(Box<(Vec<HashValue>, LedgerInfoWithSignatures)>),
}

pub struct SerializerService {
    internal: Box<dyn BlockExecutorTrait>,
    prikey: Option<Ed25519PrivateKey>,
}

impl SerializerService {
    pub fn new(internal: Box<dyn BlockExecutorTrait>, prikey: Option<Ed25519PrivateKey>) -> Self {
        Self { internal, prikey }
    }

    pub fn handle_message(&self, input_message: Vec<u8>) -> Result<Vec<u8>, Error> {
        let input = bcs::from_bytes(&input_message)?;

        let output = match input {
            ExecutionCorrectnessInput::CommittedBlockId => {
                bcs::to_bytes(&Result::<_, Error>::Ok(self.internal.committed_block_id()))
            }
            ExecutionCorrectnessInput::Reset => bcs::to_bytes(&self.internal.reset()),
            ExecutionCorrectnessInput::ExecuteBlock(block_with_parent_id) => bcs::to_bytes(
                &self
                    .internal
                    .execute_block(
                        (
                            block_with_parent_id.0.id(),
                            block_with_parent_id.0.transactions_to_execute(),
                        ),
                        block_with_parent_id.1,
                    )
                    .map(|mut result| {
                        if let Some(prikey) = self.prikey.as_ref() {
                            let vote_proposal = VoteProposal::new(
                                result.extension_proof(),
                                block_with_parent_id.0.clone(),
                                result.epoch_state().clone(),
                                false,
                            );
                            let signature = prikey.sign(&vote_proposal);
                            result.set_signature(signature);
                        }
                        result
                    }),
            ),
            ExecutionCorrectnessInput::CommitBlocks(blocks_with_li) => bcs::to_bytes(
                &self
                    .internal
                    .commit_blocks(blocks_with_li.0, blocks_with_li.1),
            ),
        };
        Ok(output?)
    }
}

pub struct SerializerClient {
    service: Box<dyn TSerializerClient>,
}

impl SerializerClient {
    pub fn new(serializer_service: Arc<SerializerService>) -> Self {
        let service = Box::new(LocalService { serializer_service });
        Self { service }
    }

    pub fn new_client(service: Box<dyn TSerializerClient>) -> Self {
        Self { service }
    }

    fn request(&self, input: ExecutionCorrectnessInput) -> Result<Vec<u8>, Error> {
        self.service.request(input)
    }
}

impl ExecutionCorrectness for SerializerClient {
    fn committed_block_id(&self) -> Result<HashValue, Error> {
        let response = self.request(ExecutionCorrectnessInput::CommittedBlockId)?;
        bcs::from_bytes(&response)?
    }

    fn reset(&self) -> Result<(), Error> {
        let response = self.request(ExecutionCorrectnessInput::Reset)?;
        bcs::from_bytes(&response)?
    }

    fn execute_block(
        &self,
        block: Block,
        parent_block_id: HashValue,
    ) -> Result<StateComputeResult, Error> {
        let response = self.request(ExecutionCorrectnessInput::ExecuteBlock(Box::new((
            block,
            parent_block_id,
        ))))?;
        bcs::from_bytes(&response)?
    }

    fn commit_blocks(
        &self,
        block_ids: Vec<HashValue>,
        ledger_info_with_sigs: LedgerInfoWithSignatures,
    ) -> Result<(), Error> {
        let response = self.request(ExecutionCorrectnessInput::CommitBlocks(Box::new((
            block_ids,
            ledger_info_with_sigs,
        ))))?;
        bcs::from_bytes(&response)?
    }
}

pub trait TSerializerClient: Send + Sync {
    fn request(&self, input: ExecutionCorrectnessInput) -> Result<Vec<u8>, Error>;
}

struct LocalService {
    pub serializer_service: Arc<SerializerService>,
}

impl TSerializerClient for LocalService {
    fn request(&self, input: ExecutionCorrectnessInput) -> Result<Vec<u8>, Error> {
        let input_message = bcs::to_bytes(&input)?;
        self.serializer_service.handle_message(input_message)
    }
}

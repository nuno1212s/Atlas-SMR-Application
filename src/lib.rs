use atlas_common::channel::ChannelSyncTx;
use atlas_common::error::*;
use atlas_common::node_id::NodeId;
use crate::app::{Reply, Request, UnorderedBatch, UpdateBatch};
use crate::serialize::ApplicationData;
use std::time::Instant;
use anyhow::Context;
use atlas_common::maybe_vec::MaybeVec;

pub mod serialize;
pub mod app;
pub mod state;

pub enum ExecutionRequest<O> {
    // Poll the state channel
    // As we have an incoming state update
    PollStateChannel,

    // Catch up to the current execution by
    // Executing the given requests
    CatchUp(MaybeVec<UpdateBatch<O>>),

    // update the state of the service
    Update((UpdateBatch<O>, Instant)),
    // same as above, and include the application state
    // in the reply, used for local checkpoints
    UpdateAndGetAppstate((UpdateBatch<O>, Instant)),

    //Execute an un ordered batch of requests
    ExecuteUnordered(UnorderedBatch<O>),

    // read the state of the service
    Read(NodeId),
}

/// Represents a handle to the client request executor.
pub struct ExecutorHandle<D: ApplicationData> {
    e_tx: ChannelSyncTx<ExecutionRequest<D::Request>>,
}

impl<D: ApplicationData> ExecutorHandle<D>
{
    pub fn new(tx: ChannelSyncTx<ExecutionRequest<D::Request>>) -> Self {
        ExecutorHandle { e_tx: tx }
    }

    /// Sets the current state of the execution layer to the given value.
    pub fn poll_state_channel(&self) -> Result<()> {
        self.e_tx
            .send(ExecutionRequest::PollStateChannel)
            .context("Failed to place poll order into executor channel")
    }

    pub fn catch_up_to_quorum(&self, requests: MaybeVec<UpdateBatch<D::Request>>) -> Result<()> {
        self.e_tx
            .send(ExecutionRequest::CatchUp(requests))
            .context("Failed to place catch up order into executor channel")
    }

    /// Queues a batch of requests `batch` for execution.
    pub fn queue_update(&self, batch: UpdateBatch<D::Request>)
                        -> Result<()> {
        self.e_tx
            .send(ExecutionRequest::Update((batch, Instant::now())))
            .context("Failed to place update order into executor channel")
    }

    /// Queues a batch of unordered requests for execution
    pub fn queue_update_unordered(&self, requests: UnorderedBatch<D::Request>)
                                  -> Result<()> {
        self.e_tx
            .send(ExecutionRequest::ExecuteUnordered(requests))
            .context("Failed to place unordered update order into executor channel")
    }

    /// Same as `queue_update()`, additionally reporting the serialized
    /// application state.
    ///
    /// This is useful during local checkpoints.
    pub fn queue_update_and_get_appstate(
        &self,
        batch: UpdateBatch<D::Request>,
    ) -> Result<()> {
        self.e_tx
            .send(ExecutionRequest::UpdateAndGetAppstate((batch, Instant::now())))
            .context("Failed to place update and get appstate order into executor channel")
    }
}

impl<D: ApplicationData> Clone for ExecutorHandle<D> {
    fn clone(&self) -> Self {
        let e_tx = self.e_tx.clone();
        Self { e_tx }
    }
}
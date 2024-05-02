// Source code for the Substrate Telemetry Server.
// Copyright (C) 2021 Parity Technologies (UK) Ltd.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

use std::collections::VecDeque;

use crate::find_location;
use common::node_message::{IntervalKind, SystemInterval};
use common::node_types::{
    Block, BlockDetails, NodeDetails, NodeHardware, NodeHwBench, NodeIO, NodeLocation, NodeStats,
    Timestamp,
};
use common::time;
use primitive_types::H256;

/// Minimum time between block below broadcasting updates to the browser gets throttled, in ms.
const THROTTLE_THRESHOLD: u64 = 100;
/// Minimum time of intervals for block updates sent to the browser when throttled, in ms.
const THROTTLE_INTERVAL: u64 = 1000;

pub struct Node {
    /// Static details
    details: NodeDetails,
    /// Basic stats
    stats: NodeStats,
    /// Node IO stats
    io: NodeIO,
    /// Best block
    best: BlockDetails,
    /// Finalized block
    finalized: Block,
    /// Timer for throttling block updates
    throttle: u64,
    /// Hardware stats over time
    hardware: NodeHardware,
    /// Physical location details
    location: find_location::Location,
    /// Flag marking if the node is stale (not syncing or producing blocks)
    stale: bool,
    /// Unix timestamp for when node started up (falls back to connection time)
    startup_time: Option<Timestamp>,
    /// Hardware benchmark results for the node
    hwbench: Option<NodeHwBench>,
    /// TODO
    historical_data: HistoricalData,
}

impl Node {
    pub fn new(mut details: NodeDetails) -> Self {
        let startup_time = details
            .startup_time
            .take()
            .and_then(|time| time.parse().ok());

        Node {
            details,
            stats: NodeStats::default(),
            io: NodeIO::default(),
            best: BlockDetails::default(),
            finalized: Block::zero(),
            throttle: 0,
            hardware: NodeHardware::default(),
            location: None,
            stale: false,
            startup_time,
            hwbench: None,
            historical_data: HistoricalData::default(),
        }
    }

    pub fn details(&self) -> &NodeDetails {
        &self.details
    }

    pub fn stats(&self) -> &NodeStats {
        &self.stats
    }

    pub fn io(&self) -> &NodeIO {
        &self.io
    }

    pub fn best(&self) -> &Block {
        &self.best.block
    }

    pub fn best_timestamp(&self) -> u64 {
        self.best.block_timestamp
    }

    pub fn finalized(&self) -> &Block {
        &self.finalized
    }

    pub fn hardware(&self) -> &NodeHardware {
        &self.hardware
    }

    pub fn location(&self) -> Option<&NodeLocation> {
        self.location.as_deref()
    }

    pub fn update_location(&mut self, location: find_location::Location) {
        self.location = location;
    }

    pub fn block_details(&self) -> &BlockDetails {
        &self.best
    }

    pub fn hwbench(&self) -> Option<&NodeHwBench> {
        self.hwbench.as_ref()
    }

    pub fn update_hwbench(&mut self, hwbench: NodeHwBench) -> Option<NodeHwBench> {
        self.hwbench.replace(hwbench)
    }

    pub fn update_block(&mut self, block: Block) -> bool {
        if block.height > self.best.block.height {
            self.stale = false;
            self.best.block = block;

            true
        } else {
            false
        }
    }

    pub fn update_details(
        &mut self,
        timestamp: u64,
        propagation_time: Option<u64>,
    ) -> Option<&BlockDetails> {
        self.best.block_time = timestamp - self.best.block_timestamp;
        self.best.block_timestamp = timestamp;
        self.best.propagation_time = propagation_time;
        self.best.sync_time = None;
        self.best.proposal_time = None;
        self.best.import_time = None;

        if self.throttle < timestamp {
            if self.best.block_time <= THROTTLE_THRESHOLD {
                self.throttle = timestamp + THROTTLE_INTERVAL;
            }

            Some(&self.best)
        } else {
            None
        }
    }

    pub fn update_hardware(&mut self, interval: &SystemInterval) -> bool {
        let mut changed = false;

        if let Some(upload) = interval.bandwidth_upload {
            changed |= self.hardware.upload.push(upload);
        }
        if let Some(download) = interval.bandwidth_download {
            changed |= self.hardware.download.push(download);
        }
        self.hardware.chart_stamps.push(time::now() as f64);

        changed
    }

    pub fn update_stats(&mut self, interval: &SystemInterval) -> Option<&NodeStats> {
        let mut changed = false;

        if let Some(peers) = interval.peers {
            if peers != self.stats.peers {
                self.stats.peers = peers;
                changed = true;
            }
        }
        if let Some(txcount) = interval.txcount {
            if txcount != self.stats.txcount {
                self.stats.txcount = txcount;
                changed = true;
            }
        }

        if changed {
            Some(&self.stats)
        } else {
            None
        }
    }

    pub fn update_io(&mut self, interval: &SystemInterval) -> Option<&NodeIO> {
        let mut changed = false;

        if let Some(size) = interval.used_state_cache_size {
            changed |= self.io.used_state_cache_size.push(size);
        }

        if changed {
            Some(&self.io)
        } else {
            None
        }
    }

    pub fn update_finalized(&mut self, block: Block) -> Option<&Block> {
        if block.height > self.finalized.height {
            self.finalized = block;
            Some(self.finalized())
        } else {
            None
        }
    }

    pub fn update_stale(&mut self, threshold: u64) -> bool {
        if self.best.block_timestamp < threshold {
            self.stale = true;
        }

        self.stale
    }

    pub fn stale(&self) -> bool {
        self.stale
    }

    pub fn set_validator_address(&mut self, addr: Box<str>) -> bool {
        if self.details.validator.as_ref() == Some(&addr) {
            false
        } else {
            self.details.validator = Some(addr);
            true
        }
    }

    pub fn startup_time(&self) -> Option<Timestamp> {
        self.startup_time
    }

    pub fn insert_block_details_interval(&mut self, duration: u64, kind: IntervalKind) {
        match kind {
            IntervalKind::Proposal => {
                self.best.proposal_time = Some(duration);
            }
            IntervalKind::Sync => {
                self.best.sync_time = Some(duration);
            }
            IntervalKind::Import => {
                self.best.import_time = Some(duration);
            }
        }
    }

    pub fn insert_historical_block_data(
        &mut self,
        block_hash: H256,
        block_height: u32,
        duration: u64,
        kind: IntervalKind,
    ) {
        self.historical_data
            .insert_block_time(duration, block_hash, block_height, kind);
    }
}

#[derive(Default)]
pub struct HistoricalData {
    blocks: VecDeque<BlockHistoricalData>,
}

impl HistoricalData {
    pub fn insert_block_time(
        &mut self,
        value: u64,
        block_hash: H256,
        block_height: u32,
        kind: IntervalKind,
    ) {
        let block = self.get_or_insert_block_mut(block_hash, block_height);
        match kind {
            IntervalKind::Proposal => {
                block.proposal_time = Some(value);
            }
            IntervalKind::Sync => {
                block.sync_time = Some(value);
            }
            IntervalKind::Import => {
                block.import_time = Some(value);
            }
        }
    }

    fn new_block(&mut self, block_hash: H256, block_height: u32) {
        const MAX_QUEUE_SIZE: usize = 128;

        if self.blocks.len() > MAX_QUEUE_SIZE {
            self.blocks.pop_front();
        }

        self.blocks.push_back(BlockHistoricalData {
            block_hash,
            block_height,
            ..Default::default()
        });
    }

    fn get_or_insert_block_mut(
        &mut self,
        block_hash: H256,
        block_height: u32,
    ) -> &mut BlockHistoricalData {
        let index = self
            .blocks
            .iter()
            .position(|b| b.block_hash == b.block_hash);

        if let Some(index) = index {
            self.blocks.get_mut(index).unwrap()
        } else {
            self.new_block(block_hash, block_height);
            self.blocks.back_mut().unwrap()
        }
    }
}

#[derive(Default)]
pub struct BlockHistoricalData {
    pub block_height: u32,
    pub block_hash: H256,
    pub proposal_time: Option<u64>,
    pub sync_time: Option<u64>,
    pub import_time: Option<u64>,
}

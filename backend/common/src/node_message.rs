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

//! This is the internal representation of telemetry messages sent from nodes.
//! There is a separate JSON representation of these types, because internally we want to be
//! able to serialize these messages to bincode, and various serde attributes aren't compatible
//! with this, hence this separate internal representation.

use std::{cmp::Ordering, collections::HashMap, fmt::Write};

use crate::node_types::{Block, BlockHash, BlockNumber, NodeDetails};
use serde::{Deserialize, Serialize};

pub type NodeMessageId = u64;

#[derive(Serialize, Deserialize, Debug)]
pub enum NodeMessage {
    V1 { payload: Payload },
    V2 { id: NodeMessageId, payload: Payload },
}

impl NodeMessage {
    /// Returns the ID associated with the node message, or 0
    /// if the message has no ID.
    pub fn id(&self) -> NodeMessageId {
        match self {
            NodeMessage::V1 { .. } => 0,
            NodeMessage::V2 { id, .. } => *id,
        }
    }
    /// Return the payload associated with the message.
    pub fn into_payload(self) -> Payload {
        match self {
            NodeMessage::V1 { payload, .. } | NodeMessage::V2 { payload, .. } => payload,
        }
    }
}

impl From<NodeMessage> for Payload {
    fn from(msg: NodeMessage) -> Payload {
        msg.into_payload()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Payload {
    SystemConnected(SystemConnected),
    SystemInterval(SystemInterval),
    BlockImport(Block),
    NotifyFinalized(Finalized),
    AfgAuthoritySet(AfgAuthoritySet),
    HwBench(NodeHwBench),
    BlobSubmission(BlobSubmission),
    BlobRequest(BlobRequest),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlobSubmission {
    pub hash: BlockHash,
    pub size: Option<u64>,
    // Milliseconds past UNIX EPOCH
    pub submission_tracked: Option<u64>,
    // Milliseconds past UNIX EPOCH
    pub added_to_pool_timestamp: Option<u64>,
    // In bytes
    pub compression_size: Option<u64>,
    // In ms
    pub compression_duration: Option<u64>,
    // Milliseconds past UNIX EPOCH
    pub poly_grid_build_start_timestamp: Option<u64>,
    // Milliseconds past UNIX EPOCH
    pub poly_grid_build_end_timestamp: Option<u64>,
    // 0 means it was full
    pub queue_capacity: Option<u32>,
    // At what point in time did we measure the queue capacity
    pub queue_capacity_timestamp: Option<u64>,
    // Milliseconds past UNIX EPOCH
    pub commitment_grid_build_start_timestamp: Option<u64>,
    // Milliseconds past UNIX EPOCH
    pub commitment_grid_build_end_timestamp: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlobRequest {
    pub size: usize,
    pub hash: BlockHash,
    pub start: u64,
    pub end: u64,
    pub from: Box<str>,
    pub to: Box<str>,
    pub success: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlobRequestData {
    pub start: u64,
    pub end: u64,
    pub duration: u64,
    pub from: Box<str>,
    pub to: Box<str>,
    pub success: bool,
}

#[derive(Debug, Clone, Default)]
pub struct NodeBlobView {
    pub size: Option<u64>,
    // Milliseconds past UNIX EPOCH
    pub submission_tracked: Option<u64>,
    // Milliseconds past UNIX EPOCH
    pub added_to_pool_timestamp: Option<u64>,
    // In bytes
    pub compression_size: Option<u64>,
    // In ms
    pub compression_duration: Option<u64>,
    // Milliseconds past UNIX EPOCH
    pub poly_grid_build_start_timestamp: Option<u64>,
    // Milliseconds past UNIX EPOCH
    pub poly_grid_build_end_timestamp: Option<u64>,
    // 0 means it was full
    pub queue_capacity: Option<u32>,
    // At what point in time did we measure the queue capacity
    pub queue_capacity_timestamp: Option<u64>,
    // Milliseconds past UNIX EPOCH
    pub commitment_grid_build_start_timestamp: Option<u64>,
    // Milliseconds past UNIX EPOCH
    pub commitment_grid_build_end_timestamp: Option<u64>,
    pub request: Option<BlobRequestData>,
    // Peer Address
    pub network_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Blob {
    pub hash: BlockHash,
    // usize is ChainNodeId
    pub map: HashMap<usize, NodeBlobView>,
}

impl Blob {
    pub fn new(hash: BlockHash) -> Self {
        Self {
            hash: hash,
            map: HashMap::default(),
        }
    }

    pub fn serialize(&self) -> String {
        let mut main_text = std::format!("BLOB_HASH: {:?}", self.hash);
        _ = std::writeln!(&mut main_text, "");

        let mut list: Vec<(usize, NodeBlobView)> = self.map.clone().into_iter().collect();
        list.sort_by(|x, y| {
            if x.1.submission_tracked.is_some() {
                return Ordering::Less;
            }

            if y.1.submission_tracked.is_some() {
                return Ordering::Greater;
            }

            x.1.request
                .as_ref()
                .map(|x| x.start)
                .cmp(&y.1.request.as_ref().map(|x| x.start))
        });

        for (key, view) in list {
            let mut sub_text = std::format!("NODE_ID: {}", key);

            if let Some(value) = &view.network_id {
                _ = std::write!(&mut sub_text, " PEER_ID: {}", value);
            }

            if let Some(start) = &view.submission_tracked
                && let Some(end) = &view.added_to_pool_timestamp
            {
                if let Some(value) = chrono::DateTime::from_timestamp_millis(*start as i64) {
                    _ = std::write!(&mut sub_text, " START: {}", value);
                }
                if let Some(value) = chrono::DateTime::from_timestamp_millis(*end as i64) {
                    _ = std::write!(&mut sub_text, " END: {}", value);
                }

                _ = std::write!(
                    &mut sub_text,
                    " SUBMISSION_DURATION: {} ms",
                    end.saturating_sub(*start)
                );
            }

            if let Some(value) = &view.size {
                _ = std::write!(&mut sub_text, " SIZE: {} bytes", value);
            }

            if let Some(start) = &view.poly_grid_build_start_timestamp
                && let Some(end) = &view.poly_grid_build_end_timestamp
            {
                _ = std::write!(
                    &mut sub_text,
                    " POLY_GRID_BUILD_DURATION: {} ms",
                    end.saturating_sub(*start)
                );
            }

            if let Some(start) = &view.commitment_grid_build_start_timestamp
                && let Some(end) = &view.commitment_grid_build_end_timestamp
            {
                _ = std::write!(
                    &mut sub_text,
                    " COMMITMENT_BUILD_DURATION: {} ms",
                    end.saturating_sub(*start)
                );
            }

            if let Some(value) = &view.compression_size {
                _ = std::write!(&mut sub_text, " COMPRESSED_SIZE: {} bytes", value);
            }
            if let Some(value) = &view.compression_duration {
                _ = std::write!(&mut sub_text, " COMPRESSION_DURATION: {} ms", value);
            }
            if let Some(value) = &view.queue_capacity {
                _ = std::write!(&mut sub_text, " QUEUE_CAPACITY: {}", value);
            }

            if let Some(request) = &view.request {
                if let Some(value) = chrono::DateTime::from_timestamp_millis(request.start as i64) {
                    _ = std::write!(&mut sub_text, " START: {}", value);
                }
                if let Some(value) = chrono::DateTime::from_timestamp_millis(request.end as i64) {
                    _ = std::write!(&mut sub_text, " END: {}", value);
                }

                _ = std::write!(&mut sub_text, " REQUEST FROM: {}", request.from);
                _ = std::write!(&mut sub_text, " REQUEST TO: {}", request.to);
                _ = std::write!(&mut sub_text, " DURATION: {} ms", request.duration);

                _ = std::write!(&mut sub_text, " SUCCESS: {}", request.success);
            }

            _ = std::writeln!(&mut main_text, "\t{}", sub_text);
        }

        main_text
    }

    pub fn submission(&mut self, node_id: usize, submission: &BlobSubmission) {
        let view = self.get(node_id);
        if let Some(value) = submission.size {
            view.size = Some(value);
        }
        if let Some(value) = submission.submission_tracked {
            view.submission_tracked = Some(value);
        }
        if let Some(value) = submission.added_to_pool_timestamp {
            view.added_to_pool_timestamp = Some(value);
        }
        if let Some(value) = submission.compression_size {
            view.compression_size = Some(value);
        }
        if let Some(value) = submission.compression_duration {
            view.compression_duration = Some(value);
        }
        if let Some(value) = submission.poly_grid_build_start_timestamp {
            view.poly_grid_build_start_timestamp = Some(value);
        }
        if let Some(value) = submission.poly_grid_build_end_timestamp {
            view.poly_grid_build_end_timestamp = Some(value);
        }
        if let Some(value) = submission.queue_capacity {
            view.queue_capacity = Some(value);
        }
        if let Some(value) = submission.queue_capacity_timestamp {
            view.queue_capacity_timestamp = Some(value);
        }
        if let Some(value) = submission.commitment_grid_build_start_timestamp {
            view.commitment_grid_build_start_timestamp = Some(value);
        }
        if let Some(value) = submission.commitment_grid_build_end_timestamp {
            view.commitment_grid_build_end_timestamp = Some(value);
        }
    }

    pub fn request(&mut self, node_id: usize, value: &BlobRequest) {
        let rq_data = BlobRequestData {
            duration: value.end.saturating_sub(value.start),
            end: value.end,
            start: value.start,
            from: value.from.clone(),
            to: value.to.clone(),
            success: value.success,
        };

        let view = self.get(node_id);
        view.request = Some(rq_data);
    }

    // Get or create
    fn get(&mut self, node_id: usize) -> &mut NodeBlobView {
        if !self.map.contains_key(&node_id) {
            self.map.insert(node_id, NodeBlobView::default());
        }

        self.map.get_mut(&node_id).expect("Just inserted. qed")
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SystemConnected {
    pub genesis_hash: BlockHash,
    pub node: NodeDetails,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SystemInterval {
    pub peers: Option<u64>,
    pub txcount: Option<u64>,
    pub bandwidth_upload: Option<f64>,
    pub bandwidth_download: Option<f64>,
    pub finalized_height: Option<BlockNumber>,
    pub finalized_hash: Option<BlockHash>,
    pub block: Option<Block>,
    pub used_state_cache_size: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Finalized {
    pub hash: BlockHash,
    pub height: Box<str>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AfgAuthoritySet {
    pub authority_id: Box<str>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NodeHwBench {
    pub cpu_hashrate_score: u64,
    pub memory_memcpy_score: u64,
    pub disk_sequential_write_score: Option<u64>,
    pub disk_random_write_score: Option<u64>,
    pub parallel_cpu_hashrate_score: Option<u64>,
    //// Dev note: this exists but isn't needed yet:
    // pub parallel_cpu_cores: Option<usize>,
}

impl Payload {
    pub fn best_block(&self) -> Option<&Block> {
        match self {
            Payload::BlockImport(block) => Some(block),
            Payload::SystemInterval(SystemInterval { block, .. }) => block.as_ref(),
            _ => None,
        }
    }

    pub fn finalized_block(&self) -> Option<Block> {
        match self {
            Payload::SystemInterval(interval) => Some(Block {
                hash: interval.finalized_hash?,
                height: interval.finalized_height?,
            }),
            Payload::NotifyFinalized(finalized) => Some(Block {
                hash: finalized.hash,
                height: finalized.height.parse().ok()?,
            }),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrayvec::ArrayString;
    use bincode::Options;

    // Without adding a derive macro and marker trait (and enforcing their use), we don't really
    // know whether things can (de)serialize to bincode or not at runtime without failing unless
    // we test the different types we want to (de)serialize ourselves. We just need to test each
    // type, not each variant.
    fn bincode_can_serialize_and_deserialize<'de, T>(item: T)
    where
        T: Serialize + serde::de::DeserializeOwned,
    {
        let bytes = bincode::serialize(&item).expect("Serialization should work");
        let _: T = bincode::deserialize(&bytes).expect("Deserialization should work");
    }

    #[test]
    fn bincode_can_serialize_and_deserialize_node_message_system_connected() {
        bincode_can_serialize_and_deserialize(NodeMessage::V1 {
            payload: Payload::SystemConnected(SystemConnected {
                genesis_hash: BlockHash::zero(),
                node: NodeDetails {
                    chain: "foo".into(),
                    name: "foo".into(),
                    implementation: "foo".into(),
                    version: "foo".into(),
                    target_arch: Some("x86_64".into()),
                    target_os: Some("linux".into()),
                    target_env: Some("env".into()),
                    validator: None,
                    network_id: ArrayString::new(),
                    startup_time: None,
                    sysinfo: None,
                    ip: Some("127.0.0.1".into()),
                },
            }),
        });
    }

    #[test]
    fn bincode_can_serialize_and_deserialize_node_message_system_interval() {
        bincode_can_serialize_and_deserialize(NodeMessage::V1 {
            payload: Payload::SystemInterval(SystemInterval {
                peers: None,
                txcount: None,
                bandwidth_upload: None,
                bandwidth_download: None,
                finalized_height: None,
                finalized_hash: None,
                block: None,
                used_state_cache_size: None,
            }),
        });
    }

    #[test]
    fn bincode_can_serialize_and_deserialize_node_message_block_import() {
        bincode_can_serialize_and_deserialize(NodeMessage::V1 {
            payload: Payload::BlockImport(Block {
                hash: BlockHash([0; 32]),
                height: 0,
            }),
        });
    }

    #[test]
    fn bincode_can_serialize_and_deserialize_node_message_notify_finalized() {
        bincode_can_serialize_and_deserialize(NodeMessage::V1 {
            payload: Payload::NotifyFinalized(Finalized {
                hash: BlockHash::zero(),
                height: "foo".into(),
            }),
        });
    }

    #[test]
    fn bincode_can_serialize_and_deserialize_node_message_afg_authority_set() {
        bincode_can_serialize_and_deserialize(NodeMessage::V1 {
            payload: Payload::AfgAuthoritySet(AfgAuthoritySet {
                authority_id: "foo".into(),
            }),
        });
    }

    #[test]
    fn bincode_block_zero() {
        let raw = Block::zero();

        let bytes = bincode::options().serialize(&raw).unwrap();

        let deserialized: Block = bincode::options().deserialize(&bytes).unwrap();

        assert_eq!(raw.hash, deserialized.hash);
        assert_eq!(raw.height, deserialized.height);
    }
}

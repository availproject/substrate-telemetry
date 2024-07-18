use common::node_message::IntervalKind;
use common::node_types::{Block, NodeDetails, NodeStats};
use common::node_types::{BlockHash, BlockNumber};
use serde::Serialize;
use std::collections::VecDeque;

// This is the struct that will returned back by /overview/ endpoint
#[derive(Serialize, Debug, Clone)]
pub struct ChainOverviewEx {
    pub genesis_hash: BlockHash,
    pub max_nodes: usize,
    pub node_count: usize,
    pub best_block: Block,
    pub finalized_block: Block,
    pub average_block_time: Option<u64>,
    pub forks: Vec<BlockNumberToHashes>,
    pub node_implementations: Vec<NodeImplementation>,
    pub blocks: Vec<BlockNumberToBlockData>,
    pub nodes: Vec<NodeDetailsEx>,
}

const MAX_BLOCKS_SIZE: usize = 15;
//const MAX_FORKS_SIZE: usize = 50;

#[derive(Debug, Clone)]
pub struct ChainOverview {
    blocks: VecDeque<(BlockNumber, Vec<BlockHashStats>)>,
}

impl ChainOverview {
    pub fn new() -> Self {
        Self {
            blocks: VecDeque::with_capacity(MAX_BLOCKS_SIZE + 5),
        }
    }

    pub fn get_forks(&self) -> Vec<BlockNumberToHashes> {
        let mut forks: Vec<BlockNumberToHashes> = self
            .blocks
            .iter()
            .filter(|(_, value)| value.len() > 1)
            .map(|(key, value)| (*key, value.iter().map(|h| h.hash).collect()))
            .map(|(block_number, block_hashes)| BlockNumberToHashes {
                block_number,
                block_hashes,
            })
            .collect();

        forks.sort_by(|a, b| b.block_number.cmp(&a.block_number));

        forks
    }

    pub fn get_blocks(&self) -> Vec<BlockNumberToBlockData> {
        let mut result: Vec<BlockNumberToBlockData> = Vec::new();

        for (block_number, blocks) in self.blocks.iter() {
            for block in blocks.iter() {
                let mut data = BlockNumberToBlockData {
                    block_number: block_number.clone(),
                    block_hashes: block.hash.clone(),
                    block_producer: None,
                };
                if let Some(proposed_kind) =
                    block.data.iter().find(|d| d.kind == IntervalKind::Proposal)
                {
                    data.block_producer = Some(BlockProducer {
                        proposed_time: proposed_kind.end.clone(),
                        node_id: proposed_kind.node_id.clone(),
                        node_name: proposed_kind.node_name.clone(),
                        node_network_id: proposed_kind.node_network_id.clone(),
                    });
                }

                result.push(data);
            }
        }

        result.sort_by(|a, b| b.block_number.cmp(&a.block_number));

        result
    }

    pub fn new_data(
        &mut self,
        node_id: usize,
        node_name: Box<str>,
        node_network_id: Box<str>,
        block_hash: BlockHash,
        block_height: BlockNumber,
        interval_kind: IntervalKind,
        interval_start: String,
        interval_end: String,
        interval_duration: u64,
    ) {
        let value = BlockStats {
            node_id,
            node_name,
            node_network_id,
            kind: interval_kind,
            start: interval_start,
            end: interval_end,
            duration_in_ms: interval_duration,
        };

        let existing_height = self
            .blocks
            .iter_mut()
            .find(|(height, _)| *height == block_height);

        if let Some(height) = existing_height {
            let existing_hash = height.1.iter_mut().find(|h| h.hash == block_hash);
            match existing_hash {
                Some(stats) => stats.data.push(value),
                None => {
                    height.1.push(BlockHashStats {
                        hash: block_hash,
                        data: vec![value],
                    });
                }
            };
        } else {
            let mut should_sort = false;
            if let Some(front) = self.blocks.front() {
                should_sort = block_height < front.0;
            }

            self.blocks.push_front((
                block_height,
                vec![BlockHashStats {
                    hash: block_hash,
                    data: vec![value],
                }],
            ));
            if should_sort {
                self.blocks.make_contiguous().sort_by(|a, b| b.0.cmp(&a.0));
            }
        }

        if self.blocks.len() > MAX_BLOCKS_SIZE {
            self.blocks.pop_back();
        }
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct NodeOverview {
    pub id: usize,
    pub details: NodeDetails,
    pub stats: NodeStats,
    pub best_block: Block,
    pub finalized_block: Block,
}

#[derive(Serialize, Debug, Clone)]
pub struct BlockHashStats {
    hash: BlockHash,
    #[serde(skip_serializing)]
    data: Vec<BlockStats>,
}

#[derive(Serialize, Debug, Clone)]
pub struct BlockStats {
    node_id: usize,
    node_name: Box<str>,
    node_network_id: Box<str>,
    kind: IntervalKind,
    start: String,
    end: String,
    duration_in_ms: u64,
}

#[derive(Serialize, Debug, Clone)]
pub struct NodeImplementation {
    pub version: Box<str>,
    pub count: usize,
    pub nodes: Vec<NodeImplementationData>,
}

#[derive(Serialize, Debug, Clone)]
pub struct NodeImplementationData {
    pub id: usize,
    pub node_name: Box<str>,
    pub network_id: Box<str>,
}

#[derive(Serialize, Debug, Clone)]
pub struct BlockNumberToHashes {
    pub block_number: BlockNumber,
    pub block_hashes: Vec<BlockHash>,
}

#[derive(Serialize, Debug, Clone)]
pub struct BlockNumberToBlockData {
    pub block_number: BlockNumber,
    pub block_hashes: BlockHash,
    pub block_producer: Option<BlockProducer>,
}

#[derive(Serialize, Debug, Clone)]
pub struct BlockProducer {
    pub proposed_time: String,
    pub node_id: usize,
    pub node_name: Box<str>,
    pub node_network_id: Box<str>,
}

#[derive(Serialize, Debug, Clone)]
pub struct NodeDetailsEx {
    pub node_id: usize,
    pub details: NodeDetails,
    pub best_block: Block,
    pub finalized_block: Block,
    pub best_block_timestamp: u64,
    pub peers: u64,
    pub txcount: u64,
    pub stale: bool,
}

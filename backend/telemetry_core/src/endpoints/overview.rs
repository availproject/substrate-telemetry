use std::collections::HashMap;

use common::node_types::{Block, NodeDetails};
use common::node_types::{BlockHash, BlockNumber};
use serde::Serialize;

use crate::state;
use crate::state::blocks::StoredBlocks;

use super::shared::{BlockProducer, SUniqueNodeIdentity};

// This is the struct that will returned back by /overview/ endpoint
#[derive(Serialize, Debug, Clone)]
pub struct ChainOverview {
    pub genesis_hash: BlockHash,
    pub best_block: Block,
    pub finalized_block: Block,
    pub max_nodes: usize,
    pub node_count: usize,
    pub average_block_time: Option<u64>,
    pub implementations: OverviewImplementations,
    pub forks: OverviewForks,
    pub blocks: OverviewBlocks,
}

#[derive(Serialize, Debug, Clone)]
pub struct OverviewFork {
    pub block_height: BlockNumber,
    pub blocks: Vec<OverviewForkBlock>,
}

impl OverviewFork {
    pub fn new(block_height: BlockNumber) -> Self {
        Self {
            block_height,
            blocks: Default::default(),
        }
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct OverviewForkBlock {
    pub block_hash: BlockHash,
    pub block_producer: Option<BlockProducer>,
    pub number_of_witnesses: usize,
}

#[derive(Serialize, Debug, Clone)]
pub struct OverviewForks(Vec<OverviewFork>);

impl From<&StoredBlocks> for OverviewForks {
    fn from(value: &StoredBlocks) -> Self {
        let mut forks: Vec<OverviewFork> = Vec::new();

        for (block_height, blocks) in value.0.iter().rev() {
            if blocks.len() < 2 {
                continue;
            }

            let mut fork = OverviewFork::new(*block_height);

            for (block_hash, nodes) in blocks {
                let block_producer: Option<BlockProducer> =
                    nodes.iter().find_map(|(id, details)| {
                        if let Some(proposal) = &details.proposal {
                            Some(BlockProducer {
                                identity: id.into(),
                                start: proposal.start_timestamp.into(),
                                end: proposal.end_timestamp.into(),
                            })
                        } else {
                            None
                        }
                    });

                fork.blocks.push(OverviewForkBlock {
                    block_hash: *block_hash,
                    block_producer,
                    number_of_witnesses: nodes.len(),
                })
            }
            forks.push(fork)
        }

        Self(forks)
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct OverviewImplementation {
    pub version: Box<str>,
    pub count: usize,
}

#[derive(Serialize, Debug, Clone)]
pub struct OverviewImplementations(Vec<OverviewImplementation>);

impl From<&state::Chain> for OverviewImplementations {
    fn from(value: &state::Chain) -> Self {
        let mut implementations: HashMap<Box<str>, Vec<SUniqueNodeIdentity>> = HashMap::new();

        for node in value.nodes_slice() {
            let Some(node) = node else { continue };
            let imp = node.details().version.clone();
            let entry = implementations.entry(imp).or_default();
            entry.push((&node.identity()).into());
        }

        let mut implementations: Vec<OverviewImplementation> = implementations
            .into_iter()
            .map(|(key, value)| OverviewImplementation {
                version: key,
                count: value.len(),
            })
            .collect();

        implementations.sort_by(|a, b| b.version.cmp(&a.version));

        Self(implementations)
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct OverviewBlocks(Vec<OverviewBlock>);

#[derive(Serialize, Debug, Clone)]
pub struct OverviewBlock {
    pub block_height: BlockNumber,
    pub block_hash: BlockHash,
    pub block_producer: Option<BlockProducer>,
}

impl From<&StoredBlocks> for OverviewBlocks {
    fn from(value: &StoredBlocks) -> Self {
        let mut overview_blocks: Vec<OverviewBlock> = Vec::new();

        for (block_height, blocks) in value.0.iter().rev() {
            for (block_hash, nodes) in blocks {
                let mut block_producer: Option<BlockProducer> = None;

                for (id, details) in nodes {
                    if let Some(proposal) = &details.proposal {
                        let value = BlockProducer {
                            identity: id.into(),
                            start: proposal.start_timestamp.into(),
                            end: proposal.end_timestamp.into(),
                        };
                        block_producer = Some(value);
                        break;
                    }
                }

                overview_blocks.push(OverviewBlock {
                    block_height: *block_height,
                    block_hash: *block_hash,
                    block_producer,
                })
            }
        }

        Self(overview_blocks)
    }
}

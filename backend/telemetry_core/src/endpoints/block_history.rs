use common::{node_message::IntervalFromNode, node_types::BlockHash};
use serde::Serialize;

use crate::state::blocks::StoredBlocks;

use super::shared::{SDateTime, SUniqueNodeIdentity};

// This is the struct that will returned back by /block_history/ endpoint
#[derive(Serialize, Debug, Clone)]
pub struct BlockHistory(Vec<BlockHistoryBlockHeight>);

#[derive(Serialize, Debug, Clone)]

pub struct BlockHistoryBlockHeight {
    pub block_height: u64,
    pub blocks: Vec<BlockHistoryBlock>,
}

#[derive(Serialize, Debug, Clone)]
pub struct BlockHistoryBlock {
    pub block_hash: BlockHash,
    pub nodes: Vec<BlockHistoryNodeData>,
}

#[derive(Serialize, Debug, Clone)]
pub struct BlockHistoryNodeData {
    pub identity: SUniqueNodeIdentity,
    pub proposal: Option<BlockHistoryDetail>,
    pub import: Option<BlockHistoryDetail>,
    pub sync: Option<BlockHistoryDetail>,
}

#[derive(Serialize, Debug, Clone)]
pub struct BlockHistoryDetail {
    pub peer_id: Option<String>,
    pub start_timestamp: SDateTime,
    pub end_timestamp: SDateTime,
}

impl From<&IntervalFromNode> for BlockHistoryDetail {
    fn from(value: &IntervalFromNode) -> Self {
        Self {
            peer_id: value.peer_id.clone(),
            start_timestamp: value.start_timestamp.into(),
            end_timestamp: value.end_timestamp.into(),
        }
    }
}

impl From<&StoredBlocks> for BlockHistory {
    fn from(value: &StoredBlocks) -> Self {
        let mut result: Vec<BlockHistoryBlockHeight> = Vec::new();
        for (block_height, blocks) in value.0.iter().rev() {
            let mut bh_blocks: Vec<BlockHistoryBlock> = Vec::new();
            for (block_hash, block) in blocks {
                let mut nodes: Vec<BlockHistoryNodeData> = Vec::new();
                for (identity, data) in block {
                    let detail = BlockHistoryNodeData {
                        identity: identity.into(),
                        proposal: data.proposal.as_ref().and_then(|p| Some(p.into())),
                        import: data.import.as_ref().and_then(|p| Some(p.into())),
                        sync: data.sync.as_ref().and_then(|p| Some(p.into())),
                    };
                    nodes.push(detail);
                }
                let bh_block = BlockHistoryBlock {
                    block_hash: block_hash.clone(),
                    nodes,
                };
                bh_blocks.push(bh_block);
            }
            result.push(BlockHistoryBlockHeight {
                block_height: *block_height,
                blocks: bh_blocks,
            });
        }

        Self(result)
    }
}

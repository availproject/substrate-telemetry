use std::collections::BTreeMap;

use common::{
    node_message::IntervalFromNode,
    node_types::{Block, BlockHash, BlockNumber},
};

use super::node::UniqueNodeIdentity;

const MAX_BLOCK_HEIGHT_STORED: usize = 30;

#[derive(Debug, Clone, Default)]
pub struct BlockIntervalDetails {
    pub proposal: Option<IntervalFromNode>,
    pub import: Option<IntervalFromNode>,
    pub sync: Option<IntervalFromNode>,
}

#[derive(Debug, Clone, Default)]
pub struct StoredBlocks(
    pub  BTreeMap<
        BlockNumber,
        BTreeMap<BlockHash, BTreeMap<UniqueNodeIdentity, BlockIntervalDetails>>,
    >,
);

impl StoredBlocks {
    pub fn new_entry(
        &mut self,
        identity: UniqueNodeIdentity,
        block: Block,
        proposal: Option<IntervalFromNode>,
        import: Option<IntervalFromNode>,
        sync: Option<IntervalFromNode>,
    ) {
        let data = BlockIntervalDetails {
            proposal,
            import,
            sync,
        };

        let blocks = self.0.entry(block.height).or_default();
        let block = blocks.entry(block.hash).or_default();
        block.insert(identity, data);

        // Trim Old Data if needed.
        while self.0.len() > MAX_BLOCK_HEIGHT_STORED {
            self.0.pop_first();
        }
    }
}

use std::collections::HashMap;

use common::node_types::{Block, NodeDetails};
use serde::Serialize;

use crate::state::{Chain, Node};

use super::shared::SUniqueNodeIdentity;

// This is the struct that will returned back by /block_history/ endpoint
#[derive(Serialize, Debug, Clone)]
pub struct NodeList {
    implementations: Vec<NodeListImplementations>,
    nodes: Vec<NodeListNodeDetails>,
}

// This is what we expose and serialize
#[derive(Serialize, Debug, Clone)]
pub struct NodeListImplementations {
    pub version: Box<str>,
    pub nodes: Vec<SUniqueNodeIdentity>,
    pub count: usize,
}

impl From<&Chain> for NodeList {
    fn from(value: &Chain) -> Self {
        let mut implementations: HashMap<Box<str>, Vec<SUniqueNodeIdentity>> = HashMap::new();

        for node in value.nodes_slice() {
            let Some(node) = node else { continue };
            let imp = node.details().version.clone();
            let entry = implementations.entry(imp).or_default();
            entry.push((&node.identity()).into());
        }

        let mut implementations: Vec<NodeListImplementations> = implementations
            .into_iter()
            .map(|(key, value)| NodeListImplementations {
                version: key,
                count: value.len(),
                nodes: value,
            })
            .collect();

        implementations.sort_by(|a, b| b.version.cmp(&a.version));

        let nodes: Vec<NodeListNodeDetails> = value
            .nodes_slice()
            .iter()
            .flatten()
            .map(|n| n.into())
            .collect();

        implementations.sort_by(|a, b| b.version.cmp(&a.version));

        Self {
            implementations,
            nodes,
        }
    }
}

// This is what we expose and serialize
#[derive(Serialize, Debug, Clone)]
pub struct NodeListNodeDetails {
    pub identity: SUniqueNodeIdentity,
    pub details: NodeDetails,
    pub best_block: Block,
    pub finalized_block: Block,
    pub best_block_timestamp: u64,
    pub peers: u64,
    pub txcount: u64,
    pub stale: bool,
    pub is_authority: Option<bool>,
}

impl From<&Node> for NodeListNodeDetails {
    fn from(value: &Node) -> Self {
        Self {
            identity: value.identity().into(),
            details: value.details().clone(),
            best_block: value.best().clone(),
            finalized_block: value.finalized().clone(),
            best_block_timestamp: value.best_timestamp(),
            peers: value.stats().peers,
            txcount: value.stats().txcount,
            stale: value.stale(),
            is_authority: value.is_authority(),
        }
    }
}

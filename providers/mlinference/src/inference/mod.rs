mod engine;

mod tract;
use tract::TractSession;

use serde::{Deserialize, Serialize};
use std::{
    collections::{btree_map::Keys, BTreeMap}
};

/// Graph
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct Graph(u32);

/// GraphExecutionContext
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct GraphExecutionContext {
    pub gec: u32,
}

#[derive(Default)]
pub struct MlState {
    pub executions: BTreeMap<GraphExecutionContext, TractSession>,
    pub models: BTreeMap<Graph, Vec<u8>>,
}

impl MlState {
    /// Helper function that returns the key that is supposed to be inserted next.
    pub fn key<K: Into<u32> + From<u32> + Copy, V>(&self, keys: Keys<K, V>) -> K {
        match keys.last() {
            Some(&k) => {
                let last: u32 = k.into();
                K::from(last + 1)
            }
            None => K::from(0),
        }
    }
}

pub type IEResult<T> = Result<T, IEError>;

#[derive(Debug, thiserror::Error)]
pub enum IEError {
    #[error("Some error")]
    SomeError,
}
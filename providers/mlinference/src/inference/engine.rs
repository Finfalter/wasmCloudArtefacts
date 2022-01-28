use async_trait::async_trait;

use super::{IEResult, Graph};

/// InferenceEngine
#[async_trait]
pub trait InferenceEngine {
    async fn load(&self) -> IEResult<Graph>;
}



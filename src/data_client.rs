use dyn_clone::DynClone;

use crate::prometheus::PromMetric;

#[async_trait]
pub trait DataClient: DynClone {
    async fn get_metrics(&self) -> Result<Vec<PromMetric>, reqwest::Error>;
}

dyn_clone::clone_trait_object!(DataClient);

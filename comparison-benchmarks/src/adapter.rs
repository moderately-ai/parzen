use serde::{Deserialize, Serialize};

use crate::{
    HarnessResult,
    cli::RunConfig,
    fixtures::{FixtureTrial, Value},
    scenarios::Scenario,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Support {
    pub supported: bool,
    pub reason: Option<String>,
}

impl Support {
    #[must_use]
    pub const fn yes() -> Self {
        Self {
            supported: true,
            reason: None,
        }
    }

    #[must_use]
    pub fn no(reason: impl Into<String>) -> Self {
        Self {
            supported: false,
            reason: Some(reason.into()),
        }
    }
}

pub trait Backend: Sized {
    const NAME: &'static str;
    const VERSION: &'static str;

    fn support(scenario: Scenario) -> Support;
    fn semantics(config: &RunConfig) -> Vec<String>;
    fn create(config: &RunConfig) -> HarnessResult<Self>;
    fn ingest(&mut self, trial: &FixtureTrial) -> HarnessResult<()>;
    fn suggest(&mut self) -> HarnessResult<Vec<Value>>;
    fn complete(&mut self, objective: f64) -> HarnessResult<()>;
    fn abort(&mut self) -> HarnessResult<()>;
    fn observations(&self) -> usize;
}

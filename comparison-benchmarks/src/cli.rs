use std::{ffi::OsString, str::FromStr, time::Duration};

use serde::{Deserialize, Serialize};

use crate::{
    HarnessResult,
    scenarios::{Operation, ParzenHistory, Scenario},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    Human,
    Json,
}

impl FromStr for OutputFormat {
    type Err = crate::HarnessError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "human" => Ok(Self::Human),
            "json" => Ok(Self::Json),
            _ => Err(format!("unknown output format `{value}`").into()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunConfig {
    pub scenario: Scenario,
    pub operation: Operation,
    pub history: usize,
    pub dimensions: usize,
    pub iterations: usize,
    pub budget: usize,
    pub seed: u64,
    pub samples: usize,
    pub warmup: usize,
    pub profile_seconds: u64,
    pub parzen_history: ParzenHistory,
    pub machine_label: String,
}

impl Default for RunConfig {
    fn default() -> Self {
        Self {
            scenario: Scenario::LinearFloat,
            operation: Operation::Cycle,
            history: 1_000,
            dimensions: 0,
            iterations: 0,
            budget: 100,
            seed: 42,
            samples: 10,
            warmup: 3,
            profile_seconds: 30,
            parzen_history: ParzenHistory::Full,
            machine_label: "unlabelled".to_owned(),
        }
    }
}

impl RunConfig {
    pub fn validate(&self) -> HarnessResult<()> {
        if self.dimensions == 0 {
            return Err("dimensions must be positive".into());
        }
        if self.samples == 0 {
            return Err("samples must be positive".into());
        }
        if self.budget < 10 && self.operation == Operation::Quality {
            return Err("quality budget must include the ten startup trials".into());
        }
        if self.profile_seconds == 0 && self.operation == Operation::Profile {
            return Err("profile duration must be positive".into());
        }
        Ok(())
    }

    #[must_use]
    pub const fn profile_duration(&self) -> Duration {
        Duration::from_secs(self.profile_seconds)
    }
}

#[derive(Debug, Clone)]
pub struct BackendCli {
    pub config: RunConfig,
    pub format: OutputFormat,
}

impl BackendCli {
    pub fn parse<I, S>(args: I) -> HarnessResult<Self>
    where
        I: IntoIterator<Item = S>,
        S: Into<OsString>,
    {
        let mut config = RunConfig::default();
        let mut format = OutputFormat::Human;
        let mut args = args.into_iter().map(Into::into);
        while let Some(flag) = args.next() {
            let flag = flag.into_string().map_err(|_| "arguments must be UTF-8")?;
            let mut value = || -> HarnessResult<String> {
                args.next()
                    .ok_or_else(|| -> crate::HarnessError {
                        format!("missing value for `{flag}`").into()
                    })?
                    .into_string()
                    .map_err(|_| "arguments must be UTF-8".into())
            };
            match flag.as_str() {
                "--scenario" => config.scenario = value()?.parse()?,
                "--operation" => config.operation = value()?.parse()?,
                "--history" => config.history = value()?.parse()?,
                "--dimensions" => config.dimensions = value()?.parse()?,
                "--iterations" => config.iterations = value()?.parse()?,
                "--budget" => config.budget = value()?.parse()?,
                "--seed" => config.seed = value()?.parse()?,
                "--samples" => config.samples = value()?.parse()?,
                "--warmup" => config.warmup = value()?.parse()?,
                "--profile-seconds" => config.profile_seconds = value()?.parse()?,
                "--parzen-history" => config.parzen_history = value()?.parse()?,
                "--machine-label" => config.machine_label = value()?,
                "--format" => format = value()?.parse()?,
                "--help" | "-h" => return Err(Self::usage().into()),
                _ => return Err(format!("unknown argument `{flag}`\n{}", Self::usage()).into()),
            }
        }
        if config.dimensions == 0 {
            config.dimensions = config.scenario.default_dimensions();
        }
        config.validate()?;
        Ok(Self { config, format })
    }

    #[must_use]
    pub const fn usage() -> &'static str {
        "options: --scenario NAME --operation NAME --history N --dimensions N \
         --iterations N --budget N --seed N --samples N --warmup N \
         --profile-seconds N --parzen-history full|bounded --format human|json"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_one_dimension_is_preserved() {
        let cli = BackendCli::parse(["--scenario", "independent-float", "--dimensions", "1"])
            .expect("CLI");
        assert_eq!(cli.config.dimensions, 1);
    }

    #[test]
    fn scenario_default_dimension_is_used_when_omitted() {
        let cli = BackendCli::parse(["--scenario", "correlated-numeric"]).expect("CLI");
        assert_eq!(cli.config.dimensions, 4);
    }

    #[test]
    fn quality_rejects_budget_below_shared_design() {
        assert!(BackendCli::parse(["--operation", "quality", "--budget", "9"]).is_err());
    }
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProfileWorkload {
    FixedSuggest,
    Cycle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BenchmarkProtocol {
    Quick,
    Checkpoint,
    Curated,
}

impl BenchmarkProtocol {
    #[must_use]
    pub const fn checksum_tag(self) -> u64 {
        match self {
            Self::Quick => 0x5155_4943_4b00_0001,
            Self::Checkpoint => 0x4348_4543_4b50_0002,
            Self::Curated => 0x4355_5241_5445_0003,
        }
    }

    #[must_use]
    pub const fn warmups(self) -> usize {
        match self {
            Self::Quick => 1,
            Self::Checkpoint => 2,
            Self::Curated => 3,
        }
    }

    #[must_use]
    pub const fn calibration_ms(self) -> u64 {
        match self {
            Self::Quick => 25,
            Self::Checkpoint => 100,
            Self::Curated => 250,
        }
    }

    #[must_use]
    pub const fn samples(self) -> usize {
        match self {
            Self::Quick => 3,
            Self::Checkpoint => 5,
            Self::Curated => 10,
        }
    }

    #[must_use]
    pub const fn rounds(self) -> usize {
        match self {
            Self::Quick => 2,
            Self::Checkpoint => 4,
            Self::Curated => 8,
        }
    }

    #[must_use]
    pub const fn case_timeout_seconds(self) -> u64 {
        match self {
            Self::Quick => 45,
            Self::Checkpoint => 120,
            Self::Curated => 300,
        }
    }

    #[must_use]
    pub const fn suite_timeout_seconds(self) -> u64 {
        match self {
            Self::Quick => 8 * 60,
            Self::Checkpoint => 30 * 60,
            Self::Curated => 45 * 60,
        }
    }

    #[must_use]
    pub const fn max_calibration_iterations(self, state_growing: bool) -> usize {
        match (self, state_growing) {
            (Self::Quick, true) => 25,
            (Self::Checkpoint, true) => 50,
            (Self::Curated, true) => 100,
            (Self::Quick, false) => 65_536,
            (Self::Checkpoint, false) => 262_144,
            (Self::Curated, false) => 1_048_576,
        }
    }
}

impl FromStr for BenchmarkProtocol {
    type Err = crate::HarnessError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "quick" => Ok(Self::Quick),
            "checkpoint" => Ok(Self::Checkpoint),
            "curated" => Ok(Self::Curated),
            _ => Err(format!("unknown benchmark protocol `{value}`").into()),
        }
    }
}

impl std::fmt::Display for BenchmarkProtocol {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Quick => formatter.write_str("quick"),
            Self::Checkpoint => formatter.write_str("checkpoint"),
            Self::Curated => formatter.write_str("curated"),
        }
    }
}

impl ProfileWorkload {
    #[must_use]
    pub const fn checksum_tag(self) -> u64 {
        match self {
            Self::FixedSuggest => 0x4649_5845_445f_5355,
            Self::Cycle => 0x4359_434c_455f_5052,
        }
    }
}

impl FromStr for ProfileWorkload {
    type Err = crate::HarnessError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "fixed-suggest" => Ok(Self::FixedSuggest),
            "cycle" => Ok(Self::Cycle),
            _ => Err(format!("unknown profile workload `{value}`").into()),
        }
    }
}

impl std::fmt::Display for ProfileWorkload {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FixedSuggest => formatter.write_str("fixed-suggest"),
            Self::Cycle => formatter.write_str("cycle"),
        }
    }
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
    pub protocol: BenchmarkProtocol,
    pub scenario: Scenario,
    pub operation: Operation,
    pub history: usize,
    pub dimensions: usize,
    pub integer_cardinality: usize,
    pub iterations: usize,
    pub budget: usize,
    pub seed: u64,
    pub samples: usize,
    pub warmup: usize,
    pub calibration_ms: u64,
    pub profile_seconds: u64,
    pub profile_workload: ProfileWorkload,
    pub parzen_history: ParzenHistory,
    pub machine_label: String,
}

impl Default for RunConfig {
    fn default() -> Self {
        Self {
            protocol: BenchmarkProtocol::Quick,
            scenario: Scenario::LinearFloat,
            operation: Operation::Cycle,
            history: 1_000,
            dimensions: 0,
            integer_cardinality: 201,
            iterations: 0,
            budget: 100,
            seed: 42,
            samples: BenchmarkProtocol::Quick.samples(),
            warmup: 1,
            calibration_ms: BenchmarkProtocol::Quick.calibration_ms(),
            profile_seconds: 30,
            profile_workload: ProfileWorkload::Cycle,
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
        if self.integer_cardinality < 2 {
            return Err("integer cardinality must be at least two".into());
        }
        if self.integer_cardinality > i32::MAX as usize {
            return Err("integer cardinality exceeds backend range limits".into());
        }
        if self.operation == Operation::Quality
            && self.scenario == Scenario::Integer
            && self.integer_cardinality < 118
        {
            return Err(
                "integer quality requires a domain containing the known optimum at 17".into(),
            );
        }
        if self.calibration_ms == 0 && self.iterations == 0 && self.operation.is_batchable() {
            return Err("calibration duration must be positive for automatic timing".into());
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

    #[must_use]
    pub const fn calibration_duration(&self) -> Duration {
        Duration::from_millis(self.calibration_ms)
    }
}

#[derive(Debug, Clone)]
pub struct BackendCli {
    pub config: RunConfig,
    pub format: OutputFormat,
    pub calibrated_iterations: Option<usize>,
}

impl BackendCli {
    pub fn parse<I, S>(args: I) -> HarnessResult<Self>
    where
        I: IntoIterator<Item = S>,
        S: Into<OsString>,
    {
        let mut config = RunConfig::default();
        let mut format = OutputFormat::Human;
        let mut calibrated_iterations = None;
        let mut profile_workload_explicit = false;
        let mut samples_explicit = false;
        let mut warmup_explicit = false;
        let mut calibration_explicit = false;
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
                "--protocol" => config.protocol = value()?.parse()?,
                "--scenario" => config.scenario = value()?.parse()?,
                "--operation" => config.operation = value()?.parse()?,
                "--history" => config.history = value()?.parse()?,
                "--dimensions" => config.dimensions = value()?.parse()?,
                "--integer-cardinality" => config.integer_cardinality = value()?.parse()?,
                "--iterations" => config.iterations = value()?.parse()?,
                "--budget" => config.budget = value()?.parse()?,
                "--seed" => config.seed = value()?.parse()?,
                "--samples" => {
                    config.samples = value()?.parse()?;
                    samples_explicit = true;
                }
                "--warmup" => {
                    config.warmup = value()?.parse()?;
                    warmup_explicit = true;
                }
                "--calibration-ms" => {
                    config.calibration_ms = value()?.parse()?;
                    calibration_explicit = true;
                }
                "--calibrated-iterations" => calibrated_iterations = Some(value()?.parse()?),
                "--profile-seconds" => config.profile_seconds = value()?.parse()?,
                "--profile-workload" => {
                    config.profile_workload = value()?.parse()?;
                    profile_workload_explicit = true;
                }
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
        if !samples_explicit {
            config.samples = config.protocol.samples();
        }
        if !warmup_explicit {
            config.warmup = config.protocol.warmups();
        }
        if !calibration_explicit {
            config.calibration_ms = config.protocol.calibration_ms();
        }
        if profile_workload_explicit && config.operation != Operation::Profile {
            return Err("--profile-workload requires --operation profile".into());
        }
        config.validate()?;
        if calibrated_iterations == Some(0) {
            return Err("reused calibration iterations must be positive".into());
        }
        if calibrated_iterations.is_some() && config.iterations != 0 {
            return Err("--calibrated-iterations conflicts with --iterations".into());
        }
        Ok(Self {
            config,
            format,
            calibrated_iterations,
        })
    }

    #[must_use]
    pub const fn usage() -> &'static str {
        "options: --protocol quick|checkpoint|curated --scenario NAME --operation NAME --history N --dimensions N \
         --iterations N --budget N --seed N --samples N --warmup N \
         --integer-cardinality N \
         --calibration-ms N --profile-seconds N \
         --profile-workload fixed-suggest|cycle --parzen-history full|bounded \
         --format human|json"
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

    #[test]
    fn integer_cardinality_is_validated() {
        let cli = BackendCli::parse(["--scenario", "integer", "--integer-cardinality", "4096"])
            .expect("CLI");
        assert_eq!(cli.config.integer_cardinality, 4_096);
        assert!(BackendCli::parse(["--integer-cardinality", "1"]).is_err());
        assert!(
            BackendCli::parse([
                "--scenario",
                "integer",
                "--operation",
                "quality",
                "--integer-cardinality",
                "8",
            ])
            .is_err()
        );
    }

    #[test]
    fn routine_timing_defaults_are_bounded() {
        let cli = BackendCli::parse(["--scenario", "linear-float"]).expect("CLI");
        assert_eq!(cli.config.samples, 3);
        assert_eq!(cli.config.warmup, 1);
        assert_eq!(cli.config.calibration_ms, 25);
    }

    #[test]
    fn benchmark_protocols_parse() {
        for (name, expected) in [
            ("quick", BenchmarkProtocol::Quick),
            ("checkpoint", BenchmarkProtocol::Checkpoint),
            ("curated", BenchmarkProtocol::Curated),
        ] {
            let cli = BackendCli::parse(["--protocol", name]).expect("protocol");
            assert_eq!(cli.config.protocol, expected);
            assert_eq!(cli.config.samples, expected.samples());
            assert_eq!(cli.config.warmup, expected.warmups());
            assert_eq!(cli.config.calibration_ms, expected.calibration_ms());
        }
    }

    #[test]
    fn explicit_timing_values_override_protocol_defaults() {
        let cli = BackendCli::parse([
            "--protocol",
            "curated",
            "--samples",
            "2",
            "--warmup",
            "0",
            "--calibration-ms",
            "7",
        ])
        .expect("CLI");
        assert_eq!(cli.config.samples, 2);
        assert_eq!(cli.config.warmup, 0);
        assert_eq!(cli.config.calibration_ms, 7);
    }

    #[test]
    fn reused_calibration_is_separate_from_requested_iterations() {
        let cli = BackendCli::parse(["--calibrated-iterations", "17"]).expect("CLI");
        assert_eq!(cli.config.iterations, 0);
        assert_eq!(cli.calibrated_iterations, Some(17));
        assert!(BackendCli::parse(["--calibrated-iterations", "17", "--iterations", "2"]).is_err());
    }

    #[test]
    fn profile_workloads_parse_and_cycle_is_the_default() {
        let default = BackendCli::parse(["--operation", "profile"]).expect("default profile");
        assert_eq!(default.config.profile_workload, ProfileWorkload::Cycle);
        let fixed = BackendCli::parse([
            "--operation",
            "profile",
            "--profile-workload",
            "fixed-suggest",
        ])
        .expect("fixed profile");
        assert_eq!(fixed.config.profile_workload, ProfileWorkload::FixedSuggest);
    }

    #[test]
    fn profile_workload_is_rejected_for_non_profile_operations() {
        assert!(
            BackendCli::parse(["--operation", "cycle", "--profile-workload", "cycle"]).is_err()
        );
    }
}

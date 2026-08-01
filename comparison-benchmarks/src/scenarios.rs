use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};

use crate::HarnessError;

pub const QUALITY_SEEDS: [u64; 32] = [
    0x243f_6a88_85a3_08d3,
    0x1319_8a2e_0370_7344,
    0xa409_3822_299f_31d0,
    0x082e_fa98_ec4e_6c89,
    0x4528_21e6_38d0_1377,
    0xbe54_66cf_34e9_0c6c,
    0xc0ac_29b7_c97c_50dd,
    0x3f84_d5b5_b547_0917,
    0x9216_d5d9_8979_fb1b,
    0xd131_0ba6_98df_b5ac,
    0x2ffd_72db_d01a_dfb7,
    0xb8e1_afed_6a26_7e96,
    0xba7c_9045_f12c_7f99,
    0x24a1_9947_b391_6cf7,
    0x0801_f2e2_858e_fc16,
    0x6369_20d8_7157_4e69,
    0xa458_fea3_f493_3d7e,
    0x0d95_748f_728e_b658,
    0x718b_cd58_8215_4aee,
    0x7b54_a41d_c25a_59b5,
    0x9c30_d539_2af2_6013,
    0xc5d1_b023_2860_85f0,
    0xca41_7918_b8db_38ef,
    0x8e79_dcb0_603a_180e,
    0x6c9e_0e8b_b01e_8a3e,
    0xd715_77c1_bd31_4b27,
    0x78af_2fda_5560_5c60,
    0xe655_25f3_aa55_ab94,
    0x5748_9862_63e8_1440,
    0x55ca_396a_2aab_10b6,
    0xb4cc_5c34_1141_e8ce,
    0xa154_86af_7c72_e993,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Scenario {
    LinearFloat,
    IndependentFloat,
    Categorical,
    Integer,
    SteppedInteger,
    LogFloat,
    MixedIndependent,
    Conditional,
    CorrelatedNumeric,
    CorrelatedMixed,
}

impl Scenario {
    pub const COMPARATIVE: [Self; 9] = [
        Self::LinearFloat,
        Self::IndependentFloat,
        Self::Categorical,
        Self::Integer,
        Self::SteppedInteger,
        Self::LogFloat,
        Self::MixedIndependent,
        Self::Conditional,
        Self::CorrelatedNumeric,
    ];

    #[must_use]
    pub const fn default_dimensions(self) -> usize {
        match self {
            Self::IndependentFloat => 4,
            Self::CorrelatedNumeric => 4,
            Self::MixedIndependent => 3,
            Self::Conditional => 2,
            Self::CorrelatedMixed => 4,
            _ => 1,
        }
    }
}

impl fmt::Display for Scenario {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = serde_json::to_value(self)
            .ok()
            .and_then(|value| value.as_str().map(str::to_owned))
            .unwrap_or_else(|| "unknown".to_owned());
        f.write_str(&name)
    }
}

impl FromStr for Scenario {
    type Err = HarnessError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        serde_json::from_value(serde_json::Value::String(value.to_owned())).map_err(Into::into)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Operation {
    Construct,
    Ingest,
    ColdSuggest,
    Suggest,
    Update,
    Cycle,
    Quality,
    Memory,
    Profile,
}

impl Operation {
    /// Whether multiple operations can be timed in one representative batch.
    ///
    /// Ingest is one whole-history operation. Cold suggestion requires a fresh
    /// optimizer and history for each observation, making automatic batching
    /// mostly untimed setup work.
    #[must_use]
    pub const fn is_batchable(self) -> bool {
        matches!(
            self,
            Self::Construct | Self::Suggest | Self::Update | Self::Cycle
        )
    }
}

impl fmt::Display for Operation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = serde_json::to_value(self)
            .ok()
            .and_then(|value| value.as_str().map(str::to_owned))
            .unwrap_or_else(|| "unknown".to_owned());
        f.write_str(&name)
    }
}

impl FromStr for Operation {
    type Err = HarnessError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        serde_json::from_value(serde_json::Value::String(value.to_owned())).map_err(Into::into)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ParzenHistory {
    Full,
    Bounded,
}

impl FromStr for ParzenHistory {
    type Err = HarnessError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        serde_json::from_value(serde_json::Value::String(value.to_owned())).map_err(Into::into)
    }
}

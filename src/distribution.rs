// Copyright 2026 Thomas Santerre and Moderately AI Inc.
//
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Validated parameter distributions.

use crate::{ParamValue, ParzenError};

/// A categorical, floating-point, or integer search distribution.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Distribution {
    /// Categorical indices in `0..num_choices`.
    Categorical(CategoricalDistribution),
    /// A bounded floating-point distribution.
    Float(FloatDistribution),
    /// A bounded integer distribution.
    Int(IntDistribution),
}

/// A categorical distribution represented by choice indices.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct CategoricalDistribution {
    num_choices: u32,
}

impl CategoricalDistribution {
    /// Create a distribution with indices in `0..num_choices`.
    pub fn new(num_choices: u32) -> Result<Self, ParzenError> {
        if num_choices == 0 {
            return Err(ParzenError::InvalidDistribution(
                "categorical choice count must be positive".into(),
            ));
        }
        Ok(Self { num_choices })
    }

    /// Number of categorical choices.
    #[must_use]
    pub const fn num_choices(self) -> u32 {
        self.num_choices
    }
}

/// Scale used by a floating-point distribution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum FloatScale {
    /// Uniform linear coordinate space.
    Linear,
    /// Natural-log coordinate space.
    Log,
}

/// A bounded floating-point distribution.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct FloatDistribution {
    low: f64,
    high: f64,
    scale: FloatScale,
    step: Option<f64>,
}

impl FloatDistribution {
    /// Create a linear continuous distribution.
    pub fn linear(low: f64, high: f64) -> Result<Self, ParzenError> {
        Self::new(low, high, FloatScale::Linear)
    }

    /// Create a log-scaled continuous distribution.
    pub fn log(low: f64, high: f64) -> Result<Self, ParzenError> {
        Self::new(low, high, FloatScale::Log)
    }

    fn new(low: f64, high: f64, scale: FloatScale) -> Result<Self, ParzenError> {
        if !low.is_finite() || !high.is_finite() || low >= high {
            return Err(ParzenError::InvalidDistribution(
                "float bounds must be finite and low < high".into(),
            ));
        }
        if scale == FloatScale::Log && low <= 0.0 {
            return Err(ParzenError::InvalidDistribution(
                "log-float bounds must be positive".into(),
            ));
        }
        let transformed_low = match scale {
            FloatScale::Linear => low,
            FloatScale::Log => low.ln(),
        };
        let transformed_high = match scale {
            FloatScale::Linear => high,
            FloatScale::Log => high.ln(),
        };
        if !transformed_low.is_finite()
            || !transformed_high.is_finite()
            || transformed_low >= transformed_high
            || !(transformed_high - transformed_low).is_finite()
        {
            return Err(ParzenError::InvalidDistribution(
                "float bounds must define a finite, non-empty transformed interval".into(),
            ));
        }
        Ok(Self {
            low,
            high,
            scale,
            step: None,
        })
    }

    /// Quantize samples to a positive step anchored at `low`.
    pub fn with_step(mut self, step: f64) -> Result<Self, ParzenError> {
        if self.scale == FloatScale::Log {
            return Err(ParzenError::InvalidDistribution(
                "log-float distributions cannot have a step".into(),
            ));
        }
        if !step.is_finite() || step <= 0.0 {
            return Err(ParzenError::InvalidDistribution(
                "float step must be finite and positive".into(),
            ));
        }
        let units = ((self.high - self.low) / step).floor();
        const MAX_EXACT_INTEGER: f64 = 9_007_199_254_740_991.0;
        if !units.is_finite() || units > MAX_EXACT_INTEGER {
            return Err(ParzenError::InvalidDistribution(
                "float step creates too many exactly representable grid points".into(),
            ));
        }
        let highest = step.mul_add(units, self.low);
        if !highest.is_finite() || highest < self.low || highest > self.high {
            return Err(ParzenError::InvalidDistribution(
                "float step produces invalid grid arithmetic".into(),
            ));
        }
        if units >= 1.0 && step.mul_add(1.0, self.low) <= self.low {
            return Err(ParzenError::InvalidDistribution(
                "float step does not produce distinct grid values at this magnitude".into(),
            ));
        }
        let adapted_low = self.low - step / 2.0;
        let adapted_high = highest + step / 2.0;
        if !adapted_low.is_finite()
            || !adapted_high.is_finite()
            || adapted_low >= adapted_high
            || !(adapted_high - adapted_low).is_finite()
        {
            return Err(ParzenError::InvalidDistribution(
                "float step must define a finite discrete kernel domain".into(),
            ));
        }
        self.step = Some(step);
        Ok(self)
    }

    /// Inclusive lower bound.
    #[must_use]
    pub const fn low(self) -> f64 {
        self.low
    }
    /// Inclusive upper bound after clamping.
    #[must_use]
    pub const fn high(self) -> f64 {
        self.high
    }
    /// Coordinate scale.
    #[must_use]
    pub const fn scale(self) -> FloatScale {
        self.scale
    }
    /// Optional quantization step.
    #[must_use]
    pub const fn step(self) -> Option<f64> {
        self.step
    }

    pub(crate) fn transform(self, value: f64) -> f64 {
        match self.scale {
            FloatScale::Linear => value,
            FloatScale::Log => value.ln(),
        }
    }

    pub(crate) fn untransform(self, value: f64) -> f64 {
        let raw = match self.scale {
            FloatScale::Linear => value,
            FloatScale::Log => value.exp(),
        };
        self.quantize(raw)
    }

    pub(crate) fn quantize(self, value: f64) -> f64 {
        let value = if let Some(step) = self.step {
            let highest_unit = ((self.high - self.low) / step).floor();
            let unit = ((value - self.low) / step).round().clamp(0.0, highest_unit);
            step.mul_add(unit, self.low)
        } else {
            value
        };
        value.clamp(self.low, self.high)
    }

    pub(crate) fn max_step_index(self) -> Option<u64> {
        self.step
            .map(|step| ((self.high - self.low) / step).floor() as u64)
    }

    pub(crate) fn grid_value(self, index: u64) -> f64 {
        self.step
            .map_or(self.low, |step| step.mul_add(index as f64, self.low))
    }
}

/// Scale used by an integer distribution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum IntScale {
    Linear,
    Log,
}

/// A bounded integer distribution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct IntDistribution {
    low: i64,
    high: i64,
    scale: IntScale,
    step: u64,
}

impl IntDistribution {
    /// Create a linear integer distribution with unit step.
    pub fn linear(low: i64, high: i64) -> Result<Self, ParzenError> {
        Self::new(low, high, IntScale::Linear)
    }
    /// Create a positive logarithmic integer distribution with unit step.
    pub fn log(low: i64, high: i64) -> Result<Self, ParzenError> {
        Self::new(low, high, IntScale::Log)
    }
    fn new(low: i64, high: i64, scale: IntScale) -> Result<Self, ParzenError> {
        if low > high {
            return Err(ParzenError::InvalidDistribution(
                "integer low must be <= high".into(),
            ));
        }
        if scale == IntScale::Log && low <= 0 {
            return Err(ParzenError::InvalidDistribution(
                "log-integer bounds must be positive".into(),
            ));
        }
        Ok(Self {
            low,
            high,
            scale,
            step: 1,
        })
    }
    /// Set a positive linear step anchored at `low`.
    pub fn with_step(mut self, step: u64) -> Result<Self, ParzenError> {
        if step == 0 {
            return Err(ParzenError::InvalidDistribution(
                "integer step must be positive".into(),
            ));
        }
        if self.scale == IntScale::Log && step != 1 {
            return Err(ParzenError::InvalidDistribution(
                "log-integer distributions require unit step".into(),
            ));
        }
        self.step = step;
        Ok(self)
    }
    #[must_use]
    pub const fn low(self) -> i64 {
        self.low
    }
    #[must_use]
    pub const fn high(self) -> i64 {
        self.high
    }
    #[must_use]
    pub const fn scale(self) -> IntScale {
        self.scale
    }
    #[must_use]
    pub const fn step(self) -> u64 {
        self.step
    }

    pub(crate) fn transform(self, value: i64) -> f64 {
        let value = value as f64;
        match self.scale {
            IntScale::Linear => value,
            IntScale::Log => value.ln(),
        }
    }

    pub(crate) fn untransform(self, value: f64) -> i64 {
        let raw = match self.scale {
            IntScale::Linear => value,
            IntScale::Log => value.exp(),
        };
        let low = self.low as i128;
        let rounded = raw.round().clamp(self.low as f64, self.high as f64) as i128;
        let step = i128::from(self.step);
        let aligned = low + ((rounded - low + step / 2) / step) * step;
        let high = i128::from(self.high);
        let highest_legal = low + ((high - low) / step) * step;
        aligned.clamp(low, highest_legal) as i64
    }

    pub(crate) fn max_step_index(self) -> u64 {
        let width = i128::from(self.high) - i128::from(self.low);
        (width / i128::from(self.step)) as u64
    }

    pub(crate) fn grid_value(self, index: u64) -> i64 {
        (i128::from(self.low) + i128::from(index) * i128::from(self.step)) as i64
    }
}

impl Distribution {
    pub(crate) fn canonicalize(&self, value: ParamValue) -> Option<ParamValue> {
        match (self, value) {
            (Self::Categorical(dist), ParamValue::Categorical(choice)) => {
                (choice < dist.num_choices).then_some(ParamValue::Categorical(choice))
            }
            (Self::Float(dist), ParamValue::Float(value)) => {
                if !value.is_finite() || value < dist.low || value > dist.high {
                    return None;
                }
                let canonical = dist.quantize(value);
                if dist.step.is_none() || ulp_distance(value, canonical) <= 4 {
                    Some(ParamValue::Float(canonical))
                } else {
                    None
                }
            }
            (Self::Int(dist), ParamValue::Int(value)) => {
                (value >= dist.low && value <= dist.high && {
                    let delta = i128::from(value) - i128::from(dist.low);
                    delta % i128::from(dist.step) == 0
                })
                .then_some(ParamValue::Int(value))
            }
            _ => None,
        }
    }
}

fn ulp_distance(left: f64, right: f64) -> u64 {
    fn ordered(value: f64) -> u64 {
        let bits = value.to_bits();
        if bits >> 63 == 0 {
            bits | (1 << 63)
        } else {
            !bits
        }
    }
    ordered(left).abs_diff(ordered(right))
}

#[cfg(feature = "serde")]
mod deserialize {
    use serde::{Deserialize, Deserializer, de::Error as _};

    use super::*;

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct CategoricalWire {
        num_choices: u32,
    }

    impl<'de> Deserialize<'de> for CategoricalDistribution {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            let wire = CategoricalWire::deserialize(deserializer)?;
            Self::new(wire.num_choices).map_err(D::Error::custom)
        }
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct FloatWire {
        low: f64,
        high: f64,
        scale: FloatScale,
        step: Option<f64>,
    }

    impl<'de> Deserialize<'de> for FloatDistribution {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            let wire = FloatWire::deserialize(deserializer)?;
            let distribution = match wire.scale {
                FloatScale::Linear => Self::linear(wire.low, wire.high),
                FloatScale::Log => Self::log(wire.low, wire.high),
            }
            .map_err(D::Error::custom)?;
            wire.step.map_or(Ok(distribution), |step| {
                distribution.with_step(step).map_err(D::Error::custom)
            })
        }
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct IntWire {
        low: i64,
        high: i64,
        scale: IntScale,
        step: u64,
    }

    impl<'de> Deserialize<'de> for IntDistribution {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            let wire = IntWire::deserialize(deserializer)?;
            let distribution = match wire.scale {
                IntScale::Linear => Self::linear(wire.low, wire.high),
                IntScale::Log => Self::log(wire.low, wire.high),
            }
            .map_err(D::Error::custom)?;
            distribution.with_step(wire.step).map_err(D::Error::custom)
        }
    }
}

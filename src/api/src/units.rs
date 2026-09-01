//! Basic implementation of common units, mostly for formatting and display not for complex arithmetic.
//! Not all units are implemented only those relevant to the API.
//! Supported units are only SI base units, derived units and °C, no US customs nor imperial units.

use core::f64;
use derive_more::{self, Add, Div, Mul, Sub};
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use utoipa::ToSchema;

fn log_to_base(x: f64, base: f64) -> f64 {
    if x == 0.0 {
        return f64::NAN;
    };

    x.ln() / base.ln()
}

struct Configuration<'a> {
    suffixes: &'a [&'a str],
    base: Option<f64>,
    value: f64,
}

impl Display for Configuration<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let suffixes = self.suffixes;
        let val = self.value;
        let suffix;
        let formatted_value;

        if let Some(base) = self.base {
            let power_of_thousand = if val != 0.0 {
                log_to_base(val, base).floor() as i32
            } else {
                0
            };

            if power_of_thousand > 0 {
                formatted_value = val / base.powi(power_of_thousand);
                suffix = suffixes
                    .get(power_of_thousand as usize)
                    .unwrap_or(&suffixes[0]);
            } else {
                formatted_value = val;
                suffix = &suffixes[0];
            };
            write!(f, "{:.2}{suffix}", formatted_value)
        } else {
            write!(f, "{:.2}{}", val, suffixes[0])
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, ToSchema, Add, Sub, Div, Mul)]
pub struct Hertz(pub f64);

impl Display for Hertz {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let c = Configuration {
            suffixes: &["Hz", "kHz", "MHz", "GHz", "THz", "PHz", "EHz"],
            base: Some(1000.0_f64),
            value: self.0,
        };
        write!(f, "{c}")
    }
}

#[derive(
    Serialize,
    Deserialize,
    Debug,
    Clone,
    Copy,
    Eq,
    PartialEq,
    PartialOrd,
    ToSchema,
    Add,
    Sub,
    Div,
    Mul,
)]
pub struct Bytes(pub u64);

impl Display for Bytes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let c = Configuration {
            suffixes: &["B", "KiB", "MiB", "GiB", "TiB", "PiB", "EiB"],
            base: Some(1024.0_f64),
            value: self.0 as f64,
        };
        write!(f, "{c}")
    }
}

#[derive(
    Serialize, Deserialize, Debug, Clone, Copy, PartialEq, PartialOrd, ToSchema, Add, Sub, Div, Mul,
)]
pub struct Celcius(pub f64);

impl Display for Celcius {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let c = Configuration {
            suffixes: &["°C"],
            base: None,
            value: self.0,
        };
        write!(f, "{c}")
    }
}

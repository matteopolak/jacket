use crate::Error;
use jai_vm::Limits;
use std::{
    ffi::{OsStr, OsString},
    fmt,
    num::{NonZeroU64, NonZeroUsize, ParseIntError},
};

#[derive(Clone, Copy, Debug)]
enum Counter {
    StackDepth,
    EvaluationDepth,
    Allocations,
    ValueCells,
}

#[derive(Clone, Copy, Debug)]
enum Limit {
    Fuel,
    Counter(Counter),
}

enum Setting {
    Fuel(NonZeroU64),
    Counter(Counter, NonZeroUsize),
}

#[derive(Debug)]
pub struct ParseError {
    limit: Limit,
    cause: ValueError,
}

#[derive(Debug)]
enum ValueError {
    NonUtf8,
    Integer(ParseIntError),
}

impl Limit {
    fn environment_key(self) -> &'static str {
        match self {
            Self::Fuel => "JAI_RS_CT_FUEL",
            Self::Counter(Counter::StackDepth) => "JAI_RS_CT_STACK_DEPTH",
            Self::Counter(Counter::EvaluationDepth) => "JAI_RS_CT_EVALUATION_DEPTH",
            Self::Counter(Counter::Allocations) => "JAI_RS_CT_ALLOCATIONS",
            Self::Counter(Counter::ValueCells) => "JAI_RS_CT_VALUE_CELLS",
        }
    }

    fn parse(self, value: &OsStr) -> Result<Setting, ParseError> {
        let text = value.to_str().ok_or(ParseError {
            limit: self,
            cause: ValueError::NonUtf8,
        })?;
        let error = |cause| ParseError {
            limit: self,
            cause: ValueError::Integer(cause),
        };
        match self {
            Self::Fuel => text.parse().map(Setting::Fuel).map_err(error),
            Self::Counter(counter) => text
                .parse()
                .map(|value| Setting::Counter(counter, value))
                .map_err(error),
        }
    }
}

impl Setting {
    fn apply(self, limits: &mut Limits) {
        match self {
            Self::Fuel(value) => limits.fuel = value.get(),
            Self::Counter(counter, value) => match counter {
                Counter::StackDepth => limits.stack_depth = value.get(),
                Counter::EvaluationDepth => limits.evaluation_depth = value.get(),
                Counter::Allocations => limits.allocations = value.get(),
                Counter::ValueCells => limits.value_cells = value.get(),
            },
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, output: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            output,
            "{} requires a positive integer: ",
            self.limit.environment_key()
        )?;
        match &self.cause {
            ValueError::NonUtf8 => output.write_str("value is not UTF-8"),
            ValueError::Integer(cause) => fmt::Display::fmt(cause, output),
        }
    }
}

pub fn parse(environment: &impl Fn(&str) -> Option<OsString>) -> Result<Limits, Error> {
    let mut limits = Limits::default();
    for limit in [
        Limit::Fuel,
        Limit::Counter(Counter::StackDepth),
        Limit::Counter(Counter::EvaluationDepth),
        Limit::Counter(Counter::Allocations),
        Limit::Counter(Counter::ValueCells),
    ] {
        if let Some(value) = environment(limit.environment_key()) {
            limit
                .parse(&value)
                .map_err(Error::CompileTimeLimit)?
                .apply(&mut limits);
        }
    }
    Ok(limits)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn configured(values: &[(&str, &str)]) -> Result<Limits, Error> {
        parse(&|key| {
            values
                .iter()
                .find_map(|(name, value)| (*name == key).then(|| OsString::from(value)))
        })
    }

    #[test]
    fn overrides_are_independent_and_preserve_other_resource_bounds() {
        let limits = configured(&[
            ("JAI_RS_CT_FUEL", "20000000"),
            ("JAI_RS_CT_STACK_DEPTH", "64"),
        ])
        .unwrap();
        assert_eq!(limits.fuel, 20_000_000);
        assert_eq!(limits.stack_depth, 64);
        assert_eq!(limits.value_cells, Limits::default().value_cells);
        assert_eq!(limits.allocations, Limits::default().allocations);
        assert_eq!(limits.evaluation_depth, Limits::default().evaluation_depth);
    }

    #[test]
    fn invalid_budgets_report_the_exact_setting() {
        for value in ["0", "-1", "", "unlimited", "18446744073709551616"] {
            let error = configured(&[("JAI_RS_CT_FUEL", value)]).unwrap_err();
            assert!(error.to_string().starts_with("JAI_RS_CT_FUEL requires"));
        }
    }
}

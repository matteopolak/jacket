use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Number;
use std::cmp::Ordering;

/// A JSON-RPC request id: a string or a JSON number. A number is an integer, or a float
/// with no fraction such as `1.0`, which some clients send; a response echoes the id in the
/// form it arrived in. `1` and `1.0` name the same request (a cancellation for one
/// cancels the other), as they are the same JSON number.
#[derive(Clone, Debug)]
pub enum RequestId {
    Number(i64),
    /// A whole number written as a float, kept so the response says `1.0`, not `1`.
    Float(f64),
    String(String),
}

impl RequestId {
    /// What equality and ordering compare: numbers by value (before strings), strings by text.
    fn key(&self) -> (bool, i64, &str) {
        match self {
            Self::Number(n) => (false, *n, ""),
            Self::Float(f) => (false, *f as i64, ""),
            Self::String(text) => (true, 0, text),
        }
    }
}

impl PartialEq for RequestId {
    fn eq(&self, other: &Self) -> bool {
        self.key() == other.key()
    }
}

impl Eq for RequestId {
}

impl PartialOrd for RequestId {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for RequestId {
    fn cmp(&self, other: &Self) -> Ordering {
        self.key().cmp(&other.key())
    }
}

impl Serialize for RequestId {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Number(n) => serializer.serialize_i64(*n),
            Self::Float(f) => Number::from_f64(*f)
                .ok_or_else(|| serde::ser::Error::custom("request id is not finite"))?
                .serialize(serializer),
            Self::String(text) => serializer.serialize_str(text),
        }
    }
}

impl<'de> Deserialize<'de> for RequestId {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use serde::de::Error;
        const MESSAGE: &str = "request id must be a string or a whole number";
        match serde_json::Value::deserialize(deserializer)? {
            serde_json::Value::String(text) => Ok(Self::String(text)),
            serde_json::Value::Number(number) => {
                if let Some(n) = number.as_i64() {
                    return Ok(Self::Number(n));
                }
                // 2^63 is the first float an `i64` cannot hold.
                number
                    .as_f64()
                    .filter(|f| f.fract() == 0.0 && f.abs() < 9_223_372_036_854_775_808.0)
                    .map(Self::Float)
                    .ok_or_else(|| Error::custom(MESSAGE))
            }
            _ => Err(Error::custom(MESSAGE)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn parse(text: &str) -> Option<RequestId> {
        serde_json::from_str(text).ok()
    }

    #[test]
    fn accepts_integers_whole_floats_and_strings() {
        assert_eq!(parse("7"), Some(RequestId::Number(7)));
        assert_eq!(parse("-3"), Some(RequestId::Number(-3)));
        assert_eq!(
            parse("9007199254740993"),
            Some(RequestId::Number(9_007_199_254_740_993))
        );
        assert_eq!(parse("1.0"), Some(RequestId::Number(1)));
        assert!(matches!(parse("1.0"), Some(RequestId::Float(_))));
        assert_eq!(parse("2e3"), Some(RequestId::Number(2000)));
        assert!(matches!(parse("2e3"), Some(RequestId::Float(_))));
        assert_eq!(parse("\"abc\""), Some(RequestId::String("abc".into())));
    }

    #[test]
    fn rejects_everything_else() {
        for text in [
            "1.5",
            "true",
            "null",
            "[]",
            "{}",
            "1e300",
            "-1e19",
            "18446744073709551615",
        ] {
            assert_eq!(parse(text), None, "{text}");
        }
    }

    #[test]
    fn echoes_the_form_it_received() {
        for text in ["7", "-3", "1.0", "2000.0", "\"abc\"", "\"1\""] {
            let id = parse(text).unwrap();
            assert_eq!(serde_json::to_string(&id).unwrap(), text);
            assert_eq!(json!(id).to_string(), text);
        }
    }

    #[test]
    fn one_and_one_point_zero_are_the_same_id_but_not_the_string() {
        assert_eq!(parse("1"), parse("1.0"));
        assert_ne!(parse("1"), parse("\"1\""));
        assert!(parse("1") < parse("\"0\""));
        assert!(parse("-1") < parse("0.0"));
    }
}

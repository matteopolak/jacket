//! Independent TOML test oracle. Runtime configuration continues using the same pinned Rust parser.
use serde_json::{Value as Json, json};
use std::io::{self, Read};
use std::process::ExitCode;

fn tag(value: &toml::Value) -> Json {
    let (kind, text) = match value {
        toml::Value::Table(table) => {
            return Json::Object(table.iter().map(|(k, v)| (k.clone(), tag(v))).collect());
        }
        toml::Value::Array(array) => return Json::Array(array.iter().map(tag).collect()),
        toml::Value::String(text) => ("string", text.clone()),
        toml::Value::Integer(value) => ("integer", value.to_string()),
        toml::Value::Float(value) => ("float", value.to_string().to_lowercase()),
        toml::Value::Boolean(value) => ("bool", value.to_string()),
        toml::Value::Datetime(value) => {
            let kind = match (value.date, value.time, value.offset) {
                (Some(_), Some(_), Some(_)) => "datetime",
                (Some(_), Some(_), None) => "datetime-local",
                (Some(_), None, None) => "date-local",
                (None, Some(_), None) => "time-local",
                _ => unreachable!("parser returned invalid datetime"),
            };
            let mut text = value.to_string();
            if value.time.is_some() {
                let at = if value.date.is_some() {
                    16
                } else {
                    5
                };
                if text.len() == at
                    || text
                        .as_bytes()
                        .get(at)
                        .is_some_and(|b| matches!(b, b'Z' | b'z' | b'+' | b'-'))
                {
                    text.insert_str(at, ":00");
                }
            }
            (kind, text)
        }
    };
    json!({"type": kind, "value": text})
}

fn untag(value: &Json) -> Result<toml::Value, String> {
    if let Some(array) = value.as_array() {
        return array
            .iter()
            .map(untag)
            .collect::<Result<Vec<_>, _>>()
            .map(toml::Value::Array);
    }
    let table = value
        .as_object()
        .ok_or("tagged value must be an object or array")?;
    if table.len() == 2
        && let (Some(kind), Some(text)) = (
            table.get("type").and_then(Json::as_str),
            table.get("value").and_then(Json::as_str),
        )
    {
        return match kind {
            "string" => Ok(toml::Value::String(text.into())),
            "integer" => text
                .parse()
                .map(toml::Value::Integer)
                .map_err(|e| format!("{e}")),
            "float" => text
                .parse()
                .map(toml::Value::Float)
                .map_err(|e| format!("{e}")),
            "bool" => text
                .parse()
                .map(toml::Value::Boolean)
                .map_err(|e| format!("{e}")),
            "datetime" | "datetime-local" | "date-local" | "time-local" => text
                .parse()
                .map(toml::Value::Datetime)
                .map_err(|e| format!("{e}")),
            _ => Err(format!("unknown tag {kind}")),
        };
    }
    table
        .iter()
        .map(|(k, v)| Ok((k.clone(), untag(v)?)))
        .collect::<Result<toml::Table, String>>()
        .map(toml::Value::Table)
}

fn run() -> Result<(), String> {
    let args: Vec<_> = std::env::args().collect();
    let mut mode = args.get(1).cloned().unwrap_or_default();
    if mode.is_empty()
        && args
            .first()
            .is_some_and(|p| p.ends_with("rust-encoder") || p.ends_with("rust-encoder.exe"))
    {
        mode = "encode".into();
    }
    let batch = mode.starts_with("batch-");
    let mut input_bytes = Vec::new();
    if batch {
        input_bytes =
            std::fs::read(args.get(2).ok_or("missing input path")?).map_err(|e| e.to_string())?;
    } else {
        io::stdin()
            .read_to_end(&mut input_bytes)
            .map_err(|e| e.to_string())?;
    }
    let input = if batch {
        ""
    } else {
        std::str::from_utf8(&input_bytes).map_err(|e| e.to_string())?
    };
    if matches!(
        mode.as_str(),
        "batch-read"
            | "batch-read-write"
            | "batch-write"
            | "batch-lookup"
            | "batch-reject"
            | "batch-verify"
            | "batch-write-verify"
            | "batch-verify-error"
    ) {
        let iterations: usize = args
            .get(3)
            .ok_or("missing iterations")?
            .parse()
            .map_err(|e| format!("{e}"))?;
        if iterations == 0 {
            return Err("iterations must be positive".into());
        }
        let prepared =
            if mode == "batch-write" || mode == "batch-write-verify" || mode == "batch-lookup" {
                Some(
                    toml::from_str::<toml::Table>(
                        std::str::from_utf8(&input_bytes).map_err(|e| format!("{e}"))?,
                    )
                    .map_err(|e| format!("{e}"))?,
                )
            } else {
                None
            };
        let mut checksum = 0u64;
        let mut output_bytes = 0usize;
        let mut started = std::time::Instant::now();
        let first: isize = if args.get(4).is_some_and(|flag| flag == "cold") {
            0
        } else {
            -3
        };
        for iteration in first..iterations as isize {
            if iteration == 0 {
                started = std::time::Instant::now();
                checksum = 0;
                output_bytes = 0;
            }
            if mode == "batch-reject" || mode == "batch-verify-error" {
                let parsed = toml::from_str::<toml::Table>(
                    std::str::from_utf8(std::hint::black_box(input_bytes.as_slice()))
                        .map_err(|e| format!("{e}"))?,
                );
                if mode == "batch-verify-error" {
                    println!("{}", json!({"rejected": parsed.is_err()}));
                    return Ok(());
                }
                std::hint::black_box(parsed.err().ok_or("invalid benchmark input was accepted")?);
                checksum += 1;
                continue;
            }
            let parsed;
            let value = if let Some(value) = &prepared {
                value
            } else {
                parsed = toml::from_str::<toml::Table>(
                    std::str::from_utf8(std::hint::black_box(input_bytes.as_slice()))
                        .map_err(|e| format!("{e}"))?,
                )
                .map_err(|e| format!("{e}"))?;
                &parsed
            };
            std::hint::black_box(value);
            if mode == "batch-lookup" {
                for key in value.keys() {
                    std::hint::black_box(
                        value
                            .get(std::hint::black_box(key))
                            .ok_or("lookup disappeared")?,
                    );
                    checksum += 1;
                }
            } else if mode == "batch-read-write"
                || mode == "batch-write"
                || mode == "batch-verify"
                || mode == "batch-write-verify"
            {
                let text = toml::to_string(&value).map_err(|e| format!("{e}"))?;
                if iteration == 0 && (mode == "batch-verify" || mode == "batch-write-verify") {
                    print!("{text}");
                    return Ok(());
                }
                checksum = text
                    .bytes()
                    .fold(checksum, |sum, b| sum.wrapping_add(u64::from(b)));
                output_bytes += text.len();
            }
        }
        let nanoseconds = started.elapsed().as_nanos();
        println!(
            "{}",
            json!({"nanoseconds": nanoseconds, "iterations": iterations, "checksum": checksum, "output_bytes": output_bytes})
        );
    } else if mode == "encode" {
        let value: Json = serde_json::from_str(input).map_err(|e| e.to_string())?;
        let output = toml::to_string(&untag(&value)?).map_err(|e| e.to_string())?;
        print!("{output}");
    } else if mode == "settings" {
        let (config, unknown) = jacket_config::parse_noting(input)?;
        config.format.validate()?;
        config.doc.format.validate_doc()?;
        let resolved: serde_json::Map<String, Json> = config
            .format
            .resolved()
            .into_iter()
            .map(|(key, text)| (key.into(), Json::String(text)))
            .collect();
        let overrides: serde_json::Map<String, Json> = config
            .doc
            .format
            .overrides()
            .into_iter()
            .map(|(key, text)| (key.into(), Json::String(text)))
            .collect();
        println!(
            "{}",
            json!({"resolved": resolved, "overrides": overrides, "ignore": config.format.ignore, "unknown": unknown})
        );
    } else {
        let table: toml::Table = toml::from_str(input).map_err(|e: toml::de::Error| {
            json!({"message": e.message(), "span": e.span().map(|r| [r.start, r.end])}).to_string()
        })?;
        println!("{}", tag(&toml::Value::Table(table)));
    }
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

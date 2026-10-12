//! Reads `hex(pattern)\thex(text)` lines and prints, per line, every match's capture spans
//! (`start-end`, or `x` for a group that did not take part), comma-separated, matches separated
//! by spaces and ended by `|`; `E` if the pattern does not compile.

use std::io::{self, BufRead, Write};

fn unhex(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}

fn main() {
    let stdin = io::stdin();
    let mut out = io::BufWriter::new(io::stdout());
    for line in stdin.lock().lines() {
        let line = line.unwrap();
        let mut parts = line.split('\t');
        let pattern = String::from_utf8(unhex(parts.next().unwrap())).unwrap();
        let text = String::from_utf8(unhex(parts.next().unwrap_or(""))).unwrap();
        let Ok(re) = regex::RegexBuilder::new(&pattern).size_limit(1 << 26).build() else {
            writeln!(out, "E").unwrap();
            continue;
        };
        let matches: Vec<String> = re
            .captures_iter(&text)
            .map(|caps| {
                (0..caps.len())
                    .map(|i| match caps.get(i) {
                        Some(m) => format!("{}-{}", m.start(), m.end()),
                        None => "x".into(),
                    })
                    .collect::<Vec<_>>()
                    .join(",")
            })
            .collect();
        writeln!(out, "{}|", matches.join(" ")).unwrap();
    }
}

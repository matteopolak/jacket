// Regex benchmark for Rust: the `regex` crate (compiled at run time) and regex-automata's dense
// DFA (built by build.rs, deserialized zero-copy). Usage: regex-bench CORPUS RUNS [CASE...]
// Output: RESULT engine case status count check mbps compile_us note
use regex::Regex;
use regex_automata::{dfa::{dense, Automaton}, Input};
use std::time::Instant;

#[repr(align(16))]
struct Aligned<T: ?Sized>(T);

const CASES: &str = include_str!("../../cases.tsv");

macro_rules! dfa_bytes {
    ($($id:literal),*) => {
        fn dfa_for(id: &str) -> Option<(&'static [u8], &'static str)> {
            match id {
                $($id => {
                    static BYTES: &Aligned<[u8]> = &Aligned(*include_bytes!(concat!(env!("OUT_DIR"), "/", $id, ".dfa")));
                    Some((&BYTES.0, include_str!(concat!(env!("OUT_DIR"), "/", $id, ".err"))))
                })*
                _ => None,
            }
        }
    };
}
dfa_bytes!("date", "email", "greek", "nomatch", "nocase", "wordb", "alt", "literal", "ing", "caps");

fn mbps(bytes: usize, secs: f64) -> f64 {
    bytes as f64 / 1e6 / secs
}

fn best_of<F: FnMut() -> (usize, u64)>(runs: usize, mut f: F) -> (usize, u64, f64) {
    let mut best = f64::MAX;
    let mut out = (0, 0);
    for _ in 0..runs {
        let t = Instant::now();
        out = f();
        best = best.min(t.elapsed().as_secs_f64());
    }
    (out.0, out.1, best)
}

fn na(engine: &str, id: &str, why: &str) {
    println!("RESULT\t{engine}\t{id}\tna\t0\t0\t0\t0\t{why}");
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let data = std::fs::read(&args[1]).unwrap();
    let text = std::str::from_utf8(&data).unwrap();
    let runs: usize = args[2].parse().unwrap();
    let wanted: Vec<&str> = args[3..].iter().map(String::as_str).collect();
    for line in CASES.lines() {
        if line.starts_with('#') || line.trim().is_empty() {
            continue;
        }
        let mut f = line.split('\t');
        let id = f.next().unwrap();
        let pattern = f.next().unwrap();
        if !wanted.is_empty() && !wanted.contains(&id) {
            continue;
        }
        let captures = id == "caps";

        // regex crate: compile time is the best of 20, search is the best of RUNS.
        let mut compile = f64::MAX;
        for _ in 0..20 {
            let t = Instant::now();
            let re = Regex::new(pattern).unwrap();
            compile = compile.min(t.elapsed().as_secs_f64());
            drop(re);
        }
        let re = Regex::new(pattern).unwrap();
        let (count, check, best) = if captures {
            best_of(runs, || {
                let mut n = 0;
                let mut sum = 0u64;
                for c in re.captures_iter(text) {
                    n += 1;
                    for g in 1..=3 {
                        sum += c[g].parse::<u64>().unwrap();
                    }
                }
                (n, sum)
            })
        } else {
            best_of(runs, || (re.find_iter(text).count(), 0))
        };
        println!("RESULT\trust-regex\t{id}\tok\t{count}\t{check}\t{}\t{}\t", mbps(data.len(), best), compile * 1e6);

        // regex-automata: dense DFA from build.rs. A DFA has no capture groups.
        let engine = "rust-dfa";
        if captures {
            na(engine, id, "a DFA reports match bounds only, not capture groups");
            continue;
        }
        let (bytes, err) = dfa_for(id).unwrap();
        if bytes.is_empty() {
            na(engine, id, &format!("no dense DFA: {err}"));
            continue;
        }
        let (dfa, _) = dense::DFA::from_bytes(bytes).unwrap();
        let (count, _, best) = best_of(runs, || {
            let mut n = 0;
            let mut pos = 0;
            while pos <= text.len() {
                let input = Input::new(text).range(pos..);
                match dfa.try_search_fwd(&input).unwrap() {
                    Some(m) => {
                        n += 1;
                        pos = m.offset().max(pos + 1);
                    }
                    None => break,
                }
            }
            (n, 0)
        });
        println!("RESULT\t{engine}\t{id}\tok\t{count}\t0\t{}\t0\t", mbps(data.len(), best));
    }
}
